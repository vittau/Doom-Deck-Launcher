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
use ddl_core::local::{self, Compatibility, CustomIwad, CustomSetup};
use ddl_core::profiles::Profile;

use crate::cli::{self, CustomCommand, GroupArg, ProfileExtrasCommand};
use crate::services::{
    CliFailure, DoctorReport, EngineStatus, Environment, ExtrasOutcome, LaunchOverrides, NewIwad,
    Services,
};

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
            let engine =
                services.install_engine(version.as_deref(), force_extract, &mut |update| {
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
        cli::Command::MapsetStatus { id } => {
            let services = open(env)?;
            for (id, status, summary) in services.mapset_status(id.as_deref())? {
                let kinds = match &status {
                    ddl_core::library::RecipeStatus::Update(c) => {
                        let mut kinds = Vec::new();
                        if c.content {
                            kinds.push("content change");
                        }
                        if c.launch {
                            kinds.push("launch change");
                        }
                        if c.components() {
                            kinds.push("components changed");
                        }
                        kinds.join(", ")
                    }
                    _ => status.label().into(),
                };
                writeln!(out, "{id}: {kinds} — {summary}").map_err(output_error)?;
            }
            Ok(())
        }
        cli::Command::MapsetUpdate { id } => {
            let mut services = open(env)?;
            let mut progress = ProgressPrinter::default();
            let before = services.mapset_status(Some(&id))?;
            if before.iter().any(|(_, status, _)| matches!(status, ddl_core::library::RecipeStatus::Update(c) if c.content)) {
                writeln!(err, "Old saves may not load in the new release.").map_err(output_error)?;
            }
            let manifest = services.update_mapset(&id, &mut |p| progress.write(p, err))?;
            writeln!(out, "Updated {} {}", manifest.recipe_id, manifest.release)
                .map_err(output_error)?;
            let snapshot = services
                .snapshot()
                .map_err(|e| CliFailure::message(e.to_string()))?;
            if let Some(item) = snapshot.items.iter().find(|i| i.recipe.id == id) {
                for notice in &item.update_notice {
                    writeln!(out, "{notice}").map_err(output_error)?;
                }
            }
            Ok(())
        }
        cli::Command::MapsetUninstall {
            id,
            delete_profiles,
        } => {
            let mut services = open(env)?;
            let outcome = services.uninstall_mapset(&id, delete_profiles)?;
            write!(
                out,
                "Uninstalled {} ({})",
                outcome.id,
                size_label(outcome.bytes)
            )
            .map_err(output_error)?;
            if delete_profiles {
                write!(out, "; deleted {} profile(s)", outcome.profiles).map_err(output_error)?;
            }
            if outcome.imported {
                write!(out, "; removed imported components").map_err(output_error)?;
            }
            writeln!(out).map_err(output_error)
        }
        cli::Command::MapsetInstall { id, components } => {
            let mut services = open(env)?;
            let mut progress = ProgressPrinter::default();
            let manifest = services
                .install_mapset(&id, &components, &mut |update| progress.write(update, err))?;
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
            profile,
            components,
            skill,
            pistol_start,
            accept_changes,
        } => {
            let mut services = open(env)?;
            apply_iwad_seam(&mut services)?;
            if services.has_recipe(&id) {
                if accept_changes {
                    let accepted = services.accept_extra_changes(&id, profile.as_deref())?;
                    if accepted > 0 {
                        writeln!(err, "Re-recorded {accepted} extra file(s).")
                            .map_err(output_error)?;
                    }
                }
                let overrides = LaunchOverrides {
                    profile,
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
            } else if services.has_custom(&id) {
                if !components.is_empty() {
                    return Err(CliFailure::message(
                        "a custom setup has no components; drop `--with`",
                    ));
                }
                if dry_run {
                    let launch =
                        services.custom_resolution(&id, profile.as_deref(), accept_changes)?;
                    print_launch_plan(&launch.plan, out)
                } else {
                    let outcome = services.run_custom(&id, profile.as_deref(), accept_changes)?;
                    writeln!(err, "DSDA-Doom exited: {}", exit_label(outcome)).map_err(output_error)
                }
            } else {
                Err(CliFailure::message(format!(
                    "no catalog recipe or custom setup with id `{id}`"
                )))
            }
        }
        cli::Command::Custom(command) => {
            let mut services = open(env)?;
            apply_iwad_seam(&mut services)?;
            exec_custom(&mut services, command, out, err)
        }
        cli::Command::ProfileExtras(command) => {
            let mut services = open(env)?;
            apply_iwad_seam(&mut services)?;
            exec_profile_extras(&mut services, command, out, err)
        }
        cli::Command::Doctor => {
            let mut services = open(env)?;
            apply_iwad_seam(&mut services)?;
            let report = services.doctor()?;
            print_doctor(&report, out)
        }
        cli::Command::Run(_) | cli::Command::Help(_) | cli::Command::Version => Ok(()),
        cli::Command::CatalogLint { .. } => Ok(()),
        cli::Command::CatalogBundle {
            out: dest,
            catalog,
            serial,
            commit,
            min_ddl_version,
        } => {
            let serial = serial.unwrap_or_else(env_serial);
            let commit = commit.unwrap_or_else(env_commit);
            let min_ddl_version = min_ddl_version
                .unwrap_or_else(|| ddl_core::catalog::DEFAULT_MIN_DDL_VERSION.to_owned());
            match ddl_core::catalog::build_bundle(&catalog, serial, &commit, &min_ddl_version) {
                Ok(bundle) => {
                    let mut json = serde_json::to_vec_pretty(&bundle).map_err(|error| {
                        CliFailure::message(format!("could not serialize the bundle: {error}"))
                    })?;
                    json.push(b'\n');
                    std::fs::write(&dest, &json).map_err(|error| {
                        CliFailure::message(format!("could not write {}: {error}", dest.display()))
                    })?;
                    writeln!(
                        out,
                        "Wrote {} ({} recipes, serial {}{})",
                        dest.display(),
                        bundle.recipe_files().len(),
                        serial,
                        ddl_core::catalog::serial_date(serial)
                            .map_or_else(String::new, |date| format!(", {date}"))
                    )
                    .map_err(output_error)
                }
                Err(errors) => {
                    for error in &errors {
                        writeln!(err, "{error}").map_err(output_error)?;
                    }
                    Err(CliFailure::message(format!(
                        "catalog lint failed with {} error(s); no bundle was written",
                        errors.len()
                    )))
                }
            }
        }
        cli::Command::CatalogStatus => {
            let services = open(env)?;
            print_catalog_status(&services.catalog_status(), out)
        }
        cli::Command::CatalogUpdate => {
            let mut services = open(env)?;
            let outcome = services.catalog_update()?;
            print_catalog_outcome(&outcome, out)
        }
    }
}

/// The bundle serial from the environment (spec 019, R1): the source commit's
/// UTC Unix time, which CI sets, falling back to the reproducible-build epoch.
fn env_serial() -> u64 {
    ["DDL_CATALOG_SERIAL", "SOURCE_DATE_EPOCH"]
        .iter()
        .find_map(|key| std::env::var(key).ok())
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0)
}

