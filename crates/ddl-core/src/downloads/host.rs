//! The host allowlist that bounds where a download may go (spec 006, R1).
//!
//! Recipes and the engine list name explicit sources; the fetcher is only
//! allowed to contact those hosts (D-010, D-015). Matching is an exact,
//! case-insensitive comparison of the DNS host, with no wildcard or suffix
//! matching: `example.com` never permits `evil.example.com`.

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
}
