//! PWAD browsing order and category membership (spec 090, spec 112).
//!
//! Discover leads with installed and favorite mapsets (spec 112, R1) and keeps
//! the ascending first-public-release order inside each group (spec 090, R2);
//! Library keeps that chronological order around its own installed/favorite
//! grouping. The date is the explicit, optional
//! [`ReleaseDate`](super::ReleaseDate) — never parsed from the free-form
//! `release` version label — and unknown dates sort last.
//!
//! The Community Projects tab hides Doomer Boards recipes (R1) because that
//! collection has its own category; the tags themselves stay accurate, so the
//! taxonomy, profiles and installations are untouched.

use std::cmp::Ordering;

use super::Recipe;

/// The tag whose Discover tab excludes Doomer Boards entries (spec 090, R1).
pub const COMMUNITY_PROJECTS: &str = "community-projects";
/// The dedicated Doomer Boards tag.
pub const DOOMER_BOARDS: &str = "doomer-boards";

/// Whether a recipe appears under `tag` in Discover (spec 090, R1/R2).
///
/// A recipe belongs to a category when it carries the tag. The one exception is
/// the owner's requested split: Community Projects no longer duplicates the
/// Doomer Boards collection, so a `doomer-boards` recipe is hidden there while
/// staying visible under its dedicated tag and any other earned tags.
#[must_use]
pub fn shows_in_category(recipe: &Recipe, tag: &str) -> bool {
    recipe.tags.iter().any(|held| held == tag)
        && !(tag == COMMUNITY_PROJECTS && recipe.tags.iter().any(|held| held == DOOMER_BOARDS))
}

