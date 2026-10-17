//! Per-mapset publication record: one rename selects a fully verified generation.
use super::{InstallManifest, ManifestError};
use crate::catalog::Recipe;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// Versioned active install. Profiles remain in ProfileStore; pending ids make
/// reconciliation after publication recoverable across process interruption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveInstall {
    pub schema_version: u32,
    pub recipe: Recipe,
    pub manifest: InstallManifest,
    pub previous_recipe: Option<Recipe>,
    #[serde(default)]
    pub pending_profiles: Vec<String>,
    #[serde(default)]
    pub notices: Vec<String>,
    #[serde(default)]
    pub retired_dirs: Vec<PathBuf>,
}
impl ActiveInstall {
    pub fn load(path: &Path) -> Result<Option<Self>, ManifestError> {
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(ManifestError::Io(e)),
        };
        let mut bytes = Vec::new();
        file.take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(ManifestError::Io)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(ManifestError::TooLarge(16 * 1024 * 1024));
        }
        let held: Self =
            serde_json::from_slice(&bytes).map_err(|e| ManifestError::Parse(e.to_string()))?;
        if held.schema_version != 1 {
            return Err(ManifestError::UnsupportedSchema(held.schema_version));
        }
        if held.recipe.id != held.manifest.recipe_id || held.recipe.release != held.manifest.release
        {
            return Err(ManifestError::Parse(
                "active recipe and manifest disagree".into(),
            ));
        }
        Ok(Some(held))
    }
    pub fn save(&self, path: &Path) -> Result<(), ManifestError> {
        fs::create_dir_all(
            path.parent()
                .ok_or_else(|| ManifestError::Parse("missing active record parent".into()))?,
        )
        .map_err(ManifestError::Io)?;
        let temp = super::manifest::temporary_path(path);
        let write = || -> Result<(), ManifestError> {
            let mut file = fs::File::create(&temp).map_err(ManifestError::Io)?;
            let bytes =
                serde_json::to_vec_pretty(self).map_err(|e| ManifestError::Parse(e.to_string()))?;
            file.write_all(&bytes).map_err(ManifestError::Io)?;
            file.sync_all().map_err(ManifestError::Io)?;
            fs::rename(&temp, path).map_err(ManifestError::Io)
        };
        let result = write();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result
    }
}
