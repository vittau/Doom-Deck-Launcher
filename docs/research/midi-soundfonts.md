# FluidSynth soundfonts

Verified 2026-10-21 for spec 067. Engine baseline: DSDA-Doom v0.30.1.

## Engine behavior — Confirmed

- [`configuration.c`](https://github.com/kraflab/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c)
  registers `snd_midiplayer` with default `"fluidsynth"` and `snd_soundfont` with default
  empty string (lines 752–763).
- [`flplayer.c`](https://github.com/kraflab/dsda-doom/blob/v0.30.1/prboom2/src/MUSIC/flplayer.c#L268-L318)
  loads a nonempty `snd_soundfont` filename with `fluid_synth_sfload`. An empty setting uses
  the `SNDFONT` lump. A replacement SNDFONT supplied outside the port WAD takes priority over
  the configured external file; failed external loading also falls back to the lump. Preserve
  this engine behavior; DDL must not claim its selector overrides a mapset's embedded font.
- The [bundled font credits](https://github.com/kraflab/dsda-doom/blob/v0.30.1/prboom2/data/lumps/soundfont/CREDITS.txt)
  identify the engine soundfont as deemster's SC-55 font. The owner's requested UI label is
  simply **Default**. No extra download is needed for Default.
- DDL's existing `-assign` override path supports integers, not strings. Do not put an SF2
  path through it. Use the guarded DSDA config manager between engine runs; strings have
  no escape syntax in DSDA's configuration grammar (existing engine research §3–4).

## Arachno 1.0 distribution — Confirmed

Primary source: Maxime Abbey's
[SF2 download page](https://www.arachnosoft.com/main/download.php?id=soundfont-sf2).
It links the mirrors below. The SF2 ZIP already contains the decompressed font; its generic
Read Me still mentions sfArk, but this package needs no converter or executable.

- Author-linked Dropbox URL:
  `https://www.dropbox.com/s/2rnpya9ecb9m4jh/arachno-soundfont-10-sf2.zip?dl=1`
- Dropbox currently redirects the share to
  `https://www.dropbox.com/scl/fi/o7ctduftiauo5sivcani2/arachno-soundfont-10-sf2.zip?rlkey=l55b9ry7z4bt20htue35joi9z&dl=1`
  and a temporary download host. Reuse DDL's existing narrowly scoped Dropbox redirect policy.
- Author's second-server link is HTTP:
  `http://maxime.abbey.free.fr/mirror/arachnosoft/files/soundfonts/arachno-soundfont-10-sf2.zip`.
  Do not weaken HTTPS download policy to use it.
- Owner also supplied `https://musical-artifacts.com/artifacts/2045`; both browser fetch and
  curl returned HTTP 403. Its identity/bytes/terms were not established; do not treat it as
  a reviewed mirror or substitute another soundfont.

Downloaded the complete author-linked Dropbox SF2 ZIP, without executing or installing it:

| Object | Exact bytes | SHA-256 |
| --- | ---: | --- |
| ZIP | 143356181 | `5c0c573abe6c5e1a3ccceba5d52a4229db24283620c9a3eb1f403018e2aa22df` |
| `Arachno SoundFont - Version 1.0.sf2` | 155405818 | `9a57fb3b6714e69dda12390e351b087e81fc3b1eca15c6b4bbe172799f4cf3cd` |
| `Read Me.txt` | 3388 | `d7dab992c37c68e61decea3406d7b232b5382a5c4b7fd89af9ec09117e046873` |
| `Documentation/Arachno SoundFont Documentation - Version 1.2 - English.html` | 441987 | `ed23308db66faa6dfee13019e0978dd1700d9a08a3d59672801613819426ca92` |

The archive has 454 entries, total declared uncompressed size 173559660 bytes. The SF2 starts
with RIFF and form type `sfbk`; the little-endian RIFF size plus eight equals its file length.
Only install the reviewed font and notices. No sample MIDI files, executables, URL shortcuts
or third-party utilities are needed. Keep exact SF2 bytes unchanged.

## Terms and attribution — Confirmed

The author's [English documentation](https://www.arachnosoft.com/main/soundfont.php?documentation=)
(introduction and copyright sections) and the ZIP's Read Me describe a freeware bank assembled
from multiple authors' samples, primarily for private, noncommercial use; commercial use needs
permission from the credited original authors. This is not a GPL or public-domain asset.

DDL provides an optional download directly from the author's listed mirror for the player's
personal use, credits Maxime Abbey, retains the supplied notices and links the original terms.
Do not bundle it into DDL, its Flatpak, repository or release assets, and do not rehost it.
Bundled HTML is untrusted data: retain it as a notice, never embed/render it as application UI
or execute content from it. Use the original HTTPS documentation link for an online Terms action.

## Needs Deck Validation

Audible switch between Default and Arachno for MIDI/MUS music with the official AppImage;
Flatpak sandbox path access; next-launch application and return to Default. No performance or
audio-quality improvement is claimed from the file inspection alone. Non-MIDI prerecorded
soundtracks are outside this feature.
