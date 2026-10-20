//! IWAD library (spec 007; spec 021 adds GOG): the known-IWAD table,
//! content-based identification, bounded Steam, GOG/store and user-folder
//! discovery, a persisted registry and the canonical symlinks DSDA-Doom needs.
//!
//! The module is pure logic with no UI dependencies. It never bundles,
//! downloads or executes IWADs; discovery is read-only and bounded, and every
//! remote or on-disk input is treated as untrusted.
//!
//! A typical flow:
//!
//! 1. [`steam::discover_paths`], [`stores::discover`] and
//!    [`folders::scan_folder`] propose files.
//! 2. [`Identifier::identify`] identifies each by content.
//! 3. [`Registry::record`] stores the result; the user resolves
//!    [`Outcome::Ambiguous`] files with [`Registry::choose`].
//! 4. [`Registry::best_for`] answers a recipe's request.
//! 5. [`CanonicalLinks::canonical_path`] hands the launch plan a path whose
//!    name makes DSDA-Doom pick the right mission.

pub mod base_games;
pub mod canonical;
pub mod folders;
pub mod identify;
pub mod registry;
pub mod steam;
pub mod stores;
pub mod table;
pub mod visibility;

pub use canonical::{CanonicalLinks, LinkError};
pub use folders::{FolderError, FolderScan, scan_folder};
pub use identify::{Family, Identification, Identifier, IdentifyError, Outcome, identify};
pub use registry::{FileTime, RefreshSummary, Registry, RegistryEntry, RegistryError, Source};
pub use steam::{SteamLibrary, candidate_iwads, discover_paths, libraries, steam_roots};
pub use stores::{GogLauncher, StoreCandidate, StoreDiscovery, standard_paths, standard_roots};
pub use table::{Game, HashKind, IwadTable, IwadVariant, TableError};
