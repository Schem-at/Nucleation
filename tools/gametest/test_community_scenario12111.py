"""Guard reference-version provenance when testing saved states across versions."""
import copy
import unittest

from run_community_capture12111 import validate_scenario


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


if __name__ == '__main__':
    unittest.main()
