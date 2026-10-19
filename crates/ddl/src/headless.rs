//! Spec 008: off-screen software rendering and real synthetic keyboard dispatch.
use std::path::PathBuf;
use std::rc::Rc;

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Key, Platform, PlatformError, WindowAdapter, WindowEvent};
use slint::{ComponentHandle, Model as _};
use slint::{PhysicalSize, Rgb8Pixel, SharedPixelBuffer};

use crate::AppWindow;

/// Steam Deck panel resolution (spec 001, R4).
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;

/// A minimal platform whose only window is the off-screen one.
struct HeadlessPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

type QueuedCallback = Box<dyn FnOnce() + Send>;
static EVENTS: std::sync::Mutex<Vec<QueuedCallback>> = std::sync::Mutex::new(Vec::new());
struct HeadlessProxy;
impl slint::platform::EventLoopProxy for HeadlessProxy {
    fn quit_event_loop(&self) -> Result<(), slint::EventLoopError> {
        Ok(())
    }
    fn invoke_from_event_loop(&self, event: QueuedCallback) -> Result<(), slint::EventLoopError> {
        EVENTS.lock().unwrap().push(event);
        Ok(())
    }
}
pub(crate) fn pump_events() {
    let events = std::mem::take(&mut *EVENTS.lock().unwrap());
    for event in events {
        event();
    }
}
impl Platform for HeadlessPlatform {
    fn new_event_loop_proxy(&self) -> Option<Box<dyn slint::platform::EventLoopProxy>> {
        Some(Box::new(HeadlessProxy))
    }

    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }
}

/// Installs the headless platform and creates the fixture component at
/// 1280×800 with a scale factor of 1 (what the Deck reports).
pub(crate) fn setup() -> (Rc<MinimalSoftwareWindow>, AppWindow) {
    setup_at(1.0, PhysicalSize::new(WIDTH, HEIGHT))
}

/// Like [`setup`], but the window reports `reported_scale` and is `size`
/// physical pixels (spec 029, A1): the display fit is exercised against a
/// window that lies about its DPI.
pub(crate) fn setup_at(
    reported_scale: f32,
    size: PhysicalSize,
) -> (Rc<MinimalSoftwareWindow>, AppWindow) {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(HeadlessPlatform {
        window: window.clone(),
    }))
    .expect("this test is the only thing installing a Slint platform in its thread");

    let ui = AppWindow::new().expect("ui/app-window.slint must compile and instantiate");
    assert_eq!(
        (
            ui.get_app_icon().size().width,
            ui.get_app_icon().size().height
        ),
        (256, 256)
    );
    assert!(include_str!("../ui/app-window.slint").contains("icon: app-icon;"));

    window.dispatch_event(WindowEvent::ScaleFactorChanged {
        scale_factor: reported_scale,
    });
    window.set_size(size);
    (window, ui)
}

/// Sends one key press through the same path a keyboard backend uses.
fn press(window: &MinimalSoftwareWindow, key: Key) {
    window.dispatch_event(WindowEvent::KeyPressed { text: key.into() });
}

/// Draws one frame into a fresh 1280×800 buffer with the software renderer.
pub(crate) fn render(window: &MinimalSoftwareWindow) -> SharedPixelBuffer<Rgb8Pixel> {
    render_at(window, WIDTH, HEIGHT)
}

