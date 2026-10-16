//! `ddl` — DoomDeckLauncher, a native DOOM mapset launcher for Steam Deck.
//!
//! Controller-first UI backed by core services (spec 010).

mod cli;
mod copy;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod headless;
mod input;
mod logging;
mod runtime;
mod services;
#[cfg(test)]
mod shell;
mod wiring_copy;

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

/// Builds and runs the production shell until the user quits.
fn run(options: cli::Options) -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    let state = runtime::attach(&ui)?;
    // Only the confirmed Quit action reaches this callback.
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

    use slint::ComponentHandle;
    let weak = ui.as_weak();
    let timer = slint::Timer::default();
    let mut gamepads = input::Gamepads::new();
    timer.start(slint::TimerMode::Repeated, input::CONFIG.poll, move || {
        if let Some(ui) = weak.upgrade() {
            let (actions, connections) =
                gamepads.poll(ui.get_input_active() && !state.borrow().playing);
            state.borrow_mut().neutral_poll(actions.is_empty());
            for connected in connections {
                state.borrow().hotplug(&ui, connected);
            }
            for action in actions {
                state.borrow_mut().action(&ui, action);
            }
        }
    });
    ui.run()?;
    Ok(())
}