/// The source commit from the environment, which CI sets.
fn env_commit() -> String {
    ["DDL_SOURCE_COMMIT", "GITHUB_SHA"]
        .iter()
        .find_map(|key| std::env::var(key).ok())
        .unwrap_or_default()
}

/// One line for `ddl catalog status` (spec 019, R5).
fn print_catalog_status(
    info: &crate::services::CatalogInfo,
    out: &mut dyn Write,
) -> Result<(), CliFailure> {
    let date = ddl_core::catalog::serial_date(info.serial).unwrap_or_else(|| "unknown date".into());
    if info.updated {
        writeln!(
            out,
            "Catalog: updated {date} (serial {}, commit {}, fetched {})",
            info.serial,
            if info.source_commit.is_empty() {
                "unknown"
            } else {
                &info.source_commit
            },
            info.fetched_at
                .and_then(ddl_core::catalog::serial_date)
                .unwrap_or_else(|| "unknown".into())
        )
        .map_err(output_error)?;
    } else {
        writeln!(out, "Catalog: built-in {date} (serial {})", info.serial).map_err(output_error)?;
    }
    writeln!(
        out,
        "Automatic checks: {} (last checked {})",
        if info.auto_update { "on" } else { "off" },
        info.last_check
            .and_then(ddl_core::catalog::serial_date)
            .unwrap_or_else(|| "never".into())
    )
    .map_err(output_error)
}

/// One line for `ddl catalog update` (spec 019, R5).
fn print_catalog_outcome(
    outcome: &crate::services::CatalogOutcome,
    out: &mut dyn Write,
) -> Result<(), CliFailure> {
    use crate::services::CatalogOutcome as O;
    match outcome {
        O::Updated {
            serial,
            source_commit,
            dropped,
        } => {
            let date = ddl_core::catalog::serial_date(*serial).unwrap_or_else(|| "unknown".into());
            write!(out, "Catalog updated to {date} (serial {serial}").map_err(output_error)?;
            if !source_commit.is_empty() {
                write!(out, ", commit {source_commit}").map_err(output_error)?;
            }
            if *dropped > 0 {
                write!(out, ", {dropped} recipe(s) dropped by lint").map_err(output_error)?;
            }
            writeln!(out, ")").map_err(output_error)
        }
        O::Current { serial } => {
            let date = ddl_core::catalog::serial_date(*serial).unwrap_or_else(|| "unknown".into());
            writeln!(out, "Catalog is up to date ({date})").map_err(output_error)
        }
        O::Disabled => writeln!(out, "Automatic catalog updates are off").map_err(output_error),
        O::NotDue => writeln!(out, "Checked less than 24 h ago").map_err(output_error),
    }
}

fn open(env: Environment) -> Result<Services, CliFailure> {
    Services::open(env).map_err(|problem| CliFailure::message(problem.to_string()))
}

/// Maps the CLI group onto the core group (spec 026, R3).
fn core_group(group: GroupArg) -> local::Group {
    match group {
        GroupArg::Files => local::Group::Files,
        GroupArg::Patches => local::Group::Patches,
    }
}

/// Parses `--complevel` (default `auto`, which omits the flag).
fn compatibility_from(value: Option<&str>) -> Result<Compatibility, CliFailure> {
    match value {
        None => Ok(Compatibility::Auto),
        Some(text) => {
            Compatibility::parse(text).map_err(|error| CliFailure::message(error.to_string()))
        }
    }
}

