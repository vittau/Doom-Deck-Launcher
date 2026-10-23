# 110 — DSDA-Doom sound precache crash: upstream handoff

**Status:** root cause located; upstream fix not written yet · **Depends on:** 106, 109
**Purpose:** give an agent on a Linux machine everything needed to reproduce this crash in
DSDA-Doom, fix it upstream and open a pull request against `dsda-org/dsda-doom`.
**Not a DDL defect:** the mapset crashes the official engine; DDL only launches it.

## Symptom

DSDA-Doom **v0.30.1** (official Linux x86-64 AppImage) aborts at startup with the engine's own
handler:

```
dsda-doom v0.30.1 ...
Playing: DOOM 2: Hell on Earth
 adding <iwad>/doom2.wad
 adding <engine>/dsda-doom.wad
 adding <library>/headless-chicken/RC1-2023-12-09/free/HDLSCHKN.wad
Loading DEH lump from .../HDLSCHKN.wad
Using data file directory: .../dsda_doom_data/doom2/hdlschkn
The game has crashed!
Please report the following information: Segmentation fault (0x0000)
```

With `-verbose` the last lines are:

```
I_InitSound:  configured audio device with 1024 samples/slice
Fluidplayer: Fluidsynth version 2.2.5
portmidiplayer device list:
  ALSA:Midi Through Port-0
portmidiplayer: Opening device ALSA:Midi Through Port-0 for output
I_InitSound: sound module ready
S_Init: Setting up sound.
 Precaching all sound effects... The game has crashed!
```

## Facts established so far

| Observation | Result |
| --- | --- |
| `-nosound` | Game starts and is playable. |
| `-nodraw` (audio only) | Still crashes; video is not involved. |
| `-assign snd_midiplayer=opl` and `=fluidsynth` | Both crash; the music backend is not involved. |
| `-complevel boom` and `-complevel mbf21` | Both crash; compatibility level is not involved. |
| Arachno soundfont installed or not | Both crash; the soundfont is not involved. |
| Shorter install path (spec 109) | Still crashes; path length/characters are not involved. |
| Same WAD, official **macOS** 0.30.1 build | Does **not** crash; Linux build only. |
| Repeated on a CachyOS host (owner) | Same crash. |

Backtrace from the official AppImage, reproduced during this investigation with a one-off
GitHub Actions diagnostic job on `ubuntu-latest` (run 39980316374; the temporary workflow was
removed, the steps are below):

```
Thread 1 "dsda-doom" received signal SIGSEGV, Segmentation fault.
#0  libSDL2-2.0.so.0
#1  libSDL2-2.0.so.0
#2  GetSndData ()
#3  I_CacheSounds ()
#4  S_Init ()
```

## Root cause chain (high confidence)

1. The mapset's WAD embeds a **ZDoom-style `SNDINFO` lump** that names 51 sounds, for example:

   ```
   dsamiss1	dsamiss1
   dsaokay	dsaokay
   tally	tally
   ```

2. `dsda_LoadSndInfo()` (`prboom2/src/dsda/sndinfo.c:46`) calls
   `dsda_LoadAmbientSndInfo()` for non-Hexen games.
3. `dsda_ParseSndInfoLine` (`prboom2/src/dsda/ambient.cpp:176`) stores every `name <lump>`
   pair whose lump exists, and `dsda_ResolveAmbientSounds` (`ambient.cpp:214`) calls
   `dsda_NewSFX(&id)` for each one, **appending new entries to `S_sfx` and raising
   `num_sfx`** (ids 700+ in the log; `-verbose` prints `Named sound: dsamiss1 -> dsamiss1 727`).
4. `S_Init` (`prboom2/src/s_sound.c:173`) calls `dsda_CacheSoundLumps()` and then
   `I_CacheSounds()` (`s_sound.c:207-208`).
5. `I_CacheSounds` (`prboom2/src/SDL/i_sound.c:348`) walks every `S_sfx` entry. For a lump
   that is not DMX it calls `GetSndData()` (`i_sound.c:210`).
6. `GetSndData` decodes with `Load_SNDFile()` (libsndfile) and, when the decoded spec is not
   mono/S16/engine-rate, calls `ConvertAudioFormat()` (`i_sound.c:164`), which uses
   `SDL_BuildAudioCVT` + `SDL_ConvertAudio`. **The crash is inside libSDL2 called from this
   function.**

The mapset ships **17 non-DMX sound lumps**: 16 Ogg Vorbis (`OggS`, the `dsa*` announcer
lines) and one RIFF/WAV (`DSTALLY`). Every other `DS*` lump is DMX version 3 and fine.
Because `IsDMXSound` (`i_sound.c:343`) only accepts DMX `0x03 0x00`, exactly these Ogg/WAV
lumps take the libsndfile path.

