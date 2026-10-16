//! Test helper for the launch-plan runner tests (spec 003, A3).
//!
//! The runner tests must not run DSDA-Doom, and `AGENTS.md` forbids proving
//! process behaviour with a shell string, so they start this tiny program
//! instead. It is a separate binary target of `ddl-core`, built by
//! `cargo test`, and never shipped: only `crates/ddl` is released.
//!
//! Behaviour is chosen with the `DDL_LAUNCH_PROBE_MODE` environment variable,
//! which lets a test drive every runner outcome while the argv stays exactly
//! the one `LaunchPlan::to_command` produced:
//!
//! | Mode | Behaviour |
//! | --- | --- |
//! | unset or `ok` | print argv to stdout, diagnostics to stderr, exit 0 |
//! | `exit:<n>` | the same output, then exit with status `<n>` |
//! | `signal` | the same output, then abort (death by `SIGABRT`) |
//! | `dump-env` | print the four scrubbed `DOOM*` variables, `DDL_PROBE_KEEP` and the working directory |

use std::process::ExitCode;

/// The variables the real runner must remove before starting the engine.
const SCRUBBED_ENV: [&str; 4] = ["DOOMWADDIR", "DOOMWADPATH", "DOOMDATADIR", "DOOMSAVEDIR"];

/// A variable the runner must leave alone, so a scrubbing test can tell
/// "removed" apart from "the whole environment was dropped".
const KEPT_ENV: &str = "DDL_PROBE_KEEP";

fn main() -> ExitCode {
    let mode = std::env::var("DDL_LAUNCH_PROBE_MODE").unwrap_or_else(|_| String::from("ok"));

    if let Some(code) = mode.strip_prefix("exit:") {
        let Ok(code) = code.parse::<u8>() else {
            eprintln!("ddl-launch-probe: `exit:` needs an 8-bit status, got {code:?}");
            return ExitCode::from(2);
        };
        report();
        return ExitCode::from(code);
    }

    match mode.as_str() {
        "ok" | "" => {
            report();
            ExitCode::SUCCESS
        }
        "signal" => {
            report();
            // SIGABRT, so the parent observes an exit *without* a status code.
            std::process::abort();
        }
        "dump-env" => {
            for key in SCRUBBED_ENV {
                println!(
                    "{key}={}",
                    std::env::var(key).unwrap_or_else(|_| String::from("unset"))
                );
            }
            println!(
                "{KEPT_ENV}={}",
                std::env::var(KEPT_ENV).unwrap_or_else(|_| String::from("unset"))
            );
            let cwd = std::env::current_dir().unwrap_or_default();
            println!("cwd={}", cwd.display());
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("ddl-launch-probe: unknown DDL_LAUNCH_PROBE_MODE {other:?}");
            ExitCode::from(2)
        }
    }
}

/// Writes the marker lines the runner tests assert on.
///
/// The argv is printed with `{:?}` so the log shows argument boundaries: a test
/// can prove that, for example, an IWAD path stayed a single token.
fn report() {
    println!("probe ok");
    println!("probe argv={:?}", std::env::args().collect::<Vec<_>>());
    eprintln!("probe diagnostics");
}
