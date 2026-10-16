//! Compiles the `.slint` user interface into Rust at build time (spec 001, R1/R4).
//!
//! `slint_build::compile` embeds the generated code through `slint::include_modules!()`
//! in `src/main.rs`; rerun tracking for the `.slint` file is handled by slint-build.

fn main() {
    slint_build::compile("ui/app-window.slint").expect("failed to compile ui/app-window.slint");
}
