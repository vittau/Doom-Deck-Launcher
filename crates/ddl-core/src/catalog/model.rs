use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub release: String,
    pub page: String,
    pub description: String,
    pub permission: String,
    /// Reviewed category tags from `catalog/tags.toml` (spec 015). Absent = none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Bundled representative screenshots (spec 023, R2; D-024). Absent = the UI
    /// falls back to the painted concept art for this recipe.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screenshots: Vec<Screenshot>,
    #[serde(default)]
    pub images: Vec<Image>,
    pub base_game: BaseGame,
    pub packages: Vec<Package>,
    pub components: Vec<Component>,
    pub compatibility: Compatibility,
    pub verification: Verification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub path: String,
    pub permission: String,
}

/// One bundled screenshot a recipe references (spec 023, R2; D-024).
///
/// The image itself lives under `crates/ddl/ui/assets/screenshots/` and is
/// committed to this repository (D-024); `asset` names it relative to that
/// directory, so a community pull request can add a screenshot by dropping the
/// file, its `SOURCES.toml` entry and this reference. DDL never downloads a
/// screenshot at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Screenshot {
    /// Path under `crates/ddl/ui/assets/screenshots/`, e.g. `sigil/01.jpg`.
    pub asset: String,
    /// Short attribution the UI shows as the card/detail caption.
    pub credit: String,
}

/// `crates/ddl/ui/assets/screenshots/<recipe-id>/SOURCES.toml` (spec 023, R1):
/// the provenance and permission record kept beside the bundled images.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenshotSources {
    pub schema_version: u32,
    #[serde(default)]
    pub screenshot: Vec<ScreenshotSource>,
}

