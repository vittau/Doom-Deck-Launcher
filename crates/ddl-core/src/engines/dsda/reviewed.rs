//! The reviewed DSDA-Doom list (spec 004, R1/R9; decision D-007).
//!
//! [`ReviewedList`] is the parsed, validated form of
//! `crates/ddl-core/data/engines.toml`. Each entry pins one upstream release:
//! its tag, the exact Linux x86-64 AppImage asset name, the asset size and
//! SHA-256, whether it has passed Deck validation, and the attribution data the
//! Settings screen shows (licence, release page, tagged source tree).
//!
//! Only releases in this list are offered by default ([`Availability::Reviewed`]
//! in [`super::discovery`]); anything else is an explicit, unreviewed opt-in.
//!
//! The type is deliberately strict: loading rejects a wrong schema version, an
//! empty list, duplicate tags, a tag that is not `v` + a path-safe version, a
//! non-bare asset name, a zero size, a malformed hash and a missing/unattributed
//! URL. The `validated` flag defaults to `false`, so an entry never claims Deck
//! validation it has not recorded.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::downloads::Sha256;

/// The reviewed list embedded in the binary.
pub const EMBEDDED_LIST: &str = include_str!("../../../data/engines.toml");

/// Schema version this build understands.
pub const SCHEMA_VERSION: u32 = 1;

/// Base URL of the official release downloads. Asset URLs are
/// `<base>/<tag>/<asset>`.
pub const RELEASE_DOWNLOAD_BASE: &str = "https://github.com/dsda-org/dsda-doom/releases/download";

/// Base URL of the upstream tag pages and source trees (`<base>/<tag>`).
pub const RELEASE_TAG_BASE: &str = "https://github.com/dsda-org/dsda-doom/releases/tag";

/// Base URL of the tagged source trees (`<base>/<tag>`).
pub const SOURCE_TREE_BASE: &str = "https://github.com/dsda-org/dsda-doom/tree";

/// One reviewed release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedVersion {
    /// Upstream git tag, e.g. `v0.30.1`.
    pub tag: String,
    /// Exact Linux x86-64 AppImage asset name for `tag`.
    pub asset: String,
    /// Asset size in bytes.
    pub size: u64,
    /// SHA-256 of the asset, 64 lowercase hex digits.
    pub sha256: String,
    /// Whether this version passed the Deck validation (spec 004 A5).
    #[serde(default)]
    pub validated: bool,
    /// Upstream licence of the engine payload.
    pub license: String,
    /// GitHub release page (attribution).
    pub release_url: String,
    /// GitHub tagged source tree (attribution).
    pub source_url: String,
}

impl ReviewedVersion {
    /// The version without the leading `v`, e.g. `0.30.1`. Also the directory
    /// name DDL installs the engine into.
    #[must_use]
    pub fn version(&self) -> &str {
        self.tag.strip_prefix('v').unwrap_or(&self.tag)
    }

    /// The parsed SHA-256.
    ///
    /// The value is validated at load time, so this cannot fail for a list that
    /// came from [`ReviewedList`].
    #[must_use]
    pub fn sha256(&self) -> Sha256 {
        Sha256::parse_hex(&self.sha256).expect("validated at load time")
    }

    /// The asset download URL for this version.
    #[must_use]
    pub fn download_url(&self) -> String {
        format!("{RELEASE_DOWNLOAD_BASE}/{}/{}", self.tag, self.asset)
    }
}

/// The parsed reviewed list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedList {
    /// Schema version of the source document.
    pub schema_version: u32,
    /// One entry per reviewed release, in table order (newest first).
    #[serde(rename = "version", default)]
    pub versions: Vec<ReviewedVersion>,
}

/// Why loading or validating the reviewed list failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListError {
    /// The TOML could not be parsed.
    Parse(String),
    /// The document declares a schema version this build does not know.
    UnsupportedSchema(u32),
    /// The list contains no entries.
    Empty,
    /// Two entries share a tag.
    DuplicateTag(String),
    /// A field of an entry is missing or malformed.
    Invalid {
        /// The tag (or a placeholder when the tag itself is bad).
        tag: String,
        /// The field name.
        field: &'static str,
        /// What is wrong.
        detail: String,
    },
}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(message) => write!(f, "engines.toml is not valid TOML: {message}"),
            Self::UnsupportedSchema(version) => write!(
                f,
                "engines.toml schema_version {version} is not supported (expected {SCHEMA_VERSION})"
            ),
            Self::Empty => f.write_str("engines.toml defines no reviewed versions"),
            Self::DuplicateTag(tag) => write!(f, "engines.toml defines the tag `{tag}` twice"),
            Self::Invalid { tag, field, detail } => {
                write!(f, "engines.toml version `{tag}` field `{field}`: {detail}")
            }
        }
    }
}

