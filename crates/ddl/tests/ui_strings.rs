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
                || (matches!(value.as_str(), "x" | "y") && before.ends_with("event.text =="));
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
