# Outreach plan

**Status:** Proposed (2026-10-26), awaiting the owner's go. Nothing has been posted.
Earlier work only listed channels ([report 080](reports/080-seo-and-ai-discovery-audit.md)
F10 and §7 item 4); this file turns that list into an ordered plan with drafts.

**Goal:** get the first real Steam Deck players and their feedback, and earn the inbound
links that search and AI answers rely on. The goal is not mass reach: 0.0.x is experimental
(D-042), and the first wave should be small enough to answer every comment.

## 1. Gates before the first post

| # | Gate | Owner | State |
| --- | --- | --- | --- |
| G1 | README and Deck guide match reality: drop "Pre-alpha" / "Physical Deck installation checks are still pending" where the owner has validated them (Flatpak install, Gaming Mode, PD1–PD3), keep **Needs Deck Validation** only where it is still true | Coordinator | Open |
| G2 | A short **Known issues** list in the README: Headless Chicken crashes on Linux with DSDA-Doom 0.30.1 ([dsda-doom#1049](https://github.com/dsda-org/dsda-doom/issues/1049)); four mapsets without screenshots; experimental state may reset (D-042) | Coordinator | Open |
| G3 | Bug intake: GitHub issue templates (bug report asking for DDL version, install type, `ddl doctor` output; mapset request pointing at CONTRIBUTING). Issues are already enabled; Discussions stay off | Coordinator | Open |
| G4 | Real Deck media: 3–4 Gaming Mode screenshots (Steam + R1) and a 30–60 s clip from Home → Discover → install → play | **Owner** (agents never capture screens) | Open |
| G5 | DF1–DF2 on 0.0.18 pass, or the post goes out on a build where they pass | **Owner** | Open |
| G6 | Read each channel's current rules on the day of posting (Reddit and Doomworld block automated reads; see §4) | **Owner** | Open |

## 2. Channels, in order

Post in waves so feedback from one shapes the next. Space them two to four days apart.

1. **r/SteamDeck**, the core audience. Text post with the clip or a screenshot gallery, the
   flair the rules require for software/tools, and the owner present for the first two hours.
   The rules aggregator lists r/SteamDeck self-promotion as "allowed, with limits"
   ([threadfox, read 2026-10-11](https://threadfox.vip/rules/for/games)); confirm the limits.
2. **r/linux_gaming**, listed there as "allowed". Lead with Flatpak, Rust/Slint and GPL.
3. **Doomworld**, in the Source Ports subforum, as the project's long-lived release thread
   (one thread, updated per release, which is the community's norm for tools). The DOOM
   audience is more critical; go there after the first fixes from waves 1–2.
4. **Passive, permanent links** (no thread to babysit): the SteamGridDB page for
   `DoomDeckLauncher` with the existing Steam art ([research 043](research/steam-shortcuts.md)),
   and a news tip to GamingOnLinux once the Doomworld thread exists.
5. **Later, only if invited or relevant:** r/Doom, r/boomershooters. Do **not** post in the
   DSDA-Doom, UZDeck or GZDeck repositories or Discords to advertise DDL; mention it there
   only when it answers someone's question.

## 3. Risks and how the drafts handle them

- **AI-assisted development.** Doomworld has an open debate on generative AI
  ([Generative AI and policy](https://www.doomworld.com/forum/topic/156896-generative-ai-and-policy/)),
  with no staff ban found. Disclose it in one plain line, as the README already does, rather
  than let someone discover it. DDL generates no maps, art or music.
- **Third-party content.** Screenshots are bundled with credit and removed on request
  (D-024); downloads come from the authors' own hosts and /idgames. Say so up front.
- **Doomer Boards Projects** are in the catalog. Their Doomworld history is contentious
  ([2021 thread](https://www.doomworld.com/forum/topic/123980-ban-lifted-doomer-boards-projects-will-no-longer-be-permitted-on-the-forums/)):
  do not lead with them on Doomworld.
- **Expectations.** Say "experimental" and "DSDA-Doom only" in the first lines, so GZDoom/UZDoom
  mod users are not misled.

## 4. Drafts

Placeholders in `<angle brackets>`. Final wording is the owner's; the owner posts from their
own accounts.

### 4.1 r/SteamDeck

**Title:** I made DoomDeckLauncher: a controller-first launcher for community DOOM maps (free, open source)

> Hi all! I've been building **DoomDeckLauncher (DDL)**, a native launcher that turns the DOOM
> games you already own on Steam into a library of community mapsets, all playable from
> Gaming Mode with the controller.
>
> - **Discover** 118 curated mapsets (SIGIL, Eviternity, Sunlust, Back to Saturn X, Ancient
>   Aliens, Scythe and more) by category and tags, then install with one button.
> - DDL finds your Steam (or GOG) DOOM files and sets up the official **DSDA-Doom** engine,
>   load order and compatibility level for you.
> - Profiles, saves and settings are kept per mapset; widescreen status bars where available;
>   optional MIDI soundfont.
> - Signed **Flatpak**: install from the website in Desktop Mode, add to Steam, play.
>
> It's **experimental (v0.0.18)**: expect rough edges, and tell me about them. DSDA-Doom only,
> so GZDoom/UZDoom mods aren't supported. DDL never downloads the commercial games; you bring
> your own.
>
> Site and install steps: https://doomdecklauncher.com/
> Source (GPL-3.0): https://github.com/vittau/Doom-Deck-Launcher
>
> Built with the help of AI coding agents, under my direction and review. Map downloads come
> from the authors' own pages and /idgames, and every screenshot is credited.
>
> I'd love feedback on controller feel and on which mapsets you'd want next.

### 4.2 r/linux_gaming

Same body as 4.1, with the title *DoomDeckLauncher: a Rust/Slint launcher for DOOM mapsets on
Steam Deck, shipped as a signed Flatpak* and one added paragraph: catalog recipes are TOML files
contributed by pull request, downloads are hash-verified and archives are never executed.

### 4.3 Doomworld (Source Ports)

**Title:** DoomDeckLauncher — controller-first mapset launcher for Steam Deck (DSDA-Doom)

> DoomDeckLauncher is a free, GPL launcher for Steam Deck that installs community mapsets for
> **DSDA-Doom** and launches them from Gaming Mode with the controller.
>
> **What it does:** curated recipes record each mapset's files, SHA-256, load order and
> complevel; DDL downloads from the author's page or /idgames, verifies the hashes, finds your
> IWADs (Steam, GOG) and runs the official DSDA-Doom AppImage with separate arguments. You can
> also run any local WAD/DEH with your own load order.
>
> **What it doesn't do:** no other ports, no bundled IWADs, no redistribution of maps. Bundled
> screenshots are credited and removed on request from the author.
>
> **State:** experimental, v0.0.18. Known issue: <Headless Chicken + dsda-doom#1049 status>.
>
> Recipes are TOML files, and corrections from mapset authors are especially welcome:
> <CONTRIBUTING link>. Disclosure: the code is written with AI coding agents under my direction
> and review; no maps, art or music are AI-generated.
>
> <links, 3–4 screenshots>

## 5. After each post

- Answer every comment in the first 48 hours; turn reproducible bugs into specs.
- Record each post (date, link, notable feedback) in `docs/journal.yml`.
- Watch Search Console referrals and GitHub traffic one week later; report numbers, not guesses.
