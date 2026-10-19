//! The host allowlist that bounds where a download may go (spec 006, R1).
//!
//! Recipes and the engine list name explicit sources; the fetcher is only
//! allowed to contact those hosts (D-010, D-015). Matching is an exact,
//! case-insensitive comparison of the DNS host, with no wildcard or suffix
//! matching: `example.com` never permits `evil.example.com`.
//!
//! The **one** named exception is Dropbox's per-request download host
//! (spec 038, R1): `www.dropbox.com/...?dl=1` redirects to a fresh host such as
//! `ucXXXX.dl.dropboxusercontent.com`, so it cannot be allowlisted ahead of
//! time. [`is_dropbox_download_host`] recognises exactly one DNS label in front
//! of `.dl.dropboxusercontent.com`; the fetcher applies it only inside a chain
//! that began at [`DROPBOX_REDIRECT_ORIGIN`], and then only to a hop back to
//! the same host.

/// The suffix Dropbox appends to a per-request download host, with its leading
/// dot, so exactly one DNS label precedes it (spec 038, R1). Three identical
/// requests produced three different labels, which is why the whole host
/// cannot be listed in the allowlist.
pub const DROPBOX_DOWNLOAD_SUFFIX: &str = ".dl.dropboxusercontent.com";

/// The only origin that may start a chain landing on a
/// [`DROPBOX_DOWNLOAD_SUFFIX`] host: the allowlisted Dropbox share URL
/// (spec 038, R1).
pub const DROPBOX_REDIRECT_ORIGIN: &str = "www.dropbox.com";

/// Whether `host` is a Dropbox per-request download host: exactly one DNS label
/// (no dot, not empty) followed by `.dl.dropboxusercontent.com` (spec 038, R1).
///
/// Deliberately narrower than a suffix match: the bare
/// `dl.dropboxusercontent.com`, any deeper subdomain, and any host that merely
/// ends with the string (say `dl.dropboxusercontent.com.evil.example`) are all
/// refused. The host is case-insensitive, like the allowlist.
#[must_use]
pub fn is_dropbox_download_host(host: &str) -> bool {
    normalise(host)
        .strip_suffix(DROPBOX_DOWNLOAD_SUFFIX)
        .is_some_and(|label| !label.is_empty() && !label.contains('.'))
}

/// The set of hosts a caller permits for a download.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostAllowlist {
    hosts: Vec<String>,
}

impl HostAllowlist {
    /// Builds an allowlist from host names. Entries are trimmed and
    /// lowercased; empty entries are dropped.
    #[must_use]
    pub fn new<I, S>(hosts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let hosts = hosts
            .into_iter()
            .map(|host| normalise(host.as_ref()))
            .filter(|host| !host.is_empty())
            .collect();
        Self { hosts }
    }

    /// Whether `host` is permitted. `host` may include an IPv6 literal in
    /// brackets; the brackets are stripped before comparison.
    #[must_use]
    pub fn permits(&self, host: &str) -> bool {
        let host = normalise(host);
        self.hosts.iter().any(|allowed| allowed == &host)
    }

    /// Whether the allowlist has no entries at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hosts.is_empty()
    }

    /// The permitted hosts, lowercased.
    #[must_use]
    pub fn hosts(&self) -> &[String] {
        &self.hosts
    }
}

/// Lowercases a host and strips the surrounding brackets of an IPv6 literal.
fn normalise(host: &str) -> String {
    let host = host.trim();
    let host = host
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(host);
    host.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_case_insensitively() {
        let allow = HostAllowlist::new(["www.gamers.org", "ROMERO.com"]);
        assert!(allow.permits("www.gamers.org"));
        assert!(allow.permits("WWW.GAMERS.ORG"));
        assert!(allow.permits("romero.com"));
    }

    #[test]
    fn does_not_match_subdomains_or_suffixes() {
        let allow = HostAllowlist::new(["example.com"]);
        assert!(!allow.permits("evil.example.com"));
        assert!(!allow.permits("example.com.evil.test"));
        assert!(!allow.permits("notexample.com"));
    }

    #[test]
    fn empty_entries_are_dropped_and_empty_list_permits_nothing() {
        let allow = HostAllowlist::new(["", "  "]);
        assert!(allow.is_empty());
        assert!(!allow.permits("anything"));
        assert!(!HostAllowlist::default().permits("anything"));
    }

    #[test]
    fn strips_ipv6_brackets() {
        let allow = HostAllowlist::new(["[::1]"]);
        assert!(allow.permits("[::1]"));
        assert!(allow.permits("::1"));
    }

    #[test]
    fn matches_a_dropbox_per_request_host_with_one_label() {
        // The exact host observed from a real `?dl=1` redirect chain.
        assert!(is_dropbox_download_host(
            "uc3b1e38517d45e48a062e295c44.dl.dropboxusercontent.com"
        ));
        // Another per-request label, and case-insensitively.
        assert!(is_dropbox_download_host(
            "ucabcdef.dl.dropboxusercontent.com"
        ));
        assert!(is_dropbox_download_host(
            "UCABCDEF.DL.DROPBOXUSERCONTENT.COM"
        ));
    }

    #[test]
    fn refuses_dropbox_hosts_without_exactly_one_label() {
        // The bare suffix is not a per-request host.
        assert!(!is_dropbox_download_host("dl.dropboxusercontent.com"));
        // Two labels is a deeper subdomain.
        assert!(!is_dropbox_download_host("a.b.dl.dropboxusercontent.com"));
        // An empty label.
        assert!(!is_dropbox_download_host(".dl.dropboxusercontent.com"));
        // A lookalike that merely ends with the suffix string.
        assert!(!is_dropbox_download_host(
            "dl.dropboxusercontent.com.evil.example"
        ));
        // The parent domain and an unrelated host.
        assert!(!is_dropbox_download_host("dropboxusercontent.com"));
        assert!(!is_dropbox_download_host("evil.example"));
        assert!(!is_dropbox_download_host(""));
    }
}
