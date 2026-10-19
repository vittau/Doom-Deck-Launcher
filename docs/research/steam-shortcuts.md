# Steam library shortcuts for installed mapsets

**Spec:** 043 (R1–R2) · **Status:** research complete 2026-10-19 · **Scope:** Steam library
shortcuts and artwork for installed mapsets. No Steam Input code, no controller layouts, no
change to DDL's launch model. **No code was written for this task.**

This document answers R1 (how a non-Steam shortcut is represented and added on SteamOS) and R2
(what a per-mapset shortcut would mean for DDL). R3, the recommendation, is in
`docs/reports/043-steam-shortcuts-research.md`.

Every factual claim carries one of these labels:

- **Confirmed** — observed in the pinned source, upstream documentation, or DDL's own code.
- **Proposed** — a DDL design position suggested here; not yet proven.
- **Needs Deck Validation** — depends on a physical Steam Deck, the current Steam client, or
  Gaming Mode behaviour we cannot reproduce on the development host.

## 1. Bottom line

| Question | Finding | Label |
| --- | --- | --- |
| Where are shortcuts stored? | One binary VDF per Steam account: `<root>/userdata/<account-id>/config/shortcuts.vdf`; artwork in the sibling `config/grid/`. | Confirmed |
| Is there a supported API? | No Steamworks API and no documented file schema. The format is described on Valve's community wiki and implemented by third-party tools. | Confirmed |
| Can DDL add a shortcut while Steam runs? | Tools require Steam to be closed; a running client can overwrite the file and will not show the new entry until it restarts. | Confirmed |
| Can a `steam://` URL add a specific shortcut? | `steam://AddNonSteamGame` only opens Steam's checklist. No documented URL creates a shortcut with a given executable and arguments. | Confirmed |
| Can DDL name shortcuts deterministically? | Yes: tools write their own 32-bit reserved-range `appid`; artwork is named from that id. Valve's own UI uses randomised IDs. | Confirmed (tools) / Proposed (DDL) |
| Is automatic creation safe in Gaming Mode? | DDL is itself launched by Steam there, so Steam is running; closing it is disruptive and the write path is unsupported. | Confirmed (architecture) / Needs Deck Validation (session behaviour) |
| Is per-mapset artwork available? | No. DDL has 1280×720 screenshots, not 600×900 capsules, 3840×1240 heroes, or transparent logos. | Confirmed |

## 2. Sources and pins

