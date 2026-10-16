//! Headless subcommand execution (spec 014, R1/R2).
//!
//! Every subcommand drives the same `ddl-core` services the UI uses through
//! [`crate::services::Services`], so the CLI and the UI share one XDG state.
//! Nothing here creates a window.
//!
//! Output is written to an explicit writer so the subcommands can be exercised
//! against the test fakes without capturing a process's stdout.

use std::io::Write;
use std::path::PathBuf;

use ddl_core::downloads::{InstallProgress, InstallStage};
use ddl_core::engines::dsda;

use crate::cli;
use crate::services::{CliFailure, DoctorReport, EngineStatus, Environment, LaunchOverrides, Services};

/// Runs a parsed subcommand against the real user environment.
///
/// # Errors
///
/// Returns a [`CliFailure`] that the binary prints as `ddl: <message>`.
pub fn run(command: cli::Command) -> Result<(), CliFailure> {
    let env = environment()?;
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    run_io(command, env, &mut out, &mut err)
}

/// Builds DDL's environment from the real process environment.
fn environment() -> Result<Environment, CliFailure> {
    let paths = ddl_core::platform::paths::resolve(&|key: &str| std::env::var(key).ok())
        .map_err(|error| CliFailure::message(error.to_string()))?;
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| CliFailure::message("HOME is not set; DDL cannot find your folders"))?;
    Ok(Environment {
        paths,
        home,
        #[cfg(test)]
        test: None,
    })
}

/// Runs a parsed subcommand against `env`, writing normal output to `out` and
/// progress/diagnostics to `err`.
///
/// # Errors
///
/// See [`run`].
pub fn run_io(
    command: cli::Command,
    env: Environment,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<(), CliFailure> {
    match command {
        cli::Command::EngineStatus => {
            let services = open(env)?;
            print_engine_status(&services.engine_status()?, out)
        }
        cli::Command::EngineInstall {
            version,
            force_extract,
        } => {
            let services = open(env)?;
            let mut progress = ProgressPrinter::default();
            let engine = services.install_engine(version.as_deref(), force_extract, &mut |update| {
                progress.write(update, err)
            })?;
            writeln!(
                out,
                "Installed DSDA-Doom {} at {} ({})",
                engine.version,
                engine.path.display(),
                mode_label(engine.mode)
            )
            .map_err(output_error)
        }
        cli::Command::IwadScan { paths } => {
            let mut services = open(env)?;
            let report = services.scan_iwads(&paths)?;
            print_iwad_scan(&report.entries, &report.unrecognized, out)
        }
        cli::Command::MapsetList => {
            let services = open(env)?;
            print_mapset_list(&services.mapset_list()?, out)
        }
        cli::Command::MapsetInstall { id, components } => {
            let mut services = open(env)?;
            let mut progress = ProgressPrinter::default();
            let manifest = services.install_mapset(&id, &components, &mut |update| {
                progress.write(update, err)
            })?;
            writeln!(out, "Installed {} {}", manifest.recipe_id, manifest.release)
                .map_err(output_error)?;
            for member in &manifest.members {
                writeln!(out, "  {}", member.path.display()).map_err(output_error)?;
            }
            Ok(())
        }
        cli::Command::Launch {
            id,
            dry_run,
            components,
            skill,
            pistol_start,
        } => {
            let mut services = open(env)?;
            apply_iwad_seam(&mut services)?;
            let overrides = LaunchOverrides {
                components,
                skill,
                pistol_start,
            };
            if dry_run {
                let resolution = services.resolve_mapset(&id, &overrides)?;
                print_plan(&resolution, out)
            } else {
                let outcome = services.run_mapset(&id, &overrides)?;
                writeln!(err, "DSDA-Doom exited: {}", exit_label(outcome)).map_err(output_error)
            }
        }
        cli::Command::Doctor => {
            let mut services = open(env)?;
            apply_iwad_seam(&mut services)?;
            let report = services.doctor()?;
            print_doctor(&report, out)
        }
        cli::Command::Run(_) | cli::Command::Help(_) | cli::Command::Version => {
            Ok(())
        }
        cli::Command::CatalogLint { .. } => Ok(()),
    }
}

fn open(env: Environment) -> Result<Services, CliFailure> {
    Services::open(env).map_err(|problem| CliFailure::message(problem.to_string()))
}

/// Test-only IWAD seam (spec 014, R3): when the `test-support` feature is built
/// and `DDL_TEST_IWAD_SEAM` names a directory, register a fake IWAD for every
/// variant the catalog needs before resolving.
fn apply_iwad_seam(services: &mut Services) -> Result<(), CliFailure> {
    #[cfg(feature = "test-support")]
    {
        if let Some(dir) = std::env::var_os("DDL_TEST_IWAD_SEAM") {
            services
                .register_fake_iwads(std::path::Path::new(&dir))
                .map_err(|problem| CliFailure::message(problem.to_string()))?;
        }
    }
    #[cfg(not(feature = "test-support"))]
    let _ = services;
    Ok(())
}

fn output_error(error: std::io::Error) -> CliFailure {
    CliFailure::message(format!("could not write output: {error}"))
}

/// The lowercase mode label for `ddl engine status`.
fn mode_label(mode: dsda::Mode) -> &'static str {
    match mode {
        dsda::Mode::Appimage => "appimage",
        dsda::Mode::Extracted => "extracted",
        dsda::Mode::Manual => "manual",
    }
}

