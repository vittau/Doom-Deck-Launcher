# Outreach plan

**Status:** Proposed, awaiting the owner's go. Nothing has been posted.
Merged on 2026-10-26 from the owner's earlier plan (`PUBLISHING.MD`, revision 2026-10-21, kept
outside the repository) and the channel list in [report 080](reports/080-seo-and-ai-discovery-audit.md)
F10. Channel rules marked *(2026-10-21)* come from that plan; agents cannot re-read them
(Reddit blocks automated reads, Doomworld returns 403), so the owner re-checks each on the day.

**Audience:** classic DOOM players on Steam Deck first; WAD fans, Doomworld, Linux gaming and
tool developers second. **Goal:** first real players and feedback, then the inbound links search
and AI answers rely on. Not mass reach: 0.0.x is experimental (D-042).

## 1. The story, not an ad

"I wanted to find and play WADs on my Steam Deck without organizing everything by hand, so I
started building a launcher for it." No launch speech, no forced comparison with GZDeck/UZDeck.

- Call it **experimental**, never ready or stable. The Flatpak is **signed, from the project's
  own repository, not Flathub**. Steam Input is configured manually. DSDA-Doom only.
- Never hide authorship or AI assistance. DDL generates no maps, art or music.
- Do not mention donations where ads/commerce are restricted (above all Steam); no star or
  upvote requests; no indiscriminate crossposting.
- Where a channel forbids LLM-written posts, the owner writes the post from scratch. The draft in
  §4 is a starting point to rewrite per community, never a template to fire everywhere.
- Players get the **site** as the main link (Releases alone means tarball, checksum and manual
  setup); technical communities get site **and** GitHub.

## 2. Gates before the first post

| # | Gate | Who | State |
| --- | --- | --- | --- |
| G1 | README and Deck guide match reality, README kept lean: drop "Pre-alpha" and "Physical Deck installation checks are still pending"; the 0.0.16 owner pass validated update → owned games found → install → play → return with data, input and Flatpak audio OK ([deck-testing](deck-testing.md)) | Coordinator | Done 2026-10-26 |
| G2 | Bug intake: GitHub issue templates (DDL version, install type, `ddl doctor` output; mapset requests point at CONTRIBUTING). Issues are enabled; Discussions stay off | Coordinator | Done 2026-10-26 |
| G3 | Optional: a few Gaming Mode screenshots (Steam + R1) for posts. No video (owner, 2026-10-26). Site images are off-screen renders with sample data; never present them as Deck captures | **Owner** | Optional |
| G4 | Open `doomdecklauncher.com` in the Deck browser and do one fresh install from the `.flatpakref` (HTTPS, layout, links, remote identity, permissions) | **Owner** | Open |
| G5 | DF1–DF2 on 0.0.18 pass, or the post points at a build where they pass | **Owner** | Open |
| G6 | Re-read the channel's rules and get moderation approval where §3 says so | **Owner** | Per channel |

## 3. Channel order

