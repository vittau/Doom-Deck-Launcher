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
//! 2. `B` is absent and `A` is the binding's `upstream` default -> take `T`
//!    (the template starts owning a binding the player never touched; spec 034).
//! 3. `T == B` -> keep `A` (the user changed it, DDL did not).
//! 4. `A == T` -> keep `A` (nothing to do).
//! 5. `A` is absent -> take `T` (a partial active file must not lose the key).
//! 6. otherwise -> **conflict**: keep `A` and report it.
//!
//! Rule 2 exists because DSDA rewrites every binding when it saves, so the first
//! template version that binds an action sees the engine's default in `A` while
//! `B` (written by the previous, binding-less template) has nothing. Comparing
//! against the researched v0.30.1 default tells "untouched" from "rebound in
//! DSDA's menu"; a rebound binding falls through to rule 6 and is kept.
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

    let wanted_entries = template
        .values
        .iter()
        .map(|(key, value)| (key.clone(), Value::Int(*value), None))
        .chain(
            template
                .strings
                .iter()
                .map(|(key, value)| (key.clone(), Value::Str(value.clone()), None)),
        )
        .chain(template.bindings.iter().map(|binding| {
            (
                binding.action.clone(),
                Value::Binding(binding.wanted.clone()),
                Some(Value::Binding(binding.upstream.clone())),
            )
        }));

    for (key, wanted, upstream) in wanted_entries {
        let current = active.get(&key);
        let previous = base.get(&key);

        let chosen = if current == previous {
            updated.push(key.clone());
            Some(wanted)
        } else if previous.is_none() && upstream.as_ref() == current {
            // The key is new to the template and still holds DSDA's own default.
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
            Some(value) => document.set(&key, value),
            None => document.remove(&key),
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
    use crate::dsda_config::format::{Binding, BindingProfile};
    use crate::dsda_config::template::TemplateBinding;

    fn template(version: &str, values: &[(&str, i64)]) -> Template {
        Template {
            template_version: version.to_owned(),
            dsda_version_min: "0.30.1".to_owned(),
            dsda_version_max: "0.30.1".to_owned(),
            values: values
                .iter()
                .map(|(key, value)| ((*key).to_owned(), *value))
                .collect(),
            strings: Vec::new(),
            bindings: Vec::new(),
        }
    }

    fn binding(joy: i64) -> Binding {
        let profile = BindingProfile {
            keys: vec![9],
            mouse: -1,
            joy,
        };
        Binding {
            profiles: vec![profile.clone(), profile.clone(), profile],
        }
    }

    fn binding_template(wanted: i64, upstream: i64) -> Template {
        let mut template = template("2", &[]);
        template.bindings.push(TemplateBinding {
            action: "input_map".to_owned(),
            wanted: binding(wanted),
            upstream: binding(upstream),
        });
        template
    }

    #[test]
    fn an_untouched_new_binding_takes_the_template_button() {
        // The previous template owned no bindings, so the base has nothing;
        // DSDA saved its own default. That is not a user change.
        let base = document("sensitivity 10\n");
        let active = document("input_map 9 -1 21 | 9 -1 21 | 9 -1 21\n");
        let result = merge(&base, &active, &binding_template(4, 21));
        assert_eq!(
            result
                .document
                .get("input_map")
                .map(Value::render)
                .as_deref(),
            Some("9 -1 4 | 9 -1 4 | 9 -1 4")
        );
        assert!(result.conflicts.is_empty());
        assert_eq!(result.updated, vec!["input_map"]);
    }

    #[test]
    fn a_new_binding_the_user_rebound_is_kept_and_reported() {
        let base = document("sensitivity 10\n");
        let active = document("input_map 9 -1 6 | 9 -1 6 | 9 -1 6\n");
        let result = merge(&base, &active, &binding_template(4, 21));
        assert_eq!(
            result
                .document
                .get("input_map")
                .map(Value::render)
                .as_deref(),
            Some("9 -1 6 | 9 -1 6 | 9 -1 6"),
            "the player's own button wins"
        );
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].key, "input_map");
    }

    #[test]
    fn a_user_changed_binding_survives_a_later_template_change() {
        let base = document("input_map 9 -1 21 | 9 -1 21 | 9 -1 21\n");
        let active = document("input_map 9 -1 6 | 9 -1 6 | 9 -1 6\n");
        let result = merge(&base, &active, &binding_template(4, 21));
        assert_eq!(
            result
                .document
                .get("input_map")
                .map(Value::render)
                .as_deref(),
            Some("9 -1 6 | 9 -1 6 | 9 -1 6")
        );
        assert_eq!(result.conflicts.len(), 1);
    }

    #[test]
    fn a_keyboard_change_to_a_new_binding_is_also_kept() {
        // The player changed only the keyboard key and left the pad button at
        // DSDA's default: the whole binding is a user choice.
        let base = document("sensitivity 10\n");
        let active = document("input_map 5 -1 21 | 5 -1 21 | 5 -1 21\n");
        let result = merge(&base, &active, &binding_template(4, 21));
        assert_eq!(
            result
                .document
                .get("input_map")
                .map(Value::render)
                .as_deref(),
            Some("5 -1 21 | 5 -1 21 | 5 -1 21")
        );
        assert_eq!(result.conflicts.len(), 1);
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
