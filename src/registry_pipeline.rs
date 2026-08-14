//! Storage-backed, policy-driven registry ingestion.
//!
//! Inputs are streamed through [`Store`] with a hard byte ceiling, decoded by
//! the bounded format readers, transformed atomically, and routed to accept,
//! quarantine, or reject. Extension hooks are deliberately declarative JSON:
//! Python and compiled tools may create rules or consume reports out of
//! process, but Nucleation never loads arbitrary code into the ingest process.

use crate::formats::limits::DecodeLimits;
use crate::formats::manager::get_manager;
use crate::formats::snapshot::to_snapshot;
use crate::processing::{TransformError, TransformPlan, TransformReport};
use crate::store::{Store, StoreError};
use serde::{Deserialize, Serialize};
use std::io::Read;

/// Registry destination selected for one object.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryRoute {
    #[default]
    Accept,
    Quarantine,
    Reject,
}

/// A sandbox-safe rule evaluated against stable report counters.
///
/// Rules can only escalate a route. This makes independently authored hooks
/// composable and prevents a later rule from overriding a core rejection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryHookRule {
    pub summary_code: String,
    #[serde(default = "one")]
    pub minimum: u64,
    pub route: RegistryRoute,
}

fn one() -> u64 {
    1
}

/// Stable report persisted for every ingest attempt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryIngestReport {
    pub schema_version: u32,
    pub source_key: String,
    pub object_id: String,
    pub route: RegistryRoute,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_key: Option<String>,
    /// Route-specific key. For accepted/quarantined objects this equals the
    /// snapshot output; for rejection it is a JSON rejection record.
    pub route_key: String,
    pub report_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformReport>,
}

/// Serializable configuration shared by Rust and language bindings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryPipeline {
    #[serde(default)]
    pub decode_limits: DecodeLimits,
    pub plan: TransformPlan,
    #[serde(default = "accept_prefix")]
    pub accept_prefix: String,
    #[serde(default = "quarantine_prefix")]
    pub quarantine_prefix: String,
    #[serde(default = "reject_prefix")]
    pub reject_prefix: String,
    #[serde(default = "report_prefix")]
    pub report_prefix: String,
    #[serde(default)]
    pub hooks: Vec<RegistryHookRule>,
}

fn accept_prefix() -> String {
    "registry/accepted".into()
}
fn quarantine_prefix() -> String {
    "registry/quarantine".into()
}
fn reject_prefix() -> String {
    "registry/rejected".into()
}
fn report_prefix() -> String {
    "registry/reports".into()
}