/// A one-line description of a setup's IWAD.
fn iwad_label(iwad: &CustomIwad) -> String {
    match iwad {
        CustomIwad::Identified { variant } => format!("identified {variant}"),
        CustomIwad::File { file } => file.path.display().to_string(),
    }
}

/// Prints a custom setup; with `check`, names any file that is missing or
/// changed (spec 026, R4).
fn print_setup(setup: &CustomSetup, check: bool, out: &mut dyn Write) -> Result<(), CliFailure> {
    writeln!(out, "{}  {}", setup.id, setup.name).map_err(output_error)?;
    writeln!(out, "  IWAD: {}", iwad_label(&setup.iwad)).map_err(output_error)?;
    writeln!(out, "  compatibility: {}", setup.compatibility).map_err(output_error)?;
    for (label, list) in [
        ("files (load order)", &setup.files),
        ("patches", &setup.patches),
    ] {
        if list.is_empty() {
            writeln!(out, "  {label}: (none)").map_err(output_error)?;
        } else {
            writeln!(out, "  {label}:").map_err(output_error)?;
            for (index, file) in list.iter().enumerate() {
                writeln!(
                    out,
                    "    {index}: {}  {}  {} bytes",
                    file.path.display(),
                    file.kind,
                    file.size
                )
                .map_err(output_error)?;
            }
        }
    }
    if check {
        let issues = local::verify_setup(setup);
        if issues.is_empty() {
            writeln!(out, "  files: ok").map_err(output_error)?;
        } else {
            for issue in issues {
                writeln!(out, "  {issue}").map_err(output_error)?;
            }
        }
    }
    Ok(())
}

/// One line summarising a setup's two groups.
fn print_setup_counts(setup: &CustomSetup, out: &mut dyn Write) -> Result<(), CliFailure> {
    writeln!(
        out,
        "{}: {} file(s), {} patch(es)",
        setup.id,
        setup.files.len(),
        setup.patches.len()
    )
    .map_err(output_error)
}

/// Prints a profile's extras as the two engine groups (spec 026, R5).
fn print_extras(profile: &Profile, out: &mut dyn Write) -> Result<(), CliFailure> {
    writeln!(
        out,
        "{} ({}) — {} extra file(s)",
        profile.name,
        profile.id,
        profile.extras.len()
    )
    .map_err(output_error)?;
    for (label, group) in [
        ("files (load order)", local::Group::Files),
        ("patches", local::Group::Patches),
    ] {
        let indices = local::group_indices(&profile.extras, group);
        if indices.is_empty() {
            writeln!(out, "  {label}: (none)").map_err(output_error)?;
            continue;
        }
        writeln!(out, "  {label}:").map_err(output_error)?;
        for (index, at) in indices.iter().enumerate() {
            let file = &profile.extras[*at];
            writeln!(
                out,
                "    {index}: {}  {}  {} bytes",
                file.path.display(),
                file.kind,
                file.size
            )
            .map_err(output_error)?;
        }
    }
    Ok(())
}

/// Warns that changing extras can strand existing saves (spec 026, R5).
fn warn_saves(outcome: &ExtrasOutcome, err: &mut dyn Write) -> Result<(), CliFailure> {
    if outcome.saves_warn {
        writeln!(
            err,
            "Warning: existing saves for this profile may not load with different files."
        )
        .map_err(output_error)?;
    }
    Ok(())
}

/// `ddl custom …` (spec 026, R7).
fn exec_custom(
    services: &mut Services,
    command: CustomCommand,
    out: &mut dyn Write,
    _err: &mut dyn Write,
) -> Result<(), CliFailure> {
    match command {
        CustomCommand::List => {
            let setups = services.custom_list()?;
            if setups.is_empty() {
                return writeln!(out, "No custom setups.").map_err(output_error);
            }
            for setup in setups {
                writeln!(
                    out,
                    "{:<20} {:<6} {:<4} {}",
                    setup.id,
                    iwad_label(&setup.iwad),
                    setup.files.len() + setup.patches.len(),
                    setup.name
                )
                .map_err(output_error)?;
            }
            Ok(())
        }
        CustomCommand::Show { id } => {
            let setup = services.custom_show(&id)?;
            print_setup(&setup, true, out)
        }
        CustomCommand::Create {
            name,
            id,
            iwad,
            iwad_file,
            compatibility,
        } => {
            let compatibility = compatibility_from(compatibility.as_deref())?;
            let iwad = match (iwad, iwad_file) {
                (Some(variant), None) => NewIwad::Variant(variant),
                (None, Some(path)) => NewIwad::File(path),
                _ => {
                    return Err(CliFailure::message(
                        "`custom create` needs exactly one of `--iwad` or `--iwad-file`",
                    ));
                }
            };
            let setup = services.custom_create(&name, id.as_deref(), iwad, compatibility)?;
            writeln!(out, "Created custom setup `{}` ({})", setup.id, setup.name)
                .map_err(output_error)
        }
        CustomCommand::AddFile { id, paths } => {
            let setup = services.custom_add_file(&id, &paths)?;
            print_setup_counts(&setup, out)
        }
        CustomCommand::Move {
            id,
            group,
            from,
            to,
        } => {
            let setup = services.custom_move(&id, core_group(group), from, to)?;
            print_setup_counts(&setup, out)
        }
        CustomCommand::RemoveFile { id, group, index } => {
            let setup = services.custom_remove_file(&id, core_group(group), index)?;
            print_setup_counts(&setup, out)
        }
        CustomCommand::Delete {
            id,
            delete_profiles,
        } => {
            services.custom_delete(&id, delete_profiles)?;
            writeln!(
                out,
                "Deleted custom setup `{id}`{}",
                if delete_profiles {
                    " and its profiles and saves"
                } else {
                    ""
                }
            )
            .map_err(output_error)
        }
    }
}

