//! Reusable scalar fields over world-space 3D coordinates.
//!
//! `Field3` is deliberately geometry-neutral: its scalar output may drive
//! geometry, materials, maps, or scatter without acquiring signed-distance
//! semantics.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::Arc;

const MAX_FIELD_JSON_BYTES: usize = 1024 * 1024;
const MAX_FBM_OCTAVES: u32 = 8;
// Keep one full lattice cell plus ample float-rounding headroom below i32::MAX.
const MAX_SAFE_LATTICE_COORD: f64 = i32::MAX as f64 - 1024.0;
const FIELD_GRAPH_VERSION: u32 = 1;
const MAX_GRAPH_NODES: usize = 5_000;
const MAX_GRAPH_ROOTS: usize = 256;
const MAX_ROOT_NAME_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldError {
    InputTooLarge,
    InvalidArgument,
    Parse,
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldError::InputTooLarge => f.write_str("field JSON exceeds the size limit"),
            FieldError::InvalidArgument => f.write_str("invalid field argument"),
            FieldError::Parse => f.write_str("invalid field JSON"),
        }
    }
}

impl std::error::Error for FieldError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum Field3Node {
    /// Deterministic value-noise FBM normalized to `[-1, 1]`.
    ValueNoiseFbm {
        frequency: f32,
        seed: i32,
        octaves: u32,
    },
}

impl Field3Node {
    fn validate(&self) -> Result<(), FieldError> {
        match self {
            Field3Node::ValueNoiseFbm {
                frequency, octaves, ..
            } => {
                if !frequency.is_finite()
                    || *frequency <= 0.0
                    || !(1..=MAX_FBM_OCTAVES).contains(octaves)
                    || (*frequency as f64) * 2.0_f64.powi((*octaves - 1) as i32)
                        > MAX_SAFE_LATTICE_COORD
                {
                    return Err(FieldError::InvalidArgument);
                }
            }
        }
        Ok(())
    }

    fn eval(&self, x: f32, y: f32, z: f32) -> f32 {
        match self {
            Field3Node::ValueNoiseFbm {
                frequency,
                seed,
                octaves,
            } => {
                let max_frequency = (*frequency as f64) * 2.0_f64.powi((*octaves - 1) as i32);
                if [x, y, z].into_iter().any(|coordinate| {
                    (coordinate as f64).abs() * max_frequency > MAX_SAFE_LATTICE_COORD
                }) {
                    return 0.0;
                }
                crate::sdf::noise::fbm3(x, y, z, *seed, *frequency, *octaves)
            }
        }
    }

    fn output_range(&self) -> Option<[f32; 2]> {
        match self {
            Field3Node::ValueNoiseFbm { .. } => Some([-1.0, 1.0]),
        }
    }
}

/// Immutable, shareable scalar field evaluated over world-space `(x, y, z)`.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Field3(Arc<Field3Node>);

impl<'de> Deserialize<'de> for Field3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let node = Field3Node::deserialize(deserializer)?;
        node.validate().map_err(serde::de::Error::custom)?;
        Ok(Self(Arc::new(node)))
    }
}

impl Field3 {
    pub(crate) fn validate(&self) -> Result<(), FieldError> {
        self.0.validate()
    }

    /// Create deterministic normalized value-noise FBM.
    pub fn value_noise_fbm(frequency: f32, seed: i32, octaves: u32) -> Result<Self, FieldError> {
        let node = Field3Node::ValueNoiseFbm {
            frequency,
            seed,
            octaves,
        };
        node.validate()?;
        Ok(Self(Arc::new(node)))
    }