/// Draws one frame into a fresh `width`×`height` buffer (spec 029, A1): the
/// display fit renders the same design at a scaled physical size.
pub(crate) fn render_at(
    window: &MinimalSoftwareWindow,
    width: u32,
    height: u32,
) -> SharedPixelBuffer<Rgb8Pixel> {
    let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);
    window.request_redraw();
    let drew = window.draw_if_needed(|renderer| {
        renderer.render(buffer.make_mut_slice(), width as usize);
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

/// Root for headless render artifacts: `DDL_HEADLESS_DIR` or `target/headless`.
fn headless_root() -> PathBuf {
    std::env::var_os("DDL_HEADLESS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/headless"))
}

/// Resolves and creates this test's own artifact directory under the root.
///
/// Every isolated test writes into `<root>/<case>/`, so parallel tests never
/// share a mutable path (spec 027, R3).
fn headless_dir() -> PathBuf {
    let dir = headless_root();
    std::fs::create_dir_all(&dir).expect("create the snapshot directory");
    dir
}

/// Runs `case` alone in a fresh subprocess and returns `true` in the parent.
///
/// Slint allows one platform per process, so every test that renders installs it
/// in its own child process. The child gets a private `DDL_HEADLESS_DIR`
/// (`<root>/<case>`) and its scratch state; the parent returns at once, keeping
/// the default parallel harness free of shared platform or artifact state.
pub(crate) fn isolated(module: &str, case: &str) -> bool {
    isolated_env(module, case, &[])
}

/// Like [`isolated`], but also sets `vars` in the child process (spec 029, A1:
/// a user `SLINT_SCALE_FACTOR` must reach the test without touching the parent
/// process environment, which the parallel harness shares).
pub(crate) fn isolated_env(module: &str, case: &str, vars: &[(&str, &str)]) -> bool {
    const KEY: &str = "DDL_HEADLESS_CASE";
    if std::env::var(KEY).as_deref() == Ok(case) {
        return false;
    }
    // `module_path!()` carries the crate name (`ddl::headless::…`); the test
    // harness names its cases without that prefix.
    let module = module
        .strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
        .unwrap_or(module);
    let dir = headless_root().join(case);
    std::fs::create_dir_all(&dir).expect("create the per-case snapshot directory");
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", &format!("{module}::{case}"), "--nocapture"])
        .env(KEY, case)
        .env("DDL_HEADLESS_DIR", &dir);
    for (name, value) in vars {
        command.env(name, value);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}

/// Writes `buffer` as a PNG and returns its path.
pub(crate) fn write_png(name: &str, buffer: &SharedPixelBuffer<Rgb8Pixel>) -> PathBuf {
    let path = headless_dir().join(name);

    let file = std::fs::File::create(&path).expect("create the snapshot file");
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(file),
        buffer.width(),
        buffer.height(),
    );
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    // Keep identical adaptive-filter scanlines; the large catalog matrix needs
    // fast lossless fixtures, while the README script recompresses for delivery.
    encoder.set_deflate_compression(png::DeflateCompression::FdeflateUltraFast);
    let mut writer = encoder.write_header().expect("write the PNG header");
    writer
        .write_image_data(buffer.as_bytes())
        .expect("write the PNG pixels");
    writer.finish().expect("finish the PNG");
    path
}

/// Exact reference and software frame side by side; no desktop screenshot involved.
fn comparison(name: &str, actual: &SharedPixelBuffer<Rgb8Pixel>) {
    let design = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/design");
    let ember = name
        .strip_prefix("ember-")
        .map(|name| design.join("visual-v2/ember-cathedral").join(name));
    let visual = ember.unwrap_or_else(|| design.join("visual").join(name));
    let reference = if visual.exists() {
        visual
    } else {
        design.join("mockups").join(name)
    };
    if !reference.exists() {
        return;
    }
    let decoder = png::Decoder::new(std::io::BufReader::new(
        std::fs::File::open(reference).unwrap(),
    ));
    let mut reader = decoder.read_info().unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    assert_eq!((info.width, info.height), (WIDTH, HEIGHT));
    assert_eq!(info.color_type, png::ColorType::Rgb);
    let mut side = SharedPixelBuffer::<Rgb8Pixel>::new(WIDTH * 2, HEIGHT);
    for y in 0..HEIGHT as usize {
        let row = &mut side.make_mut_slice()[y * WIDTH as usize * 2..(y + 1) * WIDTH as usize * 2];
        row[..WIDTH as usize]
            .copy_from_slice(&actual.as_slice()[y * WIDTH as usize..(y + 1) * WIDTH as usize]);
        for (x, pixel) in row[WIDTH as usize..].iter_mut().enumerate() {
            let index = (y * WIDTH as usize + x) * 3;
            *pixel = Rgb8Pixel {
                r: bytes[index],
                g: bytes[index + 1],
                b: bytes[index + 2],
            };
        }
    }
    write_png(&format!("comparison-{name}"), &side);
}

/// Specs 008/010: shell navigation, focus and every fixture screen render
/// headlessly. Each production journey is its own isolated test (spec 027).
#[test]
fn shell_navigation_and_focus_render_headlessly() {
    if isolated(
        module_path!(),
        "shell_navigation_and_focus_render_headlessly",
    ) {
        return;
    }
    use crate::{
        fixtures::{MapsetId, files},
        input::Action,
        shell::Screen,
    };
    let (window, ui) = setup();
    let state = crate::shell::attach(&ui);
    let quits = Rc::new(std::cell::Cell::new(0));
    let q = quits.clone();
    ui.on_quit_requested(move || q.set(q.get() + 1));
    let capture = |name: &str| {
        let original = state.borrow().focus;
        let sheet = ui.get_sheet_open();
        let nodes = if sheet {
            ui.get_sheet_nodes()
        } else {
            ui.get_nodes()
        };
        let controls: Vec<_> = (0..nodes.row_count())
            .filter_map(|i| nodes.row_data(i))
            .filter(|n| n.actionable)
            .collect();
        for node in controls {
            if !sheet {
                state.borrow_mut().focus = node.id;
                state.borrow_mut().sync(&ui);
            } else {
                ui.set_focused(node.id);
            }
            let frame = render(&window);
            let (x, y) = if sheet {
                (280. + node.x, 250. + node.y)
            } else {
                (
                    28. + node.x,
                    218. + node.y
                        - if ui.get_setup_screen() && node.x >= 700. {
                            0.
                        } else {
                            ui.get_scroll_offset()
                        },
                )
            };
            let ring_offset = if node.kind == 1 { 14. } else { 6. };
            for (px, py) in [
                (x + node.w / 2., y - ring_offset),
                (x + node.w / 2., y + node.h + ring_offset),
                (x - ring_offset, y + node.h / 2.),
                (x + node.w + ring_offset, y + node.h / 2.),
            ] {
                assert!(
                    px >= 0. && px < WIDTH as f32 && py >= 0. && py < HEIGHT as f32,
                    "ring bounds {name} target {}",
                    node.id
                );
                let pixel = frame.as_slice()[py as usize * WIDTH as usize + px as usize];
                assert_eq!(
                    (pixel.r, pixel.g, pixel.b),
                    (0xff, 0xd0, 0x78),
                    "ring edge {name} target {} at {px},{py}",
                    node.id
                );
            }
        }
        if !sheet {
            state.borrow_mut().focus = original;
            state.borrow_mut().sync(&ui);
        } else {
            ui.set_focused(original);
        }
        let frame = render(&window);
        assert!(
            count_color(&frame, (0xff, 0xd0, 0x78)) > 500,
            "amber focus pixels in {name}"
        );
        write_png(name, &frame);
        comparison(name, &frame);
    };
    let focus = || state.borrow().focus;
    assert_eq!(focus(), 1);
    capture("home.png");
    window.dispatch_event(WindowEvent::PointerMoved {
        position: slint::LogicalPosition::new(600., 470.),
    });
    assert_eq!(focus(), 1, "pointer motion ignored");
    for event in [
        WindowEvent::PointerPressed {
            position: slint::LogicalPosition::new(600., 470.),
            button: slint::platform::PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position: slint::LogicalPosition::new(600., 470.),
            button: slint::platform::PointerEventButton::Left,
        },
    ] {
        window.dispatch_event(event);
    }
    assert_eq!(
        state.borrow().screen,
        Screen::Detail,
        "click activates the same control"
    );
    press(&window, Key::Escape);
    assert_eq!(focus(), 2);
    press(&window, Key::LeftArrow);
    assert_eq!(focus(), 1);
    press(&window, Key::UpArrow);
    assert_eq!(focus(), -1);
    press(&window, Key::LeftArrow);
    assert_eq!(focus(), -1, "spatial navigation never wraps");
    press(&window, Key::RightArrow);
    assert_eq!(focus(), -2);
    press(&window, Key::LeftArrow);
    press(&window, Key::DownArrow);
    assert_eq!(focus(), 1);
    window.dispatch_event(WindowEvent::WindowActiveChanged(false));
    assert!(!ui.get_input_active());
    window.dispatch_event(WindowEvent::WindowActiveChanged(true));
    assert!(ui.get_input_active());

    press(&window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::Home);
    assert!(!ui.get_sheet_open());
    press(&window, Key::Tab);
    assert_eq!(state.borrow().screen, Screen::Discover);
    assert_eq!(focus(), 10);
    press(&window, Key::DownArrow);
    assert_eq!(focus(), 20);
    capture("discover.png");
    press(&window, Key::RightArrow);
    assert_eq!(focus(), 21);
    press(&window, Key::Return);
    assert_eq!(state.borrow().screen, Screen::Detail);
    press(&window, Key::Tab);
    assert_eq!(
        state.borrow().screen,
        Screen::Detail,
        "shoulders ignored on child screens"
    );
    press(&window, Key::Escape);
    assert_eq!(focus(), 21, "Back restores grid target");
    press(&window, Key::LeftArrow);
    assert_eq!(focus(), 20);
    press(&window, Key::Return);
    capture("mapset-detail.png");
    press(&window, Key::Return);
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert_eq!(focus(), 50);
    capture("play-setup-sigil.png");
    press(&window, Key::Return);
    assert_eq!(ui.get_toast(), "Not wired yet");
    assert_eq!(state.borrow().screen, Screen::Setup);
    press(&window, Key::LeftArrow);
    assert_eq!(focus(), 45, "nearest options row from Play");
    press(&window, Key::Return);
    assert!(state.borrow().profiles[0].pistol);
    press(&window, Key::UpArrow);
    assert_eq!(focus(), 43);
    press(&window, Key::Return);
    assert!(state.borrow().profiles[0].advanced);
    press(&window, Key::Return);
    assert!(ui.get_sheet_open());
    assert_eq!(focus(), 100);
    press(&window, Key::Escape);
    assert!(!state.borrow().profiles[0].compat);
    assert_eq!(focus(), 43);
    press(&window, Key::Return);
    press(&window, Key::RightArrow);
    press(&window, Key::Return);
    assert!(state.borrow().profiles[0].compat);
    assert_eq!(
        files(MapsetId::Sigil, &state.borrow().profiles[0]),
        vec!["SIGIL_COMPAT_V1_23.wad"]
    );
    press(&window, Key::UpArrow);
    assert_eq!(focus(), 42);
    press(&window, Key::Return);
    assert_eq!(ui.get_toast(), "Not wired yet");
    press(&window, Key::UpArrow);
    assert_eq!(focus(), 41);
    press(&window, Key::Return);
    assert_eq!(
        files(MapsetId::Sigil, &state.borrow().profiles[0]).len(),
        1,
        "required row cannot disable main"
    );
    press(&window, Key::Escape);
    assert_eq!(focus(), 30);
    press(&window, Key::Escape);
    assert_eq!(focus(), 20);
    press(&window, Key::UpArrow);
    assert_eq!(focus(), 11);
    press(&window, Key::UpArrow);
    assert_eq!(focus(), -2);
    let header = render(&window);
    let pixel = header.as_slice()[18 * WIDTH as usize + 468];
    assert_eq!(
        (pixel.r, pixel.g, pixel.b),
        (0xff, 0xd0, 0x78),
        "top tab focus ring"
    );
    press(&window, Key::DownArrow);
    assert_eq!(focus(), 11);
    press(&window, Key::F3);
    assert_eq!(state.borrow().screen, Screen::Library);
    capture("library.png");
    press(&window, Key::F4);
    assert_eq!(state.borrow().screen, Screen::Settings);
    capture("settings.png");
    press(&window, Key::DownArrow);
    assert_eq!(focus(), 61);
    press(&window, Key::RightArrow);
    assert_eq!(focus(), 70);
    press(&window, Key::Escape);
    assert_eq!(focus(), 61);
    press(&window, Key::Menu);
    assert!(ui.get_sheet_open());
    assert_eq!(focus(), 100);
    capture("quit-confirmation.png");
    press(&window, Key::Tab);
    assert_eq!(focus(), 100, "sheet traps root tab switching");
    press(&window, Key::Escape);
    assert_eq!(focus(), 61);
    press(&window, Key::F2);
    press(&window, Key::DownArrow);
    press(&window, Key::RightArrow);
    press(&window, Key::Return);
    press(&window, Key::Return);
    assert_eq!(state.borrow().mapset, MapsetId::Auger);
    assert!(
        !state.borrow().profiles[1].pistol,
        "mapset options stay independent"
    );
    for expected in [4, 5, 1, 2, 3] {
        ui.invoke_clicked(44);
        assert_eq!(state.borrow().profiles[1].skill, expected);
    }
    assert_eq!(state.borrow().profiles[0].skill, 3);
    // Use the same public click callback as pointer activation, never private rendering shortcuts.
    ui.invoke_clicked(43);
    assert_eq!(
        files(MapsetId::Auger, &state.borrow().profiles[1]),
        vec!["DBP37_AUGZEN.wad", "DBP37_AlternateBlaster.wad"]
    );
    capture("play-setup-auger.png");
    ui.invoke_clicked(43);
    assert_eq!(
        files(MapsetId::Auger, &state.borrow().profiles[1])[1],
        "DBP37_DeckardBlaster.wad"
    );
    ui.invoke_clicked(43);
    assert_eq!(files(MapsetId::Auger, &state.borrow().profiles[1]).len(), 1);
    press(&window, Key::Escape);
    press(&window, Key::Escape);
    press(&window, Key::RightArrow);
    assert_eq!(focus(), 22);
    press(&window, Key::Return);
    press(&window, Key::Return);
    assert_eq!(ui.get_toast(), "Not wired yet");
    assert_eq!(
        state.borrow().screen,
        Screen::Detail,
        "Install is a fixture action"
    );
    ui.invoke_clicked(37);
    assert_eq!(state.borrow().mapset, MapsetId::Sigil2);
    capture("play-blocked-iwad.png");
    press(&window, Key::Return);
    assert_eq!(ui.get_toast(), "Not wired yet");
    assert_eq!(state.borrow().screen, Screen::Setup);
    ui.invoke_clicked(51);
    assert_eq!(
        ui.get_toast(),
        crate::copy::text(&ui, crate::copy::Copy::Missing)
    );
    assert_eq!(state.borrow().screen, Screen::Setup);
    let before = focus();
    state.borrow().hotplug(&ui, false);
    assert_eq!(focus(), before);
    state.borrow().hotplug(&ui, true);
    assert_eq!(focus(), before);
    // Cycle through all root tabs; a full cycle restores the same control.
    press(&window, Key::Escape);
    press(&window, Key::Escape);
    for _ in 0..4 {
        press(&window, Key::Tab);
    }
    assert_eq!(focus(), 22);
    state.borrow_mut().action(&ui, Action::Menu);
    press(&window, Key::Escape);
    assert_eq!(focus(), 22);
    assert_eq!(quits.get(), 0, "B and cancelled sheets never quit");
    press(&window, Key::Menu);
    press(&window, Key::Return);
    assert_eq!(quits.get(), 0, "Keep playing is the safe default");
    press(&window, Key::Menu);
    press(&window, Key::RightArrow);
    press(&window, Key::Return);
    assert_eq!(quits.get(), 1, "only explicit confirmation quits");
    let _ = ui.window(); // ComponentHandle methods are available without showing a window.
}

#[path = "headless_production.rs"]
mod production;

#[path = "headless_journeys.rs"]
mod journeys;
