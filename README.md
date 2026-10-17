<p align="center"><img src="docs/design/logo/full-name.svg" alt="DoomDeckLauncher — Ember Gate logo" width="700"></p>

# DoomDeckLauncher

**A native, controller-first DOOM mapset launcher for Steam Deck, powered by the official [DSDA-Doom](https://github.com/dsda-org/dsda-doom) AppImage.**

Discover community maps, choose your setup, and play from Gaming Mode.
Built in Rust and [Slint](https://slint.dev).

**Pre-alpha:** these screenshots show the current source tree; published releases may lag.
**Needs Deck Validation:** Gaming Mode, input handoff and gameplay — [test checklist](docs/deck-testing.md).

| Home | Discover |
| --- | --- |
| ![Home with recently played mapsets](docs/screenshots/home.png) | ![Discover with categories, tags and a favorite](docs/screenshots/discover.png) |
| Play setup | Mapset detail |
| ![Play setup with component choices and load order](docs/screenshots/play-setup.png) | ![Mapset detail with tags and favorite](docs/screenshots/mapset-detail.png) |

*Headless renders of the real UI with sample state; artwork is original mood art.*
Click an image to inspect it at 1280×800.

## Features

- Controller-first navigation with an Ember Cathedral interface.
- Curated mapset recipes with explicit components and load order.
- Discover categories, tags and persistent favorites.
- Install the official DSDA-Doom AppImage, with an extraction fallback.
- Find your own IWADs; keep engine settings and saves isolated.
- Diagnose engine, IWAD and mapset problems with the same `ddl` command-line tools.

## Install on Steam Deck

1. In Desktop Mode, download `ddl-linux-x86_64.tar.gz` from [GitHub Releases](https://github.com/vittau/Doom-Deck-Launcher/releases) (checksum included).
2. Extract it somewhere convenient and keep the `ddl/` directory together.
3. Steam → **Games → Add a Non-Steam Game → Browse** → select the extracted `ddl` executable.
4. In the shortcut's Properties, **leave Compatibility → Force a compatibility tool off**.
5. Launch DDL, install DSDA-Doom and add a folder containing your legally owned IWADs.
6. Return to Gaming Mode; configure [Classic or Modern + Gyro controls](docs/steam-deck-guide.md#recommended-steam-deck-controller-setup), then discover a mapset and play.

[Steam Deck guide](docs/steam-deck-guide.md) ·
[Steam artwork](docs/steam-deck-guide.md#make-it-look-native-in-steam) ·
[Konsole troubleshooting](docs/steam-deck-guide.md#troubleshooting-from-konsole) ·
[Build from source](docs/steam-deck-guide.md#build-from-source) ·
[Contribute a mapset](CONTRIBUTING.md)

## License

DDL is licensed under the **GNU General Public License, version 3 or later**
(GPL-3.0-or-later). See [`LICENSE`](LICENSE).

DDL is not affiliated with, endorsed by, or sponsored by id Software, Bethesda
Softworks, ZeniMax, Microsoft, Valve Corporation, Doomworld, or the DSDA-Doom
maintainers. **DOOM** is a trademark of id Software LLC; **Steam** and **Steam Deck**
are trademarks of Valve Corporation. All trademarks belong to their respective
owners and are used here only to describe what DDL does.

## Credits

- **[UZDeck](https://github.com/karkusviktor/UZDeck)** and
  **[GZDeck](https://github.com/flegald/GZDeck)** — the projects that inspired DDL.
  DDL is an independent reimplementation; no original code or assets have been
  reused.
- **[DSDA-Doom](https://github.com/dsda-org/dsda-doom)** — the DOOM source port DDL
  targets, used unmodified as the official AppImage.
- **[Slint](https://slint.dev)** — the native UI toolkit DDL is built with.
- The DOOM mapping community, whose mapsets DDL exists to make playable.
