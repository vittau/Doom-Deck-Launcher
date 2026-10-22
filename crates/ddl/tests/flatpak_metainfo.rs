//! Keep the committed AppStream release in step with the workspace (spec 099).
use std::path::Path;

#[test]
fn newest_appstream_release_matches_workspace_version() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cargo = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    let workspace_package = cargo
        .split_once("[workspace.package]")
        .expect("workspace package section")
        .1;
    let version = workspace_package
        .lines()
        .take_while(|line| !line.starts_with('['))
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "version").then(|| value.trim().trim_matches('"'))
        })
        .expect("workspace package version");
    let metainfo = std::fs::read_to_string(
        root.join("packaging/flatpak/com.doomdecklauncher.DoomDeckLauncher.metainfo.xml"),
    )
    .unwrap();
    let newest = metainfo
        .split_once("<releases>")
        .expect("AppStream releases")
        .1
        .split_once("<release ")
        .expect("first AppStream release")
        .1
        .split_once('>')
        .expect("release opening tag")
        .0;
    let release_version = newest
        .split_once("version=\"")
        .expect("release version attribute")
        .1
        .split_once('"')
        .expect("closing version quote")
        .0;
    assert_eq!(
        release_version, version,
        "Update the checked-in AppStream release with scripts/flatpak/set-metainfo-release.py"
    );
}
