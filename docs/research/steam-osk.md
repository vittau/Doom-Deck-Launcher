# Steam on-screen keyboard for profile names (spec 031)

## Evidence and choice

**Confirmed (upstream implementation):** SDL's X11 backend opens the Steam Deck
keyboard through a `steam://open/keyboard?...` URI. See the pinned
[SDL commit 51dfca813b6c02746c782a1b1f194cde82e9dbb2](https://github.com/libsdl-org/SDL/commit/51dfca813b6c02746c782a1b1f194cde82e9dbb2),
`src/video/x11/SDL_x11keyboard.c`. DDL does not depend on SDL.
The simpler `steam steam://open/keyboard` invocation is independently reproduced in
[Valve's Steam-for-Linux issue 11404](https://github.com/ValveSoftware/steam-for-linux/issues/11404).
This is evidence of a client mechanism, not a documented stability guarantee.

**Proposed / Needs Deck Validation:** on gaining focus, DDL's profile TextInput
requests `Command::new("steam").args(["steam://open/keyboard"])` on Linux. The
helper is behind a launcher trait; tests replace the launcher and never invoke
Steam. Other hosts return Unsupported. The child is reaped away from the UI
thread. Starting a helper successfully cannot prove that the overlay appeared.
The input therefore always shows the one-line fallback hint, `Steam + X`.
Valve explicitly documents this shortcut in its
[Desktop FAQ](https://help.steampowered.com/en/faqs/view/671A-4453-E8D2-323C).

## Text delivery and focus

**Confirmed (Slint contract):** a focused TextInput receives keyboard events,
including virtual keyboard input; `focus()` requests that focus. See
[Slint focus handling](https://releases.slint.dev/1.7.0/docs/slint/src/language/concepts/focus)
and [TextInput reference](https://docs.slint.dev/latest/docs/slint/reference/keyboard-input/textinput/).
DDL uses Slint 1.18.1. The field handles text itself; root shortcuts such as x/y
and brackets must not consume typed names. Return submits, Escape backs out,
and Down/Tab moves to Save. Focus leaving the field returns keyboard navigation
to the root scope.

**Verified headlessly:** synthetic WindowEvent key events exercise the actual
Slint TextInput, focus callback, selection replacement, text insertion and
submission; a fake launcher checks program and argv on focus. This verifies
DDL's focus and text-event path without opening a visible window.

**Needs Deck Validation:** the Steam keyboard is expected to deliver ordinary
virtual-keyboard events to the focused X11/XWayland window. Check the real
non-Steam shortcut in Gaming Mode: overlay visibility, focus retention, typing
(including x/y and punctuation), dismissal, and Steam + X fallback. The macOS
software-renderer tests cannot establish Steam/gamescope event delivery or
whether the overlay obscures this field. Steam's issue tracker contains
[overlay/focus failures](https://github.com/ValveSoftware/steam-for-linux/issues/12920),
so successful argv tests must not be reported as Deck validation.