impl Default for RegistryPipeline {
    fn default() -> Self {
        Self {
            decode_limits: DecodeLimits::default(),
            plan: TransformPlan::registry_safe(),
            accept_prefix: accept_prefix(),
            quarantine_prefix: quarantine_prefix(),
            reject_prefix: reject_prefix(),
            report_prefix: report_prefix(),
            hooks: Vec::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryPipelineError {
    #[error("storage operation failed: {0}")]
    Storage(#[from] StoreError),
    #[error("registry pipeline configuration is invalid: {0}")]
    InvalidConfiguration(String),
    #[error("could not serialize registry report: {0}")]
    Serialization(String),
}

impl RegistryPipeline {
    pub fn from_json(json: &str) -> Result<Self, RegistryPipelineError> {
        let value: Self = serde_json::from_str(json)
            .map_err(|error| RegistryPipelineError::InvalidConfiguration(error.to_string()))?;
        value.validate()?;
        Ok(value)
    }

    pub fn to_json(&self) -> Result<String, RegistryPipelineError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| RegistryPipelineError::Serialization(error.to_string()))
    }

    pub fn validate(&self) -> Result<(), RegistryPipelineError> {
        self.plan
            .validate()
            .map_err(|error| RegistryPipelineError::InvalidConfiguration(error.to_string()))?;
        for prefix in [
            &self.accept_prefix,
            &self.quarantine_prefix,
            &self.reject_prefix,
            &self.report_prefix,
        ] {
            if prefix.is_empty()
                || prefix.starts_with('/')
                || prefix.split('/').any(|part| part == "..")
            {
                return Err(RegistryPipelineError::InvalidConfiguration(
                    "registry prefixes must be relative logical store keys".into(),
                ));
            }
        }
        if self.hooks.iter().any(|rule| rule.summary_code.is_empty()) {
            return Err(RegistryPipelineError::InvalidConfiguration(
                "hook summary_code must not be empty".into(),
            ));
        }
        Ok(())
    }

    /// Inspect bytes without producing an output schematic.
    pub fn dry_run_bytes(&self, source_key: &str, data: &[u8]) -> RegistryIngestReport {
        self.process_bytes(source_key, data, true).0
    }

    /// Ingest one object between arbitrary storage backends.
    pub fn ingest_store(
        &self,
        input: &dyn Store,
        source_key: &str,
        output: &dyn Store,
    ) -> Result<RegistryIngestReport, RegistryPipelineError> {
        self.validate()?;
        let mut reader = input.reader(source_key)?;
        let ceiling = self.decode_limits.max_input_bytes.saturating_add(1);
        let mut data = Vec::with_capacity(ceiling.min(8 * 1024 * 1024));
        reader
            .by_ref()
            .take(ceiling as u64)
            .read_to_end(&mut data)
            .map_err(StoreError::from)?;

        let (report, encoded) = self.process_bytes(source_key, &data, false);
        if let (Some(key), Some(bytes)) = (&report.output_key, encoded) {
            output.put(key, &bytes)?;
        }
        let report_bytes = serde_json::to_vec_pretty(&report)
            .map_err(|error| RegistryPipelineError::Serialization(error.to_string()))?;
        output.put(&report.report_key, &report_bytes)?;
        if report.route == RegistryRoute::Reject {
            output.put(&report.route_key, &report_bytes)?;
        }
        Ok(report)
    }

    fn process_bytes(
        &self,
        source_key: &str,
        data: &[u8],
        dry_run: bool,
    ) -> (RegistryIngestReport, Option<Vec<u8>>) {
        let object_id = blake3::hash(data).to_hex().to_string();
        let report_key = format!("{}/{}.json", trim(&self.report_prefix), object_id);
        let manager = get_manager();
        let guard = manager
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let decoded = guard.read_bounded_with_format(data, &self.decode_limits);
        drop(guard);

        let (detected_format, mut schematic) = match decoded {
            Ok((format, value)) => (Some(format), value),
            Err(_) => {
                return (
                    RegistryIngestReport {
                        schema_version: 1,
                        source_key: source_key.into(),
                        object_id: object_id.clone(),
                        route: RegistryRoute::Reject,
                        detected_format: None,
                        output_key: None,
                        route_key: format!("{}/{}.json", trim(&self.reject_prefix), object_id),
                        report_key,
                        error_code: Some("decode.rejected".into()),
                        transform: None,
                    },
                    None,
                );
            }
        };

        let transformed = if dry_run {
            self.plan.inspect(&schematic)
        } else {
            self.plan.apply(&mut schematic)
        };
        let (mut route, transform, error_code) = match transformed {
            Ok(report) => (
                if report.quarantined {
                    RegistryRoute::Quarantine
                } else {
                    RegistryRoute::Accept
                },
                Some(report),
                None,
            ),
            Err(TransformError::Rejected(report)) => (
                RegistryRoute::Reject,
                Some(report),
                Some("policy.rejected".into()),
            ),
            Err(TransformError::InvalidPlan(_)) => {
                (RegistryRoute::Reject, None, Some("plan.invalid".into()))
            }
        };
        if let Some(report) = &transform {
            for hook in &self.hooks {
                if report.summary.get(&hook.summary_code).copied().unwrap_or(0) >= hook.minimum {
                    route = route.max(hook.route);
                }
            }
        }
        let route_key = if route == RegistryRoute::Reject {
            format!("{}/{}.json", trim(&self.reject_prefix), object_id)
        } else {
            let prefix = if route == RegistryRoute::Quarantine {
                &self.quarantine_prefix
            } else {
                &self.accept_prefix
            };
            format!("{}/{}.nusn", trim(prefix), object_id)
        };
        let output_key = if dry_run || route == RegistryRoute::Reject {
            None
        } else {
            Some(route_key.clone())
        };
        let encoded = output_key
            .as_ref()
            .and_then(|_| to_snapshot(&schematic).ok());
        (
            RegistryIngestReport {
                schema_version: 1,
                source_key: source_key.into(),
                object_id,
                route,
                detected_format,
                output_key,
                route_key,
                report_key,
                error_code,
                transform,
            },
            encoded,
        )
    }
}

fn trim(prefix: &str) -> &str {
    prefix.trim_end_matches('/')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::snapshot::to_snapshot;
    use crate::{MemStore, UniversalSchematic};

    #[test]
    fn storage_pipeline_routes_and_persists_audit_report() {
        let input = MemStore::new();
        let output = MemStore::new();
        input
            .put(
                "incoming/example.nusn",
                &to_snapshot(&UniversalSchematic::new("demo".into())).unwrap(),
            )
            .unwrap();
        let report = RegistryPipeline::default()
            .ingest_store(&input, "incoming/example.nusn", &output)
            .unwrap();
        assert_eq!(report.route, RegistryRoute::Accept);
        assert!(output
            .exists(report.output_key.as_deref().unwrap())
            .unwrap());
        assert!(output.exists(&report.report_key).unwrap());
    }

    #[test]
    fn declarative_hook_can_only_escalate() {
        let bytes = to_snapshot(&UniversalSchematic::new("demo".into())).unwrap();
        let mut pipeline = RegistryPipeline::default();
        pipeline.hooks.push(RegistryHookRule {
            summary_code: "palette.entries_removed".into(),
            minimum: 0,
            route: RegistryRoute::Quarantine,
        });
        assert_eq!(
            pipeline.dry_run_bytes("demo.nusn", &bytes).route,
            RegistryRoute::Quarantine
        );
    }

    #[test]
    fn oversized_and_unknown_inputs_are_rejected_without_output() {
        let mut pipeline = RegistryPipeline::default();
        pipeline.decode_limits.max_input_bytes = 4;
        let report = pipeline.dry_run_bytes("bad.schem", b"not a schematic");
        assert_eq!(report.route, RegistryRoute::Reject);
        assert_eq!(report.error_code.as_deref(), Some("decode.rejected"));
        assert!(report.output_key.is_none());
    }

    #[test]
    fn rejected_store_input_writes_report_and_reject_route_record() {
        let input = MemStore::new();
        let output = MemStore::new();
        input.put("incoming/bad.schem", b"not a schematic").unwrap();
        let report = RegistryPipeline::default()
            .ingest_store(&input, "incoming/bad.schem", &output)
            .unwrap();
        assert_eq!(report.route, RegistryRoute::Reject);
        assert!(output.exists(&report.report_key).unwrap());
        assert!(output.exists(&report.route_key).unwrap());
    }
}
