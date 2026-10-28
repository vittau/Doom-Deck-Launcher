//! UI resource lint: new visible literals must enter Slint's translation resources.
use std::path::Path;

fn literals(source: &str) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    let mut chars = source.char_indices().peekable();
    while let Some((offset, ch)) = chars.next() {
        if ch == '/' && chars.peek().is_some_and(|(_, c)| *c == '/') {
            for (_, c) in chars.by_ref() {
                if c == '\n' {
                    break;
                }
            }
        } else if ch == '"' {
            let mut value = String::new();
            while let Some((_, c)) = chars.next() {
                if c == '"' {
                    break;
                }
                if c == '\\' {
                    if let Some((_, c)) = chars.next() {
                        value.push(c);
                    }
                } else {
                    value.push(c);
                }
            }
            result.push((offset, value));
        }
    }
    result
}
#[test]
fn visible_slint_literals_are_translatable() {
    let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    for entry in std::fs::read_dir(ui).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "slint") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for (offset, value) in literals(&source) {
            let before = source[..offset].trim_end();
            let resource = before.ends_with("@tr(");
            let noncopy = value.is_empty()
                || matches!(value.as_str(), "Inter Variable" | "Barlow" | "Pirata One")
                || before.ends_with("import")
                || before.ends_with("from")
                || before.ends_with("@image-url(")
                // Spec 116 R10: vector geometry for the UV-Max trophy is data,
                // not player copy.
                || before.ends_with("commands:")
                || (matches!(value.as_str(), "x" | "y" | "v" | "[" | "]")
                    && before.ends_with("event.text =="));
            assert!(
                resource || noncopy,
                "{}: untranslated literal {value:?}",
                path.display()
            );
        }
    }
    // The view builder may carry proper names and numbered filenames, but no English UI copy.
    // Fixtures are the separate, documented content-data boundary.
    let source = include_str!("../src/shell.rs");
    let data = [
        "{} / {}",
        "{}…{}",
        "SIGIL",
        "John Romero",
        "SIGIL · John Romero",
        "AUGER;ZENITH · DBP 37",
        "✓",
        "1  {}",
        "2  {f}",
    ];
    for (_, value) in literals(source) {
        assert!(
            data.contains(&value.as_str()),
            "UI copy bypasses @tr: {value}"
        );
    }
}

/// Spec 033: decorative subtitles, taglines and caption plates stay out of the UI.
#[test]
fn removed_decorative_copy_does_not_return() {
    let removed = [
        "Curated worlds",
        "Choose your next descent",
        "Requirements, content and source",
        "Your last profile, ready again",
        "Build your library and begin a new session",
        "Installed worlds, ready for your next session",
        "Review components and gameplay before launching",
        "Manage your engine, base games and settings",
        "Prepare the engine and bring your own base games",
        "Review the error and choose your next step",
        "Your next mapset starts here",
        "Original concept art",
        "Screenshot: {}",
        "Original game",
    ];
    let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    for entry in std::fs::read_dir(ui).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "slint") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for text in removed {
            assert!(
                !source.contains(text),
                "{}: removed copy {text:?} is back",
                path.display()
            );
        }
    }
}

/// Spec 045: project validation status belongs in docs, not player copy.
#[test]
fn project_process_labels_do_not_return() {
    let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    for entry in std::fs::read_dir(ui).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "slint") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for (_, value) in literals(&source) {
            let value = value.to_lowercase();
            for removed in [
                "verification pending",
                "needs deck validation",
                "research reference",
                "not yet verified",
            ] {
                assert!(
                    !value.contains(removed),
                    "{}: project-process copy {removed:?} is back",
                    path.display()
                );
            }
        }
    }
}
