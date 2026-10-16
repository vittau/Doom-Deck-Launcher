//! Structured diagnostics for `ddl` (spec 001, R5).
//!
//! All logs go to **stderr** (stdout stays clear for `--version`/`--help` and,
//! later, machine-readable output). The level comes from the `DDL_LOG`
//! environment variable and defaults to `info`.

use std::io::Write as _;
use std::str::FromStr as _;

use tracing_subscriber::filter::LevelFilter;

/// Environment variable that selects the log level.
pub const LOG_ENV: &str = "DDL_LOG";

/// Installs the global `tracing` subscriber.
///
/// Never panics: a subscriber that is already installed (for example in tests)
/// is left alone, and an unparsable `DDL_LOG` value falls back to `info` after
/// a one-line warning on stderr.
pub fn init() {
    let level = match std::env::var(LOG_ENV) {
        Ok(value) => match LevelFilter::from_str(value.trim()) {
            Ok(level) => level,
            Err(_) => {
                // Logging is not up yet, so report the bad value directly.
                let _ = writeln!(
                    std::io::stderr(),
                    "{LOG_ENV}: ignoring unknown log level `{value}`, using `info`"
                );
                LevelFilter::INFO
            }
        },
        Err(_) => LevelFilter::INFO,
    };

    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(level)
        // Plain text: logs are read from a file or a Steam launch log, where
        // escape codes are noise.
        .with_ansi(false)
        .try_init();
}