impl std::error::Error for ListError {}

impl ReviewedList {
    /// Parses and validates a reviewed-list document.
    ///
    /// # Errors
    ///
    /// Returns [`ListError`] when the TOML is malformed or fails validation
    /// (see the module docs for the rules).
    pub fn parse(text: &str) -> Result<Self, ListError> {
        let list: Self =
            toml::from_str(text).map_err(|error| ListError::Parse(error.to_string()))?;
        list.validate()?;
        Ok(list)
    }

    /// Loads the reviewed list embedded in the binary.
    ///
    /// # Panics
    ///
    /// Panics only if the embedded data is invalid, which a unit test forbids.
    #[must_use]
    pub fn embedded() -> Self {
        Self::parse(EMBEDDED_LIST).expect("embedded engines.toml is valid")
    }

    /// Validates an already-deserialized list.
    ///
    /// # Errors
    ///
    /// Returns [`ListError`] describing the first problem found.
    pub fn validate(&self) -> Result<(), ListError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ListError::UnsupportedSchema(self.schema_version));
        }
        if self.versions.is_empty() {
            return Err(ListError::Empty);
        }
        let mut seen: Vec<&str> = Vec::with_capacity(self.versions.len());
        for version in &self.versions {
            if seen.contains(&version.tag.as_str()) {
                return Err(ListError::DuplicateTag(version.tag.clone()));
            }
            seen.push(&version.tag);
            version.validate()?;
        }
        Ok(())
    }

    /// The entry with this upstream tag, if any.
    #[must_use]
    pub fn get_by_tag(&self, tag: &str) -> Option<&ReviewedVersion> {
        self.versions.iter().find(|version| version.tag == tag)
    }

    /// The entry whose version (tag without `v`) equals `version`, if any.
    #[must_use]
    pub fn get_by_version(&self, version: &str) -> Option<&ReviewedVersion> {
        self.versions
            .iter()
            .find(|entry| entry.version() == version)
    }

    /// Whether `tag` is on the reviewed list.
    #[must_use]
    pub fn contains_tag(&self, tag: &str) -> bool {
        self.get_by_tag(tag).is_some()
    }

    /// The reviewed entries, newest first (table order).
    pub fn iter(&self) -> impl Iterator<Item = &ReviewedVersion> {
        self.versions.iter()
    }

    /// The most recently released reviewed version (first table entry).
    #[must_use]
    pub fn newest(&self) -> &ReviewedVersion {
        self.versions
            .first()
            .expect("a validated list is never empty")
    }
}

impl ReviewedVersion {
    fn validate(&self) -> Result<(), ListError> {
        let invalid = |field: &'static str, detail: String| ListError::Invalid {
            tag: self.tag.clone(),
            field,
            detail,
        };

