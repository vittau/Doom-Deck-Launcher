//! Display-name validation shared by profile editing and persistence.
use super::Profile;

/// Trim and validate a proposed name without changing profile identity or data.
/// `editing` excludes the profile being renamed from the uniqueness check.
///
/// # Errors
/// Returns an inline-ready reason for an empty, overlong or duplicate name.
pub fn validate_name(
    name: &str,
    recipe_id: &str,
    editing: Option<&str>,
    profiles: &[Profile],
) -> Result<String, &'static str> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Enter a profile name.");
    }
    if name.chars().count() > 32 {
        return Err("Use 1–32 characters.");
    }
    if profiles.iter().any(|p| {
        p.recipe_id == recipe_id
            && Some(p.id.as_str()) != editing
            && p.name.to_lowercase() == name.to_lowercase()
    }) {
        return Err("A profile with this name already exists.");
    }
    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trims_counts_characters_and_enforces_mapset_uniqueness() {
        let profiles = [Profile::new("one", "Casual", "map")];
        assert_eq!(
            validate_name("  Night run  ", "map", None, &profiles),
            Ok("Night run".into())
        );
        assert!(validate_name(" \t", "map", None, &profiles).is_err());
        assert!(validate_name(&"é".repeat(32), "map", None, &profiles).is_ok());
        assert!(validate_name(&"é".repeat(33), "map", None, &profiles).is_err());
        assert!(validate_name("casUAL", "map", None, &profiles).is_err());
        assert!(validate_name("casUAL", "other", None, &profiles).is_ok());
        assert!(validate_name("Casual", "map", Some("one"), &profiles).is_ok());
    }
    #[test]
    fn legacy_preset_names_round_trip_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let store =
            super::super::ProfileStore::new(dir.path().join("config"), dir.path().join("data"));
        for (n, name) in [
            "Default",
            "Casual",
            "UV",
            "Pistol start",
            "Co-op",
            "Speedrun",
        ]
        .iter()
        .enumerate()
        {
            let profile = Profile::new(format!("p{n}"), *name, "map");
            store.save(&profile).unwrap();
            assert_eq!(store.load(&profile.id).unwrap(), profile);
        }
    }
}