## Suspects in `ConvertAudioFormat` (`prboom2/src/SDL/i_sound.c:164-196`)

- `cvt.len = *len;` then `cvt.buf = Z_Malloc(cvt.len * cvt.len_mult);` — `int` arithmetic on
  attacker-controlled lengths; an overflow allocates a short buffer before `memcpy`.
- The decoded `SDL_AudioSpec` from libsndfile is used without validation: `freq` could be 0,
  `channels` outside 1–2, or the format float, any of which can make SDL fail or misbehave.
- `GetSndData` stores `sample.freq`/`samplelen` unchecked, and `I_CacheSounds` then calls
  `FadeInOutMono16((short *)snd_data->data, …)` on whatever came back.
- A defensive fix in `I_CacheSounds` (skip a lump that cannot be decoded or converted, with
  one `LO_WARN`) would turn a crash into a missing sound effect, which is the right trade-off
  for a mapset that ships Ogg sfx.

## How to reproduce

Public data only: the mapset is on /idgames, and any IWAD works (our CI uses Freedoom).

```sh
# 1. engine: official Linux AppImage
curl -LO https://github.com/dsda-org/dsda-doom/releases/download/v0.30.1/dsda-doom-0.30.1-linux-x86_64.appimage
echo "b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0  dsda-doom-0.30.1-linux-x86_64.appimage" | sha256sum -c -
chmod +x dsda-doom-0.30.1-linux-x86_64.appimage
./dsda-doom-0.30.1-linux-x86_64.appimage --appimage-extract

# 2. mapset (Headless Chicken, idgames)
curl -LO https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/Ports/megawads/hdlschkn.zip
echo "de8926c8d3c22dc739718eb06c18da994f3a9fcd12c0f404cbc4c342ec5c4659  hdlschkn.zip" | sha256sum -c -
unzip -q hdlschkn.zip    # HDLSCHKN.wad, 126,358,308 B, sha256 5b7620307c6c8f0203508c1e380a563dda0d62172e65f32406166c3e427380ab

# 3. run headless (crashes before any map loads)
SDL_AUDIODRIVER=dummy SDL_VIDEODRIVER=dummy \
  ./squashfs-root/usr/bin/dsda-doom -iwad <your doom2.wad or freedoom2.wad> \
  -file HDLSCHKN.wad -complevel mbf21 -nodraw -verbose
```

Expected: the run stops at `Precaching all sound effects...` and prints the engine's crash
message. For a stack with symbols, build 0.30.1 from source with debug info. With the AppImage you can
still get frames from its partial symbols; note the debugger itself must not inherit the
AppImage libraries:

```sh
gdb -batch -q -x /dev/stdin --args ./squashfs-root/usr/bin/dsda-doom \
  -iwad <iwad> -file HDLSCHKN.wad -complevel mbf21 -nodraw -verbose <<'EOF'
set environment LD_LIBRARY_PATH <abs-path>/squashfs-root/usr/lib
set environment SDL_AUDIODRIVER dummy
set environment SDL_VIDEODRIVER dummy
run
bt
EOF
```

Useful extra detail for the fix: with `-verbose` the log lists `Named sound: <name> -> <lump>
<id>` for every SNDINFO entry just before the crash, so a breakpoint in `GetSndData` prints the
offending `sfxid`/lump.

## Requested deliverable on the CachyOS machine

1. Confirm the reproduction with a source build (`cmake`, debug info, gdb).
2. Identify the exact lump/SDL call and fix it in DSDA-Doom (guard + graceful skip; avoid the
   `int` length overflow).
3. Run the existing sound tests (`prboom2` has no unit suite for this path, so at minimum:
   the reproducer no longer crashes, sfx still play, Ogg/WAV sfx still work when valid).
4. Open an issue with the backtrace and a PR to `dsda-org/dsda-doom` (current release
   `v0.30.1`; check `patch_notes/` conventions).

## DDL side

- Recipe: `catalog/headless-chicken.toml` (pinned archive/WAD hashes; `complevel = "mbf21"`).
- DDL must not patch the engine or the WAD. Once upstream releases a fix, DDL adds the new
  engine version to `crates/ddl-core/data/engines.toml` with its SHA-256.
- Until then, Headless Chicken is **known broken** on Linux with DSDA-Doom 0.30.1; `-nosound`
  is the only workaround and DDL does not expose it per mapset.
- Evidence: the CI diagnostic runs 39980316374 (backtrace) and 39982006900, this report, and
  the publication history in reports 106/107/109.