/// The lowercase label for a recipe's base game.
fn game_label(game: ddl_core::catalog::Game) -> &'static str {
    match game {
        ddl_core::catalog::Game::Doom => "doom",
        ddl_core::catalog::Game::Doom2 => "doom2",
        ddl_core::catalog::Game::Tnt => "tnt",
        ddl_core::catalog::Game::Plutonia => "plutonia",
    }
}

fn print_engine_status(status: &EngineStatus, out: &mut dyn Write) -> Result<(), CliFailure> {
    if status.installed.is_empty() {
        return writeln!(out, "engine: not installed").map_err(output_error);
    }
    writeln!(out, "installed:").map_err(output_error)?;
    for engine in &status.installed {
        let active = if status.active.as_deref() == Some(engine.version.as_str()) {
            "  active"
        } else if status.previous.as_deref() == Some(engine.version.as_str()) {
            "  previous"
        } else {
            ""
        };
        writeln!(
            out,
            "  {}  {}  {}{}",
            engine.version,
            mode_label(engine.mode),
            if engine.reviewed {
                "reviewed"
            } else {
                "unreviewed"
            },
            active
        )
        .map_err(output_error)?;
    }
    match &status.active {
        Some(active) => writeln!(out, "active: {active}").map_err(output_error),
        None => writeln!(out, "active: none").map_err(output_error),
    }
}

fn print_iwad_scan(
    entries: &[ddl_core::iwads::RegistryEntry],
    unrecognized: &[PathBuf],
    out: &mut dyn Write,
) -> Result<(), CliFailure> {
    if entries.is_empty() && unrecognized.is_empty() {
        return writeln!(out, "No IWAD files found.").map_err(output_error);
    }
    for entry in entries {
        let identification = match entry.variant.as_deref() {
            Some(variant) => format!(
                "{variant}, {}",
                entry.game.map_or_else(|| "-".to_owned(), |game| game.to_string())
            ),
            None if !entry.candidates.is_empty() => {
                format!("ambiguous: {}", entry.candidates.join(", "))
            }
            None => "unresolved".to_owned(),
        };
        writeln!(
            out,
            "{} ({}) sha256={}",
            entry.path.display(),
            identification,
            entry.sha256
        )
        .map_err(output_error)?;
    }
    for path in unrecognized {
        writeln!(out, "unrecognized: {}", path.display()).map_err(output_error)?;
    }
    Ok(())
}

fn print_mapset_list(
    rows: &[crate::services::MapsetRow],
    out: &mut dyn Write,
) -> Result<(), CliFailure> {
    for row in rows {
        writeln!(
            out,
            "{:<18} {:<8} {:<14} {}",
            row.id,
            game_label(row.game),
            if row.installed {
                "installed"
            } else {
                "not installed"
            },
            row.title
        )
        .map_err(output_error)?;
    }
    Ok(())
}

fn print_plan(
    resolution: &ddl_core::resolver::Resolution,
    out: &mut dyn Write,
) -> Result<(), CliFailure> {
    writeln!(out, "program: {}", resolution.plan().executable.display()).map_err(output_error)?;
    for token in resolution.plan().to_command() {
        writeln!(out, "{}", token.to_string_lossy()).map_err(output_error)?;
    }
    writeln!(
        out,
        "working directory: {}",
        resolution.plan().working_dir.display()
    )
    .map_err(output_error)
}

