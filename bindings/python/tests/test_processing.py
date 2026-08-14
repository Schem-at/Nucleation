import importlib.util
import json
from pathlib import Path
import sys


PROCESSING = Path(__file__).parents[1] / "nucleation" / "processing.py"
SPEC = importlib.util.spec_from_file_location("nucleation_processing_contract", PROCESSING)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


def test_registry_safe_python_facade_contains_core_policy_sections():
    plan = json.loads(MODULE.TransformPlan.registry_safe().to_json())

    assert plan["schema_version"] == 1
    assert plan["name"] == "registry-safe-v1"
    assert [entry["type"] for entry in plan["passes"]] == [
        "canonicalize_palette",
        "content_policy",
    ]

    policy = plan["passes"][1]["policy"]
    assert policy["text"]["strip_keys"] == [
        "CustomName",
        "pages",
        "filtered_pages",
        "author",
        "title",
    ]
    assert policy["nbt"]["executable_action"] == "remove"
    assert policy["nbt"]["profile_action"] == "remove"
    assert policy["nbt"]["volatile_action"] == "remove"
    assert policy["entities"]["excess_action"] == "quarantine"
    assert policy["uuids"]["mode"] == "regenerate_deterministic"
    assert policy["uuids"]["representation"] == "int_array"


if __name__ == "__main__":
    test_registry_safe_python_facade_contains_core_policy_sections()
    print("Python processing contract: ok")
