//! Real-binary round trip for `ddl custom …` and `ddl profile extras …`
//! (spec 026, R7/A1) in a scratch XDG root.
//!
//! The test drives the released `ddl` binary with `HOME` and every XDG variable
//! redirected into a temporary directory, so it never touches the developer's
//! own DDL state. `SLINT_BACKEND` is set to an invalid value so a command that
//! tried to create a window would fail rather than open one. No network and no
//! engine are used.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// A scratch XDG root plus the player files the test references in place.
struct Scratch {
    _temp: TempDir,
    home: PathBuf,
    wad: PathBuf,
    deh: PathBuf,
    iwad: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        let temp = TempDir::new().expect("tempdir");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let games = temp.path().join("games");
        fs::create_dir_all(&games).unwrap();
        let wad = games.join("player.wad");
        fs::write(&wad, pwad(&["MAP01"])).unwrap();
        let deh = games.join("fix.deh");
        fs::write(
            &deh,
            b"Patch File for DeHackEd v3.0\nThing 1\nHit points = 30\n",
        )
        .unwrap();
        let iwad = games.join("mine.wad");
        fs::write(&iwad, wad_bytes(b"IWAD", &["PLAYPAL"])).unwrap();
        Self {
            _temp: temp,
            home,
            wad,
            deh,
            iwad,
        }
    }

    /// Runs `ddl` with this scratch root.
    fn ddl(&self, args: &[&str]) -> Output {
        let dir = self.home.join("state");
        fs::create_dir_all(&dir).unwrap();
        Command::new(env!("CARGO_BIN_EXE_ddl"))
            .args(args)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", self.home.join("config"))
            .env("XDG_DATA_HOME", self.home.join("data"))
            .env("XDG_CACHE_HOME", self.home.join("cache"))
            .env("XDG_STATE_HOME", &dir)
            .env("SLINT_BACKEND", "invalid-backend-proves-headless")
            .output()
            .expect("run ddl")
    }

    /// Runs a command and asserts success, returning its stdout.
    fn ok(&self, args: &[&str]) -> String {
        let output = self.ddl(args);
        assert!(
            output.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    /// Runs a command and asserts failure, returning its stderr.
    fn err(&self, args: &[&str]) -> String {
        let output = self.ddl(args);
        assert!(
            !output.status.success(),
            "{args:?} unexpectedly succeeded: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        String::from_utf8_lossy(&output.stderr).into_owned()
    }
}

fn pwad(lumps: &[&str]) -> Vec<u8> {
    wad_bytes(b"PWAD", lumps)
}

fn wad_bytes(magic: &[u8; 4], lumps: &[&str]) -> Vec<u8> {
    let mut directory = Vec::new();
    for name in lumps {
        directory.extend_from_slice(&0_i32.to_le_bytes());
        directory.extend_from_slice(&0_i32.to_le_bytes());
        let mut padded = [0_u8; 8];
        let len = name.len().min(8);
        padded[..len].copy_from_slice(&name.as_bytes()[..len]);
        directory.extend_from_slice(&padded);
    }
    let mut out = Vec::new();
    out.extend_from_slice(magic);
    out.extend_from_slice(&(lumps.len() as i32).to_le_bytes());
    out.extend_from_slice(&12_i32.to_le_bytes());
    out.extend_from_slice(&directory);
    out
}

fn path(path: &Path) -> String {
    path.display().to_string()
}

#[test]
fn custom_round_trip_in_a_scratch_xdg_root() {
    let scratch = Scratch::new();

    // Create from a user file whose header is IWAD, pinning a complevel.
    let created = scratch.ok(&[
        "custom",
        "create",
        "My Setup",
        "--id",
        "mine",
        "--iwad-file",
        &path(&scratch.iwad),
        "--complevel",
        "21",
    ]);
    assert!(created.contains("Created custom setup `mine`"), "{created}");

    // `list` and `show` round-trip the document.
    let list = scratch.ok(&["custom", "list"]);
    assert!(list.contains("mine"), "{list}");
    let shown = scratch.ok(&["custom", "show", "mine"]);
    assert!(shown.contains("  IWAD: "), "{shown}");
    assert!(shown.contains(&path(&scratch.iwad)), "{shown}");
    assert!(shown.contains("compatibility: 21"), "{shown}");
    assert!(shown.contains("files: ok"), "{shown}");

    // `add-file` routes by content: a WAD to files, a DEH to patches.
    let added = scratch.ok(&[
        "custom",
        "add-file",
        "mine",
        &path(&scratch.wad),
        &path(&scratch.deh),
    ]);
    assert!(added.contains("1 file(s), 1 patch(es)"), "{added}");
    let shown = scratch.ok(&["custom", "show", "mine"]);
    assert!(shown.contains("files (load order):"), "{shown}");
    assert!(shown.contains("patches:"), "{shown}");

    // A `.pk3` is refused because the engine's `-file` has no branch for it.
    let stray = scratch.home.join("stray.pk3");
    fs::write(&stray, b"PK\x03\x04payload").unwrap();
    let refused = scratch.err(&["custom", "add-file", "mine", &path(&stray)]);
    assert!(refused.contains(".pk3"), "{refused}");
    assert!(refused.contains(".wad"), "{refused}");

    // `move` reorders only the record; the file is untouched.
    let moved = scratch.ok(&["custom", "move", "mine", "files", "0", "0"]);
    assert!(moved.contains("1 file(s), 1 patch(es)"), "{moved}");

    // Changing a referenced file blocks the launch and names the file (R4).
    fs::write(&scratch.wad, pwad(&["MAP02", "MAP03"])).unwrap();
    let blocked = scratch.err(&["launch", "mine", "--dry-run"]);
    assert!(blocked.contains(&path(&scratch.wad)), "{blocked}");
    assert!(blocked.contains("changed"), "{blocked}");
    let shown = scratch.ok(&["custom", "show", "mine"]);
    assert!(shown.contains(&path(&scratch.wad)), "{shown}");
    assert!(shown.contains("changed"), "{shown}");

    // `--accept-changes` re-records the hash, then the launch fails only on the
    // missing engine.
    let accepted = scratch.err(&["launch", "mine", "--dry-run", "--accept-changes"]);
    assert!(
        accepted.contains("DSDA-Doom is not installed"),
        "{accepted}"
    );
    assert!(
        scratch
            .ok(&["custom", "show", "mine"])
            .contains("files: ok")
    );

    // `remove-file` drops the record and never the file; `delete` removes DDL's
    // record even with `--delete-profiles`.
    let removed = scratch.ok(&["custom", "remove-file", "mine", "files", "0"]);
    assert!(removed.contains("0 file(s), 1 patch(es)"), "{removed}");
    assert!(scratch.wad.exists(), "the player's file must stay");
    let deleted = scratch.ok(&["custom", "delete", "mine", "--delete-profiles"]);
    assert!(deleted.contains("Deleted custom setup `mine`"), "{deleted}");
    assert!(scratch.ok(&["custom", "list"]).contains("No custom setups"));
    assert_eq!(fs::read(&scratch.wad).unwrap(), pwad(&["MAP02", "MAP03"]));
    assert!(
        scratch
            .err(&["custom", "show", "mine"])
            .contains("no custom setup")
    );

    // An identified IWAD variant needs no file at create time.
    let created = scratch.ok(&["custom", "create", "Identified", "--iwad", "doom2-1.9"]);
    assert!(
        created.contains("Created custom setup `identified`"),
        "{created}"
    );
}

#[test]
fn profile_extras_round_trip_warns_about_saves_and_keeps_files() {
    let scratch = Scratch::new();

    // The recipe is embedded; no install or engine is needed to edit extras.
    let listed = scratch.ok(&["profile", "extras", "sigil", "list"]);
    assert!(listed.contains("0 extra file(s)"), "{listed}");

    let added = scratch.ok(&[
        "profile",
        "extras",
        "sigil",
        "--profile",
        "Default",
        "add",
        &path(&scratch.wad),
    ]);
    assert!(added.contains("1 extra file(s)"), "{added}");
    assert!(added.contains("files (load order):"), "{added}");

    // A save slot in the profile's save directory triggers the R5 warning.
    let save_dir = scratch.home.join("data/ddl/profiles/sigil-default/save");
    fs::create_dir_all(&save_dir).unwrap();
    fs::write(save_dir.join("slot0.dsg"), b"save").unwrap();
    let output = scratch.ddl(&[
        "profile",
        "extras",
        "sigil",
        "--profile",
        "Default",
        "add",
        &path(&scratch.deh),
    ]);
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("may not load"), "{stderr}");

    let listed = scratch.ok(&["profile", "extras", "sigil", "list"]);
    assert!(listed.contains("2 extra file(s)"), "{listed}");
    assert!(listed.contains("files (load order):"), "{listed}");
    assert!(listed.contains("patches:"), "{listed}");

    // Removing drops the record only; the file stays on disk.
    let removed = scratch.ok(&["profile", "extras", "sigil", "remove", "files", "0"]);
    assert!(removed.contains("1 extra file(s)"), "{removed}");
    assert!(scratch.wad.exists());

    let cleared = scratch.ok(&["profile", "extras", "sigil", "clear"]);
    assert!(cleared.contains("0 extra file(s)"), "{cleared}");
    assert!(scratch.deh.exists());
    assert!(
        scratch
            .err(&["profile", "extras", "sigil", "remove", "files", "0"])
            .contains("past")
    );
}
