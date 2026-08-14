"""Idiomatic Python facade for Nucleation transform plans and reports.

The Rust core accepts a versioned JSON contract so exactly the same policy can
be used by every binding.  These small dataclasses make that contract pleasant
to construct from Python while keeping arbitrary future policy fields possible.
"""

from __future__ import annotations

import dataclasses
import json
from typing import Any, Dict, Iterable, Mapping, Optional, Sequence, Tuple, Union


PolicyMapping = Mapping[str, Any]


@dataclasses.dataclass(frozen=True)
class DecodeLimits:
    """Allocation limits for untrusted schematic decoding."""

    max_input_bytes: int = 256 * 1024 * 1024
    max_decompressed_bytes: int = 1024 * 1024 * 1024
    max_dimension: int = 16_384
    max_volume: int = 536_870_912
    max_regions: int = 16_384
    max_palette_entries: int = 1_048_576
    max_entities: int = 1_000_000
    max_block_entities: int = 16_000_000
    max_nbt_depth: int = 64
    max_nbt_string_bytes: int = 1024 * 1024
    max_nbt_collection_items: int = 536_870_912
    max_nbt_nodes: int = 64_000_000

    def to_json(self) -> str:
        return json.dumps(dataclasses.asdict(self), sort_keys=True, separators=(",", ":"))


@dataclasses.dataclass(frozen=True)
class UuidPolicy:
    """Control UUID identity, representation, collisions, and references.

    ``mode`` is ``preserve``, ``remove``, ``regenerate_random`` or
    ``regenerate_deterministic``.  Deterministic UUIDs derive from their stable
    schematic path plus ``salt``, making the operation idempotent.  References
    are rewritten through the definition map rather than independently.
    """

    mode: str = "preserve"
    representation: str = "preserve"
    scope: str = "definitions_and_references"
    salt: str = ""
    identity_basis: str = "stable_path"
    assign_missing: bool = False
    collision: str = "warn"
    dangling: str = "warn"
    definition_keys: Tuple[str, ...] = ("UUID", "uuid")
    reference_keys: Tuple[str, ...] = (
        "Owner",
        "OwnerUUID",
        "Leash",
        "LoveCause",
        "ConversionPlayer",
        "HurtBy",
        "AngryAt",
        "Thrower",
        "Trusted",
    )

    def as_dict(self) -> Dict[str, Any]:
        result = dataclasses.asdict(self)
        result["definition_keys"] = list(self.definition_keys)
        result["reference_keys"] = list(self.reference_keys)
        return result


@dataclasses.dataclass(frozen=True)
class MaterialProfile:
    """A named, versioned mapping between block-material conventions."""

    name: str
    mappings: Mapping[str, str]
    family_mappings: Sequence[Mapping[str, Any]] = ()
    version: int = 1
    target_data_version: Optional[int] = None
    preserve_unmentioned_properties: bool = False
    safety: str = "behavior_preserving"

    def as_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "version": self.version,
            "target_data_version": self.target_data_version,
            "mappings": dict(self.mappings),
            "family_mappings": [dict(rule) for rule in self.family_mappings],
            "preserve_unmentioned_properties": self.preserve_unmentioned_properties,
            "safety": self.safety,
        }


@dataclasses.dataclass(frozen=True)
class ContentPolicy:
    """Serializable content rules; nested sections remain forward-compatible."""

    allowed_namespaces: Optional[Sequence[str]] = None
    namespace_action: str = "warn"
    text: PolicyMapping = dataclasses.field(default_factory=dict)
    nbt: PolicyMapping = dataclasses.field(default_factory=dict)
    items: PolicyMapping = dataclasses.field(default_factory=dict)
    blocks: PolicyMapping = dataclasses.field(default_factory=dict)
    entities: PolicyMapping = dataclasses.field(default_factory=dict)
    block_entities: PolicyMapping = dataclasses.field(default_factory=dict)
    uuids: Union[UuidPolicy, PolicyMapping] = dataclasses.field(default_factory=UuidPolicy)

    def as_dict(self) -> Dict[str, Any]:
        uuids = self.uuids.as_dict() if isinstance(self.uuids, UuidPolicy) else dict(self.uuids)
        return {
            "allowed_namespaces": (
                list(self.allowed_namespaces) if self.allowed_namespaces is not None else None
            ),
            "namespace_action": self.namespace_action,
            "text": dict(self.text),
            "nbt": dict(self.nbt),
            "items": dict(self.items),
            "blocks": dict(self.blocks),
            "entities": dict(self.entities),
            "block_entities": dict(self.block_entities),
            "uuids": uuids,
        }


