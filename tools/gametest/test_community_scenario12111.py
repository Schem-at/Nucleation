"""Guard reference-version provenance when testing saved states across versions."""
import copy
import json
import pathlib
import tempfile
import unittest

from run_community_capture12111 import (INNER_SHA256, MAPPINGS_SHA1, sampled_ticks,
                                       validate_capture, validate_scenario)


class ScenarioValidation(unittest.TestCase):
    def setUp(self):
        self.scenario = {
            'format_version': 1, 'data_version': 4671, 'case': 'note_trigger',
            'negative_control': False, 'source': {'data_version': 4671, 'sha256': 'a' * 64},
            'size': [1, 1, 1], 'origin': [0, 100, 0],
            'watch': {'min': [-1] * 3, 'max': [1] * 3}, 'rng_seed': 0, 'capture_ticks': 10,
            'fixture': [{'pos': [0, 0, 0], 'state': 'minecraft:note_block[instrument=harp,note=0,powered=false]'}],
            'actions': [{'tick': 1, 'type': 'use_note_block', 'pos': [0, 0, 0]}],
        }

    def test_native_note_use_is_accepted(self):
        self.assertEqual(validate_scenario(self.scenario), 27)

    def test_cross_version_is_not_silently_accepted(self):
        self.scenario['source']['data_version'] = 5023
        with self.assertRaisesRegex(ValueError, 'explicit'):
            validate_scenario(self.scenario)

    def test_explicit_mode_preserves_original_version(self):
        self.scenario['source']['data_version'] = 5023
        self.scenario['source_mode'] = 'exact_states_no_datafix'
        before = copy.deepcopy(self.scenario)
        validate_scenario(self.scenario)
        self.assertEqual(self.scenario, before)
        self.assertEqual(self.scenario['source']['data_version'], 5023)

    def test_compatibility_mode_refuses_block_entity_payloads(self):
        self.scenario['source_mode'] = 'exact_states_no_datafix'
        self.scenario['fixture'][0]['nbt'] = '{}'
        with self.assertRaisesRegex(ValueError, 'block entities'):
            validate_scenario(self.scenario)

    def test_compatibility_mode_does_not_allow_entity_import(self):
        self.scenario['source_mode'] = 'exact_states_no_datafix'
        self.scenario['entities'] = [{'id': 'minecraft:minecart'}]
        with self.assertRaisesRegex(ValueError, 'Entity loading'):
            validate_scenario(self.scenario)

    def test_reference_version_cannot_be_relabelled(self):
        self.scenario['data_version'] = 5023
        with self.assertRaisesRegex(ValueError, 'native 4671'):
            validate_scenario(self.scenario)

    def test_note_use_cannot_smuggle_a_state_write(self):
        self.scenario['actions'][0]['state'] = 'minecraft:air'
        with self.assertRaisesRegex(ValueError, 'action'):
            validate_scenario(self.scenario)

    def test_sampling_retains_initial_multiples_and_nondivisible_final(self):
        self.scenario['capture_stride'] = 4
        self.assertEqual(sampled_ticks(self.scenario), [0, 4, 8, 10])
        validate_scenario(self.scenario)  # Input action at unsampled tick 1 remains valid.
        self.scenario['capture_stride'] = 5
        self.assertEqual(sampled_ticks(self.scenario), [0, 5, 10])

    def test_sampling_rejects_invalid_strides(self):
        for stride in [True, False, 0, -1, 11, 1.5, 1.0, None, '2']:
            with self.subTest(stride=stride):
                self.scenario['capture_stride'] = stride
                with self.assertRaisesRegex(ValueError, 'capture_stride'):
                    validate_scenario(self.scenario)

    def test_sparse_capture_does_not_relax_action_order_or_tick_limit(self):
        self.scenario['capture_stride'] = 5
        self.scenario['actions'] *= 2
        self.scenario['actions'][0] = dict(self.scenario['actions'][0], tick=6)
        with self.assertRaisesRegex(ValueError, 'Actions must be ordered'):
            validate_scenario(self.scenario)
        self.scenario['actions'] = []
        self.scenario['capture_ticks'] = 5001
        with self.assertRaisesRegex(ValueError, 'simulated-tick'):
            validate_scenario(self.scenario)

    def test_budget_counts_captured_cells_without_loosening_cell_limit(self):
        self.scenario['watch'] = {'min': [0, 0, 0], 'max': [199, 9, 99]}
        self.scenario['capture_ticks'] = 2000
        with self.assertRaisesRegex(ValueError, 'cell/snapshot'):
            validate_scenario(self.scenario)
        self.scenario['capture_stride'] = 20
        self.assertEqual(validate_scenario(self.scenario), 200000)
        self.scenario['watch']['max'][0] = 299
        with self.assertRaisesRegex(ValueError, 'cell/snapshot'):
            validate_scenario(self.scenario)

    def capture(self, stride=None):
        scenario = copy.deepcopy(self.scenario)
        if stride is not None:
            scenario['capture_stride'] = stride
        ticks = sampled_ticks(scenario)
        row = [0, 0, 0, scenario['fixture'][0]['state']]
        frames = [{'tick': t, 'players': 0, 'blocks': [row],
                   'inventories': [], 'comparator_outputs': []} for t in ticks]
        capture = dict(scenario, mc_version='1.21.11', mappings_sha1=MAPPINGS_SHA1,
                       server_inner_sha256=INNER_SHA256, scenario_sha256='b' * 64,
                       settle='in_world', initial=frames[0], snapshots=frames[1:],
                       warmup_ticks=2, java_runtime_version='21.0.13',
                       scope='sampled tick-end blocks' if stride and stride > 1 else 'tick-end blocks')
        if stride is not None:
            capture['sampled_ticks'] = ticks
        if stride is not None and stride > 1:
            capture['final_activity'] = {'tick': ticks[-1], 'block_ticks': 0, 'fluid_ticks': 0,
                'block_events': 0, 'rescheduled_block_events': 0, 'moving_pistons_in_watch': 0,
                'scope': 'Server-level queues; watched moving pistons; no runtime-entity census.'}
        return scenario, capture

    def validate_output(self, scenario, capture):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / 'capture.json'
            path.write_text(json.dumps(capture))
            return validate_capture(path, scenario, 'b' * 64)

    def test_default_output_schema_and_sequence_are_unchanged(self):
        scenario, capture = self.capture()
        result = self.validate_output(scenario, capture)
        self.assertNotIn('capture_stride', result)
        self.assertNotIn('sampled_ticks', result)
        capture['sampled_ticks'] = list(range(11))
        with self.assertRaisesRegex(ValueError, 'undeclared sampling'):
            self.validate_output(scenario, capture)

    def test_sparse_output_declares_exact_ticks_and_limits(self):
        scenario, capture = self.capture(4)
        result = self.validate_output(scenario, capture)
        self.assertEqual(result['sampled_ticks'], [0, 4, 8, 10])
        self.assertFalse(result['every_tick_captured'])
        capture['scope'] = 'tick-end blocks'
        with self.assertRaisesRegex(ValueError, 'sampled tick-end scope'):
            self.validate_output(scenario, capture)

    def test_sparse_output_rejects_missing_final_reordering_and_wrong_declaration(self):
        for mutation in ['missing_final', 'reordered', 'wrong_list', 'wrong_stride']:
            with self.subTest(mutation=mutation):
                scenario, capture = self.capture(4)
                if mutation == 'missing_final':
                    capture['snapshots'].pop()
                elif mutation == 'reordered':
                    capture['snapshots'].reverse()
                elif mutation == 'wrong_list':
                    capture['sampled_ticks'] = [0, 4, 8]
                else:
                    capture['capture_stride'] = 2
                with self.assertRaises(ValueError):
                    self.validate_output(scenario, capture)

    def test_sparse_output_still_checks_source_and_initial_blocks(self):
        for mutation in ['source', 'initial']:
            with self.subTest(mutation=mutation):
                scenario, capture = self.capture(4)
                if mutation == 'source':
                    capture['source'] = dict(capture['source'], sha256='c' * 64)
                else:
                    capture['initial']['blocks'] = []
                with self.assertRaises(ValueError):
                    self.validate_output(scenario, capture)

    def test_final_activity_requires_exact_counters_and_matches_final_frame(self):
        for value in [True, -1, 1.5, None]:
            with self.subTest(value=value):
                scenario, capture = self.capture(4)
                capture['final_activity']['block_ticks'] = value
                with self.assertRaisesRegex(ValueError, 'activity counter'):
                    self.validate_output(scenario, capture)
        scenario, capture = self.capture(4)
        capture['final_activity']['moving_pistons_in_watch'] = 1
        with self.assertRaisesRegex(ValueError, 'moving-piston count'):
            self.validate_output(scenario, capture)


if __name__ == '__main__':
    unittest.main()