/// Ascending first-public-release order for PWAD browsing lists (spec 090, R2).
///
/// `None` (an unknown date) sorts last, deterministically. Equal dates fall back
/// to the case-insensitive title and then the recipe id, which is always unique,
/// so favorite status, catalog order or insertion order never decides a tie.
#[must_use]
pub fn compare_browsing_order(a: &Recipe, b: &Recipe) -> Ordering {
    match (a.original_release, b.original_release) {
        (Some(left), Some(right)) => left
            .ordering_key()
            .cmp(&right.ordering_key())
            .then_with(|| compare_title_and_id(a, b)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => compare_title_and_id(a, b),
    }
}

fn compare_title_and_id(a: &Recipe, b: &Recipe) -> Ordering {
    a.title
        .to_lowercase()
        .cmp(&b.title.to_lowercase())
        .then_with(|| a.id.cmp(&b.id))
}

/// Discover's leading group for one mapset (spec 112, R1). A higher group sorts
/// first: installed favorite (3), installed (2), favorite only (1), the rest (0).
fn discover_group(installed: bool, favorite: bool) -> u8 {
    match (installed, favorite) {
        (true, true) => 3,
        (true, false) => 2,
        (false, true) => 1,
        (false, false) => 0,
    }
}

/// Discover order for every category in Cards and List (spec 112, R1).
///
/// The `(installed, favorite)` tuple sorts descending — installed favorites,
/// installed mapsets, favorite-only mapsets, then the rest — and
/// [`compare_browsing_order`] keeps the chronological order inside each group.
/// `installed` is the card's installed state (an install manifest is present),
/// whatever the base-game state.
#[must_use]
pub fn compare_discover_order(
    a: &Recipe,
    a_installed: bool,
    a_favorite: bool,
    b: &Recipe,
    b_installed: bool,
    b_favorite: bool,
) -> Ordering {
    discover_group(b_installed, b_favorite)
        .cmp(&discover_group(a_installed, a_favorite))
        .then_with(|| compare_browsing_order(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::ReleaseDate;

    fn recipe(id: &str, title: &str, release: Option<ReleaseDate>) -> Recipe {
        let mut recipe: Recipe = toml::from_str(&format!(
            r#"
schema_version = 1
id = "{id}"
title = "{title}"
authors = ["Someone"]
release = "1.0"
page = "https://www.gamers.org/"
description = "A mapset."
permission = "Free to redistribute."
[base_game]
game = "doom2"
[compatibility]
family = "boom"
complevel = "boom"
[compatibility.engine]
range = "=0.30.1"
status = "inspected"
[verification]
status = "inspected"
tester = "test"
date = "2026-10-22"
engine_version = "0.30.1"
inspected = ["fixture"]
notes = "fixture"
limitations = []
[[packages]]
id = "free"
page = "https://www.gamers.org/"
urls = ["https://www.gamers.org/a.zip"]
size = 1
sha256 = "{sha}"
kind = "zip"
[[packages.members]]
id = "maps"
path = "a.wad"
size = 1
sha256 = "{sha}"
[[components]]
id = "main"
label = "Main"
description = "Main."
optional = false
default = true
position = 1
[[components.members]]
package = "free"
member = "maps"
role = "maps"
"#,
            sha = "0".repeat(64)
        ))
        .expect("fixture recipe parses");
        recipe.original_release = release;
        recipe
    }

    fn date(year: u16, month: Option<u8>, day: Option<u8>) -> ReleaseDate {
        ReleaseDate { year, month, day }
    }

    #[test]
    fn dates_sort_ascending_by_year_month_and_day() {
        let old = recipe("old", "Zeta", Some(date(1998, Some(5), Some(1))));
        let new = recipe("new", "Alpha", Some(date(2001, Some(1), Some(1))));
        assert_eq!(compare_browsing_order(&old, &new), Ordering::Less);
        assert_eq!(compare_browsing_order(&new, &old), Ordering::Greater);
    }

    #[test]
    fn equal_dates_tie_by_title_then_id() {
        let day = Some(date(2005, Some(9), Some(30)));
        let alpha = recipe("zeta-id", "Alpha", day);
        let beta = recipe("alpha-id", "Beta", day);
        assert_eq!(compare_browsing_order(&alpha, &beta), Ordering::Less);
        let first = recipe("a", "Same", day);
        let second = recipe("b", "Same", day);
        assert_eq!(compare_browsing_order(&first, &second), Ordering::Less);
    }

    #[test]
    fn unknown_and_partial_dates_order_honestly() {
        // Unknown dates sort last.
        let known = recipe("known", "Zeta", Some(date(1999, None, None)));
        let unknown = recipe("unknown", "Alpha", None);
        assert_eq!(compare_browsing_order(&known, &unknown), Ordering::Less);

        // A year-only date is not early January: it follows the year's known
        // months. A month-only date follows that month's known days.
        let year_only = recipe("year", "A", Some(date(1997, None, None)));
        let december = recipe("dec", "B", Some(date(1997, Some(12), Some(31))));
        let may = recipe("may", "C", Some(date(1997, Some(5), None)));
        let may_day = recipe("may-day", "D", Some(date(1997, Some(5), Some(20))));
        let june = recipe("june", "E", Some(date(1997, Some(6), Some(1))));
        assert_eq!(compare_browsing_order(&may_day, &may), Ordering::Less);
        assert_eq!(compare_browsing_order(&may, &june), Ordering::Less);
        assert_eq!(compare_browsing_order(&june, &december), Ordering::Less);
        assert_eq!(
            compare_browsing_order(&december, &year_only),
            Ordering::Less,
            "year-only sorts after the year's dated entries"
        );
    }

    #[test]
    fn iso_keeps_only_the_evidenced_precision() {
        assert_eq!(date(1997, None, None).iso(), "1997");
        assert_eq!(date(1997, Some(5), None).iso(), "1997-05");
        assert_eq!(date(1997, Some(5), Some(9)).iso(), "1997-05-09");
    }

    /// Spec 112, R1: installed favorites lead, then installed mapsets, then
    /// favorite-only mapsets, then the rest, whatever the release dates.
    #[test]
    fn discover_leads_with_installed_and_favorite_groups() {
        let old =
            |id: &str, title: &str, year: u16| recipe(id, title, Some(date(year, None, None)));
        let installed_favorite = old("if", "Installed favorite", 2015);
        let installed = old("i", "Installed", 2014);
        let favorite = old("f", "Favorite", 2013);
        let rest = old("r", "Rest", 2012);

        let compare = |a: (&Recipe, bool, bool), b: (&Recipe, bool, bool)| {
            compare_discover_order(a.0, a.1, a.2, b.0, b.1, b.2)
        };
        let if_ = (&installed_favorite, true, true);
        let i_ = (&installed, true, false);
        let f_ = (&favorite, false, true);
        let r_ = (&rest, false, false);

        assert_eq!(compare(if_, i_), Ordering::Less);
        assert_eq!(compare(i_, f_), Ordering::Less);
        assert_eq!(compare(f_, r_), Ordering::Less);
        assert_eq!(compare(r_, if_), Ordering::Greater);
        assert_eq!(compare(if_, if_), Ordering::Equal);
    }

    /// Spec 112, R1: inside each group the existing chronological browsing order
    /// decides, including the title/id tie-break for equal and unknown dates.
    #[test]
    fn discover_keeps_chronology_inside_each_group() {
        let day = Some(date(2005, Some(9), Some(30)));
        let earlier = recipe("later-id", "Earlier", day);
        let later = recipe("earlier-id", "Later", Some(date(2010, None, None)));
        let unknown = recipe("unknown", "Unknown", None);

        for (installed, favorite) in [(true, true), (true, false), (false, true), (false, false)] {
            let group: Vec<&Recipe> = vec![&earlier, &later, &unknown];
            let mut sorted = group.clone();
            sorted.sort_by(|a, b| {
                compare_discover_order(a, installed, favorite, b, installed, favorite)
            });
            assert_eq!(
                sorted,
                vec![&earlier, &later, &unknown],
                "group ({installed}, {favorite}) keeps chronology"
            );
        }

        // Equal dates fall back to the case-insensitive title then the recipe id.
        let alpha = recipe("zeta-id", "Alpha", day);
        let beta = recipe("alpha-id", "Beta", day);
        assert_eq!(
            compare_discover_order(&alpha, true, true, &beta, true, true),
            Ordering::Less
        );
        assert_eq!(
            compare_discover_order(&beta, false, false, &alpha, false, false),
            Ordering::Greater
        );
    }

    #[test]
    fn discover_groups_are_the_descending_tuple() {
        assert_eq!(discover_group(true, true), 3);
        assert_eq!(discover_group(true, false), 2);
        assert_eq!(discover_group(false, true), 1);
        assert_eq!(discover_group(false, false), 0);
    }

    #[test]
    fn community_projects_hides_doomer_boards_only() {
        let mut dbp = recipe("dbp", "DBP", Some(date(2021, Some(7), Some(26))));
        dbp.tags = vec!["community-projects".into(), "doomer-boards".into()];
        let mut community = recipe("cp", "CP", Some(date(2020, Some(1), Some(2))));
        community.tags = vec!["community-projects".into()];
        let mut doomer_boards_only = recipe("dbp2", "DBP2", Some(date(2022, Some(1), Some(1))));
        doomer_boards_only.tags = vec!["doomer-boards".into()];

        assert!(!shows_in_category(&dbp, COMMUNITY_PROJECTS));
        assert!(shows_in_category(&community, COMMUNITY_PROJECTS));
        assert!(!shows_in_category(&doomer_boards_only, COMMUNITY_PROJECTS));
        // Both stay in their dedicated category and in any other earned tag.
        assert!(shows_in_category(&dbp, DOOMER_BOARDS));
        assert!(shows_in_category(&doomer_boards_only, DOOMER_BOARDS));
    }
}
