//! Bounded, synchronous world changes for experimental input plans.
use crate::{Pos, StateId};
use std::{collections::VecDeque, sync::Arc};

const CAPACITY: usize = 512;

pub(crate) struct MutationJournal {
    identity: Arc<()>,
    revision: u64,
    changes: VecDeque<(Pos, StateId, StateId)>,
}

impl Default for MutationJournal {
    fn default() -> Self {
        Self { identity: Arc::new(()), revision: 0, changes: VecDeque::new() }
    }
}
// A copied/restored world must never share validity with the world copied from.
impl Clone for MutationJournal { fn clone(&self) -> Self { Self::default() } }
impl PartialEq for MutationJournal { fn eq(&self, _: &Self) -> bool { true } }
impl Eq for MutationJournal {}

impl MutationJournal {
    pub(crate) fn identity(&self) -> &Arc<()> { &self.identity }
    pub(crate) fn revision(&self) -> u64 { self.revision }
    pub(crate) fn reset(&mut self) {
        self.identity = Arc::new(());
        self.revision = 0;
        self.changes.clear();
    }
    pub(crate) fn record(&mut self, pos: Pos, old: StateId, new: StateId) {
        if self.revision == u64::MAX { self.reset(); }
        self.revision += 1;
        if self.changes.len() == CAPACITY { self.changes.pop_front(); }
        self.changes.push_back((pos, old, new));
    }
    pub(crate) fn changes_since(&self, revision: u64)
        -> Option<impl Iterator<Item = (Pos, StateId, StateId)> + '_>
    {
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
}
