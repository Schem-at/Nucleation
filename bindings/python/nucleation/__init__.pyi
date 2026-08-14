"""Typed public surface for the compiled core and Python veneers."""

from . import nucleation as core
from .nucleation import *
from .curation import (
    CuratedCorpus as CuratedCorpus,
    CurationDecision as CurationDecision,
    CurationPolicy as CurationPolicy,
    MetricRule as MetricRule,
    curate_corpus as curate_corpus,
    write_registry_archives as write_registry_archives,
    write_top_owner_archives as write_top_owner_archives,
)
from .design import (
    Bus as Bus,
    CheckReport as CheckReport,
    Design as Design,
    DesignCheckError as DesignCheckError,
    Executor as Executor,
    Flat as Flat,
    Gate as Gate,
    Style as Style,
)
from .processing import (
    ContentPolicy as ContentPolicy,
    DecodeLimits as DecodeLimits,
    RegistryHookRule as RegistryHookRule,
    RegistryPipelineConfig as RegistryPipelineConfig,
    MaterialProfile as MaterialProfile,
    TransformPlan as TransformPlan,
    TransformReport as TransformReport,
    UuidPolicy as UuidPolicy,
    apply_transform as apply_transform,
    decode_bounded as decode_bounded,
    inspect_transform as inspect_transform,
)
