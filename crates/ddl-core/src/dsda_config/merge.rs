//! Three-way merge of the active DSDA configuration when the template version
//! changes (spec 005, R5).
//!
//! For every key the new template owns, DDL compares three values:
//!
//! | Symbol | Meaning |
//! | --- | --- |
//! | `B` | the baseline DDL last applied (`base/<old-version>.cfg`) |
//! | `A` | the value currently in the active file (DSDA may have rewritten it) |
//! | `T` | the value the new template wants |
//!
//! Rules, in order:
//!
//! 1. `A == B` -> take `T` (the user never changed it).
//! 2. `T == B` -> keep `A` (the user changed it, DDL did not).
//! 3. `A == T` -> keep `A` (nothing to do).
//! 4. `A` is absent -> take `T` (a partial active file must not lose the key).
//! 5. otherwise -> **conflict**: keep `A` and report it.
//!
//! Keys the user changed that the template does not own are always kept, and a
//! key removed from the template reverts to user ownership (it is left alone).

use super::format::{Document, Value};
use super::state::Conflict;
use super::template::Template;

/// The merged document plus what the merge decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeResult {
    /// The document to write as the new active file.
    pub document: Document,
    /// Template keys left at their active value because both sides changed them.
    pub conflicts: Vec<Conflict>,
    /// Template keys the user had changed (kept as-is this migration).
    pub user_changed: Vec<String>,
    /// Template keys the user had not touched (updated to the new value).
    pub updated: Vec<String>,
}

/// Merges `base` (last applied baseline), `active` (current file) and
/// `template` (new baseline) into the document DDL should write.
#[must_use]
pub fn merge(base: &Document, active: &Document, template: &Template) -> MergeResult {
    let mut document = active.clone();
    let mut conflicts = Vec::new();
    let mut user_changed = Vec::new();
    let mut updated = Vec::new();

    for (key, template_value) in &template.values {
        let wanted = Value::Int(*template_value);
        let current = active.get(key);
        let previous = base.get(key);

        let chosen = if current == previous {
            updated.push(key.clone());
            Some(wanted)
        } else if previous == Some(&wanted) {
            user_changed.push(key.clone());
            current.cloned()
        } else if current == Some(&wanted) {
            updated.push(key.clone());
            current.cloned()
        } else if current.is_none() {
            updated.push(key.clone());
            Some(wanted)
        } else {
            conflicts.push(Conflict {
                key: key.clone(),
                base: previous.map(Value::render),
                active: current.map(Value::render),
                template: Some(wanted.render()),
            });
            current.cloned()
        };

        match chosen {
            Some(value) => document.set(key, value),
            None => document.remove(key),
        }
    }

    MergeResult {
        document,
        conflicts,
        user_changed,
        updated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(version: &str, values: &[(&str, i64)]) -> Template {
        Template {
            template_version: version.to_owned(),
            dsda_version_min: "0.30.1".to_owned(),
            dsda_version_max: "0.30.1".to_owned(),
            values: values
                .iter()
                .map(|(key, value)| ((*key).to_owned(), *value))
                .collect(),
        }
    }

    fn document(text: &str) -> Document {
        Document::parse(text)
    }

    #[test]
    fn untouched_keys_take_the_template_value() {
        let base = document("keep 1\nchange 2\n");
        let active = document("keep 1\nchange 2\n");
        let result = merge(
            &base,
            &active,
            &template("2", &[("keep", 9), ("change", 4)]),
        );
        assert_eq!(result.document.get("keep"), Some(&Value::Int(9)));
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn user_only_changes_are_kept() {
        let base = document("sensitivity 10\n");
        let active = document("sensitivity 20\n");
        // Template unchanged for this key.
        let result = merge(&base, &active, &template("2", &[("sensitivity", 10)]));
        assert_eq!(result.document.get("sensitivity"), Some(&Value::Int(20)));
        assert_eq!(result.user_changed, vec!["sensitivity"]);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn template_only_changes_win() {
        let base = document("sensitivity 10\n");
        let active = document("sensitivity 10\n");
        let result = merge(&base, &active, &template("2", &[("sensitivity", 12)]));
        assert_eq!(result.document.get("sensitivity"), Some(&Value::Int(12)));
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn both_sides_changing_the_same_key_is_a_conflict_that_keeps_the_user_value() {
        let base = document("sensitivity 10\n");
        let active = document("sensitivity 20\n");
        let result = merge(&base, &active, &template("2", &[("sensitivity", 12)]));
        assert_eq!(result.document.get("sensitivity"), Some(&Value::Int(20)));
        assert_eq!(
            result.conflicts,
            vec![Conflict {
                key: "sensitivity".to_owned(),
                base: Some("10".to_owned()),
                active: Some("20".to_owned()),
                template: Some("12".to_owned()),
            }]
        );
    }

    #[test]
    fn keys_removed_from_the_template_revert_to_user_ownership() {
        let base = document("removed 1\nkept 2\n");
        let active = document("removed 7\nkept 2\n");
        let result = merge(&base, &active, &template("2", &[("kept", 3)]));
        assert_eq!(
            result.document.get("removed"),
            Some(&Value::Int(7)),
            "a key the template dropped is left exactly as the user had it"
        );
        assert_eq!(result.document.get("kept"), Some(&Value::Int(3)));
    }

    #[test]
    fn non_template_keys_are_always_kept() {
        let base = document("mystery 1\n");
        let active = document("mystery 99\n");
        let result = merge(&base, &active, &template("2", &[]));
        assert_eq!(result.document.get("mystery"), Some(&Value::Int(99)));
    }

    #[test]
    fn a_missing_active_key_takes_the_template_value() {
        let base = document("sensitivity 10\n");
        let active = document("# the key was dropped\n");
        let result = merge(&base, &active, &template("2", &[("sensitivity", 12)]));
        assert_eq!(result.document.get("sensitivity"), Some(&Value::Int(12)));
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn comments_and_unknown_lines_in_the_active_file_survive_the_merge() {
        let base = document("sensitivity 10\n");
        let active = document("# engine comment\nsensitivity 10\nunknown_key value\n");
        let result = merge(&base, &active, &template("2", &[("sensitivity", 12)]));
        let text = result.document.to_text();
        assert!(text.contains("# engine comment\n"));
        assert!(text.contains("unknown_key value\n"));
        assert!(text.contains("sensitivity 12\n"));
    }
}