/// `ddl profile extras …` (spec 026, R5/R7).
fn exec_profile_extras(
    services: &mut Services,
    command: ProfileExtrasCommand,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<(), CliFailure> {
    match command {
        ProfileExtrasCommand::List { recipe, profile } => {
            let profile = services.extras_list(&recipe, profile.as_deref())?;
            print_extras(&profile, out)
        }
        ProfileExtrasCommand::Add {
            recipe,
            profile,
            paths,
        } => {
            let outcome = services.extras_add(&recipe, profile.as_deref(), &paths)?;
            print_extras(&outcome.profile, out)?;
            warn_saves(&outcome, err)
        }
        ProfileExtrasCommand::Remove {
            recipe,
            profile,
            group,
            index,
        } => {
            let outcome =
                services.extras_remove(&recipe, profile.as_deref(), core_group(group), index)?;
            print_extras(&outcome.profile, out)?;
            warn_saves(&outcome, err)
        }
        ProfileExtrasCommand::Move {
            recipe,
            profile,
            group,
            from,
            to,
        } => {
            let outcome =
                services.extras_move(&recipe, profile.as_deref(), core_group(group), from, to)?;
            print_extras(&outcome.profile, out)?;
            warn_saves(&outcome, err)
        }
        ProfileExtrasCommand::Clear { recipe, profile } => {
            let outcome = services.extras_clear(&recipe, profile.as_deref())?;
            print_extras(&outcome.profile, out)?;
            warn_saves(&outcome, err)
        }
    }
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

/// IEC byte units for the one-line uninstall summary (spec 022, R5).
fn size_label(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut amount = bytes as f64;
    let mut unit = 0;
    while amount >= 1024. && unit < UNITS.len() - 1 {
        amount /= 1024.;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{amount:.1} {}", UNITS[unit])
    }
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
                entry
                    .game
                    .map_or_else(|| "-".to_owned(), |game| game.to_string())
            ),
            None if !entry.candidates.is_empty() => {
                format!("ambiguous: {}", entry.candidates.join(", "))
            }
            None => "unresolved".to_owned(),
        };
        // Spec 021 R3: name the source discovered from ("GOG via Heroic",
        // "Steam", …).
        writeln!(
            out,
            "{}: {} ({}) sha256={}",
            entry.source.label(),
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
    print_launch_plan(resolution.plan(), out)
}

/// Prints a launch plan exactly as `ddl launch --dry-run` does (spec 014, R1).
fn print_launch_plan(plan: &dsda::LaunchPlan, out: &mut dyn Write) -> Result<(), CliFailure> {
    writeln!(out, "program: {}", plan.executable.display()).map_err(output_error)?;
    for token in plan.to_command() {
        writeln!(out, "{}", token.to_string_lossy()).map_err(output_error)?;
    }
    writeln!(out, "working directory: {}", plan.working_dir.display()).map_err(output_error)
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
            writeln!(
                out,
                "  {}  {}  {}",
                engine.version,
                mode_label(engine.mode),
                engine.path.display()
            )
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
                "  [{}] {} ({})",
                entry.source.label(),
                entry.path.display(),
                entry.variant.as_deref().unwrap_or("ambiguous")
            )
            .map_err(output_error)?;
        }
    }

    writeln!(out, "profiles:").map_err(output_error)?;
    for (mapset, profiles) in &report.profiles {
        for profile in profiles {
            writeln!(out, "  {mapset}: {} ({})", profile.name, profile.id).map_err(output_error)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{Environment, TestEnvironment};
    use ddl_core::{
        catalog::Catalog,
        downloads::{
            Sha256,
            test_http::{Response, Server},
        },
        engines::dsda,
        resolver::Blocker,
    };
    use std::sync::Arc;

    /// A script that answers both the `-v` smoke test and `--appimage-extract`,
    /// exactly like the reviewed AppImage does.
    const APPIMAGE: &str = r#"#!/bin/sh
case "$1" in
  -v) echo "dsda-doom v0.30.1"; exit 0 ;;
  --appimage-extract)
    mkdir -p squashfs-root
    printf '#!/bin/sh\necho "dsda-doom v0.30.1"\n' > squashfs-root/AppRun
    chmod +x squashfs-root/AppRun
    exit 0 ;;
