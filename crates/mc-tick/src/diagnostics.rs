//! Diagnostic options are sampled once per process, on first use.
use std::ffi::{OsStr, OsString};
use std::sync::OnceLock;

macro_rules! diag_flag {
    ($fn_name:ident, $env_name:literal) => {
        #[inline]
        pub(crate) fn $fn_name() -> Option<&'static OsStr> {
            static SLOT: OnceLock<Option<OsString>> = OnceLock::new();
            SLOT.get_or_init(|| std::env::var_os($env_name)).as_deref()
        }
    };
}

diag_flag!(trace_events, "MC_TICK_TRACE_EVENTS");
diag_flag!(trace_line, "MC_TICK_TRACE_LINE");
diag_flag!(trace_notify, "MC_TICK_TRACE_NOTIFY");
diag_flag!(trace_power, "MC_TICK_TRACE_POWER");
diag_flag!(trace_reach, "MC_TICK_TRACE_REACH");
diag_flag!(trace_retract, "MC_TICK_TRACE_RETRACT");
diag_flag!(trace_sched, "MC_TICK_TRACE_SCHED");
diag_flag!(trace_schedule, "MC_TICK_TRACE_SCHEDULE");
diag_flag!(trace_shove, "MC_TICK_TRACE_SHOVE");
diag_flag!(trace_side, "MC_TICK_TRACE_SIDE");
diag_flag!(trace_updates, "MC_TICK_TRACE_UPDATES");
diag_flag!(trace_write, "MC_TICK_TRACE_WRITE");
diag_flag!(debug_comparator, "MC_TICK_DEBUG_COMPARATOR");

#[inline]
pub(crate) fn trace_notify_enabled() -> bool {
    trace_notify().is_some()
}

#[inline]
pub(crate) fn trace_side_enabled() -> bool {
    trace_side().is_some()
}

#[inline]
pub(crate) fn var_os(name: &str) -> Option<&'static OsStr> {
    match name {
        "MC_TICK_TRACE_EVENTS" => trace_events(),
        "MC_TICK_TRACE_LINE" => trace_line(),
        "MC_TICK_TRACE_NOTIFY" => trace_notify(),
        "MC_TICK_TRACE_POWER" => trace_power(),
        "MC_TICK_TRACE_REACH" => trace_reach(),
        "MC_TICK_TRACE_RETRACT" => trace_retract(),
        "MC_TICK_TRACE_SCHED" => trace_sched(),
        "MC_TICK_TRACE_SCHEDULE" => trace_schedule(),
        "MC_TICK_TRACE_SHOVE" => trace_shove(),
        "MC_TICK_TRACE_SIDE" => trace_side(),
        "MC_TICK_TRACE_UPDATES" => trace_updates(),
        "MC_TICK_TRACE_WRITE" => trace_write(),
        "MC_TICK_DEBUG_COMPARATOR" => debug_comparator(),
        _ => unreachable!("unregistered diagnostic option"),
    }
}

#[inline]
pub(crate) fn var(name: &str) -> Result<&'static str, ()> {
    var_os(name).and_then(OsStr::to_str).ok_or(())
}
