"""Auditable post-extraction curation for schematic corpora.

Extraction should be lossless.  This module creates reproducible *views* over
that raw corpus for registry ingestion, rankings, and distribution packages.
Rules never delete source schematics; every rejection is written with a reason.
"""

from __future__ import annotations

import collections
import csv
import dataclasses
import datetime as _datetime
import hashlib
import json
import operator
import re
import zipfile
from pathlib import Path
from typing import Any, Callable, Dict, Iterable, List, Mapping, Optional, Sequence, Tuple


_OPERATORS: Dict[str, Callable[[Any, Any], bool]] = {
    "eq": operator.eq,
    "ne": operator.ne,
    "ge": operator.ge,
    "gt": operator.gt,
    "le": operator.le,
    "lt": operator.lt,
    "in": lambda actual, expected: actual in expected,
    "not_in": lambda actual, expected: actual not in expected,
}


def _utc_now() -> str:
    return _datetime.datetime.now(_datetime.timezone.utc).isoformat().replace("+00:00", "Z")


def _resolve(record: Mapping[str, Any], field: str) -> Any:
    value: Any = record
    for part in field.split("."):
        if not isinstance(value, Mapping) or part not in value:
            raise KeyError("curation field {!r} is absent".format(field))
        value = value[part]
    return value


@dataclasses.dataclass(frozen=True)
class MetricRule:
    """One required relationship over a metric or catalogue field."""

    field: str
    operator: str
    value: Any
    reason: str
    source: str = "metric"

    def __post_init__(self) -> None:
        if self.operator not in _OPERATORS:
            raise ValueError("unsupported curation operator: {}".format(self.operator))
        if self.source not in ("metric", "catalog"):
            raise ValueError("curation rule source must be metric or catalog")

    def accepts(self, metric: Mapping[str, Any], catalog: Mapping[str, Any]) -> bool:
        record = metric if self.source == "metric" else catalog
        return bool(_OPERATORS[self.operator](_resolve(record, self.field), self.value))

    def as_dict(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)


@dataclasses.dataclass(frozen=True)
class CurationDecision:
    accepted: bool
    reasons: Tuple[str, ...] = ()


@dataclasses.dataclass(frozen=True)
class CurationPolicy:
    """Serializable rules plus optional named Python predicates.

    Predicates receive ``(metric, catalog)`` and return truthy to keep the
    schematic.  Their names are included in the policy hash, while callers are
    responsible for versioning predicate code alongside the run manifest.
    """

    name: str
    rules: Tuple[MetricRule, ...]
    predicates: Tuple[Tuple[str, Callable[[Mapping[str, Any], Mapping[str, Any]], bool]], ...] = ()
    schema_version: int = 1

    @classmethod
    def minima(
        cls,
        *,
        min_blocks: int = 1,
        min_palette_names: int = 1,
        name: str = "minimum-complexity",
    ) -> "CurationPolicy":
        if min_blocks < 0 or min_palette_names < 0:
            raise ValueError("curation minima cannot be negative")
        return cls(
            name=name,
            rules=(
                MetricRule("block_count", "ge", min_blocks, "block_count_below_{}".format(min_blocks)),
                MetricRule(
                    "palette_names",
                    "ge",
                    min_palette_names,
                    "palette_names_below_{}".format(min_palette_names),
                ),
            ),
        )

    def evaluate(
        self,
        metric: Mapping[str, Any],
        catalog: Optional[Mapping[str, Any]] = None,
    ) -> CurationDecision:
        catalog = catalog or {}
        reasons = [rule.reason for rule in self.rules if not rule.accepts(metric, catalog)]
        for name, predicate in self.predicates:
            if not predicate(metric, catalog):
                reasons.append(name)
        return CurationDecision(not reasons, tuple(reasons))

    def as_dict(self) -> Dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "name": self.name,
            "rules": [rule.as_dict() for rule in self.rules],
            "predicates": [name for name, _predicate in self.predicates],
        }

    def content_id(self) -> str:
        encoded = json.dumps(self.as_dict(), sort_keys=True, separators=(",", ":")).encode()
        return hashlib.sha256(encoded).hexdigest()


@dataclasses.dataclass
class CuratedCorpus:
    root: Path
    policy: CurationPolicy
    accepted: List[Tuple[Dict[str, Any], Dict[str, Any]]]
    rejected_count: int
    rejection_reasons: Dict[str, int]