        let Some(version) = self.tag.strip_prefix('v') else {
            return Err(invalid(
                "tag",
                "must start with `v` (e.g. `v0.30.1`)".to_owned(),
            ));
        };
        if !is_safe_version(version) {
            return Err(invalid(
                "tag",
                "must be `v` followed by a path-safe version of letters, digits, '-', '_' or '.'"
                    .to_owned(),
            ));
        }
        if !is_bare_filename(&self.asset) {
            return Err(invalid(
                "asset",
                "must be a bare file name without path separators".to_owned(),
            ));
        }
        if self.size == 0 {
            return Err(invalid("size", "must be greater than zero".to_owned()));
        }
        if !is_lower_hex_64(&self.sha256) {
            return Err(invalid(
                "sha256",
                "must be 64 lowercase hex digits".to_owned(),
            ));
        }
        if self.license.trim().is_empty() {
            return Err(invalid(
                "license",
                "must name the upstream licence".to_owned(),
            ));
        }
        for (field, url) in [
            ("release_url", &self.release_url),
            ("source_url", &self.source_url),
        ] {
            if !is_https_url(url) {
                return Err(invalid(
                    field,
                    "must be an absolute https URL with a host".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

/// `true` when `version` is a non-empty, path-safe single token.
fn is_safe_version(version: &str) -> bool {
    !version.is_empty()
        && version != "."
        && version != ".."
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

/// `true` when `name` is a bare file name with no directory components.
fn is_bare_filename(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\'])
        && !name.contains('\0')
}

/// `true` when `value` is exactly 64 lowercase hex digits.
fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// `true` when `value` parses as an absolute `https` URL with a host.
fn is_https_url(value: &str) -> bool {
    url::Url::parse(value)
        .map(|url| url.scheme() == "https" && url.host_str().is_some())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_list_parses_and_matches_the_research() {
        let list = ReviewedList::embedded();
        assert_eq!(list.schema_version, SCHEMA_VERSION);
        assert!(!list.versions.is_empty());

        let seed = list.get_by_tag("v0.30.1").expect("seed entry present");
        assert_eq!(seed.version(), "0.30.1");
        assert_eq!(seed.asset, "dsda-doom-0.30.1-linux-x86_64.appimage");
        assert_eq!(seed.size, 14_608_888);
        assert_eq!(
            seed.sha256(),
            Sha256::parse_hex("b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0")
                .unwrap()
        );
        assert!(!seed.validated, "Deck validation (A5) has not happened yet");
        assert_eq!(seed.license, "GPL-2.0-or-later");
        assert_eq!(
            seed.download_url(),
            "https://github.com/dsda-org/dsda-doom/releases/download/v0.30.1/dsda-doom-0.30.1-linux-x86_64.appimage"
        );
    }

    #[test]
    fn lookups_work_by_tag_and_version() {
        let list = ReviewedList::embedded();
        assert!(list.contains_tag("v0.30.1"));
        assert!(!list.contains_tag("v0.30.0"));
        assert_eq!(
            list.get_by_version("0.30.1").map(|v| v.tag.clone()),
            Some("v0.30.1".to_owned())
        );
        assert!(list.get_by_version("9.9.9").is_none());
    }

    #[test]
    fn duplicate_tags_are_rejected() {
        let text = r#"
schema_version = 1

[[version]]
tag = "v1.0.0"
asset = "a.appimage"
size = 1
sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
license = "GPL-2.0-or-later"
release_url = "https://example.invalid/r"
source_url = "https://example.invalid/s"

[[version]]
tag = "v1.0.0"
asset = "b.appimage"
size = 2
sha256 = "1111111111111111111111111111111111111111111111111111111111111111"
license = "GPL-2.0-or-later"
release_url = "https://example.invalid/r"
source_url = "https://example.invalid/s"
"#;
        assert_eq!(
            ReviewedList::parse(text),
            Err(ListError::DuplicateTag("v1.0.0".to_owned()))
        );
    }

    #[test]
    fn a_tag_without_v_is_rejected() {
        let text = minimal("0.30.1", "a.appimage", 1);
        let error = ReviewedList::parse(&text).expect_err("tag");
        assert!(matches!(error, ListError::Invalid { field: "tag", .. }));
    }

    #[test]
    fn an_unsafe_version_is_rejected() {
        let text = minimal("v../escape", "a.appimage", 1);
        let error = ReviewedList::parse(&text).expect_err("path traversal");
        assert!(matches!(error, ListError::Invalid { field: "tag", .. }));
    }

    #[test]
    fn a_non_https_attribution_url_is_rejected() {
        let text = minimal("v1.0.0", "a.appimage", 1)
            .replace("https://example.invalid/s", "http://example.invalid/s");
        let error = ReviewedList::parse(&text).expect_err("plain http");
        assert!(matches!(
            error,
            ListError::Invalid {
                field: "source_url",
                ..
            }
        ));
    }

    #[test]
    fn a_zero_size_is_rejected() {
        let text = minimal("v1.0.0", "a.appimage", 0);
        let error = ReviewedList::parse(&text).expect_err("zero size");
        assert!(matches!(error, ListError::Invalid { field: "size", .. }));
    }

    #[test]
    fn wrong_schema_and_empty_list_are_rejected() {
        assert_eq!(
            ReviewedList::parse("schema_version = 99\n"),
            Err(ListError::UnsupportedSchema(99))
        );
        assert_eq!(
            ReviewedList::parse("schema_version = 1\n"),
            Err(ListError::Empty)
        );
    }

    /// A minimal one-entry document for the rejection tests.
    fn minimal(tag: &str, asset: &str, size: u64) -> String {
        format!(
            r#"
schema_version = 1

[[version]]
tag = "{tag}"
asset = "{asset}"
size = {size}
sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
license = "GPL-2.0-or-later"
release_url = "https://example.invalid/r"
source_url = "https://example.invalid/s"
"#
        )
    }
}
