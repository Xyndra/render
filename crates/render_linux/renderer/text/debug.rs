// WARNING: AI GENERATED; UNDER REVIEW

/// Compile-time gate for verbose text-rendering debug logging.
/// When `false`, every `debug!` call compiles away to nothing.
pub(crate) const TEXT_DEBUG: bool = false;

/// Debug logging gated by [`TEXT_DEBUG`]. Compiles to a no-op when disabled.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::renderer::text::debug::TEXT_DEBUG {
            eprintln!($($arg)*);
        }
    };
}