def _catalog_index(root: Path) -> Dict[str, Dict[str, Any]]:
    result: Dict[str, Dict[str, Any]] = {}
    for path in sorted((root / "catalog").glob("*.jsonl")):
        with path.open() as handle:
            for line_number, line in enumerate(handle, 1):
                record = json.loads(line)
                build_id = record["stable_build_id"]
                if build_id in result:
                    raise ValueError("duplicate catalogue ID: {}".format(build_id))
                record["catalog_source"] = {"file": path.name, "line": line_number}
                result[build_id] = record
    return result


def curate_corpus(root: Path, output: Path, policy: CurationPolicy) -> CuratedCorpus:
    """Evaluate a metrics JSONL stream and write an auditable curated view."""

    root = Path(root)
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    catalog = _catalog_index(root)
    accepted: List[Tuple[Dict[str, Any], Dict[str, Any]]] = []
    rejected_count = 0
    reason_counts: collections.Counter[str] = collections.Counter()
    seen = set()

    with (root / "analysis" / "metrics.jsonl").open() as metrics_handle, \
            (output / "accepted-ids.txt").open("w") as accepted_handle, \
            (output / "rejected.jsonl").open("w") as rejected_handle:
        for line in metrics_handle:
            metric = json.loads(line)
            build_id = metric["id"]
            if build_id in seen:
                raise ValueError("duplicate metrics ID: {}".format(build_id))
            seen.add(build_id)
            if build_id not in catalog:
                raise ValueError("metrics ID missing from catalogue: {}".format(build_id))
            record = catalog[build_id]
            decision = policy.evaluate(metric, record)
            if decision.accepted:
                accepted_handle.write(build_id + "\n")
                accepted.append((metric, record))
            else:
                rejected_count += 1
                reason_counts.update(decision.reasons)
                rejected_handle.write(
                    json.dumps(
                        {
                            "id": build_id,
                            "reasons": list(decision.reasons),
                            "block_count": metric["block_count"],
                            "palette_names": metric["palette_names"],
                            "dominant_block": metric["dominant_block"],
                            "tier": record.get("tier"),
                        },
                        separators=(",", ":"),
                    )
                    + "\n"
                )

    if seen != set(catalog):
        raise ValueError("catalogue and metrics ID sets differ")

    policy_payload = policy.as_dict()
    policy_payload["content_id"] = policy.content_id()
    (output / "policy.json").write_text(json.dumps(policy_payload, indent=2) + "\n")
    summary = {
        "schema_version": 1,
        "created_at": _utc_now(),
        "policy": policy.name,
        "policy_content_id": policy.content_id(),
        "source_schematics": len(seen),
        "accepted": len(accepted),
        "rejected": rejected_count,
        "rejection_reasons": dict(sorted(reason_counts.items())),
    }
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return CuratedCorpus(root, policy, accepted, rejected_count, dict(reason_counts))


def _safe_name(value: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]+", "-", value).strip("-._") or "unknown"


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _suffix(index: int) -> str:
    """A fixed-width sequence token whose lexical order is numeric order."""

    if index < 0:
        raise ValueError("archive part index cannot be negative")
    return "{:04d}".format(index + 1)


def write_registry_archives(
    corpus: CuratedCorpus,
    output: Path,
    *,
    max_files: int = 750,
    max_bytes: int = 90 * 1024 * 1024,
) -> Dict[str, Any]:
    """Package only accepted schematics into deterministic registry batches."""

    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    paths = sorted(corpus.root / "schematics" / (record["stable_build_id"] + ".schem") for _metric, record in corpus.accepted)
    groups: List[List[Path]] = []
    current: List[Path] = []
    current_bytes = 0
    for path in paths:
        size = path.stat().st_size
        if current and (len(current) >= max_files or current_bytes + size > max_bytes):
            groups.append(current)
            current, current_bytes = [], 0
        current.append(path)
        current_bytes += size
    if current:
        groups.append(current)

    parts = []
    for index, group in enumerate(groups):
        part = _suffix(index)
        listing = output / "files-{}".format(part)
        listing.write_text("".join("schematics/{}\n".format(path.name) for path in group))
        archive = output / "ore-build-20260811080019-part-{}.zip".format(part)
        with zipfile.ZipFile(str(archive), "w", compression=zipfile.ZIP_STORED, allowZip64=True) as bundle:
            for path in group:
                bundle.write(str(path), "schematics/{}".format(path.name))
        with zipfile.ZipFile(str(archive)) as bundle:
            bad = bundle.testzip()
            if bad:
                raise IOError("corrupt registry member {} in {}".format(bad, archive))
        parts.append(
            {
                "part": part,
                "files": len(group),
                "archive": archive.name,
                "archive_bytes": archive.stat().st_size,
                "sha256": _sha256(archive),
                "file_list": listing.name,
            }
        )
    payload = {
        "schema_version": 1,
        "created_at": _utc_now(),
        "policy_content_id": corpus.policy.content_id(),
        "schematics": len(paths),
        "parts": parts,
    }
    (output / "index.json").write_text(json.dumps(payload, indent=2) + "\n")
    (output / "README.md").write_text(
        "# Curated schematic registry batches\n\n"
        "Policy: `{}` (`{}`)  \n"
        "Accepted schematics: {:,}  \n"
        "Archive parts: {:,}\n\n"
        "Raw extraction remains available beside this derived view. Each ZIP "
        "is stored without recompression and CRC-checked after creation. "
        "`index.json` contains SHA-256 checksums.\n".format(
            corpus.policy.name,
            corpus.policy.content_id(),
            len(paths),
            len(parts),
        )
    )
    return payload


