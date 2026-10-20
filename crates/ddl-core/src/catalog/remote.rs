//! Fetching the published catalog bundle (spec 019, R3).
//!
//! The app reads one document from the CI-maintained `catalog` branch. The
//! fetch runs through spec 006's hardened download stack (HTTPS only, host
//! allowlist, redirect limit, size-scaled deadline, hard size cap) with no
//! SHA-256 pin, because the bundle is not signed (spec 019 out of scope) — so
//! everything that arrives is untrusted input and is validated in
//! [`super::bundle`] before it can be used.
//!
//! [`RemoteFetch`] is the injection seam: production uses [`HttpsRemoteFetch`],
//! tests use a fake or a loopback server, and no test touches the network.

use std::path::PathBuf;

use crate::downloads::{CancellationToken, Fetcher, HostAllowlist};

use super::bundle::{
    AcceptedBundle, BundleRejection, MAX_BUNDLE_BYTES, accept_bundle, parse_bundle,
};

/// Where the published bundle lives (spec 019, R2). `raw.githubusercontent.com`
/// is already on the embedded download allowlist.
pub const BUNDLE_URL: &str =
    "https://raw.githubusercontent.com/vittau/Doom-Deck-Launcher/catalog/catalog.json";

/// The download boundary for the catalog bundle, so tests never touch a socket.
pub trait RemoteFetch {
    /// Fetches `url`, enforcing HTTPS, the host allowlist and `max_bytes`, and
    /// returns the document bytes.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason for a network, host, size or transport
    /// failure.
    fn fetch(
        &mut self,
        url: &str,
        max_bytes: u64,
        allowlist: &HostAllowlist,
    ) -> Result<Vec<u8>, String>;
}

/// The real boundary, backed by spec 006's [`Fetcher`].
pub struct HttpsRemoteFetch {
    cache_dir: PathBuf,
    #[cfg(any(test, feature = "test-support"))]
    allow_http: bool,
}

impl HttpsRemoteFetch {
    /// Creates a fetcher whose document is cached under `cache_dir/downloads`.
    #[must_use]
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            #[cfg(any(test, feature = "test-support"))]
            allow_http: false,
        }
    }

    /// Allows plain-HTTP URLs. Test-only: production is HTTPS-only.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn allow_http_for_tests(mut self) -> Self {
        self.allow_http = true;
        self
    }
}

impl RemoteFetch for HttpsRemoteFetch {
    fn fetch(
        &mut self,
        url: &str,
        max_bytes: u64,
        allowlist: &HostAllowlist,
    ) -> Result<Vec<u8>, String> {
        #[allow(unused_mut)]
        let mut fetcher = Fetcher::new(&self.cache_dir);
        #[cfg(any(test, feature = "test-support"))]
        {
            if self.allow_http {
                fetcher = fetcher.allow_http_for_tests();
            }
        }
        fetcher
            .download_document(url, max_bytes, allowlist.clone(), &CancellationToken::new())
            .map_err(|error| error.to_string())
    }
}

/// Why a catalog update did not happen (spec 019, R3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogUpdateError {
    /// The document could not be fetched (offline Deck, HTTP error, size cap).
    Fetch(String),
    /// The bundle was refused as a whole.
    Rejected(BundleRejection),
}

impl std::fmt::Display for CatalogUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fetch(detail) => write!(f, "could not fetch the catalog bundle: {detail}"),
            Self::Rejected(rejection) => write!(f, "{rejection}"),
        }
    }
}

impl std::error::Error for CatalogUpdateError {}