@dataclasses.dataclass(frozen=True)
class TransformPlan:
    name: str
    passes: Tuple[Mapping[str, Any], ...]
    schema_version: int = 1
    record_history: bool = True

    @classmethod
    def canonical(cls) -> "TransformPlan":
        return cls("canonical", ({"type": "canonicalize_palette"},))

    @classmethod
    def registry_safe(cls) -> "TransformPlan":
        # The core owns the exact bundled preset. This representation is useful
        # when callers want to customize it before applying it.
        return cls(
            "registry-safe-v1",
            (
                {"type": "canonicalize_palette"},
                {
                    "type": "content_policy",
                    "policy": ContentPolicy(
                        text={
                            "strip_keys": [
                                "CustomName",
                                "pages",
                                "filtered_pages",
                                "author",
                                "title",
                            ],
                            "suspicious_patterns": [
                                "ignore previous instructions",
                                "system prompt",
                                "<script",
                                "javascript:",
                                "${jndi:",
                            ],
                            "suspicious_action": "warn",
                        },
                        entities={
                            "denied_ids": [
                                "minecraft:item",
                                "minecraft:experience_orb",
                                "minecraft:area_effect_cloud",
                            ],
                            "denied_action": "remove",
                            "max_total": 512,
                            "excess_action": "quarantine",
                        },
                        uuids=UuidPolicy(
                            mode="regenerate_deterministic",
                            representation="int_array",
                            salt="nucleation:registry-safe:v1",
                        ),
                    ).as_dict(),
                },
            ),
        )

    @classmethod
    def from_passes(
        cls, name: str, passes: Iterable[Mapping[str, Any]]
    ) -> "TransformPlan":
        return cls(name, tuple(dict(item) for item in passes))

    def as_dict(self) -> Dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "name": self.name,
            "record_history": self.record_history,
            "passes": [dict(item) for item in self.passes],
        }

    def to_json(self) -> str:
        return json.dumps(self.as_dict(), sort_keys=True, separators=(",", ":"))


@dataclasses.dataclass(frozen=True)
class TransformReport:
    schema_version: int
    plan: str
    dry_run: bool
    rejected: bool
    quarantined: bool
    summary: Mapping[str, int]
    findings: Tuple[Mapping[str, Any], ...]

    @classmethod
    def from_json(cls, value: str) -> "TransformReport":
        data = json.loads(value)
        data["findings"] = tuple(data.get("findings", ()))
        return cls(**data)


@dataclasses.dataclass(frozen=True)
class RegistryHookRule:
    """Sandbox-safe report rule; it may only escalate an ingest route."""

    summary_code: str
    route: str
    minimum: int = 1


@dataclasses.dataclass(frozen=True)
class RegistryPipelineConfig:
    """JSON contract for the Rust storage-backed registry pipeline.

    Python or compiled extensions can emit these declarative rules and consume
    reports out of process; the ingest process never imports callback code.
    """

    plan: Union[TransformPlan, PolicyMapping]
    decode_limits: DecodeLimits = dataclasses.field(default_factory=DecodeLimits)
    accept_prefix: str = "registry/accepted"
    quarantine_prefix: str = "registry/quarantine"
    reject_prefix: str = "registry/rejected"
    report_prefix: str = "registry/reports"
    hooks: Tuple[RegistryHookRule, ...] = ()

    def to_json(self) -> str:
        plan = self.plan.as_dict() if isinstance(self.plan, TransformPlan) else dict(self.plan)
        value = {
            "decode_limits": dataclasses.asdict(self.decode_limits),
            "plan": plan,
            "accept_prefix": self.accept_prefix,
            "quarantine_prefix": self.quarantine_prefix,
            "reject_prefix": self.reject_prefix,
            "report_prefix": self.report_prefix,
            "hooks": [dataclasses.asdict(hook) for hook in self.hooks],
        }
        return json.dumps(value, sort_keys=True, separators=(",", ":"))


def _plan_json(plan: Union[TransformPlan, PolicyMapping, str]) -> str:
    if isinstance(plan, TransformPlan):
        return plan.to_json()
    if isinstance(plan, str):
        return plan
    return json.dumps(dict(plan), sort_keys=True, separators=(",", ":"))


def inspect_transform(schematic: Any, plan: Union[TransformPlan, PolicyMapping, str]) -> TransformReport:
    """Dry-run ``plan`` using the same core path as a real transformation."""

    return TransformReport.from_json(schematic.inspect_transform_plan_json(_plan_json(plan)))


def apply_transform(schematic: Any, plan: Union[TransformPlan, PolicyMapping, str]) -> TransformReport:
    """Atomically apply ``plan`` and return its audit report."""

    return TransformReport.from_json(schematic.apply_transform_plan_json(_plan_json(plan)))


def decode_bounded(data: bytes, limits: Optional[DecodeLimits] = None) -> Any:
    """Decode untrusted bytes without permitting unbounded parser allocations."""

    from . import Schematic

    return Schematic.from_data_bounded(data, (limits or DecodeLimits()).to_json())
