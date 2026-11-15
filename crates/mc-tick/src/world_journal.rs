//! Bounded, synchronous world changes for experimental input plans.
use crate::{Pos, StateId};
use std::{collections::VecDeque, sync::Arc};
#[cfg(feature = "experiment-lazy-journal")]
use std::sync::atomic::{AtomicBool, Ordering};

const CAPACITY: usize = 512;

pub(crate) struct MutationJournal {
    identity: Arc<()>,
    revision: u64,
    changes: VecDeque<(Pos, StateId, StateId)>,
    #[cfg(feature = "experiment-lazy-journal")]
    active: AtomicBool,
}

impl Default for MutationJournal {
    fn default() -> Self {
        Self { identity: Arc::new(()), revision: 0, changes: VecDeque::new(),
            #[cfg(feature = "experiment-lazy-journal")]
            active: AtomicBool::new(false),
        }
    }
}
// A copied/restored world must never share validity with the world copied from.
impl Clone for MutationJournal { fn clone(&self) -> Self { Self::default() } }
impl PartialEq for MutationJournal { fn eq(&self, _: &Self) -> bool { true } }
impl Eq for MutationJournal {}

impl MutationJournal {
    pub(crate) fn identity(&self) -> &Arc<()> { &self.identity }
    pub(crate) fn revision(&self) -> u64 {
        // Queries borrow World immutably; a concurrent write requires an
        // exclusive borrow and cannot race activation or plan construction.
        #[cfg(feature = "experiment-lazy-journal")]
        if !self.active.load(Ordering::Relaxed) { self.active.store(true, Ordering::Relaxed); }
        self.revision
    }
    pub(crate) fn reset(&mut self) {
        self.identity = Arc::new(());
        self.revision = 0;
        self.changes.clear();
    }
    pub(crate) fn record(&mut self, pos: Pos, old: StateId, new: StateId) {
        #[cfg(feature = "experiment-lazy-journal")]
        if !*self.active.get_mut() { return; }
        if self.revision == u64::MAX { self.reset(); }
        self.revision += 1;
        if self.changes.len() == CAPACITY { self.changes.pop_front(); }
        self.changes.push_back((pos, old, new));
    }
    pub(crate) fn changes_since(&self, revision: u64)
        -> Option<impl Iterator<Item = (Pos, StateId, StateId)> + '_>
    {
        #[cfg(feature = "experiment-lazy-journal")]
        if !self.active.load(Ordering::Relaxed) { self.active.store(true, Ordering::Relaxed); }
        let count = self.revision.checked_sub(revision)?;
        if count > self.changes.len() as u64 { return None; }
        Some(self.changes.iter().skip(self.changes.len() - count as usize).copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_history_clone_reset_and_rollover_are_explicit() {
        let mut log = MutationJournal::default();
        assert_eq!(log.revision(), 0);
        let token = Arc::clone(log.identity());
        for n in 0..600 { log.record(Pos::new(n, -2, 3), StateId(1), StateId(2)); }
        assert!(log.changes_since(0).is_none());
        assert_eq!(log.changes_since(88).unwrap().count(), CAPACITY);
        assert_eq!(log.changes_since(599).unwrap().next().unwrap().0.x, 599);
        assert!(log.changes_since(601).is_none());
        assert!(!Arc::ptr_eq(log.clone().identity(), &token));
        log.revision = u64::MAX;
        log.record(Pos::new(0, 0, 0), StateId(2), StateId(3));
        assert_eq!(log.revision(), 1);
        assert!(!Arc::ptr_eq(log.identity(), &token));
        assert_eq!(log.changes_since(0).unwrap().count(), 1);
    }

    #[cfg(feature = "experiment-lazy-journal")]
    #[test]
    fn lazy_recording_activates_before_first_plan_and_stays_active_after_reset() {
        let mut log = MutationJournal::default();
        let pos = Pos::new(-3, 5, 7);
        for n in 0..20 { log.record(pos, StateId(n), StateId(n + 1)); }
        assert!(log.changes.is_empty());
        assert_eq!(log.revision(), 0);
        log.record(pos, StateId(20), StateId(21));
        assert_eq!(log.revision(), 1);
        assert_eq!(log.changes_since(0).unwrap().collect::<Vec<_>>(),
            vec![(pos, StateId(20), StateId(21))]);
        let mut copy = log.clone();
        copy.record(pos, StateId(21), StateId(22));
        assert!(copy.changes.is_empty());
        assert!(copy.changes_since(0).unwrap().next().is_none());
        copy.record(pos, StateId(22), StateId(23));
        assert_eq!(copy.revision(), 1);
        log.reset();
        log.record(pos, StateId(21), StateId(22));
        assert_eq!(log.revision(), 1);
        fn send_sync<T: Send + Sync>() {}
        send_sync::<MutationJournal>();
    }
}
