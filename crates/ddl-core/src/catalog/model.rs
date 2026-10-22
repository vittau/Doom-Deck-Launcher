use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub release: String,
    /// First public release date, with only the precision the evidence supports
    /// (spec 090, R3). Absent means unknown, not "January 1".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_release: Option<ReleaseDate>,
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

/// The first public release date of a mapset, carrying only the precision the
/// evidence supports (spec 090, R3). A missing `month` or `day` is unknown, never
/// January or the first of the month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseDate {
    pub year: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day: Option<u8>,
}

impl ReleaseDate {
    /// Ascending browsing key (spec 090, R2/R3): year first, then month, then day.
    /// A missing month or day sorts after the known values of its period, so a
    /// year-only date is never presented as an early-January release and a
    /// month-only date is never presented as its first day.
    #[must_use]
    pub const fn ordering_key(self) -> (u16, u8, u8) {
        let month = match self.month {
            Some(month) => month,
            None => 13,
        };
        let day = match self.day {
            Some(day) => day,
            None => 32,
        };
        (self.year, month, day)
    }

    /// The date as an ISO prefix with only the evidenced precision: `YYYY`,
    /// `YYYY-MM` or `YYYY-MM-DD`.
    #[must_use]
    pub fn iso(&self) -> String {
        match (self.month, self.day) {
            (Some(month), Some(day)) => format!("{:04}-{month:02}-{day:02}", self.year),
            (Some(month), None) => format!("{:04}-{month:02}", self.year),
            (None, _) => format!("{:04}", self.year),
        }
    }
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

impl Game {
    /// The number DSDA-Doom's `vanilla` word resolves to when this game's IWAD
    /// is loaded: 4 for TNT/Plutonia, 2 for Doom II, otherwise 3
    /// (`docs/research/dsda-engine.md` §1, D-017).
    #[must_use]
    pub const fn vanilla_complevel(self) -> u8 {
        match self {
            Self::Doom => 3,
            Self::Doom2 => 2,
            Self::Tnt | Self::Plutonia => 4,
        }
    }
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
    /// Another optional component that must be active with this one. Only one
    /// dependency level is supported (`requires` may not point at a component
    /// that itself requires another).
    #[serde(default)]
    pub requires: Option<String>,
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

impl ComplevelAlias {
    /// The alias as it is written in a recipe.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Boom => "boom",
            Self::Mbf => "mbf",
            Self::Mbf21 => "mbf21",
        }
    }

