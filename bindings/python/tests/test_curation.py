import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).parents[1] / "nucleation" / "curation.py"
SPEC = importlib.util.spec_from_file_location("nucleation_curation", MODULE_PATH)
curation = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
import sys
sys.modules[SPEC.name] = curation
SPEC.loader.exec_module(curation)


class CurationPolicyTests(unittest.TestCase):
    def test_minimum_policy_reports_every_failed_rule(self):
        policy = curation.CurationPolicy.minima(min_blocks=2, min_palette_names=2)
        decision = policy.evaluate({"block_count": 1, "palette_names": 1})
        self.assertFalse(decision.accepted)
        self.assertEqual(
            decision.reasons,
            ("block_count_below_2", "palette_names_below_2"),
        )

    def test_large_monochrome_build_is_rejected_only_by_palette(self):
        policy = curation.CurationPolicy.minima(min_blocks=2, min_palette_names=2)
        decision = policy.evaluate({"block_count": 50_000, "palette_names": 1})
        self.assertEqual(decision.reasons, ("palette_names_below_2",))

    def test_catalogue_fields_and_custom_predicates_are_supported(self):
        policy = curation.CurationPolicy(
            name="custom",
            rules=(curation.MetricRule("tier", "in", ["Confident"], "not_confident", "catalog"),),
            predicates=(("owner_not_test", lambda _metric, catalog: catalog["partition_metadata"]["owner"] != "test"),),
        )
        decision = policy.evaluate(
            {"block_count": 100},
            {"tier": "Confident", "partition_metadata": {"owner": "test"}},
        )
        self.assertEqual(decision.reasons, ("owner_not_test",))

    def test_curate_corpus_writes_accepted_and_rejected_audit_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "corpus"
            output = Path(directory) / "curated"
            (root / "analysis").mkdir(parents=True)
            (root / "catalog").mkdir()
            metrics = [
                {"id": "a", "block_count": 1, "palette_names": 1, "dominant_block": "minecraft:barrel"},
                {"id": "b", "block_count": 20, "palette_names": 3, "dominant_block": "minecraft:stone"},
            ]
            (root / "analysis" / "metrics.jsonl").write_text("".join(json.dumps(row) + "\n" for row in metrics))
            catalogs = [
                {"stable_build_id": "a", "tier": "Debris"},
                {"stable_build_id": "b", "tier": "Probable"},
            ]
            (root / "catalog" / "part.jsonl").write_text("".join(json.dumps(row) + "\n" for row in catalogs))
            corpus = curation.curate_corpus(
                root,
                output,
                curation.CurationPolicy.minima(min_blocks=2, min_palette_names=2),
            )
            self.assertEqual([record[1]["stable_build_id"] for record in corpus.accepted], ["b"])
            self.assertEqual((output / "accepted-ids.txt").read_text(), "b\n")
            rejection = json.loads((output / "rejected.jsonl").read_text())
            self.assertEqual(rejection["id"], "a")
            self.assertEqual(corpus.rejected_count, 1)


if __name__ == "__main__":
    unittest.main()