/// One `[[screenshot]]` entry in a `SOURCES.toml`: everything a reviewer needs
/// to trace an image back to where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenshotSource {
    /// File name inside the recipe's screenshot directory, e.g. `01.jpg`.
    pub file: String,
    /// Exact URL the committed bytes were originally retrieved from.
    pub source_url: String,
    /// Page a human can open to see the image in context.
    pub source_page: String,
    /// Who made/owns the image, for the credit caption.
    pub credit: String,
    /// Dimensions of the downloaded original before DDL's resize/crop.
    pub original_size: String,
    /// Retrieval date, `YYYY-MM-DD`.
    pub retrieved: String,
    /// Any permission/licence text found, or that none was found.
    pub permission: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseGame {
    pub game: Game,
    #[serde(default)]
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Doom,
    Doom2,
    Tnt,
    Plutonia,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub id: String,
    pub page: String,
    pub urls: Vec<String>,
    pub size: u64,
    pub sha256: String,
    pub kind: PackageKind,
    #[serde(default)]
    pub idgames_path: Option<String>,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
    Zip,
    Wad,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub id: String,
    pub path: String,
    pub size: u64,
    pub sha256: String,
    #[serde(default)]
    pub keep: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
    pub label: String,
    pub description: String,
    pub optional: bool,
    pub default: bool,
    pub position: u32,
    #[serde(default)]
    pub exclusive_group: Option<String>,
    /// A replacement suppresses main; it does not load alongside it.
    #[serde(default)]
    pub replaces: Option<String>,
    #[serde(default)]
    pub user_supplied: bool,
    #[serde(default)]
    pub obtain: Option<String>,
    #[serde(default)]
    pub known_sha256: Vec<String>,
    pub members: Vec<ComponentMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentMember {
    /// None only for the single user-imported WAD/patch.
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub member: Option<String>,
    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Maps,
    Music,
    Graphics,
    Sounds,
    Dehacked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    pub family: PortFamily,
    pub complevel: Complevel,
    pub complevel_status: EvidenceStatus,
    pub engine: EngineEvidence,
    #[serde(default)]
    pub runtime: RuntimeDefaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortFamily {
    Vanilla,
    LimitRemoving,
    Boom,
    Mbf,
    Mbf21,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Complevel {
    Number(u8),
    Alias(ComplevelAlias),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplevelAlias {
    Vanilla,
    Boom,
    Mbf,
    Mbf21,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceStatus {
    Confirmed,
    Proposed,
    NeedsDeckValidation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineEvidence {
    /// A semver requirement, not a claim of support beyond the recorded evidence.
    pub range: String,
    pub status: VerificationStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDefaults {
    /// DSDA -warp integers (map, or episode/map); no free-form argv.
    #[serde(default)]
    pub warp: Option<Vec<u8>>,
    /// Suggestion only; the player's profile owns the actual setting.
    #[serde(default)]
    pub pistol_start: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    Indexed,
    Inspected,
    Verified,
    Partial,
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub status: VerificationStatus,
    pub tester: String,
    pub date: String,
    pub engine_version: String,
    pub inspected: Vec<String>,
    pub notes: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hosts {
    pub schema_version: u32,
    pub pages: Vec<String>,
    pub downloads: Vec<String>,
}

/// The reviewed tag vocabulary, `catalog/tags.toml` (spec 015, R1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagVocabulary {
    pub schema_version: u32,
    #[serde(default)]
    pub tags: Vec<TagDef>,
}

/// One tag: a stable id, an English label (the UI translates by id), a short
/// description and the position of its category in Discover.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagDef {
    pub id: String,
    pub label: String,
    pub description: String,
    pub order: u32,
}

impl TagVocabulary {
    /// The definition with this id, if any.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&TagDef> {
        self.tags.iter().find(|tag| tag.id == id)
    }

    /// Definitions sorted by `order` (ties by id), the Discover strip order.
    #[must_use]
    pub fn ordered(&self) -> Vec<&TagDef> {
        let mut tags: Vec<_> = self.tags.iter().collect();
        tags.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.id.cmp(&b.id)));
        tags
    }
}

/// Symbolic order only: downstream installation/resolution maps these refs to files.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct LoadOrder<'a> {
    pub files: Vec<&'a ComponentMember>,
    pub deh: Vec<&'a ComponentMember>,
}

impl Recipe {
    /// The ordered components for one selection, honouring replacement and
    /// exclusive groups. `enabled` contains optional component ids; main is
    /// implicitly required and is dropped when exactly one enabled component
    /// replaces it.
    ///
    /// `catalog::validate` proves these invariants for every allowed selection,
    /// but this method still checks them, so a hand-built or stale recipe can
    /// only fail with a message rather than produce an ambiguous launch.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason for a missing main, an unknown or
    /// duplicated optional component, two replacements, two components of one
    /// exclusive group, or two co-enabled components at the same position.
    pub fn active_components<'a>(&'a self, enabled: &[&str]) -> Result<Vec<&'a Component>, String> {
        let main = self
            .components
            .iter()
            .find(|c| c.id == "main")
            .ok_or("declare a required main component")?;
        let mut active = vec![main];
        for id in enabled {
            let c = self
                .components
                .iter()
                .find(|c| c.id == *id && c.optional)
                .ok_or_else(|| format!("unknown optional component `{id}`"))?;
            if active.contains(&c) {
                return Err(format!("duplicate enabled component `{id}`"));
            }
            active.push(c);
        }
        let replacements = active.iter().filter(|c| c.replaces.is_some()).count();
        if replacements > 1 {
            return Err("choose at most one replacement for main".into());
        }
        if replacements == 1 {
            active.retain(|c| c.id != "main");
        }
        let mut groups = std::collections::HashSet::new();
        for c in &active {
            if let Some(group) = &c.exclusive_group
                && !groups.insert(group)
            {
                return Err(format!("choose at most one component in group `{group}`"));
            }
        }
        active.sort_by_key(|c| c.position);
        if active.windows(2).any(|cs| cs[0].position == cs[1].position) {
            return Err("give co-enabled components distinct load positions".into());
        }
        Ok(active)
    }

    /// The optional component ids a fresh profile enables by default (spec 017,
    /// R5). Lint proves this combination is legal (`load_order`).
    #[must_use]
    pub fn default_components(&self) -> Vec<String> {
        self.components
            .iter()
            .filter(|c| c.optional && c.default)
            .map(|c| c.id.clone())
            .collect()
    }

    /// Checks one selection; validation proves the invariants for every selection.
    /// `enabled` contains optional component ids; main is implicitly required.
    pub fn load_order<'a>(&'a self, enabled: &[&str]) -> Result<LoadOrder<'a>, String> {
        let mut order = LoadOrder::default();
        for component in self.active_components(enabled)? {
            for member in &component.members {
                if member.role == Role::Dehacked {
                    order.deh.push(member);
                } else {
                    order.files.push(member);
                }
            }
        }
        Ok(order)
    }
}
