//! DSDA-Doom integration (decisions D-007, D-011, D-013).
//!
//! The single choke point for the engine:
//!
//! * [`reviewed`] is the reviewed release list shipped with DDL;
//! * [`discovery`] discovers upstream releases and caches them for offline use;
//! * [`install`] downloads, verifies and installs an AppImage, with the FUSE
//!   extraction fallback;
//! * [`state`] persists installed versions, the active one and the rollback
//!   target;
//! * [`manager`] ties those together (install, select, rollback, remove,
//!   manual registration, attribution);
//! * [`plan`] turns a resolved description of "what to play" into an argv
//!   vector and validates it;
//! * [`runner`] starts the engine without a shell and reports how it ended,
//!   with a fake for tests;
//! * [`smoke`] probes an installation with `-v`.
//!
//! Facts about DSDA-Doom come from `docs/research/dsda-engine.md` and
//! `docs/research/upstream-sources.md`, pinned to upstream tag `v0.30.1`. Every
//! value DDL emits is confirmed there or marked there as needing Deck
//! validation.

pub mod clock;
pub mod discovery;
pub mod install;
pub mod manager;
pub mod plan;
pub mod reviewed;
pub mod runner;
pub mod smoke;
pub mod state;

pub use clock::{Clock, SystemClock, format_unix_utc};
pub use discovery::{
    ApiError, ApiResponse, AssetInfo, Availability, CACHE_DIR, CACHE_FILE, CachedReleases,
    CannedApi, DiscoveredRelease, DiscoveryError, DiscoveryOutcome, DiscoveryStatus, GhAsset,
    GhRelease, GithubApi, RELEASES_URL, ReleaseApi, THROTTLE, classify_release, parse_digest,
    refresh, select_linux_appimage, version_from_tag,
};
pub use install::{
    APPIMAGE_FILE, EXTRACT_DIR, EXTRACT_ENTRY, EXTRACT_TIMEOUT, InstallError, InstallRequest,
    indicates_fuse,
};
pub use manager::{Attribution, ENGINE_LICENSE, EngineDirs, EngineManager, Error};
pub use plan::{
    AssignAllowlist, AssignOverride, Complevel, LaunchPlan, PathField, Skill, ValidationError,
    ValueRangeError, Warp,
};
pub use reviewed::{
    EMBEDDED_LIST, ListError, RELEASE_DOWNLOAD_BASE, RELEASE_TAG_BASE, ReviewedList,
    ReviewedVersion, SOURCE_TREE_BASE,
};
pub use runner::{
    EngineRunner, FakeEngineRunner, LaunchError, LaunchExit, LaunchOutcome, RealEngineRunner,
    RecordedLaunch, SCRUBBED_ENV,
};
pub use smoke::{
    RuntimeError, SMOKE_TIMEOUT, SmokeError, VERSION_PREFIX, classify_runtime, smoke_test,
    smoke_test_with_timeout,
};
pub use state::{
    InstalledEngine, InstalledState, Mode, Origin, SCHEMA_VERSION as STATE_SCHEMA_VERSION,
    STATE_FILE, StateError,
};
