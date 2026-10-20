//! Spec 008 only: replace this provider with core-backed view models in spec 010.
//! Content identities: docs/research/reference-sigil.md, upstream-sources.md §5,
//! accepted D-016. Readiness/install states are deliberately synthetic.
use crate::copy::Copy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapsetId {
    Sigil,
    Auger,
    Sigil2,
}

/// Narrow UI boundary: no network, file discovery, or engine execution.
pub struct MapsetView {
    pub title: &'static str,
    pub author: &'static str,
    pub game: Copy,
    pub description: Copy,
    pub main: &'static str,
    pub source: &'static str,
    pub release: &'static str,
    pub installed: bool,
    pub iwad_ready: bool,
    pub complevel: Copy,
    pub art: i32,
}
pub const MAPSETS: [MapsetId; 3] = [MapsetId::Sigil, MapsetId::Auger, MapsetId::Sigil2];
pub fn mapset(id: MapsetId) -> MapsetView {
    match id {
        MapsetId::Sigil => MapsetView {
            title: "SIGIL",
            author: "John Romero",
            game: Copy::Doom,
            description: Copy::SigilDescription,
            main: "SIGIL_V1_23.wad",
            source: "romero.com/sigil",
            release: "1.23",
            installed: true,
            iwad_ready: true,
            complevel: Copy::Complevel3,
            art: 0,
        },
        MapsetId::Auger => MapsetView {
            title: "AUGER;ZENITH",
            author: "Doomer Boards Project 37",
            game: Copy::Doom2,
            description: Copy::AugerDescription,
            main: "DBP37_AUGZEN.wad",
            source: "doomer.boards.net/thread/2176/dbp37-auger-zenith",
            release: "DBP 37",
            installed: true,
            iwad_ready: true,
            complevel: Copy::Complevel2,
            art: 1,
        },
        MapsetId::Sigil2 => MapsetView {
            title: "SIGIL II",
            author: "John Romero",
            game: Copy::Doom,
            description: Copy::Sigil2Description,
            main: "SIGIL_II_V1_0.WAD",
            source: "romero.com/sigil",
            release: "1.0",
            installed: false,
            iwad_ready: false,
            complevel: Copy::Complevel3,
            art: 2,
        },
    }
}
#[derive(Clone, Debug)]
pub struct ProfileView {
    pub skill: usize,
    pub pistol: bool,
    pub advanced: bool,
    pub compat: bool,
    pub addon: usize,
}
impl Default for ProfileView {
    fn default() -> Self {
        Self {
            skill: 4,
            pistol: false,
            advanced: false,
            compat: false,
            addon: 0,
        }
    }
}
/// The fixed recipe order is inspectable, never freely reordered.
pub fn files(id: MapsetId, p: &ProfileView) -> Vec<&'static str> {
    let main = if id == MapsetId::Sigil && p.compat {
        "SIGIL_COMPAT_V1_23.wad"
    } else {
        mapset(id).main
    };
    let mut result = vec![main];
    if id == MapsetId::Auger {
        match p.addon {
            1 => result.push("DBP37_AlternateBlaster.wad"),
            2 => result.push("DBP37_DeckardBlaster.wad"),
            _ => {}
        }
    }
    result
}
