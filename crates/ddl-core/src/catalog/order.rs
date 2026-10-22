//! PWAD browsing order and category membership (spec 090).
//!
//! Every Discover category, Favorites and Library lists recipes in ascending
//! first-public-release order (R2). The date is the explicit, optional
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