    /// The alias that names `number` when played on `game`'s IWAD, if any
    /// (D-030): 9 → `boom`, 11 → `mbf`, 21 → `mbf21`, and the game's own
    /// `vanilla` level (2 on Doom II, 3 on Doom, 4 on TNT/Plutonia).
    #[must_use]
    pub const fn naming(number: u8, game: Game) -> Option<Self> {
        match number {
            9 => Some(Self::Boom),
            11 => Some(Self::Mbf),
            21 => Some(Self::Mbf21),
            n if n == game.vanilla_complevel() => Some(Self::Vanilla),
            _ => None,
        }
    }
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
        self.validate_component_requirements()?;
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
        for component in &active {
            let Some(required) = &component.requires else {
                continue;
            };
            if !component.optional || component.replaces.is_some() {
                return Err(format!(
                    "component `{}` cannot depend on another component",
                    component.id
                ));
            }
            let target = self
                .components
                .iter()
                .find(|other| other.id == *required)
                .ok_or_else(|| {
                    format!(
                        "component `{}` requires unknown component `{required}`",
                        component.id
                    )
                })?;
            if !target.optional {
                return Err(format!(
                    "component `{}` requires required component `{required}`",
                    component.id
                ));
            }
            if target.requires.is_some() {
                return Err(format!(
                    "component `{}` uses an unsupported dependency chain",
                    component.id
                ));
            }
            if !active.iter().any(|other| other.id == target.id) {
                return Err(format!(
                    "component `{}` requires `{required}`",
                    component.id
                ));
            }
            if target.position >= component.position {
                return Err(format!(
                    "component `{}` must load after its prerequisite `{required}`",
                    component.id
                ));
            }
        }
        Ok(active)
    }

    /// Expands explicit optional component ids with their prerequisites and
    /// proves the result is a legal selection. A lone dependent enable may add
    /// its prerequisite, but contradictory explicit ids are rejected rather
    /// than resolved by order.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason for an unknown/required id, an
    /// unsupported dependency chain, or an illegal combination.
    pub fn explicit_selection(&self, requested: &[String]) -> Result<Vec<String>, String> {
        let expanded = self.expand_requirements(requested)?;
        self.check_selection(&expanded)?;
        Ok(expanded)
    }

    /// Applies one interactive on/off choice, keeping requirements and
    /// exclusive groups consistent. Unlike [`Self::explicit_selection`], this
    /// is intentionally order-aware because a UI action means "switch to this".
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason for an unknown/required id or an
    /// unsupported dependency chain; the input is never partially changed.
    pub fn apply_component_choice(
        &self,
        selected: &[String],
        id: &str,
        enabled: bool,
    ) -> Result<Vec<String>, String> {
        let component = self.optional_component(id)?;
        let mut next = selected.to_vec();
        if enabled {
            next.retain(|held| held != id);
            if let Some(required) = &component.requires {
                let target = self.optional_component(required)?;
                if target.requires.is_some() {
                    return Err(format!(
                        "component `{id}` uses an unsupported dependency chain"
                    ));
                }
                next = self.remove_exclusive_peers(&next, target);
                if !next.iter().any(|held| held == &target.id) {
                    next.push(target.id.clone());
                }
            }
            next = self.remove_exclusive_peers(&next, component);
            next.push(component.id.clone());
        } else {
            next = self.remove_with_dependents(&next, id);
        }
        let next = self.expand_requirements(&next)?;
        self.check_selection(&next)?;
        Ok(next)
    }

    fn validate_component_requirements(&self) -> Result<(), String> {
        for component in &self.components {
            let Some(required) = &component.requires else {
                continue;
            };
            if !component.optional || component.replaces.is_some() {
                return Err(format!(
                    "component `{}` cannot depend on another component",
                    component.id
                ));
            }
            let target = self
                .components
                .iter()
                .find(|other| other.id == *required)
                .ok_or_else(|| {
                    format!(
                        "component `{}` requires unknown component `{required}`",
                        component.id
                    )
                })?;
            if target.id == component.id || !target.optional {
                return Err(format!(
                    "component `{}` has an invalid prerequisite `{required}`",
                    component.id
                ));
            }
            if target.requires.is_some() {
                return Err(format!(
                    "component `{}` uses an unsupported dependency chain",
                    component.id
                ));
            }
            if target.position >= component.position {
                return Err(format!(
                    "component `{}` must load after its prerequisite `{required}`",
                    component.id
                ));
            }
        }
        Ok(())
    }

    fn optional_component<'a>(&'a self, id: &str) -> Result<&'a Component, String> {
        let component = self
            .components
            .iter()
            .find(|component| component.id == id)
            .ok_or_else(|| format!("unknown component `{id}`"))?;
        if !component.optional {
            return Err(format!(
                "component `{id}` is required and cannot be selected"
            ));
        }
        Ok(component)
    }

    fn expand_requirements(&self, requested: &[String]) -> Result<Vec<String>, String> {
        let mut expanded = Vec::with_capacity(requested.len());
        for id in requested {
            let component = self.optional_component(id)?;
            if component.replaces.is_some() && component.requires.is_some() {
                return Err(format!(
                    "component `{id}` cannot both replace main and require another component"
                ));
            }
            if let Some(required) = &component.requires {
                let target = self.optional_component(required)?;
                if target.requires.is_some() {
                    return Err(format!(
                        "component `{id}` uses an unsupported dependency chain"
                    ));
                }
                if target.position >= component.position {
                    return Err(format!(
                        "component `{id}` must load after its prerequisite `{required}`"
                    ));
                }
                if !expanded.iter().any(|held| held == &target.id) {
                    expanded.push(target.id.clone());
                }
            }
            if !expanded.iter().any(|held| held == id) {
                expanded.push(id.clone());
            }
        }
        Ok(expanded)
    }

    fn remove_exclusive_peers(&self, selected: &[String], component: &Component) -> Vec<String> {
        let Some(group) = &component.exclusive_group else {
            return selected.to_vec();
        };
        let mut next = selected.to_vec();
        for other in &self.components {
            if other.id != component.id && other.exclusive_group.as_ref() == Some(group) {
                next = self.remove_with_dependents(&next, &other.id);
            }
        }
        next
    }

    fn remove_with_dependents(&self, selected: &[String], id: &str) -> Vec<String> {
        let mut next = selected.to_vec();
        let mut pending = vec![id.to_owned()];
        let mut seen = std::collections::HashSet::new();
        while let Some(removed) = pending.pop() {
            if !seen.insert(removed.clone()) {
                continue;
            }
            next.retain(|held| held != &removed);
            for component in &self.components {
                if component.requires.as_deref() == Some(removed.as_str())
                    && !seen.contains(&component.id)
                {
                    pending.push(component.id.clone());
                }
            }
        }
        next
    }

    fn check_selection(&self, selected: &[String]) -> Result<(), String> {
        let enabled: Vec<&str> = selected.iter().map(String::as_str).collect();
        self.active_components(&enabled).map(|_| ())
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
