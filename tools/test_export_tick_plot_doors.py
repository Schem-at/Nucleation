"""Evidence thresholds: motion/replay must not become a door/parity certificate."""
import copy
from pathlib import Path
import unittest
from export_tick_plot_doors import alternating, inside, negative_signs, usable_trials, verified


class EvidenceTests(unittest.TestCase):
    def trial(self):
        return {'control':[1,2,3], 'deterministic_after_restore_and_reseed':True,
                'first':{'status':'completed','completed_ticks':320,'requested_ticks':320,
                         'piston_related_changes':20,'changes_away_from_control':24,
                         'actions':[{'status':'used'}],
                         'endpoints':[{'observable_state_fingerprint':x,'moving_blocks':0} for x in ['a','b','a','b','a']]}}

    def certificate(self):
        comparison={'status':'matches_capture','expected_frames':661,'matching_frames_including_initial':661,
                    'first_difference':None,'compared_capture_state_blake3':'same','compared_engine_state_blake3':'same'}
        return {'validation_status':'vanilla_tick_end_parity_pass','functional_status':'vanilla_functional_pass',
                'benchmark_status':'admitted','plot_validation':{'oracle':{'source_sha256':'source','promoted':True,
                 'functional_status':'vanilla_functional_pass','functional_failures':[],
                 'native':{'cycle':dict(comparison,actions=4),'idle':dict(comparison,actions=0)}}}}

    def test_motion_and_replay_are_not_a_verified_door(self):
        self.assertTrue(alternating(self.trial()))
        self.assertFalse(verified({'functional_status':'unproven','validation_status':'simulation_smoke_checked'},'source'))

    def test_static_or_moving_endpoints_are_not_alternating_rest_states(self):
        trial=self.trial()
        for endpoint in trial['first']['endpoints']:endpoint['observable_state_fingerprint']='a'
        self.assertFalse(alternating(trial))
        trial=self.trial();trial['first']['endpoints'][1]['moving_blocks']=1
        self.assertFalse(alternating(trial))

    def test_exact_state_must_repeat_in_both_phases(self):
        trial=self.trial();trial['first']['endpoints'][3]['observable_state_fingerprint']='inventory_changed'
        self.assertFalse(alternating(trial))

    def test_missing_control_or_no_piston_motion_is_not_useful(self):
        for change in ['missing_control','no_motion']:
            trial=self.trial()
            if change=='missing_control':trial['first']['actions'][0]['status']='control_missing_or_replaced'
            else:trial['first']['piston_related_changes']=0
            raw={'status':'assessed','native':{'assessment':{'status':'assessed','trials':[trial]}}}
            self.assertEqual(usable_trials(raw),[])

    def test_failed_restore_replay_rejects_assessment(self):
        trial=self.trial();trial['deterministic_after_restore_and_reseed']=False
        with self.assertRaises(ValueError):usable_trials({'status':'assessed','native':{'assessment':{'status':'assessed','trials':[trial]}}})

    def test_verified_requires_same_source_full_capture_and_function(self):
        status=self.certificate();self.assertTrue(verified(status,'source'))
        self.assertFalse(verified(status,'other-source'))
        for field,value in [('actions',0),('matching_frames_including_initial',660),('compared_engine_state_blake3','different')]:
            bad=copy.deepcopy(status);bad['plot_validation']['oracle']['native']['cycle'][field]=value
            self.assertFalse(verified(bad,'source'))
        status['plot_validation']['oracle']['functional_failures']=['closed center is air']
        self.assertFalse(verified(status,'source'))

    def test_broken_source_label_is_retained_for_exclusion(self):
        row={'signs':[{'text':{'front_text':['idk what i did','wrong, but its','broken now']}}]}
        self.assertEqual(negative_signs(row),row['signs'])

    def test_archive_escape_is_rejected(self):
        with self.assertRaises(ValueError):inside(Path('/tmp/archive'), '../secret')

if __name__=='__main__':unittest.main()
