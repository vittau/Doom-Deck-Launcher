//! `ddl` — DoomDeckLauncher, a native DOOM mapset launcher for Steam Deck.
//!
//! Spec 001 only needs a runnable shell: argument handling, structured logging
//! and a fullscreen placeholder menu. Gamepad input, DSDA-Doom handling and the
//! real screens arrive with later specs.

mod cli;
#[cfg(test)]
mod headless;
mod logging;

use std::process::ExitCode;

use tracing::{debug, error, info, warn};

// Generated from `ui/app-window.slint` by `build.rs`.
slint::include_modules!();

/// Exit status used for command-line errors, matching the `sysexits`-style
/// convention of "usage" style failures.
const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    match cli::parse(std::env::args_os().skip(1)) {
        Ok(cli::Command::Help) => {
            print!("{}", cli::help_text());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::Version) => {
            println!("{}", cli::version_line());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::CatalogLint { paths, fetch }) => {
            let cache = ddl_core::platform::paths::resolve(&|key: &str| std::env::var(key).ok())
                .map(|paths| paths.cache)
                .unwrap_or_else(|_| std::env::temp_dir().join("ddl-catalog-fetch"));
            let mut network = ddl_core::catalog::HttpsFetch::new(cache);
            let errors = ddl_core::catalog::lint_paths(
                &paths,
                std::path::Path::new("catalog/hosts.toml"),
                if fetch { Some(&mut network) } else { None },
            );
            if errors.is_empty() {
                println!(
                    "Catalog lint passed{}.",
                    if fetch {
                        " (fetched packages verified)"
                    } else {
                        " (offline)"
                    }
                );
                ExitCode::SUCCESS
            } else {
                for error in errors {
                    eprintln!("{error}");
                }
                ExitCode::FAILURE
            }
        }
        Ok(cli::Command::Run(options)) => {
            logging::init();
            info!(
                version = cli::VERSION,
                windowed = options.windowed,
                "starting DoomDeckLauncher"
            );
            match run(options) {
                Ok(()) => {
                    info!("DoomDeckLauncher stopped");
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    error!(error = %err, "DoomDeckLauncher stopped with an error");
                    ExitCode::FAILURE
                }
            }
        }
        Err(err) => {
            eprintln!("{}: {err}", cli::BIN);
            eprintln!("Try '{} --help' for a list of options.", cli::BIN);
            ExitCode::from(EXIT_USAGE)
        }
    }
}

/// Builds and runs the placeholder window until the user quits.
fn run(options: cli::Options) -> Result<(), Box<dyn std::error::Error>> {
    log_directories();

    let ui = AppWindow::new()?;
    ui.set_selected(0);

    // "Play" and "Settings" are placeholders; spec 001 (R4) only requires the
    // focus movement to be visible, so they log and do nothing.
    ui.on_menu_activated(|index| info!(index, "menu item activated"));

    // Both the "Quit" entry and Escape end up here and must exit with status 0.
    ui.on_quit_requested(|| {
        info!("quit requested, closing the window");
        if let Err(err) = slint::quit_event_loop() {
            warn!(error = %err, "could not stop the Slint event loop");
        }
    });

    let window = slint::ComponentHandle::window(&ui);
    if options.windowed {
        window.set_fullscreen(false);
        window.set_size(slint::LogicalSize::new(1280.0, 800.0));
        debug!("using a 1280x800 window instead of fullscreen");
    } else {
        window.set_fullscreen(true);
        debug!("using the full screen");
    }

    ui.run()?;
    Ok(())
}

/// Resolves and logs DDL's XDG directories.
///
/// A missing `HOME` is reported as a warning rather than aborting: the
/// placeholder screen needs no directories, and spec 001 (R2) only requires
/// resolution to fail cleanly.
fn log_directories() {
    use ddl_core::platform::paths;

    match paths::resolve(&|key: &str| std::env::var(key).ok()) {
        Ok(dirs) => debug!(
            config = %dirs.config.display(),
            data = %dirs.data.display(),
            cache = %dirs.cache.display(),
            state = %dirs.state.display(),
            "resolved DDL directories"
        ),
        Err(err) => warn!(error = %err, "could not resolve DDL directories"),
    }
}
