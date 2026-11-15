//! Diagnostic options are sampled once per process, on first use.
use std::ffi::{OsStr, OsString};
use std::sync::OnceLock;

pub(crate) fn var_os(name: &str) -> Option<&'static OsStr> {
    static MC_TICK_TRACE_EVENTS: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_LINE: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_NOTIFY: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_POWER: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_REACH: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_RETRACT: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_SCHED: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_SCHEDULE: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_SHOVE: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_SIDE: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_UPDATES: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_TRACE_WRITE: OnceLock<Option<OsString>> = OnceLock::new();
    static MC_TICK_DEBUG_COMPARATOR: OnceLock<Option<OsString>> = OnceLock::new();
    let slot = match name {
        "MC_TICK_TRACE_EVENTS" => &MC_TICK_TRACE_EVENTS,
        "MC_TICK_TRACE_LINE" => &MC_TICK_TRACE_LINE,
        "MC_TICK_TRACE_NOTIFY" => &MC_TICK_TRACE_NOTIFY,
        "MC_TICK_TRACE_POWER" => &MC_TICK_TRACE_POWER,
        "MC_TICK_TRACE_REACH" => &MC_TICK_TRACE_REACH,
        "MC_TICK_TRACE_RETRACT" => &MC_TICK_TRACE_RETRACT,
        "MC_TICK_TRACE_SCHED" => &MC_TICK_TRACE_SCHED,
        "MC_TICK_TRACE_SCHEDULE" => &MC_TICK_TRACE_SCHEDULE,
        "MC_TICK_TRACE_SHOVE" => &MC_TICK_TRACE_SHOVE,
        "MC_TICK_TRACE_SIDE" => &MC_TICK_TRACE_SIDE,
        "MC_TICK_TRACE_UPDATES" => &MC_TICK_TRACE_UPDATES,
        "MC_TICK_TRACE_WRITE" => &MC_TICK_TRACE_WRITE,
        "MC_TICK_DEBUG_COMPARATOR" => &MC_TICK_DEBUG_COMPARATOR,
        _ => unreachable!("unregistered diagnostic option"),
    };
    slot.get_or_init(|| std::env::var_os(name)).as_deref()
}

pub(crate) fn var(name: &str) -> Result<&'static str, ()> {
    var_os(name).and_then(OsStr::to_str).ok_or(())
}
