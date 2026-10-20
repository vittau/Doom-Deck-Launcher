//! A minimal [`Recipe`] builder for tests that only care about packages and
//! components (specs 044 link checks and e2e sample). It builds the typed
//! model directly, without the lint rules, so a test can describe any shape.

use super::Recipe;

/// One optional or required component: `(id, optional, user_supplied)`.
pub(crate) type ComponentSpec<'a> = (&'a str, bool, bool);

/// One package: `(id, mirror urls, size)`.
pub(crate) type PackageSpec<'a> = (&'a str, Vec<String>, u64);

/// A recipe with the given packages and components.
pub(crate) fn recipe(
    id: &str,
    packages: &[PackageSpec<'_>],
    components: &[ComponentSpec<'_>],
) -> Recipe {
    let empty_components = if components.is_empty() {
        "components = []\n"
    } else {
        ""
    };
    let mut text = format!(
        r#"schema_version = 1
id = "{id}"
title = "Test {id}"
authors = []
release = "2020-01-01"
page = "https://example.com/"
description = "d"
permission = "p"
{empty_components}
[base_game]
game = "doom2"

[compatibility]
family = "vanilla"
complevel = "vanilla"

[compatibility.engine]
range = "=0.30.1"
status = "inspected"

[verification]
status = "inspected"
tester = "t"
date = "2020-01-01"
engine_version = "0.30.1"
inspected = []
notes = "n"
limitations = []
"#
    );
    for (package, urls, size) in packages {
        let urls: Vec<String> = urls.iter().map(|url| format!("{url:?}")).collect();
        text.push_str(&format!(
            "\n[[packages]]\nid = {package:?}\npage = \"https://example.com/\"\nurls = [{}]\nsize = {size}\nsha256 = \"{}\"\nkind = \"zip\"\nmembers = []\n",
            urls.join(", "),
            "0".repeat(64)
        ));
    }
    for (position, (component, optional, user_supplied)) in components.iter().enumerate() {
        text.push_str(&format!(
            "\n[[components]]\nid = {component:?}\nlabel = {component:?}\ndescription = \"d\"\noptional = {optional}\ndefault = false\nposition = {position}\nuser_supplied = {user_supplied}\nmembers = []\n"
        ));
    }
    toml::from_str(&text).expect("the fixture recipe parses")
}
