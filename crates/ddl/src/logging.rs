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

    // Initialize before any window/event loop exists. Worker diagnostics and engine logs
    // then share the location exposed in Settings.
    let file = ddl_core::platform::paths::resolve(&|key: &str| std::env::var(key).ok())
        .ok()
        .and_then(|paths| {
            let dir = paths.state.join("logs");
            std::fs::create_dir_all(&dir).ok()?;
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("ddl.log"))
                .ok()
        })
        .map(|file| std::sync::Arc::new(std::sync::Mutex::new(file)));
    let _ = tracing_subscriber::fmt()
        .with_writer(move || LogWriter { file: file.clone() })
        .with_max_level(level)
        // Plain text: logs are read from a file or a Steam launch log, where
        // escape codes are noise.
        .with_ansi(false)
        .try_init();
}

struct LogWriter {
    file: Option<std::sync::Arc<std::sync::Mutex<std::fs::File>>>,
}
impl std::io::Write for LogWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        std::io::stderr().write_all(bytes)?;
        if let Some(file) = &self.file
            && let Ok(mut file) = file.lock()
        {
            file.write_all(bytes)?;
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stderr().flush()
    }
}