/// Fetches, parses and validates the published bundle (spec 019, R3).
///
/// `embedded_hosts_text` is the binary's own `hosts.toml`; `embedded_serial`
/// and `running_version` gate the bundle the same way the app does.
///
/// # Errors
///
/// Returns [`CatalogUpdateError::Fetch`] for a transport failure and
/// [`CatalogUpdateError::Rejected`] when the bundle is refused whole.
pub fn load_remote(
    fetch: &mut dyn RemoteFetch,
    url: &str,
    allowlist: &HostAllowlist,
    embedded_hosts_text: &str,
    embedded_serial: u64,
    running_version: &str,
) -> Result<AcceptedBundle, CatalogUpdateError> {
    let bytes = fetch
        .fetch(url, MAX_BUNDLE_BYTES, allowlist)
        .map_err(CatalogUpdateError::Fetch)?;
    let bundle = parse_bundle(&bytes).map_err(CatalogUpdateError::Rejected)?;
    accept_bundle(
        &bundle,
        embedded_hosts_text,
        embedded_serial,
        running_version,
    )
    .map_err(CatalogUpdateError::Rejected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::bundle::{BUNDLE_SCHEMA, Bundle};
    use crate::downloads::test_http::{Response, Server};
    use std::collections::BTreeMap;

    const HOSTS: &str = r#"
schema_version = 1
pages = ["www.doomworld.com"]
downloads = ["www.gamers.org"]
"#;

    fn recipe(id: &str, host: &str) -> String {
        format!(
            r#"
schema_version = 1
id = "{id}"
title = "Test {id}"
authors = ["Tester"]
release = "1.0"
page = "https://www.doomworld.com/{id}"
description = "A test mapset."
permission = "Free to redistribute."
base_game = {{ game = "doom2" }}
[[packages]]
id = "main"
page = "https://www.doomworld.com/{id}"
urls = ["https://{host}/{id}.zip"]
size = 4
sha256 = "{hash}"
kind = "zip"
[[packages.members]]
id = "main"
path = "{id}.wad"
size = 4
sha256 = "{hash}"
[[components]]
id = "main"
label = "Main"
description = "The mapset."
optional = false
default = true
position = 1
[[components.members]]
package = "main"
member = "main"
role = "maps"
[compatibility]
family = "vanilla"
complevel = "vanilla"
[compatibility.engine]
range = ">=0.30"
status = "inspected"
[verification]
status = "inspected"
tester = "Tester"
date = "2026-01-01"
engine_version = "0.30.1"
inspected = ["source page"]
notes = "Desk research only."
limitations = []
"#,
            hash = "0".repeat(64),
        )
    }

    fn document(serial: u64, host: &str) -> Vec<u8> {
        let mut files = BTreeMap::new();
        files.insert("hosts.toml".to_owned(), HOSTS.to_owned());
        files.insert("good.toml".to_owned(), recipe("good", host));
        let bundle = Bundle {
            bundle_schema: BUNDLE_SCHEMA,
            catalog_serial: serial,
            source_commit: "abc1234".to_owned(),
            min_ddl_version: "0.0.1".to_owned(),
            files,
        };
        serde_json::to_vec(&bundle).unwrap()
    }

    struct Fake(Vec<u8>);
    impl RemoteFetch for Fake {
        fn fetch(
            &mut self,
            _url: &str,
            _max_bytes: u64,
            _allowlist: &HostAllowlist,
        ) -> Result<Vec<u8>, String> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn a_valid_bundle_is_accepted_and_a_stale_one_is_not() {
        let allow = HostAllowlist::new(["raw.githubusercontent.com"]);
        let accepted = load_remote(
            &mut Fake(document(200, "www.gamers.org")),
            BUNDLE_URL,
            &allow,
            HOSTS,
            100,
            "0.0.4",
        )
        .expect("accepted");
        assert_eq!(accepted.serial(), 200);
        assert_eq!(accepted.catalog.ids().collect::<Vec<_>>(), vec!["good"]);

        let stale = load_remote(
            &mut Fake(document(50, "www.gamers.org")),
            BUNDLE_URL,
            &allow,
            HOSTS,
            100,
            "0.0.4",
        )
        .expect_err("stale");
        assert!(matches!(
            stale,
            CatalogUpdateError::Rejected(BundleRejection::NotNewer { .. })
        ));
    }

    #[test]
    fn malformed_unknown_schema_and_oversize_are_rejected() {
        let allow = HostAllowlist::new(["raw.githubusercontent.com"]);
        let malformed = load_remote(
            &mut Fake(b"{not json".to_vec()),
            BUNDLE_URL,
            &allow,
            HOSTS,
            100,
            "0.0.4",
        )
        .expect_err("malformed");
        assert!(matches!(
            malformed,
            CatalogUpdateError::Rejected(BundleRejection::Malformed { .. })
        ));

        let mut unknown = document(200, "www.gamers.org");
        let text = String::from_utf8(unknown)
            .unwrap()
            .replace("\"bundle_schema\":1", "\"bundle_schema\":99");
        unknown = text.into_bytes();
        let rejected = load_remote(&mut Fake(unknown), BUNDLE_URL, &allow, HOSTS, 100, "0.0.4")
            .expect_err("unknown schema");
        assert!(matches!(
            rejected,
            CatalogUpdateError::Rejected(BundleRejection::UnknownSchema { got: 99 })
        ));

        let oversized = vec![b' '; MAX_BUNDLE_BYTES as usize + 1];
        let rejected = load_remote(
            &mut Fake(oversized),
            BUNDLE_URL,
            &allow,
            HOSTS,
            100,
            "0.0.4",
        )
        .expect_err("oversize");
        assert!(matches!(
            rejected,
            CatalogUpdateError::Rejected(BundleRejection::Oversize { .. })
        ));
    }

    #[test]
    fn a_too_new_min_version_is_rejected() {
        let allow = HostAllowlist::new(["raw.githubusercontent.com"]);
        let mut files = BTreeMap::new();
        files.insert("good.toml".to_owned(), recipe("good", "www.gamers.org"));
        let bundle = Bundle {
            bundle_schema: BUNDLE_SCHEMA,
            catalog_serial: 200,
            source_commit: "abc1234".to_owned(),
            min_ddl_version: "9.9.9".to_owned(),
            files,
        };
        let rejected = load_remote(
            &mut Fake(serde_json::to_vec(&bundle).unwrap()),
            BUNDLE_URL,
            &allow,
            HOSTS,
            100,
            "0.0.4",
        )
        .expect_err("too new");
        assert!(matches!(
            rejected,
            CatalogUpdateError::Rejected(BundleRejection::TooNew { .. })
        ));
    }

    #[test]
    fn a_recipe_naming_a_disallowed_host_is_dropped_not_accepted() {
        let allow = HostAllowlist::new(["raw.githubusercontent.com"]);
        let accepted = load_remote(
            &mut Fake(document(200, "evil.example")),
            BUNDLE_URL,
            &allow,
            HOSTS,
            100,
            "0.0.4",
        )
        .expect("accepted whole");
        assert!(accepted.catalog.recipes.is_empty());
        assert_eq!(accepted.dropped().len(), 1);
    }

    #[test]
    fn the_real_fetch_runs_through_the_download_stack() {
        let body = document(200, "www.gamers.org");
        let served = body.clone();
        let server = Server::start(move |_req| Response::ok(served.clone()));
        let dir = tempfile::tempdir().unwrap();
        let mut fetch = HttpsRemoteFetch::new(dir.path()).allow_http_for_tests();
        let allow = HostAllowlist::new(["127.0.0.1"]);
        let accepted = load_remote(
            &mut fetch,
            &server.url("/catalog.json"),
            &allow,
            HOSTS,
            100,
            "0.0.4",
        )
        .expect("loopback fetch");
        assert_eq!(accepted.serial(), 200);
        assert_eq!(accepted.catalog.ids().collect::<Vec<_>>(), vec!["good"]);
    }

    #[test]
    fn a_host_outside_the_allowlist_never_reaches_the_recipe_check() {
        let body = document(200, "www.gamers.org");
        let served = body.clone();
        let server = Server::start(move |_req| Response::ok(served.clone()));
        let dir = tempfile::tempdir().unwrap();
        let mut fetch = HttpsRemoteFetch::new(dir.path()).allow_http_for_tests();
        let error = load_remote(
            &mut fetch,
            &server.url("/catalog.json"),
            &HostAllowlist::new(["raw.githubusercontent.com"]),
            HOSTS,
            100,
            "0.0.4",
        )
        .expect_err("host not allowed");
        assert!(matches!(error, CatalogUpdateError::Fetch(_)), "{error}");
    }
}