esac
exit 1
"#;

    struct Fixture {
        _temp: tempfile::TempDir,
        env: Environment,
        _server: Server,
        _api: Arc<dsda::CannedApi>,
    }

    fn zip_entries(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
        use std::io::Write as _;
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    /// Rebuilds the SIGIL package with tiny synthetic member bytes.
    fn synthetic_package(recipe: &mut ddl_core::catalog::Recipe) -> Vec<u8> {
        let mut entries = Vec::new();
        for member in &mut recipe.packages[0].members {
            let bytes = format!("synthetic member {}", member.id).into_bytes();
            member.size = bytes.len() as u64;
            member.sha256 = Sha256::of(&bytes).to_hex();
            entries.push((member.path.clone(), bytes));
        }
        let bytes = zip_entries(&entries);
        recipe.packages[0].size = bytes.len() as u64;
        recipe.packages[0].sha256 = Sha256::of(&bytes).to_hex();
        bytes
    }

    /// A real, lint-clean bundle built from this repository's `catalog/`.
    fn repository_bundle(serial: u64) -> Vec<u8> {
        let catalog = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog");
        let bundle = ddl_core::catalog::build_bundle(&catalog, serial, "test-commit", "0.0.1")
            .expect("the repository catalog lints");
        serde_json::to_vec(&bundle).unwrap()
    }

    fn fixture() -> Fixture {
        fixture_with_engine(APPIMAGE)
    }

    /// A fixture whose engine script is `engine` (its reviewed-list entry is
    /// derived from those bytes).
    fn fixture_with_engine(engine: &str) -> Fixture {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let paths = ddl_core::platform::paths::resolve(&|key: &str| {
            (key == "HOME").then(|| home.display().to_string())
        })
        .unwrap();

        let mut catalog = Catalog::embedded();
        catalog.recipes.retain(|loaded| loaded.recipe.id == "sigil");
        let package = synthetic_package(&mut catalog.recipes[0].recipe);
        let package_for_server = package.clone();
        let engine_for_server = engine.as_bytes().to_vec();
        let bundle_for_server = repository_bundle(1_700_000_000);
        let server = Server::start(move |request| {
            if request.target.contains("AppImage") {
                Response::ok(engine_for_server.clone())
            } else if request.target == "/catalog.json" {
                Response::ok(bundle_for_server.clone())
            } else {
                Response::ok(package_for_server.clone())
            }
        });
        catalog.recipes[0].recipe.packages[0].urls = vec![server.url("/sigil.zip")];
        catalog.hosts.downloads.push("127.0.0.1".into());

        let reviewed = dsda::ReviewedList::parse(&format!(
            "schema_version = 1\n[[version]]\ntag = 'v0.30.1'\nasset = 'test-linux.AppImage'\nsize = {}\nsha256 = '{}'\nlicense = 'GPL-2.0-or-later'\nrelease_url = 'https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.1'\nsource_url = 'https://github.com/dsda-org/dsda-doom/tree/v0.30.1'\n",
            engine.len(),
            Sha256::of(engine.as_bytes()).to_hex()
        ))
        .unwrap();
        let api = Arc::new(dsda::CannedApi::new());
        let runner = Arc::new(dsda::FakeEngineRunner::new(dsda::LaunchExit::Clean));
        let test = Arc::new(TestEnvironment {
            api: api.clone(),
            catalog,
            engine_base: server.url(""),
            catalog_url: server.url("/catalog.json"),
            reviewed,
            runner,
            engine_response_timeout: None,
        });
        Fixture {
            env: Environment {
                paths,
                home,
                test: Some(test),
            },
            _temp: temp,
            _server: server,
            _api: api,
        }
    }

    fn run(fx: &Fixture, command: cli::Command) -> Result<(String, String), CliFailure> {
        let mut out = Vec::new();
        let mut err = Vec::new();
        run_io(command, fx.env.clone(), &mut out, &mut err)?;
        Ok((
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        ))
    }

    fn install_engine(fx: &Fixture, force_extract: bool) {
        run(
            fx,
            cli::Command::EngineInstall {
                version: None,
                force_extract,
            },
        )
        .expect("engine install");
    }

    fn install_sigil(fx: &Fixture, components: &[&str]) {
        run(
            fx,
            cli::Command::MapsetInstall {
                id: "sigil".into(),
                components: components.iter().map(|c| (*c).to_owned()).collect(),
            },
        )
        .expect("mapset install");
    }

    #[test]
    fn engine_install_appimage_then_status_lists_it_as_active() {
        let fx = fixture();
        let (out, err) = run(
            &fx,
            cli::Command::EngineInstall {
                version: None,
                force_extract: false,
            },
        )
        .expect("install");
        assert!(out.contains("Installed DSDA-Doom 0.30.1"), "{out}");
        assert!(out.contains("(appimage)"), "{out}");
        assert!(err.contains("download"), "progress goes to stderr: {err}");
        assert!(err.contains("verify"), "{err}");
        assert!(err.contains("commit"), "{err}");

        let (status, _) = run(&fx, cli::Command::EngineStatus).expect("status");
        assert!(
            status.contains("0.30.1  appimage  reviewed  active"),
            "{status}"
        );
        assert!(status.contains("active: 0.30.1"), "{status}");
    }

    #[test]
    fn engine_install_force_extract_takes_the_fallback() {
        let fx = fixture();
        let (out, err) = run(
            &fx,
            cli::Command::EngineInstall {
                version: Some("0.30.1".into()),
                force_extract: true,
            },
        )
        .expect("forced install");
        // The script's `-v` would pass, so `(appimage)` would be the default.
        assert!(out.contains("(extracted)"), "{out}");
        assert!(err.contains("extract"), "{err}");
    }

    #[test]
    fn engine_install_reports_a_host_runtime_failure_actionably() {
        // Real stderr from CI run 38980417407: the extracted AppRun in the
        // Steam Runtime SDK container needs glibc >= 2.34.
        const OLD_GLIBC: &str = "#!/bin/sh\necho \"/lib/x86_64-linux-gnu/libc.so.6: version 'GLIBC_2.34' not found\" >&2\nexit 1\n";
        let fx = fixture_with_engine(OLD_GLIBC);
        let error = run(
            &fx,
            cli::Command::EngineInstall {
                version: None,
                force_extract: false,
            },
        )
        .unwrap_err();
        let message = error.to_string();
        assert!(message.contains("GLIBC_2.34"), "{message}");
        assert!(message.contains("Compatibility tool"), "{message}");
    }

    #[test]
    fn engine_install_reports_a_missing_library_by_name() {
        const MISSING_OPENGL: &str = "#!/bin/sh\necho 'dsda-doom.AppImage: error while loading shared libraries: libOpenGL.so.0: cannot open shared object file: No such file or directory' >&2\nexit 127\n";
        let fx = fixture_with_engine(MISSING_OPENGL);
        let error = run(
            &fx,
            cli::Command::EngineInstall {
                version: None,
                force_extract: false,
            },
        )
        .unwrap_err();
        let message = error.to_string();
        assert!(message.contains("libOpenGL.so.0"), "{message}");
        assert!(
            !message.contains("SteamOS"),
            "the missing-library message must stay neutral: {message}"
        );
    }

    #[test]
    fn mapset_list_reports_install_state_and_install_places_members() {
        let fx = fixture();
        let (before, _) = run(&fx, cli::Command::MapsetList).expect("list");
        assert!(before.contains("sigil"), "{before}");
        assert!(before.contains("not installed"), "{before}");

        install_sigil(&fx, &["compat"]);
        let (after, _) = run(&fx, cli::Command::MapsetList).expect("list");
        assert!(after.contains("installed"), "{after}");

        // The compat replacement is what the profile and install both record.
        let (out, err) = run(
            &fx,
            cli::Command::MapsetInstall {
                id: "sigil".into(),
                components: vec!["compat".into()],
            },
        )
        .expect("reinstall with compat");
        assert!(out.contains("Installed sigil 1.23"), "{out}");
        assert!(out.contains("SIGIL_COMPAT_V1_23.wad"), "{out}");
        assert!(err.contains("verify"), "{err}");
    }

    #[test]
    fn mapset_install_rejects_unknown_and_user_supplied_components() {
        let fx = fixture();
        let unknown = run(
            &fx,
            cli::Command::MapsetInstall {
                id: "sigil".into(),
                components: vec!["nope".into()],
            },
        )
        .unwrap_err();
        assert!(
            unknown.to_string().contains("has no component `nope`"),
            "{unknown}"
        );

        let user = run(
            &fx,
            cli::Command::MapsetInstall {
                id: "sigil".into(),
                components: vec!["soundtrack".into()],
            },
        )
        .unwrap_err();
        assert!(user.to_string().contains("user-supplied"), "{user}");

        let missing = run(
            &fx,
            cli::Command::MapsetInstall {
                id: "does-not-exist".into(),
                components: vec![],
            },
        )
        .unwrap_err();
        assert!(
            missing.to_string().contains("no catalog recipe"),
            "{missing}"
        );
    }

    #[test]
    fn iwad_scan_lists_ambiguous_and_unrecognized_files() {
        let fx = fixture();
        let dir = fx.env.home.join("iwads");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("maybe.wad"), commercial_wad()).unwrap();
        std::fs::write(dir.join("junk.wad"), b"not a wad").unwrap();

        let (out, _) = run(
            &fx,
            cli::Command::IwadScan {
                paths: vec![dir.clone()],
            },
        )
        .expect("scan");
        assert!(out.contains("maybe.wad"), "{out}");
        assert!(out.contains("ambiguous:"), "{out}");
        assert!(out.contains("doom2-1.9"), "{out}");
        assert!(out.contains("sha256="), "{out}");
        assert!(
            out.contains("unrecognized:") && out.contains("junk.wad"),
            "{out}"
        );
    }

    #[test]
    fn iwad_scan_labels_a_gog_install_from_heroic() {
        let fx = fixture();
        let install = fx.env.home.join("Games/Heroic/DOOM + DOOM II/base");
        std::fs::create_dir_all(&install).unwrap();
        std::fs::write(install.join("doom2.wad"), commercial_wad()).unwrap();
        let record = fx.env.home.join(".config/heroic/gog_store/installed.json");
        std::fs::create_dir_all(record.parent().unwrap()).unwrap();
        std::fs::write(
            &record,
            format!(
                "{{\"installed\":[{{\"install_path\":\"{}\"}}]}}",
                install.parent().unwrap().display()
            ),
        )
        .unwrap();

        let (out, _) = run(&fx, cli::Command::IwadScan { paths: vec![] }).expect("scan");
        assert!(out.contains("GOG via Heroic: "), "{out}");
        assert!(out.contains("doom2.wad"), "{out}");
    }

    #[test]
    fn launch_dry_run_without_an_iwad_reports_the_typed_blocker() {
        let fx = fixture();
        install_engine(&fx, false);
        install_sigil(&fx, &[]);
        let error = run(
            &fx,
            cli::Command::Launch {
                id: "sigil".into(),
                dry_run: true,
                profile: None,
                components: vec![],
                skill: None,
                pistol_start: false,
                accept_changes: false,
            },
        )
        .unwrap_err();
        match error {
            CliFailure::Blocker(Blocker::MissingIwad { game, .. }) => {
                assert_eq!(game, ddl_core::catalog::Game::Doom);
            }
            other => panic!("expected MissingIwad, got {other}"),
        }
    }

    #[test]
    fn launch_dry_run_prints_the_d011_argv_with_a_registered_iwad() {
        let fx = fixture();
        install_engine(&fx, false);
        install_sigil(&fx, &[]);
        let seam = fx.env.home.join("seam-iwads");
        {
            let mut services = Services::open(fx.env.clone()).unwrap();
            services.register_fake_iwads(&seam).expect("seam");
        }

        let (out, _) = run(
            &fx,
            cli::Command::Launch {
                id: "sigil".into(),
                dry_run: true,
                profile: None,
                components: vec![],
                skill: Some(4),
                pistol_start: true,
                accept_changes: false,
            },
        )
        .expect("dry run");

        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("program: "), "{out}");
        // D-011: one `-file` list, one `-deh`, a numeric `-complevel`, and the
        // IWAD through DDL's canonical link.
        assert_eq!(
            lines.iter().filter(|line| **line == "-file").count(),
            1,
            "{out}"
        );
        assert_eq!(
            lines.iter().filter(|line| **line == "-deh").count(),
            0,
            "{out}"
        );
        assert_eq!(
            lines.iter().filter(|line| **line == "-iwad").count(),
            1,
            "{out}"
        );
        assert_eq!(
            lines.iter().filter(|line| **line == "-noautoload").count(),
            1,
            "{out}"
        );
        assert_eq!(
            lines.iter().filter(|line| **line == "-config").count(),
            1,
            "{out}"
        );
        assert_eq!(
            lines.iter().filter(|line| **line == "-save").count(),
            1,
            "{out}"
        );
        assert_eq!(
            lines.iter().filter(|line| **line == "-data").count(),
            1,
            "{out}"
        );
        assert_eq!(
            lines.iter().filter(|line| **line == "-shotdir").count(),
            1,
            "{out}"
        );
        assert!(lines.contains(&"-pistolstart"), "{out}");

        let after = |flag: &str| {
            let index = lines.iter().position(|line| *line == flag).unwrap();
            lines[index + 1]
        };
        let iwad = after("-iwad");
        assert!(iwad.starts_with('/'), "canonical link is absolute: {iwad}");
        assert!(iwad.ends_with("doomu.wad"), "{iwad}");
        assert_eq!(after("-complevel"), "3", "numeric complevel: {out}");
        assert_eq!(after("-skill"), "4", "{out}");
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("working directory: /")),
            "{out}"
        );
        assert!(out.contains("SIGIL_V1_23.wad"), "{out}");
    }

    #[test]
    fn launch_profile_selects_isolated_paths_and_remembers_without_saving_overrides() {
        let fx = fixture();
        install_engine(&fx, false);
        install_sigil(&fx, &[]);
        let seam = fx.env.home.join("seam-iwads");
        let mut services = Services::open(fx.env.clone()).unwrap();
        services.register_fake_iwads(&seam).unwrap();
        services
            .execute(
                crate::services::Command::NewProfile("sigil".into(), false),
                &ddl_core::downloads::CancellationToken::new(),
                &mut |_| {},
            )
            .unwrap();
        let store = ddl_core::profiles::ProfileStore::new(&fx.env.paths.config, &fx.env.paths.data);
        let selected = store
            .list()
            .unwrap()
            .into_iter()
            .find(|p| p.name == "Profile 2")
            .unwrap();
        services
            .execute(
                crate::services::Command::SelectProfile("sigil".into(), "sigil-default".into()),
                &ddl_core::downloads::CancellationToken::new(),
                &mut |_| {},
            )
            .unwrap();
        for selector in [selected.name.clone(), selected.id.clone()] {
            let (out, _) = run(
                &fx,
                cli::Command::Launch {
                    id: "sigil".into(),
                    dry_run: true,
                    profile: Some(selector),
                    components: vec![],
                    skill: Some(5),
                    pistol_start: true,
                    accept_changes: false,
                },
            )
            .unwrap();
            assert!(
                out.contains(&store.dirs(&selected.id).save.display().to_string()),
                "{out}"
            );
        }
        assert_eq!(
            Services::open(fx.env.clone())
                .unwrap()
                .snapshot()
                .unwrap()
                .items[0]
                .profile
                .id,
            "sigil-default"
        );
        run(
            &fx,
            cli::Command::Launch {
                id: "sigil".into(),
                dry_run: false,
                profile: Some(selected.name),
                components: vec![],
                skill: Some(5),
                pistol_start: true,
                accept_changes: false,
            },
        )
        .unwrap();
        let restored = Services::open(fx.env.clone()).unwrap().snapshot().unwrap();
        assert_eq!(restored.items[0].profile.id, selected.id);
        assert!(restored.items[0].profile.last_played.is_some());
        assert_eq!(restored.items[0].profile.skill, 3);
        assert!(!restored.items[0].profile.pistol_start);
        let (out, _) = run(&fx, cli::Command::Doctor).unwrap();
        assert!(
            out.contains(&format!("sigil: Profile 2 ({})", selected.id)),
            "{out}"
        );
        assert!(out.contains("sigil: Default (sigil-default)"), "{out}");
    }

    #[test]
    fn doctor_reports_paths_engine_mapsets_and_config() {
        let fx = fixture();
        install_engine(&fx, false);
        install_sigil(&fx, &[]);
        let (out, _) = run(&fx, cli::Command::Doctor).expect("doctor");
        assert!(out.contains("doctor"), "{out}");
        assert!(out.contains("paths:"), "{out}");
        assert!(out.contains("engine:"), "{out}");
        assert!(out.contains("0.30.1  appimage"), "{out}");
        assert!(out.contains("active: 0.30.1"), "{out}");
        assert!(out.contains("mapsets:"), "{out}");
        assert!(out.contains("sigil") && out.contains("installed"), "{out}");
        assert!(out.contains("dsda config:"), "{out}");
        assert!(out.contains("logs:"), "{out}");
    }

    #[test]
    fn concurrent_mutation_is_refused_with_a_clear_message() {
        let fx = fixture();
        let lock = ddl_core::platform::lock::StateLock::acquire(&fx.env.paths.config)
            .expect("hold the state lock");
        let error = run(
            &fx,
            cli::Command::MapsetInstall {
                id: "sigil".into(),
                components: vec![],
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("another DDL process"), "{error}");
        drop(lock);
        // Releasing the lock lets the same command succeed.
        install_sigil(&fx, &[]);
    }

    #[test]
    fn catalog_bundle_writes_a_lint_clean_bundle_and_refuses_a_bad_one() {
        let fx = fixture();
        let out = fx.env.home.join("bundle.json");
        let catalog = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog");
        let (text, _) = run(
            &fx,
            cli::Command::CatalogBundle {
                out: out.clone(),
                catalog: catalog.clone(),
                serial: Some(1_700_000_000),
                commit: Some("deadbeef".into()),
                min_ddl_version: Some("0.0.1".into()),
            },
        )
        .expect("bundle");
        assert!(text.contains("Wrote "), "{text}");
        let bundle: ddl_core::catalog::Bundle =
            serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        assert_eq!(bundle.catalog_serial, 1_700_000_000);
        assert_eq!(bundle.source_commit, "deadbeef");
        assert_eq!(bundle.bundle_schema, ddl_core::catalog::BUNDLE_SCHEMA);
        assert!(!bundle.recipe_files().is_empty());

        // Without `--min-ddl-version` the bundle uses the deliberate constant,
        // not this binary's version (spec 019 review fix).
        let default_out = fx.env.home.join("default-bundle.json");
        run(
            &fx,
            cli::Command::CatalogBundle {
                out: default_out.clone(),
                catalog: catalog.clone(),
                serial: Some(1_700_000_001),
                commit: None,
                min_ddl_version: None,
            },
        )
        .expect("bundle with defaults");
        let default_bundle: ddl_core::catalog::Bundle =
            serde_json::from_slice(&std::fs::read(&default_out).unwrap()).unwrap();
        assert_eq!(
            default_bundle.min_ddl_version,
            ddl_core::catalog::DEFAULT_MIN_DDL_VERSION
        );

        // A catalog that does not lint never produces a file.
        let empty = fx.env.home.join("empty-catalog");
        std::fs::create_dir_all(&empty).unwrap();
        let refused = run(
            &fx,
            cli::Command::CatalogBundle {
                out: fx.env.home.join("never.json"),
                catalog: empty,
                serial: None,
                commit: None,
                min_ddl_version: None,
            },
        )
        .unwrap_err();
        assert!(refused.to_string().contains("lint failed"), "{refused}");
        assert!(!fx.env.home.join("never.json").exists());
    }

    #[test]
    fn catalog_status_and_update_report_and_persist_the_catalog() {
        let fx = fixture();
        let (before, _) = run(&fx, cli::Command::CatalogStatus).expect("status");
        assert!(before.contains("Catalog: built-in"), "{before}");
        assert!(before.contains("Automatic checks: on"), "{before}");
        assert!(before.contains("last checked never"), "{before}");

        let (updated, _) = run(&fx, cli::Command::CatalogUpdate).expect("update");
        assert!(
            updated.contains("Catalog updated to 2023-11-14"),
            "{updated}"
        );

        let (after, _) = run(&fx, cli::Command::CatalogStatus).expect("status");
        assert!(after.contains("Catalog: updated 2023-11-14"), "{after}");
        assert!(after.contains("test-commit"), "{after}");

        // A second manual check finds nothing newer.
        let (again, _) = run(&fx, cli::Command::CatalogUpdate).expect("update");
        assert!(again.contains("up to date"), "{again}");
    }

    /// A synthetic commercial WAD: an IWAD header plus MAP01..MAP32, which the
    /// identifier classifies as the ambiguous Doom II family.
    fn commercial_wad() -> Vec<u8> {
        let mut bytes = b"IWAD".to_vec();
        bytes.extend_from_slice(&32i32.to_le_bytes());
        bytes.extend_from_slice(&12i32.to_le_bytes());
        for map in 1..=32 {
            bytes.extend_from_slice(&[0; 8]);
            let mut lump = [0; 8];
            let name = format!("MAP{map:02}");
            lump[..name.len()].copy_from_slice(name.as_bytes());
            bytes.extend_from_slice(&lump);
        }
        bytes
    }
}