| Source | Pin / date | What it is used for |
| --- | --- | --- |
| [Valve Support: Add Non-Steam Games to a Library](https://help.steampowered.com/en/faqs/view/4B8B-9697-2338-40EC) | FAQ content timestamp 2021-07-10; read 2026-10-19 | Official user flow; "only adds a shortcut", no updates or ownership. |
| [Steamworks: Library Assets](https://partner.steamgames.com/doc/store/assets/libraryassets) | Read 2026-10-19 (no versioned URL) | Published capsule, header, hero and logo sizes. |
| [Steamworks: Community and Client Icons](https://partner.steamgames.com/doc/store/assets/community) | Read 2026-10-19 (no versioned URL) | Shortcut icon sizes and formats. |
| [Valve Developer Community: Steam browser protocol](https://developer.valvesoftware.com/wiki/Steam_browser_protocol) | Read 2026-10-19 (community wiki; no revision pin) | `steam://AddNonSteamGame`, `steam://rungameid/<id>`. |
| [Valve Developer Community: Adding Non-Steam Games](https://developer.valvesoftware.com/wiki/Add_Non-Steam_Game) | Read 2026-10-19 (community wiki; no revision pin) | Binary layout and field list. |
| [Valve Developer Community: Steam Library Shortcuts](https://developer.valvesoftware.com/wiki/Steam_Library_Shortcuts) | Read 2026-10-19 (community wiki; no revision pin) | Independent C# writer; "Steam must be shut downed"; app-id formula. |
| [SteamTinkerLaunch wiki: Add Non-Steam Game](https://github.com/sonic2kk/steamtinkerlaunch/wiki/Add-Non-Steam-Game/c56b4c922ed85b7d8ecbaaab708e6f65717a0104) | Wiki revision `c56b4c9` (2023-10-17) | User-facing workflow, artwork folder, restart requirement. |
| [SteamTinkerLaunch source](https://github.com/sonic2kk/steamtinkerlaunch/tree/8550ab26a712b7f5f6d0947070181446b9de61fd) | Commit `8550ab2` (2025-07-14) | Path discovery, app-id notes, artwork names, "Ensure the Steam Client is closed". |
| [BoilR](https://github.com/PhilipK/BoilR/tree/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf) | Tag `v.1.11.0`, commit `fed7df5` (2026-10-02) | Steam shutdown/wait, user enumeration, backup, artwork names, collections change. |
| [Steam ROM Manager](https://github.com/SteamGridDB/steam-rom-manager/tree/7f0f4024bf829a22a2fba56c6e6dce5182b76af4) | Tag `v2.5.44`, commit `7f0f402` (2026-08-06) | App-id formula, loginusers parsing, backup names, artwork names, casing normalisation. |
| [`steam_shortcuts_util` 1.1.9](https://crates.io/crates/steam_shortcuts_util/1.1.9) | Checksum `c7e0c9bee47630fbfad0df3951421aa55aa2f68dcf0f9a62764550d44925d59d` | Rust parser/writer; independent field order and app-id formula. |
| [ValveSoftware/steam-for-linux issue #9463](https://github.com/ValveSoftware/steam-for-linux/issues/9463) | Opened 2023-05-05; updated 2026-03-19; open | Steam-created shortcut IDs are randomised; third-party prediction broke. |
| DDL code | Commit `3867208` (task branch base; these docs add no code) | `crates/ddl-core/src/iwads/steam.rs`, `crates/ddl/src/cli.rs`, `crates/ddl-core/src/engines/dsda/runner.rs` |

The Valve Developer Community pages are community-edited, not formal Steamworks documentation.
They are the closest thing to a schema that exists, and the maintained tools above agree with
their core layout. That agreement is the evidence; the pages themselves are not a contract.

## 3. R1 — How a non-Steam shortcut works

### 3.1 Files, accounts and roots

- **Confirmed** — a non-Steam shortcut is stored in
  `<steam-root>/userdata/<account-id>/config/shortcuts.vdf`, a binary file. Custom artwork lives
  in `<steam-root>/userdata/<account-id>/config/grid/`
  ([SteamTinkerLaunch wiki](https://github.com/sonic2kk/steamtinkerlaunch/wiki/Add-Non-Steam-Game/c56b4c922ed85b7d8ecbaaab708e6f65717a0104);
  [BoilR `steam/utils.rs`](https://github.com/PhilipK/BoilR/blob/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf/crates/boilr-core/src/steam/utils.rs#L53-L105)).
- **Confirmed** — there is one file per account, not one global file. `userdata/0` is not a user
  account: BoilR skips it
  ([`utils.rs#L79-L82`](https://github.com/PhilipK/BoilR/blob/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf/crates/boilr-core/src/steam/utils.rs#L79-L82)),
  and SRM also excludes it
  ([`get-available-logins.ts`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/helpers/steam/get-available-logins.ts#L52-L75)).
- **Confirmed** — account names and the most recent login come from
  `<root>/config/loginusers.vdf`; the `userdata` folder name is the account id, equal to
  `SteamID64 - 76561197960265728` (`0x0110000100000000`)
  ([SRM `steam-id-64-to-account-id.ts`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/helpers/steam/steam-id-64-to-account-id.ts#L1-L6);
  [SteamTinkerLaunch `setSteamPaths`](https://github.com/sonic2kk/steamtinkerlaunch/blob/8550ab26a712b7f5f6d0947070181446b9de61fd/steamtinkerlaunch#L627-L707)).
- **Confirmed** — Linux roots DDL already probes are
  `~/.local/share/Steam`, `~/.steam/root`, and Flatpak
  `~/.var/app/com.valvesoftware.Steam/.local/share/Steam`
  ([`crates/ddl-core/src/iwads/steam.rs`](../../crates/ddl-core/src/iwads/steam.rs), `steam_roots`).
  BoilR also checks `~/.steam/steam` and the Flatpak `.steam/steam` path
  ([`utils.rs#L129-L145`](https://github.com/PhilipK/BoilR/blob/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf/crates/boilr-core/src/steam/utils.rs#L129-L145)).
  On a stock Deck the native root is the one that matters.
- **Confirmed** — the file is user state, not library state. It does not move with a game library
  on an SD card; only the executable path stored inside it points elsewhere.

### 3.2 Binary format

- **Confirmed** — `shortcuts.vdf` is a binary KeyValues/VDF document: a root dictionary named
  `shortcuts`, integer keys (`0`, `1`, …) for each entry, then a nested `tags` dictionary.
  The Valve community schema and all three tools write the same shape
  ([Adding Non-Steam Games](https://developer.valvesoftware.com/wiki/Add_Non-Steam_Game);
  [`shortcuts_writer.rs`](https://docs.rs/crate/steam_shortcuts_util/1.1.9/source/src/shortcuts_writer.rs)).
- **Confirmed** — the type bytes used by the writers are `0x00` dictionary, `0x01` string,
  `0x02` int32 (little-endian, four bytes), and `0x08` end-of-dictionary. Strings are
  NUL-terminated; there are no length prefixes
  ([`shortcuts_writer.rs`](https://docs.rs/crate/steam_shortcuts_util/1.1.9/source/src/shortcuts_writer.rs);
  [Valve community schema](https://developer.valvesoftware.com/wiki/Add_Non-Steam_Game)).
- **Confirmed** — the common fields are:

| Field | Type | Meaning / notes |
| --- | --- | --- |
| `appid` | int32 | Shortcut id, also the artwork key (see §3.3). |
| `AppName` / `appname` | string | Display name. Casing varies (see below). |
| `Exe` / `exe` | string | Absolute executable path, normally quoted. |
| `StartDir` | string | Working directory, normally quoted and ending in `/`. |
| `icon` | string | Absolute icon path. |
| `ShortcutPath` | string | Usually empty for NSG shortcuts. |
| `LaunchOptions` | string | Arguments appended to `Exe`. |
| `IsHidden` | int32 0/1 | Hide from the library. |
| `AllowDesktopConfig` | int32 0/1 | Desktop configuration permission. |
| `AllowOverlay` | int32 0/1 | Steam overlay permission. |
| `openvr` / `OpenVR` | int32 0/1 | VR flag; casing differs between writers. |
| `Devkit` | int32 | Devkit flag. |
| `DevkitGameID` | string | Devkit id. |
| `DevkitOverrideAppID` | int32 | Devkit override. |
| `LastPlayTime` | int32 | Unix seconds; Steam updates it. |
| `FlatpakAppID` | string | Flatpak app id, when relevant. |
| `sortas` | string | Sort name. |
| `tags` | dictionary | String keys `0`, `1`, … to collection names. |

  Sources: [Adding Non-Steam Games](https://developer.valvesoftware.com/wiki/Add_Non-Steam_Game);
  [Steam Library Shortcuts](https://developer.valvesoftware.com/wiki/Steam_Library_Shortcuts);
  [`shortcut.rs`](https://docs.rs/crate/steam_shortcuts_util/1.1.9/source/src/shortcut.rs).
- **Confirmed** — field casing is inconsistent in real files. SRM added a normalisation pass
  because Steam writes `AppName`/`Exe` for shortcuts added through its own UI while other code
  used lower-case names
  ([`vdf-shortcuts-file.ts#L101-L109`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/vdf-shortcuts-file.ts#L101-L109)).
  A parser must compare keys case-insensitively and preserve untouched entries rather than
  reserialising them from a fixed field list.
- **Confirmed** — the field set is not fixed. `FlatpakAppID` and `sortas` appear in the Valve
  community schema but not in `steam_shortcuts_util` 1.1.9's writer; Steam adds fields over time.
  Unknown fields must survive a read/write cycle.
- **Needs Deck Validation** — string encoding for non-ASCII names and paths. The Valve community
  app-id sample uses Windows-1252; the Rust and TypeScript tools use UTF-8. DDL recipe titles
  are English, but a Linux user name or an SD-card label can be non-ASCII.
- **Confirmed** — a truncated or malformed file is not recoverable by the tools; SRM reports a
  "corrupted VDF" error and BoilR returns an error naming the file. A backup is the only
  recovery path ([`vdf-shortcuts-file.ts#L78-L94`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/vdf-shortcuts-file.ts#L78-L94)).

### 3.3 App ids and artwork

- **Confirmed** — the apparent id formula is a CRC32 with the high bit set. Two variants are in
  use:
  - SRM: `crc32(exe + appname) | 0x80000000`
    ([`generate-app-id.ts#L1-L31`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/helpers/steam/generate-app-id.ts#L1-L31)).
  - `steam_shortcuts_util`: the same `exe + app_name` concatenation
    ([`app_id_generator.rs`](https://docs.rs/crate/steam_shortcuts_util/1.1.9/source/src/app_id_generator.rs)).
  - Valve community wiki: `crc32(Windows-1252(AppName + Exe + "\0")) | 0x80000000`, i.e. name
    first, executable second, with a trailing NUL
    ([Steam Library Shortcuts](https://developer.valvesoftware.com/wiki/Steam_Library_Shortcuts)).
  
  The order, separator and encoding disagree. The consequence is practical: two tools can
  compute different ids for the same shortcut. A DDL-written shortcut must therefore own its
  `appid` and use that exact value for artwork; it must not assume another tool would derive the
  same value. **Confirmed** (code paths); **Proposed** (DDL policy).
- **Confirmed** — the id is stored as a 32-bit little-endian value and used as the artwork key.
  SteamTinkerLaunch says explicitly that "you can actually write any AppID you want into the
  appid field", and that it generates one from the name and path so it is predictable
  ([wiki](https://github.com/sonic2kk/steamtinkerlaunch/wiki/Add-Non-Steam-Game/c56b4c922ed85b7d8ecbaaab708e6f65717a0104)).
  Its implementation uses an MD5-seeded value rather than CRC32, confirming that the value is
  arbitrary as long as it is well-formed
  ([source](https://github.com/sonic2kk/steamtinkerlaunch/blob/8550ab26a712b7f5f6d0947070181446b9de61fd/steamtinkerlaunch#L24812-L24845)).
- **Confirmed** — Steam's own "Add a Non-Steam Game" flow now assigns randomised NSG ids, so the
  id cannot be predicted from the path and name for a shortcut Steam created itself
  ([issue #9463](https://github.com/ValveSoftware/steam-for-linux/issues/9463), opened 2023-05-05,
  still open). This does not prevent a tool from writing its own id; it prevents a tool from
  predicting what Steam's UI would have chosen. **Confirmed** from the issue and the maintained
  writers.
- **Confirmed** — the 64-bit Big Picture id used by `steam://rungameid/<id>` is derived from the
  short id as `(short_id << 32) | 0x02000000` in SRM
  ([`generate-app-id.ts#L1-L24`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/helpers/steam/generate-app-id.ts#L1-L24)).
  **Needs Deck Validation** — whether a DDL-written id is accepted by `rungameid` on the current
  client, and whether the Big Picture id needs the shortcut to have been seen once by Steam.
- **Confirmed** — artwork is looked up by the short id, using these suffixes:

| File | Slot | Published size |
| --- | --- | --- |
| `<id>p.jpg` | Portrait capsule / library grid | 600×900 |
| `<id>.jpg` | Wide capsule / library header | 920×430 |
| `<id>_hero.jpg` | Library detail hero | 3840×1240 (1920×620 half-size) |
| `<id>_logo.png` | Transparent logo over the hero | 1280 wide and/or 720 tall |
| `<id>_icon.*` | Icon (SteamTinkerLaunch) | 256×256 or 512×512 |
| `<id>-icon.*` | Icon (BoilR spelling) | 256×256 or 512×512 |
| `<id>_bigpicture.*` | Legacy Big Picture grid (BoilR) | 920×430 |

  Sources: SteamTinkerLaunch
  ([artwork suffixes](https://github.com/sonic2kk/steamtinkerlaunch/blob/8550ab26a712b7f5f6d0947070181446b9de61fd/steamtinkerlaunch#L1861-L1868),
  [wiki](https://github.com/sonic2kk/steamtinkerlaunch/wiki/Add-Non-Steam-Game/c56b4c922ed85b7d8ecbaaab708e6f65717a0104));
  BoilR
  ([`image_type.rs#L41-L49`](https://github.com/PhilipK/BoilR/blob/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf/crates/boilr-core/src/steamgriddb/image_type.rs#L41-L49));
  SRM
  ([`available-artwork-types.ts#L47-L57`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/artwork-types/available-artwork-types.ts#L47-L57)).
  The three tools disagree on the icon spelling (`_icon` vs `-icon`), and only BoilR writes a
  `_bigpicture` file. **Needs Deck Validation** — which spelling and which Big Picture id the
  current Steam client actually reads.
- **Confirmed** — the published dimensions above are Valve's Steamworks asset sizes
  ([Library Assets](https://partner.steamgames.com/doc/store/assets/libraryassets);
  [Community and Client Icons](https://partner.steamgames.com/doc/store/assets/community)).
  Steamworks asks for a transparent PNG logo, an ICO or PNG shortcut icon, and gives capsule
  and hero sizes with design guidance but no mandated source format. None of these pages
  document the non-Steam `config/grid` loader.
- **Needs Deck Validation** — the exact accepted formats and precedence for grid files (JPEG vs
  PNG, duplicate `.jpg`/`.png`, transparency, animated files). Tools write JPEG and PNG; SRM
  normalises WebP to PNG
  ([`image-extensions.ts`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/helpers/steam/image-extensions.ts)).
  Valve does not document this loader, so no claim beyond "tools use JPEG/PNG" is safe.
- **Proposed** — DDL should treat the id as an opaque, stable, reserved-range value it owns, and
  derive it from the mapset id plus a DDL namespace (not from a user-editable display name). That
  keeps artwork names stable when a title is edited.

### 3.4 When Steam reads and writes the file

- **Confirmed** — Steam reads `shortcuts.vdf` at startup. SteamTinkerLaunch's wiki says: "When a
  Non-Steam Game is added, you must restart Steam to make the newly added shortcut available"
  ([wiki](https://github.com/sonic2kk/steamtinkerlaunch/wiki/Add-Non-Steam-Game/c56b4c922ed85b7d8ecbaaab708e6f65717a0104)).
- **Confirmed** — Steam must be closed while the file is edited. Valve's community wiki states:
  "In order to add a steam short cut, Steam must be shut downed or it wont add the shortcut"
  ([Steam Library Shortcuts](https://developer.valvesoftware.com/wiki/Steam_Library_Shortcuts)).
  SteamTinkerLaunch's own help says: "Ensure the Steam Client is closed before adding shortcuts"
  ([`howto`](https://github.com/sonic2kk/steamtinkerlaunch/blob/8550ab26a712b7f5f6d0947070181446b9de61fd/steamtinkerlaunch#L22378-L22382)).
- **Confirmed** — the maintained tools implement a stop/write/restart cycle:
  - BoilR asks the client to exit with `steam -shutdown`, waits up to 20 seconds, then falls back
    to signals, because "the client does not handle `Signal::Quit`" and a SIGKILL can interrupt a
    config write
    ([`restarter.rs#L15-L42`](https://github.com/PhilipK/BoilR/blob/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf/crates/boilr-core/src/steam/restarter.rs#L15-L42)).
    It backs up `shortcuts.vdf` before each import
    ([`backups.rs`](https://github.com/PhilipK/BoilR/blob/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf/src/backups.rs)).
  - SRM can stop Steam with `kill -15` on Linux, optionally restart it, and writes backups named
    `shortcuts.backup` and `shortcuts.firstbackup`
    ([`stop-start-steam.ts#L70-L111`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/helpers/steam/stop-start-steam.ts#L70-L111);
    [`backup.ts`](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/helpers/file/backup.ts)).
  - SteamTinkerLaunch tells the user to close Steam
    ([`howto`](https://github.com/sonic2kk/steamtinkerlaunch/blob/8550ab26a712b7f5f6d0947070181446b9de61fd/steamtinkerlaunch#L22378-L22382))
    and copies `shortcuts.vdf` to a `*_steamtinkerlaunch_backup.vdf` file before replacing
    artwork ([`getGridsForNonSteamGames`](https://github.com/sonic2kk/steamtinkerlaunch/blob/8550ab26a712b7f5f6d0947070181446b9de61fd/steamtinkerlaunch#L1791-L1793)).
- **Confirmed** — BoilR's comment gives the reason: writing while Steam runs lets the client
  "overwrite them behind our back"
  ([`restarter.rs#L20-L21`](https://github.com/PhilipK/BoilR/blob/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf/crates/boilr-core/src/steam/restarter.rs#L20-L21)).
  Whether a running client always rewrites on exit, and how long the window is, is **Needs Deck
  Validation**; the safe rule is "do not write while Steam runs".
- **Needs Deck Validation** — Gaming Mode specifics. DDL is installed as a non-Steam shortcut and
  launched from the library, so Steam is its parent process
  ([steam-deck-guide](../steam-deck-guide.md), "Install from a GitHub release"). Closing
  Steam from inside DDL would at minimum terminate the parent and may end the Gaming Mode session
  rather than returning to a usable desktop. The exact result (black screen, Steam restart, session
  logout) needs a Deck test and is not documented by Valve.
- **Confirmed** — after a write, artwork appears with the shortcut on the next Steam start. A
  restart is also required after removing artwork
  ([SteamTinkerLaunch source](https://github.com/sonic2kk/steamtinkerlaunch/blob/8550ab26a712b7f5f6d0947070181446b9de61fd/steamtinkerlaunch#L1872)).

### 3.5 Supported alternatives

- **Confirmed** — Steam's official UI path is: launch Steam → **Games** → **Add a Non-Steam Game
  to My Library…** → **Browse…** → **Add Selected Programs**
  ([Valve Support](https://help.steampowered.com/en/faqs/view/4B8B-9697-2338-40EC)). Valve warns
  that this only adds a shortcut: it does not grant updates or ownership.
- **Confirmed** — `steam://AddNonSteamGame` opens "the Steam checklist menu to add non-Steam
  games". The Valve page documents no parameters, so there is no way to prefill an executable
  or arguments
  ([Steam browser protocol](https://developer.valvesoftware.com/wiki/Steam_browser_protocol)).
- **Confirmed** — `steam://rungameid/<id>` runs an application "with support for mods and
  non-Steam shortcuts"
  ([Steam browser protocol](https://developer.valvesoftware.com/wiki/Steam_browser_protocol)).
  It launches an existing shortcut; it does not create one.
- **Confirmed** — none of the reviewed Valve pages documents a URL or Steamworks call that
  creates a shortcut with a given `Exe`, `LaunchOptions`, name and artwork. The community
  schema's note points readers to the file format, not to an API
  ([Steam browser protocol](https://developer.valvesoftware.com/wiki/Steam_browser_protocol);
  [Adding Non-Steam Games](https://developer.valvesoftware.com/wiki/Add_Non-Steam_Game)).
- **Proposed** — a `.desktop` file is not a substitute on Deck: Gaming Mode shows Steam library
  entries, not arbitrary desktop launchers. It would only help in Desktop Mode, which is outside
  DDL's primary flow.

### 3.6 Fragility

- **Confirmed** — the format is undocumented by Valve and maintained by community reverse
  engineering. The Valve Developer Community pages are not a contract and carry no compatibility
  promise.
- **Confirmed** — the client has changed ID behaviour before: Steam-created NSG ids became
  randomised, breaking third-party launch prediction in 2023
  ([issue #9463](https://github.com/ValveSoftware/steam-for-linux/issues/9463)).
- **Confirmed** — client storage keeps moving. BoilR's code notes that "Modern Steam (late
  2025+)" stores collections in a flat JSON file in `userdata`, with LevelDB and `localconfig.vdf`
  fallbacks for older clients
  ([`collections.rs#L117-L139`](https://github.com/PhilipK/BoilR/blob/fed7df598b59f1fc2c6916c06bf33bf7c27f16cf/crates/boilr-core/src/steam/collections.rs#L117-L139)).
  `shortcuts.vdf` is still written by all three tools in their 2025–2026 releases, but that is
  evidence of current practice, not a guarantee.
- **Confirmed** — field casing and field sets vary between Steam-written and tool-written files
  ([SRM comment](https://github.com/SteamGridDB/steam-rom-manager/blob/7f0f4024bf829a22a2fba56c6e6dce5182b76af4/src/lib/vdf-shortcuts-file.ts#L101-L109)).
- **Proposed** — a DDL implementation would need a round-trip parser, a schema-version guard, a
  backup, and a "refuse on unknown shape" rule. It cannot promise forward compatibility.

## 4. R2 — Fit for DDL

### 4.1 What a per-mapset shortcut would run

- **Confirmed** — the implemented CLI is
  `ddl launch <id> [--dry-run] [--profile <name|id>] [--with <component>...] [--skill <1-5>]
  [--pistol-start] [--accept-changes]`
  ([`crates/ddl/src/cli.rs`](../../crates/ddl/src/cli.rs), `launch` help). Spec 043's
  `ddl play <id>` is stale: there is no `play` subcommand. Spec 014 and spec 018 both use
  `ddl launch`, and the parser tests cover it.
- **Confirmed** — `launch` is headless. `main.rs` routes every subcommand except the UI to the
  headless runner, and the launch path blocks until DSDA exits (`child.wait()` in
  [`runner.rs`](../../crates/ddl-core/src/engines/dsda/runner.rs)).
- **Proposed** — a shortcut would store:
  - `Exe` = absolute path to the `ddl` binary (quoted);
  - `StartDir` = the directory containing `ddl`;
  - `AppName` = a DDL-owned display name, e.g. `DDL: SIGIL`;
  - `LaunchOptions` = `launch sigil` (and optionally `--profile "UV"`).
  
  Omitting `--profile` uses DDL's remembered profile for that mapset. Including it pins one
  profile and breaks if the profile is renamed or deleted. The first is closer to DDL's UI
  behaviour; the second is closer to "this shortcut is my UV run".
- **Confirmed** — the mapset must already be installed and the profile must exist. Otherwise
  `launch` exits with a typed blocker. In CLI mode that blocker goes to stderr; in Gaming Mode
  there is no terminal, so the user would see the shortcut start and stop with no explanation.
  This is a real UX cost of the `ddl launch` approach, not a file-format issue.
- **Proposed** — if per-mapset shortcuts were ever built, DDL should launch its own UI with a
  preselected mapset (a new flag) rather than a headless `launch`, so missing prerequisites can
  be shown on screen. That is a new CLI/UI feature, not just a `shortcuts.vdf` writer.

### 4.2 Safe write design (only if a future decision says go)

These are the minimum properties any DDL writer would need. All are **Proposed**; none is
implemented.

1. **Opt-in and explicit.** A settings action such as "Install Steam shortcuts for my mapsets",
   never a side effect of install, launch or first run.
2. **Refuse while Steam runs.** Detect the client process and stop with a clear explanation.
   Offer a documented "quit Steam, then run this from Desktop Mode" path rather than killing the
   parent session.
3. **Back up first.** Copy `shortcuts.vdf` to a timestamped DDL backup directory before parsing,
   and never overwrite an existing backup. BoilR and SRM both do this.
4. **Parse, do not regenerate.** Read the existing file into a lossless tree; preserve unknown
   fields and every non-DDL entry. Refuse to write if the file does not parse or if any entry
   cannot be represented.
5. **Own entries by a stable marker.** Use a DDL-namespaced `appid` and an `AppName` prefix; do
   not rely on `ShortcutPath` or `tags`, which users can edit in Steam. Removing a shortcut must
   remove only DDL-owned entries.
6. **Write atomically.** Write a temp file in the same directory, flush and sync, rename over the
   original, and re-parse the result. DDL already uses this pattern in
   [`library/manifest.rs`](../../crates/ddl-core/src/library/manifest.rs) and
   [`profiles/store.rs`](../../crates/ddl-core/src/profiles/store.rs).
7. **Artwork is copied, not generated.** Copy DDL-owned images into `config/grid/` under the
   chosen id; never delete files DDL did not create.
8. **One user at a time.** Enumerate `userdata/<numeric>` directories, exclude `0`, read
   `loginusers.vdf` for names and `MostRecent`, and require an explicit choice when there is more
   than one. Writing to the wrong account is silent and confusing.
9. **No shell.** Use `std::process::Command` with separate arguments, as AGENTS.md requires.
10. **Test with fixtures.** Golden binary fixtures from Steam, round-trip preservation, unknown
    fields, corrupt input, backup/restore, refusal while "Steam runs" (fake process check),
    multi-user selection, and atomic rename failure.

### 4.3 Multiple users, SD card, Flatpak

| Topic | Finding | Label |
| --- | --- | --- |
| Multiple Steam accounts | Each account has its own `userdata/<id>/config/shortcuts.vdf`; `loginusers.vdf` gives names and `MostRecent`. A shortcut written for one account is invisible to another. | Confirmed |
| Default selection | Picking the `MostRecent` account is reasonable only when it is unambiguous; otherwise ask. SteamTinkerLaunch does exactly this, with a first-folder fallback. | Confirmed (STL) / Proposed (DDL) |
| SD card library | The game library can live on the SD card, but `shortcuts.vdf` and `grid/` stay in the Steam root on internal storage. | Confirmed |
| SD card path stability | A shortcut stores an absolute `Exe`/`StartDir`. If DDL moves, the SD card is mounted at a different path, or the user reinstalls, the shortcut breaks. Steam does not repair it. | Confirmed (format) / Needs Deck Validation (mount behaviour) |
| Flatpak Steam | Flatpak Steam has its own data root; a host-installed `ddl` may not be visible inside the sandbox. On a stock Deck Steam is native, so this is a desktop-Linux caveat, not a Deck blocker. | Proposed |
| Steam running | In Gaming Mode DDL is launched by Steam, so the "Steam is closed" precondition is false for the primary flow. | Confirmed (architecture) |

### 4.4 Risks

| Risk | Why it matters | Label |
| --- | --- | --- |
| File corruption | A truncated or wrongly encoded `shortcuts.vdf` loses every non-Steam entry, not just DDL's. Backups and round-trip parsing are mandatory. | Confirmed (tool behaviour) |
| Steam overwrite | Steam caches the file and can rewrite it; a write while it runs can be lost or conflict. | Confirmed (tool rationale) / Needs Deck Validation (timing) |
| Client updates | Format and storage are undocumented and have changed (randomised IDs, collections JSON). | Confirmed |
| Multi-user | Wrong `userdata` folder means the shortcut never appears for the player. | Confirmed |
| Path drift | Absolute paths break when DDL or the SD card moves. | Confirmed |
| Artwork cost | DDL has screenshots, not capsules/heroes/logos; per-mapset artwork is a content and licensing project. | Confirmed (assets) |
| Error UX | Headless `ddl launch` failures are invisible in Gaming Mode. | Confirmed (CLI) |
| Scope | `OVERVIEW.md` lists "modifying Steam settings/shortcuts files" as an MVP non-goal. Automatic writing needs a decision change. | Confirmed |
| Dependency | `steam_shortcuts_util` is small (3 stars), MIT/Apache-2.0 declared but with no `LICENSE` file in the repository at 2026-10-02; a hand-rolled parser may be safer for a GPL project. | Confirmed (repository) / Proposed (policy) |

### 4.5 What DDL already has

- **Confirmed** — read-only Steam discovery:
  [`crates/ddl-core/src/iwads/steam.rs`](../../crates/ddl-core/src/iwads/steam.rs) resolves the
  same roots, parses `libraryfolders.vdf`, and never writes to Steam. It is the right base for
  locating `userdata`; it is not a shortcut writer.
- **Confirmed** — atomic write patterns exist in
  [`library/manifest.rs`](../../crates/ddl-core/src/library/manifest.rs) and
  [`profiles/store.rs`](../../crates/ddl-core/src/profiles/store.rs).
- **Confirmed** — the DSDA runner already uses `std::process::Command`, captures logs, and waits
  for the engine; no shell is involved
  ([`runner.rs`](../../crates/ddl-core/src/engines/dsda/runner.rs)).
- **Confirmed** — there is no existing code that reads or writes `shortcuts.vdf`, `loginusers.vdf`
  or `config/grid/`. This would be new surface area.

## 5. Open questions for Deck validation

1. Does a shortcut with `Exe=/path/to/ddl` and `LaunchOptions=launch <id>` start correctly from
   Gaming Mode, including paths with spaces and a quoted `Exe`? **Needs Deck Validation.**
2. Does the current client accept a DDL-written `appid` in the reserved range and show
   `grid/<id>p.jpg`, `<id>.jpg`, `<id>_hero.jpg` and `<id>_logo.png` without a Steam restart after
   the first start? **Needs Deck Validation.**
3. Which icon spelling does the current client read: `<id>_icon.*`, `<id>-icon.*`, or both?
   **Needs Deck Validation.**
4. Which image formats and which duplicate-resolution rule does `config/grid` use (JPEG vs PNG,
   alpha, big picture id)? **Needs Deck Validation.**
5. What exactly happens if Steam is asked to close while DDL is running in Gaming Mode — session
   logout, black screen, Steam restart? **Needs Deck Validation.**
6. Does `steam://rungameid/<BPID>` launch a DDL-written shortcut reliably, and is the BPID stable
   across Steam restarts? **Needs Deck Validation.**
7. Does `loginusers.vdf` `MostRecent` match the account actually using DDL on a shared Deck?
   **Needs Deck Validation.**
8. How are non-ASCII names and paths encoded in `shortcuts.vdf` on the current client?
   **Needs Deck Validation.**

## 6. What this means for R3

The mechanism is understood and reproducible, but it is unsupported, fragile, and conflicts with
DDL's normal Gaming Mode lifecycle. Automatic shortcut writing is not a small feature: it needs a
binary VDF parser/writer, a Steam shutdown protocol, multi-user selection, backups, and a
per-mapset artwork pipeline. The recommendation and the proposed next step are in
`docs/reports/043-steam-shortcuts-research.md`.
