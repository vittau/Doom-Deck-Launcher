//! Launch resolution: from a recipe, a profile and installed state to a
//! validated DSDA-Doom [`LaunchPlan`] and a "Will launch" summary (spec 009,
//! R5/R6).

mod blocker;
mod resolve;

#[cfg(test)]
mod tests;

pub use blocker::{Blocker, Engine, Summary, SummaryContent, game_label};
pub use resolve::{Iwads, Resolution, resolve};
