//! Headless verification of the placeholder window (spec 001, R4).
//!
//! The product runs on a Steam Deck, so the automated tests must not need a
//! display, a window, or a GPU — and nobody working on the repository should
//! have windows popping up while `cargo test` runs. Both are achieved here by
//! driving Slint without a windowing system:
//!
//! * a [`MinimalSoftwareWindow`] is installed as the Slint platform, so the
//!   component is created off-screen;
//! * [`WindowEvent`]s are dispatched straight at the window, which is exactly
//!   what a real keyboard backend does, so Up/Down/Return/Escape exercise the
//!   same `.slint` handlers the Deck will exercise;
//! * the CPU renderer draws the component into a pixel buffer at 1280×800,
//!   and the frames are written as PNGs (into `target/headless/` by default,
//!   or `$DDL_HEADLESS_DIR`) so a human can look at what the menu draws.
//!
//! Everything lives in one `#[test]` on purpose: Slint's context — and with it
//! the installed platform — is thread-local, and a single platform can only be
//! installed once per thread.

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use slint::Model as _;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Key, Platform, PlatformError, WindowAdapter, WindowEvent};
use slint::{PhysicalSize, Rgb8Pixel, SharedPixelBuffer};

use crate::AppWindow;

/// Steam Deck panel resolution (spec 001, R4).
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;

/// Colours from `ui/app-window.slint`, as the renderer writes them.
const BACKGROUND: (u8, u8, u8) = (0x0b, 0x0e, 0x13);
const FOCUSED_ROW: (u8, u8, u8) = (0x2f, 0x6f, 0xdb);
const UNFOCUSED_ROW: (u8, u8, u8) = (0x16, 0x1b, 0x23);

/// A minimal platform whose only window is the off-screen one.
struct HeadlessPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for HeadlessPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }
}

/// Installs the headless platform and creates the placeholder component at
/// 1280×800 with a scale factor of 1 (what the Deck reports).
fn setup() -> (Rc<MinimalSoftwareWindow>, AppWindow) {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(HeadlessPlatform {
        window: window.clone(),
    }))
    .expect("this test is the only thing installing a Slint platform in its thread");

    let ui = AppWindow::new().expect("ui/app-window.slint must compile and instantiate");

    window.dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: 1.0 });
    window.set_size(PhysicalSize::new(WIDTH, HEIGHT));
    (window, ui)
}

/// Sends one key press through the same path a keyboard backend uses.
fn press(window: &MinimalSoftwareWindow, key: Key) {
    window.dispatch_event(WindowEvent::KeyPressed { text: key.into() });
}

/// Draws one frame into a fresh 1280×800 buffer with the software renderer.
fn render(window: &MinimalSoftwareWindow) -> SharedPixelBuffer<Rgb8Pixel> {
    let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(WIDTH, HEIGHT);
    window.request_redraw();
    let drew = window.draw_if_needed(|renderer| {
        renderer.render(buffer.make_mut_slice(), WIDTH as usize);
    });
    assert!(drew, "the software renderer must produce a frame");
    buffer
}

/// Number of pixels that are exactly `color`.
fn count_color(buffer: &SharedPixelBuffer<Rgb8Pixel>, color: (u8, u8, u8)) -> usize {
    buffer
        .as_slice()
        .iter()
        .filter(|pixel| (pixel.r, pixel.g, pixel.b) == color)
        .count()
}

/// Writes `buffer` as a PNG and returns its path.
fn write_png(name: &str, buffer: &SharedPixelBuffer<Rgb8Pixel>) -> PathBuf {
    let dir = std::env::var_os("DDL_HEADLESS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/headless"));
    std::fs::create_dir_all(&dir).expect("create the snapshot directory");
    let path = dir.join(name);

    let file = std::fs::File::create(&path).expect("create the snapshot file");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), WIDTH, HEIGHT);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("write the PNG header");
    writer
        .write_image_data(buffer.as_bytes())
        .expect("write the PNG pixels");
    writer.finish().expect("finish the PNG");
    path
}

#[test]
fn placeholder_menu_is_keyboard_navigable_and_draws_a_focus_highlight() {
    let (window, ui) = setup();

    // R4: the window carries the product name and three placeholder entries.
    assert_eq!(ui.get_window_title(), "DoomDeckLauncher");
    let items = ui.get_items();
    let labels: Vec<String> = (0..items.row_count())
        .map(|index| items.row_data(index).expect("every row exists").to_string())
        .collect();
    assert_eq!(labels, ["Play", "Settings", "Quit"]);

    // Callbacks are recorded instead of acted upon: exiting the process is the
    // binary's job, not the test's.
    let activated = Rc::new(Cell::new(-1));
    let quits = Rc::new(Cell::new(0));
    {
        let activated = activated.clone();
        ui.on_menu_activated(move |index| activated.set(index));
    }
    {
        let quits = quits.clone();
        ui.on_quit_requested(move || quits.set(quits.get() + 1));
    }

    // R4: "Play" holds the initial focus.
    assert_eq!(ui.get_selected(), 0);

    // R4: Down moves the highlight and stops at the last entry.
    for (expected, why) in [(1, "Down moves to Settings"), (2, "Down moves to Quit")] {
        press(&window, Key::DownArrow);
        assert_eq!(ui.get_selected(), expected, "{why}");
    }
    press(&window, Key::DownArrow);
    assert_eq!(
        ui.get_selected(),
        2,
        "Down must not run past the last entry"
    );

    // R4: Up moves it back and stops at the first entry.
    for (expected, why) in [(1, "Up moves to Settings"), (0, "Up moves to Play")] {
        press(&window, Key::UpArrow);
        assert_eq!(ui.get_selected(), expected, "{why}");
    }
    press(&window, Key::UpArrow);
    assert_eq!(ui.get_selected(), 0, "Up must not run past the first entry");

    // R4: the highlight is clearly visible and follows the selection, which is
    // what the two rendered frames prove.
    let play_focused = render(&window);
    press(&window, Key::DownArrow);
    press(&window, Key::DownArrow);
    assert_eq!(ui.get_selected(), 2);
    let quit_focused = render(&window);

    let play_png = write_png("menu-play-focused.png", &play_focused);
    let quit_png = write_png("menu-quit-focused.png", &quit_focused);

    let highlight = count_color(&play_focused, FOCUSED_ROW);
    assert!(
        highlight > 10_000,
        "the focused row must be a solid highlight band, found {highlight} pixels"
    );
    assert!(
        count_color(&quit_focused, FOCUSED_ROW) > 10_000,
        "the highlight must also be drawn on the last row"
    );
    assert!(
        count_color(&play_focused, UNFOCUSED_ROW) > 10_000,
        "unfocused rows must still be drawn"
    );
    assert!(
        count_color(&play_focused, BACKGROUND) > 10_000,
        "the window background must be painted"
    );
    assert_ne!(
        play_focused.as_bytes(),
        quit_focused.as_bytes(),
        "moving the selection must visibly move the highlight"
    );
    assert!(
        !play_png.as_os_str().is_empty() && !quit_png.as_os_str().is_empty(),
        "frames are written to disk"
    );

    // R4: Return activates the focused entry, and "Quit" ends the session.
    press(&window, Key::Return);
    assert_eq!(activated.get(), 2, "Return activates the focused entry");
    assert_eq!(quits.get(), 1, "activating Quit asks the host to exit");

    // R4: Escape on the menu exits too, without activating anything.
    press(&window, Key::Escape);
    assert_eq!(quits.get(), 2, "Escape asks the host to exit");
    assert_eq!(activated.get(), 2, "Escape must not activate a menu entry");
}