def write_top_owner_archives(
    corpus: CuratedCorpus,
    output: Path,
    *,
    limit: int = 20,
) -> Dict[str, Any]:
    """Rank owners and package schematics after applying the curation policy."""

    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    by_owner: Dict[str, List[Tuple[Dict[str, Any], Dict[str, Any]]]] = collections.defaultdict(list)
    for metric, record in corpus.accepted:
        by_owner[record.get("partition_metadata", {}).get("owner", "unknown")].append((metric, record))
    ranking = sorted(by_owner.items(), key=lambda item: (-len(item[1]), item[0].casefold(), item[0]))[:limit]
    created = _utc_now()
    archives = []
    for rank, (owner, entries) in enumerate(ranking, 1):
        entries.sort(key=lambda item: item[1]["stable_build_id"])
        archive = output / "{:02d}-{}.zip".format(rank, _safe_name(owner))
        manifest_entries = []
        for _metric, record in entries:
            item = dict(record)
            build_id = record["stable_build_id"]
            item["schematic_file"] = "schematics/{}.schem".format(build_id)
            item["provenance_file"] = "provenance/{}.json".format(build_id)
            manifest_entries.append(item)
        manifest = {
            "schema_version": 1,
            "package_type": "nucleation-curated-owner-schematic-bundle",
            "owner": owner,
            "owner_rank": rank,
            "schematic_count": len(entries),
            "policy": corpus.policy.as_dict(),
            "policy_content_id": corpus.policy.content_id(),
            "created_at": created,
            "entries": manifest_entries,
        }
        readme = (
            "# Curated ORE schematic bundle: {}\n\n"
            "Rank: {} of {}  \n"
            "Schematics: {:,}  \n"
            "Policy: `{}` (`{}`)\n\n"
            "Selection is an exact primary-owner match after corpus curation. "
            "Schematics retain embedded provenance; JSON sidecars are included "
            "for inspection.\n"
        ).format(owner, rank, limit, len(entries), corpus.policy.name, corpus.policy.content_id())
        with zipfile.ZipFile(str(archive), "w", compression=zipfile.ZIP_STORED, allowZip64=True) as bundle:
            bundle.writestr("README.md", readme)
            bundle.writestr("manifest.json", json.dumps(manifest, indent=2) + "\n")
            for _metric, record in entries:
                build_id = record["stable_build_id"]
                bundle.write(str(corpus.root / "schematics" / (build_id + ".schem")), "schematics/{}.schem".format(build_id))
                bundle.write(str(corpus.root / "provenance" / (build_id + ".json")), "provenance/{}.json".format(build_id))
        with zipfile.ZipFile(str(archive)) as bundle:
            bad = bundle.testzip()
            if bad:
                raise IOError("corrupt owner member {} in {}".format(bad, archive))
        archives.append(
            {
                "rank": rank,
                "owner": owner,
                "schematic_count": len(entries),
                "archive": archive.name,
                "archive_bytes": archive.stat().st_size,
                "sha256": _sha256(archive),
            }
        )
    payload = {
        "schema_version": 1,
        "created_at": created,
        "policy_content_id": corpus.policy.content_id(),
        "owner_count": len(archives),
        "schematic_memberships": sum(item["schematic_count"] for item in archives),
        "archives": archives,
    }
    (output / "index.json").write_text(json.dumps(payload, indent=2) + "\n")
    with (output / "index.csv").open("w", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=["rank", "owner", "schematic_count", "archive", "archive_bytes", "sha256"],
        )
        writer.writeheader()
        writer.writerows(archives)
    (output / "README.md").write_text(
        "# Curated top ORE plot-owner archives\n\n"
        "Ranked after applying `{}` (`{}`). Raw extraction was not modified.\n".format(
            corpus.policy.name,
            corpus.policy.content_id(),
        )
    )
    return payload
