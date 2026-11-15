//! Reusable addresses into the current World geometry, never cached states.
//!
//! Eight bytes per retained input replace repeated xyz arithmetic. A geometry
//! token belongs to one World allocation/layout. Clone, replacement and growth
//! obtain fresh tokens; ordinary writes leave addresses valid and reads live.

use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_TOKEN: AtomicU32 = AtomicU32::new(1);

fn allocate_token(next: &AtomicU32) -> u32 {
    // Never wrap and accidentally bless an old address. Exhaustion permanently
    // falls back to position lookup for newly created layouts.
    next.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |token| token.checked_add(1))
        .unwrap_or(0)
}

pub(crate) struct Geometry(u32);

impl Default for Geometry {
    fn default() -> Self { Self(allocate_token(&NEXT_TOKEN)) }
}

impl Clone for Geometry {
    fn clone(&self) -> Self { Self::default() }
}

impl PartialEq for Geometry {
    fn eq(&self, _: &Self) -> bool { true }
}
impl Eq for Geometry {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Address(u64);

impl Geometry {
    pub(crate) fn address(&self, index: Option<usize>) -> Address {
        let Some(index) = index.and_then(|index| u32::try_from(index).ok()) else {
            return Address::default();
        };
        if self.0 == 0 { return Address::default(); }
        Address((u64::from(self.0) << 32) | u64::from(index))
    }

    #[inline]
    pub(crate) fn index(&self, address: Address) -> Option<usize> {
        if self.0 != 0 && (address.0 >> 32) as u32 == self.0 {
            Some(address.0 as u32 as usize)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_do_not_alias_on_clone_or_rollover() {
        let geometry = Geometry::default();
        let address = geometry.address(Some(7));
        assert_eq!(geometry.index(address), Some(7));
        assert_eq!(geometry.clone().index(address), None);
        assert_eq!(geometry.index(Address::default()), None);
        assert_eq!(Geometry(0).index(Address::default()), None);
        assert_eq!(Geometry(0).address(Some(7)), Address::default());
        let next = AtomicU32::new(u32::MAX - 1);
        assert_eq!(allocate_token(&next), u32::MAX - 1);
        assert_eq!(allocate_token(&next), 0);
        assert_eq!(allocate_token(&next), 0);
        assert_eq!(next.load(Ordering::Relaxed), u32::MAX);
        assert_eq!(std::mem::size_of::<Address>(), 8);
    }
}