    /// Evaluate the scalar field. Non-finite coordinates map to positive
    /// infinity. Finite coordinates outside this field's safe lattice domain,
    /// or an unexpected non-finite internal result, map to neutral `0`; finite
    /// results are constrained to the analytically proven output range.
    pub fn eval(&self, x: f32, y: f32, z: f32) -> f32 {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return f32::INFINITY;
        }
        let value = self.0.eval(x, y, z);
        match self.output_range() {
            Some([lo, hi]) if value.is_finite() => value.clamp(lo, hi),
            _ => 0.0,
        }
    }

    /// Proven inclusive scalar range, when known analytically.
    pub fn output_range(&self) -> Option<[f32; 2]> {
        self.0.output_range()
    }

    /// Whether two handles point at the same immutable field node.
    pub fn shares_storage_with(&self, other: &Field3) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn from_json(json: &str) -> Result<Self, FieldError> {
        if json.len() > MAX_FIELD_JSON_BYTES {
            return Err(FieldError::InputTooLarge);
        }
        let node: Field3Node = serde_json::from_str(json).map_err(|_| FieldError::Parse)?;
        node.validate()?;
        Ok(Self(Arc::new(node)))
    }

    pub fn to_json(&self) -> Result<String, FieldError> {
        serde_json::to_string(&*self.0).map_err(|_| FieldError::Parse)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Field3GraphDocument {
    version: u32,
    nodes: Vec<Field3Node>,
    roots: BTreeMap<String, usize>,
}

/// A bounded, versioned multi-root field graph.
///
/// Roots refer to a shared node table by ID. Round-tripping therefore restores
/// shared runtime identity instead of deserializing one copy per consumer.
#[derive(Debug, Clone)]
pub struct Field3Graph {
    roots: BTreeMap<String, Field3>,
}

impl Field3Graph {
    pub fn from_roots<I>(roots: I) -> Result<Self, FieldError>
    where
        I: IntoIterator<Item = (String, Field3)>,
    {
        let mut result = BTreeMap::new();
        for (name, field) in roots {
            if name.is_empty()
                || name.len() > MAX_ROOT_NAME_BYTES
                || result.len() >= MAX_GRAPH_ROOTS
                || field.validate().is_err()
                || result.insert(name, field).is_some()
            {
                return Err(FieldError::InvalidArgument);
            }
        }
        if result.is_empty() {
            return Err(FieldError::InvalidArgument);
        }
        Ok(Self { roots: result })
    }

    pub fn root(&self, name: &str) -> Option<Field3> {
        self.roots.get(name).cloned()
    }

    pub fn from_json(json: &str) -> Result<Self, FieldError> {
        if json.len() > MAX_FIELD_JSON_BYTES {
            return Err(FieldError::InputTooLarge);
        }
        let document: Field3GraphDocument =
            serde_json::from_str(json).map_err(|_| FieldError::Parse)?;
        if document.version != FIELD_GRAPH_VERSION
            || document.nodes.is_empty()
            || document.nodes.len() > MAX_GRAPH_NODES
            || document.roots.is_empty()
            || document.roots.len() > MAX_GRAPH_ROOTS
        {
            return Err(FieldError::InvalidArgument);
        }

        let mut nodes = Vec::with_capacity(document.nodes.len());
        for node in document.nodes {
            node.validate()?;
            nodes.push(Arc::new(node));
        }

        let mut roots = BTreeMap::new();
        for (name, node_id) in document.roots {
            if name.is_empty() || name.len() > MAX_ROOT_NAME_BYTES {
                return Err(FieldError::InvalidArgument);
            }
            let node = nodes
                .get(node_id)
                .ok_or(FieldError::InvalidArgument)?
                .clone();
            roots.insert(name, Field3(node));
        }
        Ok(Self { roots })
    }

    pub fn to_json(&self) -> Result<String, FieldError> {
        let mut node_ids: HashMap<usize, usize> = HashMap::new();
        let mut nodes = Vec::new();
        let mut roots = BTreeMap::new();
        for (name, field) in &self.roots {
            let pointer = Arc::as_ptr(&field.0) as usize;
            let node_id = *node_ids.entry(pointer).or_insert_with(|| {
                let id = nodes.len();
                nodes.push((*field.0).clone());
                id
            });
            roots.insert(name.clone(), node_id);
        }
        let document = Field3GraphDocument {
            version: FIELD_GRAPH_VERSION,
            nodes,
            roots,
        };
        serde_json::to_string(&document).map_err(|_| FieldError::Parse)
    }
}