fn print_doctor(report: &DoctorReport, out: &mut dyn Write) -> Result<(), CliFailure> {
    writeln!(out, "DoomDeckLauncher {} doctor", cli::VERSION).map_err(output_error)?;
    writeln!(out, "paths:").map_err(output_error)?;
    writeln!(out, "  config: {}", report.paths.config.display()).map_err(output_error)?;
    writeln!(out, "  data:   {}", report.paths.data.display()).map_err(output_error)?;
    writeln!(out, "  cache:  {}", report.paths.cache.display()).map_err(output_error)?;
    writeln!(out, "  state:  {}", report.paths.state.display()).map_err(output_error)?;
    writeln!(out, "  logs:   {}", report.logs.display()).map_err(output_error)?;

    writeln!(out, "engine:").map_err(output_error)?;
    writeln!(out, "  state file: {}", report.engine_state.display()).map_err(output_error)?;
    if report.engine.installed.is_empty() {
        writeln!(out, "  installed: none").map_err(output_error)?;
    } else {
        for engine in &report.engine.installed {
            writeln!(out, "  {}  {}  {}", engine.version, mode_label(engine.mode), engine.path.display())
                .map_err(output_error)?;
        }
    }
    writeln!(
        out,
        "  active: {}",
        report.engine.active.as_deref().unwrap_or("none")
    )
    .map_err(output_error)?;
    writeln!(
        out,
        "  previous good: {}",
        report.engine.previous.as_deref().unwrap_or("none")
    )
    .map_err(output_error)?;

    writeln!(out, "iwads:").map_err(output_error)?;
    if report.iwads.is_empty() {
        writeln!(out, "  none identified").map_err(output_error)?;
    } else {
        for entry in &report.iwads {
            writeln!(
                out,
                "  {} ({})",
                entry.path.display(),
                entry.variant.as_deref().unwrap_or("ambiguous")
            )
            .map_err(output_error)?;
        }
    }

    writeln!(out, "mapsets:").map_err(output_error)?;
    for row in &report.mapsets {
        writeln!(
            out,
            "  {:<18} {:<8} {}",
            row.id,
            game_label(row.game),
            if row.installed {
                "installed"
            } else {
                "not installed"
            }
        )
        .map_err(output_error)?;
    }

    writeln!(out, "dsda config:").map_err(output_error)?;
    writeln!(out, "  active: {}", report.config_active.display()).map_err(output_error)?;
    writeln!(
        out,
        "  usable: {}",
        if report.config_usable { "yes" } else { "no" }
    )
    .map_err(output_error)?;
    writeln!(out, "  template: {}", report.config_template).map_err(output_error)?;
    writeln!(out, "  conflicts: {}", report.config_conflicts).map_err(output_error)
}

fn exit_label(outcome: dsda::LaunchOutcome) -> String {
    match outcome.exit_status {
        dsda::LaunchExit::Clean => "clean".to_owned(),
        dsda::LaunchExit::Failed { code } => format!("failed with status {code}"),
        dsda::LaunchExit::Signalled { signal } => format!("killed by signal {signal}"),
    }
}

/// Streams install stages and byte progress to stderr without spamming.
#[derive(Default)]
struct ProgressPrinter {
    stage: Option<InstallStage>,
    percent: Option<u64>,
}

impl ProgressPrinter {
    fn write(&mut self, update: InstallProgress, err: &mut dyn Write) {
        if self.stage != Some(update.stage) {
            self.stage = Some(update.stage);
            self.percent = None;
            let label = match update.stage {
                InstallStage::Download => "download",
                InstallStage::Verify => "verify",
                InstallStage::Extract => "extract",
                InstallStage::Commit => "commit",
            };
            let _ = writeln!(err, "{label}");
            let _ = err.flush();
        }
        if let Some(progress) = update.transfer
            && let Some(total) = progress.total.filter(|total| *total > 0)
        {
            let percent = progress.downloaded.saturating_mul(100) / total;
            if self.percent != Some(percent) {
                self.percent = Some(percent);
                let _ = writeln!(
                    err,
                    "  {percent}% ({} / {} bytes)",
                    progress.downloaded, total
                );
                let _ = err.flush();
            }
        }
    }
}
