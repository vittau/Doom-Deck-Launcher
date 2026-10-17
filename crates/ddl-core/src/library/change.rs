//! Pure comparison of installed content and the catalog (spec 020).
use super::InstallManifest;
use crate::catalog::Recipe;
use crate::profiles::Profile;

/// Independent changes may occur together; metadata-only excludes all three.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Changes {
    pub launch: bool,
    pub content: bool,
    pub added_components: Vec<String>,
    pub removed_components: Vec<String>,
}
impl Changes {
    #[must_use]
    pub fn components(&self) -> bool {
        !self.added_components.is_empty() || !self.removed_components.is_empty()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeStatus {
    Current,
    MetadataOnly,
    Update(Changes),
    Removed,
}
impl RecipeStatus {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::MetadataOnly => "Metadata only",
            Self::Removed => "No longer in the catalog",
            Self::Update(_) => "Update available",
        }
    }
    #[must_use]
    pub fn summary(&self, old: &Recipe, current: Option<&Recipe>) -> String {
        let (Self::Update(changes), Some(new)) = (self, current) else {
            return self.label().into();
        };
        let mut parts = Vec::new();
        if changes.content {
            if old.release != new.release {
                parts.push(format!("New release {}", new.release));
            } else {
                parts.push("Content changed".into());
            }
        }
        if changes.launch {
            if old.compatibility.complevel != new.compatibility.complevel {
                let level = |r: &Recipe| match &r.compatibility.complevel {
                    crate::catalog::Complevel::Number(n) => n.to_string(),
                    crate::catalog::Complevel::Alias(a) => format!("{a:?}").to_lowercase(),
                };
                parts.push(format!("complevel {} → {}", level(old), level(new)));
            } else {
                parts.push("Launch settings changed".into());
            }
        }
        for id in &changes.added_components {
            parts.push(format!("New add-on: {id}"));
        }
        for id in &changes.removed_components {
            parts.push(format!("Removed add-on: {id}"));
        }
        parts.join(" · ")
    }
}

/// `installed` is the frozen recipe used to create `manifest` (D-019).
/// No disk access, network, or profile mutation occurs here.
#[must_use]
pub fn classify(
    manifest: &InstallManifest,
    installed: &Recipe,
    current: Option<&Recipe>,
) -> RecipeStatus {
    let Some(new) = current else {
        return RecipeStatus::Removed;
    };
    if installed == new {
        return RecipeStatus::Current;
    }
    let mut changes = Changes::default();
    changes.content = manifest.release != new.release
        || content(installed) != content(new)
        || manifest.packages.iter().any(|held| {
            !new.packages
                .iter()
                .any(|p| p.id == held.id && p.sha256 == held.sha256 && p.size == held.size)
        });
    changes.launch = installed.base_game != new.base_game
        || installed.compatibility.complevel != new.compatibility.complevel
        || installed.compatibility.engine != new.compatibility.engine
        || installed.compatibility.runtime.warp != new.compatibility.runtime.warp;
    for c in &installed.components {
        match new.components.iter().find(|n| n.id == c.id) {
            None if c.optional => changes.removed_components.push(c.id.clone()),
            None => changes.launch = true,
            Some(n) => {
                if c.position != n.position
                    || c.members != n.members
                    || c.replaces != n.replaces
                    || c.exclusive_group != n.exclusive_group
                    || c.user_supplied != n.user_supplied
                    || c.optional != n.optional
                    || c.default != n.default
                {
                    changes.launch = true;
                }
            }
        }
    }
    for c in &new.components {
        if c.optional && !installed.components.iter().any(|old| old.id == c.id) {
            changes.added_components.push(c.id.clone());
        }
    }
    if changes.launch || changes.content || changes.components() {
        RecipeStatus::Update(changes)
    } else {
        RecipeStatus::MetadataOnly
    }
}
fn content(
    recipe: &Recipe,
) -> Vec<(
    &str,
    &str,
    u64,
    crate::catalog::PackageKind,
    &[crate::catalog::Member],
)> {
    recipe
        .packages
        .iter()
        .map(|p| {
            (
                p.id.as_str(),
                p.sha256.as_str(),
                p.size,
                p.kind,
                p.members.as_slice(),
            )
        })
        .collect()
}

/// Preserve existing choices, drop removed ones, and apply defaults only to new add-ons.
/// Runtime save/data/shot directories are deliberately outside this pure operation.
pub fn reconcile_profile(profile: &mut Profile, old: &Recipe, new: &Recipe) -> Vec<String> {
    let dropped: Vec<_> = profile
        .components
        .iter()
        .filter(|id| !new.components.iter().any(|c| c.optional && &c.id == *id))
        .cloned()
        .collect();
    profile.components.retain(|id| !dropped.contains(id));
    for c in &new.components {
        if c.optional
            && c.default
            && !old.components.iter().any(|held| held.id == c.id)
            && !profile.components.iter().any(|id| {
                new.components.iter().any(|held| {
                    &held.id == id
                        && (held.replaces.is_some() && c.replaces.is_some()
                            || c.exclusive_group.is_some()
                                && held.exclusive_group == c.exclusive_group)
                })
            })
        {
            profile.set_component(&c.id, true);
        }
    }
    dropped
}
