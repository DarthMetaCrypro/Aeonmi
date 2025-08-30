//! Lightweight debug logging gated by AEONMI_DEBUG=1.
use std::sync::OnceLock;

static ENABLED: OnceLock<bool> = OnceLock::new();

pub fn is_enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var("AEONMI_DEBUG").ok().as_deref() == Some("1"))
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {{
        if $crate::core::debug::is_enabled() { eprintln!($($arg)*); }
    }};
}