| # | Channel | When | Hook | Main rule / care |
| --- | --- | --- | --- | --- |
| 1 | [r/DoomMods](https://www.reddit.com/r/DoomMods/) | Now (testable experimental build) | Feedback on playing mapsets on the Deck | Confirm launchers are welcome (not only mods); check flair and AI rules |
| 2 | [Steam — DOOM + DOOM II discussions](https://steamcommunity.com/app/2280/discussions/0/) | After permission | The pain of playing WADs on the Deck | Ads/commercial content restricted; FOSS is no automatic exception ([rules](https://help.steampowered.com/en/faqs/view/6862-8119-C23E-EA7B), [conduct](https://store.steampowered.com/online_conduct/)). **Ask moderation before an own thread.** No donation link. Update one thread per milestone |
| 3 | [Doomworld](https://www.doomworld.com/forum/) | After first fixes from 1–2 | DSDA-Doom, complevel, IWAD, load order | Pick the right subforum for tools; one long-lived thread. Open [generative-AI debate](https://www.doomworld.com/forum/topic/156896-generative-ai-and-policy/) (no staff ban found): disclose plainly. Don't lead with Doomer Boards Projects ([history](https://www.doomworld.com/forum/topic/123980-ban-lifted-doomer-boards-projects-will-no-longer-be-permitted-on-the-forums/)) |
| 4 | [r/SteamDeck](https://www.reddit.com/r/SteamDeck/) | Well-validated build | Discover → install → play in Gaming Mode | **Modmail approval before self-promotion** *(2026-10-21)*. Don't say "one-click install"; say not Flathub |
| 5 | [r/SteamDeckHQ](https://www.reddit.com/r/SteamDeckHQ/) | After 4 | Practical use on the Deck | Check rules/flair; short post |
| 6 | [GamingOnLinux](https://www.gamingonlinux.com/contact-us/), [SteamDeckHQ](https://steamdeckhq.com/) editorial | When strangers install and play without help | Native tool for classic maps | One short tip, not a press release; AI disclosure |
| 7 | [awesome-steam-deck](https://github.com/airscripts/awesome-steam-deck) | Documented, working release | Be found when people look for tools | Neutral PR per its guide, no donation link |
| 8 | [r/linux_gaming](https://www.reddit.com/r/linux_gaming/) | **Not before 2026-12-14**, then re-evaluate | Technical SteamOS story | Two-month minimum for substantially AI-assisted projects *(2026-10-21; repo created 2026-10-14)*; post text written by the owner |
| — | [r/Doom](https://www.reddit.com/r/Doom/) | Only with explicit moderator green light | General DOOM interest | Ads and AI rules; skip if refused |

Passive, no thread to babysit: the SteamGridDB page for `DoomDeckLauncher` with the existing
Steam art ([research](research/steam-shortcuts.md)). Do not advertise in the DSDA-Doom, UZDeck or
GZDeck repositories or Discords; mention DDL there only when it answers someone's question.
A third party recommending GZDeck in a Steam thread shows relevance, not permission to advertise.

## 4. Drafts (rewrite per channel)

**Titles** (personal, no product name needed):
- *I've been working on a Doom launcher for the Steam Deck* (default; r/DoomMods: "Doom WAD launcher")
- *I wanted an easier way to play Doom WADs on my Steam Deck, so I made one* (r/SteamDeck)
- Doomworld: *DoomDeckLauncher — a Steam Deck-focused launcher for DSDA-Doom mapsets (experimental)*

**Base post:**

> I've been playing a lot of classic Doom on my Steam Deck, and setting up custom WADs can be a
> bit of a chore — especially when you need the right IWAD, compatibility settings and load order.
>
> So I've been working on **DoomDeckLauncher**, a controller-first launcher built around
> **DSDA-Doom**. You browse community mapsets, review the setup and launch into a game from
> Gaming Mode, without doing all that setup by hand every time. It finds the DOOM games you
> already own on Steam; it never downloads the commercial games.
>
> It's **free and open source**, and **experimental** (v0.0.18). There's a signed Flatpak from the
> project's own repository (not Flathub) on the website, plus a portable Linux build. Expect
> rough edges.
>
> I build it with AI coding agents, under my own direction and review; the maps, art and music are
> the community's, credited, and downloaded from the authors' own pages and /idgames.
>
> If you play custom Doom maps on the Deck, I'd be curious what you think — especially how you
> manage WADs today and which parts annoy you most.
>
> **Website:** https://doomdecklauncher.com/
> **Source:** https://github.com/vittau/Doom-Deck-Launcher

**Doomworld additions:** why DSDA-Doom; recipes record files, SHA-256, load order and complevel;
hashes are verified and archives never executed; local WADs/DEH run with a player-set order. What
it does not do: other ports, bundled IWADs, map redistribution (screenshots credited, removed on
the author's request). Credit UZDeck and GZDeck as inspiration without competing, plus a link to CONTRIBUTING for recipe corrections.

**Editorial tip** (subject: *A new open-source Doom WAD launcher built around the Steam Deck*):

> Hey! I've been building DoomDeckLauncher, a free and open-source launcher for playing classic
> Doom mapsets on Steam Deck using DSDA-Doom. It's controller-first, with a curated discovery
> flow and a signed Flatpak from the project's own repository. It's built with AI coding agents
> under my direction. Website: https://doomdecklauncher.com/ — Source:
> https://github.com/vittau/Doom-Deck-Launcher

**awesome-steam-deck entry:** *DoomDeckLauncher — Controller-first, open-source launcher for
classic Doom mapsets on Steam Deck, using DSDA-Doom.*

## 5. Post checklist

- [ ] Rules re-read and moderation approval where required.
- [ ] Personal, Deck-focused title.
- [ ] What works vs. what is missing is clear; site renders are not passed off as Deck captures.
- [ ] Flatpak named experimental and self-hosted, not Flathub.
- [ ] Authorship and AI use disclosed.
- [ ] Site as the main link for players; GitHub second.
- [ ] No donation pitch, no star/upvote requests, no blanket crossposting.
- [ ] Owner around for the first hours; reproducible bugs become specs.

## 6. Tracking

Record each post (date, link, notable feedback) here and in `docs/journal.yml`. One week later,
read Search Console referrals and GitHub traffic; report numbers, not guesses.

| Channel | State | Link | Result |
| --- | --- | --- | --- |
| r/DoomMods | To post | — | — |
| Steam DOOM + DOOM II | Ask moderation | — | — |
| Doomworld | To post | — | — |
| r/SteamDeck | Ask moderation (modmail) | — | — |
| r/SteamDeckHQ | Check rules | — | — |
| GamingOnLinux / SteamDeckHQ | Contact when mature | — | — |
| awesome-steam-deck | PR pending | — | — |
| r/linux_gaming | Re-evaluate from 2026-12-14 | — | — |
| r/Doom | Only with approval | — | — |
