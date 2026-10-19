//! Fitting the launcher to the physical screen (spec 029, R1/R2).
//!
//! Gaming Mode runs DDL under gamescope's Xwayland, where winit reports a large
//! scale factor for the 7" 1280×800 panel (spec 029 Facts). Slint then lays the
//! 1280×800 design out at that factor, so only part of it fits. DDL therefore
//! ignores the reported DPI factor in fullscreen and derives one from the
//! window's **physical** size instead:
//!
//! ```text
//! applied = min(physical_width / 1280, physical_height / 800)
//! ```
//!
//! The Deck panel therefore gets exactly `1.0`, and a larger screen scales the
//! design up without cropping. The `.slint` root pins a 1280×800 logical size,
//! so the spare axis is **letterboxed with the window background** (the design's
//! own canvas colour), not stretched; the design stays fully visible. A factor
//! the user set through `SLINT_SCALE_FACTOR` still wins: Slint's winit backend
//! reads that variable itself, and DDL reproduces the same value here so both
//! agree.
//!
//! `--windowed` keeps a 1280×800 *logical* window and is left at the reported
//! factor, matching the documented `--windowed` behaviour.
//!
//! Everything here goes through the small [`DisplayWindow`] trait rather than a
//! `.slint` file, so the fit is a unit-testable pure function and the headless
//! tests can supply a fake window without a real backend.

use slint::{PhysicalSize, Window};
use tracing::info;

/// The logical design the `.slint` files are authored at.
pub const DESIGN_WIDTH: f32 = 1280.0;
/// The logical design the `.slint` files are authored at.
pub const DESIGN_HEIGHT: f32 = 800.0;

/// Environment variable Slint's winit backend reads itself; a user value wins.
pub const SCALE_ENV: &str = "SLINT_SCALE_FACTOR";

/// How the window is shown; it decides whether the fit is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Fill the screen; the fit is applied from the physical size (R1).
    Fullscreen,
    /// A 1280×800 logical window; the reported factor is kept (R1).
    Windowed,
}

/// Who chose the factor DDL applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The user set `SLINT_SCALE_FACTOR`; it overrides the fit (R1).
    User,
    /// DDL derived `min(width / 1280, height / 800)` from the physical size.
    Fitted,
    /// The reported backend factor was kept (`--windowed`).
    Reported,
}

impl Source {
    fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Fitted => "fitted",
            Self::Reported => "reported",
        }
    }
}

/// The three values R2 records, plus who chose the applied one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Report {
    /// The scale factor winit reported (ignored in fullscreen).
    pub reported: f32,
    /// The window's physical size.
    pub physical: PhysicalSize,
    /// The factor DDL asked Slint to use.
    pub applied: f32,
    /// Where [`Report::applied`] came from.
    pub source: Source,
}

/// The window operations the fit needs.
///
/// Implemented for [`slint::Window`] in production and by a fake in tests, so
/// the decision logic never has to touch a real backend (spec 029, R1).
pub trait DisplayWindow {
    /// The scale factor the backend reported (DPI).
    fn reported_scale_factor(&self) -> f32;
    /// The window's physical size in pixels.
    fn physical_size(&self) -> PhysicalSize;
    /// Asks Slint to use `factor` for this window.
    fn apply_scale_factor(&self, factor: f32);
}

impl DisplayWindow for Window {
    fn reported_scale_factor(&self) -> f32 {
        self.scale_factor()
    }

    fn physical_size(&self) -> PhysicalSize {
        self.size()
    }

    fn apply_scale_factor(&self, factor: f32) {
        // The public Slint API path for changing the scale factor: the backend
        // dispatches the same event when winit reports a DPI change.
        self.dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
            scale_factor: factor,
        });
        // Changing the factor alone does not re-lay the root element out; the
        // backend normally follows it with the window's (new) logical size so
        // the design is laid out at the applied factor.
        let logical = slint::LogicalSize::from_physical(self.size(), factor);
        self.dispatch_event(slint::platform::WindowEvent::Resized { size: logical });
    }
}

/// The fit factor for a physical window, ignoring the reported DPI (R1).
///
/// Always positive and finite: a window that reports no size yet yields `1.0`
/// so the design is never scaled to nothing.
#[must_use]
pub fn fit_factor(physical: PhysicalSize) -> f32 {
    let factor = (physical.width as f32 / DESIGN_WIDTH).min(physical.height as f32 / DESIGN_HEIGHT);
    sane(factor)
}

/// Parses a `SLINT_SCALE_FACTOR` value the way Slint's winit backend does: a
/// positive number, anything else ignored.
#[must_use]
pub fn parse_user_factor(value: Option<&str>) -> Option<f32> {
    let factor: f32 = value?.trim().parse().ok()?;
    (factor > 0.0 && factor.is_finite()).then_some(factor)
}

/// Computes, applies and logs the scale factor for `window` (R1/R2).
///
/// The user's `SLINT_SCALE_FACTOR` wins over the fit; otherwise the fit is
/// applied in fullscreen and the reported factor is kept in windowed mode.
pub fn configure(window: &dyn DisplayWindow, mode: Mode) -> Report {
    configure_with_user_factor(window, mode, user_factor())
}

/// [`configure`] with the user factor supplied explicitly (tests and callers
/// that already resolved the environment).
pub fn configure_with_user_factor(
    window: &dyn DisplayWindow,
    mode: Mode,
    user: Option<f32>,
) -> Report {
    let reported = window.reported_scale_factor();
    let physical = window.physical_size();
    let (applied, source) = match user {
        Some(factor) => (factor, Source::User),
        None => match mode {
            Mode::Fullscreen => (fit_factor(physical), Source::Fitted),
            Mode::Windowed => (sane(reported), Source::Reported),
        },
    };

    window.apply_scale_factor(applied);
    info!(
        reported,
        physical_width = physical.width,
        physical_height = physical.height,
        applied,
        source = source.as_str(),
        "display scale factor"
    );

    Report {
        reported,
        physical,
        applied,
        source,
    }
}

/// The largest difference between two factors that still counts as "the same".
const FACTOR_EPSILON: f32 = 0.01;

/// Whether the backend now reports a different factor than the one DDL applied.
///
/// Winit can dispatch its own `ScaleFactorChanged` *after* DDL fitted the window
/// (for example when gamescope maps the fullscreen window); the physical size
/// does not change, so only the reported factor reveals the drift.
#[must_use]
pub fn factor_drifted(reported: f32, applied: f32) -> bool {
    (reported - applied).abs() > FACTOR_EPSILON
}

/// Re-applies the fit when a later backend event changed the reported factor.
///
/// Returns the new [`Report`] (and logs it) when a drift was corrected, or
/// `None` when there is nothing to do. Only fullscreen with no user
/// `SLINT_SCALE_FACTOR` is refitted: windowed mode keeps the reported factor by
/// design, and a user value must never be fought.
pub fn refit_if_drifted(window: &dyn DisplayWindow, mode: Mode, applied: f32) -> Option<Report> {
    refit_if_drifted_with(window, mode, applied, user_factor())
}

/// [`refit_if_drifted`] with the user factor supplied explicitly (tests).
pub fn refit_if_drifted_with(
    window: &dyn DisplayWindow,
    mode: Mode,
    applied: f32,
    user: Option<f32>,
) -> Option<Report> {
    if mode != Mode::Fullscreen || user.is_some() {
        return None;
    }
    factor_drifted(window.reported_scale_factor(), applied)
        .then(|| configure_with_user_factor(window, mode, user))
}

fn user_factor() -> Option<f32> {
    parse_user_factor(std::env::var(SCALE_ENV).ok().as_deref())
}

/// Clamps a factor to a usable, positive value.
fn sane(factor: f32) -> f32 {
    if factor.is_finite() && factor > 0.0 {
        factor
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::{Arc, Mutex};

    use slint::platform::WindowEvent;
    use slint::{ComponentHandle, Rgb8Pixel, SharedPixelBuffer};

    use super::*;

    /// A `DisplayWindow` with no backend (unit tests).
    struct FakeDisplay {
        reported: f32,
        physical: PhysicalSize,
        applied: Cell<f32>,
    }

    impl FakeDisplay {
        fn new(reported: f32, physical: PhysicalSize) -> Self {
            Self {
                reported,
                physical,
                applied: Cell::new(0.0),
            }
        }
    }

    impl DisplayWindow for FakeDisplay {
        fn reported_scale_factor(&self) -> f32 {
            self.reported
        }
        fn physical_size(&self) -> PhysicalSize {
            self.physical
        }
        fn apply_scale_factor(&self, factor: f32) {
            self.applied.set(factor);
        }
    }

    /// Captures everything a test subscriber writes.
    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl Capture {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    impl std::io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
        type Writer = Capture;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn amber_at(buffer: &SharedPixelBuffer<Rgb8Pixel>, cx: f32, cy: f32, radius: i32) -> bool {
        let width = buffer.width() as i32;
        let height = buffer.height() as i32;
        let left = (cx as i32 - radius).max(0);
        let right = (cx as i32 + radius).min(width - 1);
        let top = (cy as i32 - radius).max(0);
        let bottom = (cy as i32 + radius).min(height - 1);
        for y in top..=bottom {
            for x in left..=right {
                let pixel = buffer.as_slice()[y as usize * width as usize + x as usize];
                if (pixel.r, pixel.g, pixel.b) == (0xff, 0xd0, 0x78) {
                    return true;
                }
            }
        }
        false
    }

    #[test]
    fn the_fit_is_the_smaller_axis_ratio() {
        assert_eq!(fit_factor(PhysicalSize::new(1280, 800)), 1.0);
        assert_eq!(fit_factor(PhysicalSize::new(1920, 1080)), 1.35);
        assert_eq!(fit_factor(PhysicalSize::new(2560, 1600)), 2.0);
        // A window that has no size yet must not collapse the design.
        assert_eq!(fit_factor(PhysicalSize::new(0, 0)), 1.0);
    }

    #[test]
    fn user_factor_parsing_matches_the_winit_backend() {
        assert_eq!(parse_user_factor(Some("1")), Some(1.0));
        assert_eq!(parse_user_factor(Some(" 1.5 ")), Some(1.5));
        assert_eq!(parse_user_factor(Some("0")), None);
        assert_eq!(parse_user_factor(Some("-2")), None);
        assert_eq!(parse_user_factor(Some("abc")), None);
        assert_eq!(parse_user_factor(None), None);
    }

    /// A2: the reported factor, the physical size and the applied factor all
    /// reach the log.
    #[test]
    fn reported_physical_and_applied_reach_the_log() {
        let fake = FakeDisplay::new(2.25, PhysicalSize::new(1280, 800));
        let capture = Capture::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(capture.clone())
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .finish();

        let report =
            tracing::subscriber::with_default(subscriber, || configure(&fake, Mode::Fullscreen));

        assert_eq!(report.reported, 2.25);
        assert_eq!(report.physical, PhysicalSize::new(1280, 800));
        assert_eq!(report.applied, 1.0);
        assert_eq!(report.source, Source::Fitted);
        assert_eq!(fake.applied.get(), 1.0, "the fit is applied to the window");

        let text = capture.text();
        assert!(text.contains("reported=2.25"), "log: {text}");
        assert!(text.contains("physical_width=1280"), "log: {text}");
        assert!(text.contains("physical_height=800"), "log: {text}");
        assert!(text.contains("applied=1"), "log: {text}");
    }

    /// R1: the user's `SLINT_SCALE_FACTOR` wins over the fit in both modes.
    #[test]
    fn a_user_factor_wins_over_the_fit() {
        let fake = FakeDisplay::new(2.25, PhysicalSize::new(1280, 800));
        let report = configure_with_user_factor(&fake, Mode::Fullscreen, Some(1.5));
        assert_eq!(report.source, Source::User);
        assert_eq!(report.applied, 1.5);
        assert_eq!(fake.applied.get(), 1.5);

        let fake = FakeDisplay::new(2.25, PhysicalSize::new(1920, 1080));
        let report = configure_with_user_factor(&fake, Mode::Windowed, Some(1.5));
        assert_eq!(
            report.applied, 1.5,
            "the user value wins in windowed mode too"
        );
    }

    /// R1: `--windowed` keeps the reported factor, not the fit.
    #[test]
    fn windowed_keeps_the_reported_factor() {
        let fake = FakeDisplay::new(2.25, PhysicalSize::new(1280, 800));
        let report = configure_with_user_factor(&fake, Mode::Windowed, None);
        assert_eq!(report.source, Source::Reported);
        assert_eq!(report.applied, 2.25);
    }

    #[test]
    fn drift_is_any_meaningful_factor_difference() {
        assert!(factor_drifted(2.25, 1.0));
        assert!(factor_drifted(1.0, 1.5));
        assert!(!factor_drifted(1.0, 1.0));
        assert!(!factor_drifted(1.0, 1.005));
    }

    /// R1: a later backend factor drift is corrected, but only for fullscreen
    /// and never against a user `SLINT_SCALE_FACTOR`.
    #[test]
    fn refit_only_corrects_a_fullscreen_drift_without_a_user_value() {
        let fake = FakeDisplay::new(2.25, PhysicalSize::new(1280, 800));
        let report = refit_if_drifted_with(&fake, Mode::Fullscreen, 1.0, None)
            .expect("a drift must be corrected");
        assert_eq!(report.source, Source::Fitted);
        assert_eq!(report.applied, 1.0);
        assert_eq!(fake.applied.get(), 1.0);

        // No drift: nothing to do.
        let fake = FakeDisplay::new(1.0, PhysicalSize::new(1280, 800));
        assert!(refit_if_drifted_with(&fake, Mode::Fullscreen, 1.0, None).is_none());

        // Windowed mode keeps the reported factor by design.
        let fake = FakeDisplay::new(2.25, PhysicalSize::new(1280, 800));
        assert!(refit_if_drifted_with(&fake, Mode::Windowed, 1.0, None).is_none());

        // A user value is never fought.
        let fake = FakeDisplay::new(2.25, PhysicalSize::new(1280, 800));
        assert!(refit_if_drifted_with(&fake, Mode::Fullscreen, 1.0, Some(1.5)).is_none());
    }

    /// A1: a 1280×800 physical window that reports scale 2.25 lays out exactly
    /// like the same window at scale 1.0.
    #[test]
    fn fitting_ignores_the_reported_dpi_on_the_deck_panel() {
        const CASE: &str = "fitting_ignores_the_reported_dpi_on_the_deck_panel";
        if crate::headless::isolated(module_path!(), CASE) {
            return;
        }
        let (window, ui) = crate::headless::setup_at(2.25, PhysicalSize::new(1280, 800));
        let target = ui.window();
        assert_eq!(target.scale_factor(), 2.25, "the panel reports a high DPI");

        let report = configure(target, Mode::Fullscreen);
        assert_eq!(report.reported, 2.25);
        assert_eq!(report.physical, PhysicalSize::new(1280, 800));
        assert_eq!(report.applied, 1.0);
        assert_eq!(report.source, Source::Fitted);
        assert_eq!(
            target.scale_factor(),
            1.0,
            "the fit replaced the DPI factor"
        );

        // Focus the right-most of the four tabs; its ring proves tab 4 is on
        // screen at the fitted layout.
        ui.set_focused(-4);
        let fitted = crate::headless::render(&window);
        crate::headless::write_png("029-deck-fit-1280x800-reported-2.25.png", &fitted);
        assert!(amber_at(&fitted, 828.0, 18.0, 8), "tab 4 focus ring at 1.0");

        // The same window reporting 1.0 must produce the identical frame.
        target.dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: 1.0 });
        let plain_report = configure(target, Mode::Fullscreen);
        assert_eq!(plain_report.applied, 1.0);
        let plain = crate::headless::render(&window);
        crate::headless::write_png("029-deck-fit-1280x800-reported-1.0.png", &plain);
        assert_eq!(
            plain.as_slice(),
            fitted.as_slice(),
            "scale 2.25 fitted to 1.0 is byte-identical to scale 1.0"
        );
    }

    /// A1: a 1920×1080 window renders the design scaled up and uncropped.
    #[test]
    fn fitting_scales_up_uncropped_on_a_larger_screen() {
        const CASE: &str = "fitting_scales_up_uncropped_on_a_larger_screen";
        if crate::headless::isolated(module_path!(), CASE) {
            return;
        }
        let (window, ui) = crate::headless::setup_at(2.25, PhysicalSize::new(1280, 800));
        window.set_size(PhysicalSize::new(1920, 1080));
        let target = ui.window();

        let report = configure(target, Mode::Fullscreen);
        assert_eq!(report.physical, PhysicalSize::new(1920, 1080));
        assert!(
            (report.applied - 1.35).abs() < 1e-6,
            "min(1920/1280, 1080/800) = 1.35, got {}",
            report.applied
        );
        assert!(f32::abs(target.scale_factor() - 1.35) < 1e-6);

        ui.set_focused(-4);
        let frame = crate::headless::render_at(&window, 1920, 1080);
        crate::headless::write_png("029-deck-fit-1920x1080.png", &frame);

        // Tab 4 stays on screen: its focus ring sits at x = 828 logical,
        // scaled by 1.35.
        assert!(
            amber_at(&frame, 828.0 * 1.35, 18.0 * 1.35, 8),
            "tab 4 focus ring visible after scaling"
        );
        // The design is 1280×800 logical (the root pins that size), scaled to
        // 1728×1080; the spare right strip is the window background, not a crop
        // and not unpainted black. Tokens.canvas and Tokens.deep are both
        // #120C0D.
        let edge = frame.as_slice()[20 * 1920 + 1919];
        assert_eq!(
            (edge.r, edge.g, edge.b),
            (0x12, 0x0c, 0x0d),
            "the spare axis is letterboxed with the background, not cropped"
        );
    }

    /// A1: a `SLINT_SCALE_FACTOR` the user set in the environment overrides both
    /// the fit and the reported DPI.
    #[test]
    fn a_user_scale_factor_overrides_both() {
        const CASE: &str = "a_user_scale_factor_overrides_both";
        if crate::headless::isolated_env(module_path!(), CASE, &[(SCALE_ENV, "1.5")]) {
            return;
        }
        let (window, ui) = crate::headless::setup_at(2.25, PhysicalSize::new(1280, 800));
        let target = ui.window();

        let report = configure(target, Mode::Fullscreen);
        assert_eq!(report.source, Source::User);
        assert_eq!(report.applied, 1.5);
        assert_eq!(target.scale_factor(), 1.5);

        // The same override applies to a larger screen.
        window.set_size(PhysicalSize::new(1920, 1080));
        let report = configure(target, Mode::Fullscreen);
        assert_eq!(report.source, Source::User);
        assert_eq!(report.applied, 1.5);

        let frame = crate::headless::render_at(&window, 1920, 1080);
        crate::headless::write_png("029-deck-fit-1920x1080-user-1.5.png", &frame);
    }

    /// A1/R1: a later backend `ScaleFactorChanged` with no size change (winit
    /// re-reports the panel DPI when gamescope maps the fullscreen window) must
    /// not re-scale the UI; the drift is corrected and logged.
    #[test]
    fn a_backend_scale_factor_drift_is_refitted() {
        const CASE: &str = "a_backend_scale_factor_drift_is_refitted";
        if crate::headless::isolated(module_path!(), CASE) {
            return;
        }
        let (window, ui) = crate::headless::setup_at(2.25, PhysicalSize::new(1280, 800));
        let target = ui.window();
        let report = configure(target, Mode::Fullscreen);
        assert_eq!(report.applied, 1.0);

        // winit takes over again without a resize: same physical size, DPI back.
        target.dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: 2.25 });
        assert_eq!(target.scale_factor(), 2.25);
        assert_eq!(window.size(), PhysicalSize::new(1280, 800));

        let capture = Capture::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(capture.clone())
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .finish();
        let refit = tracing::subscriber::with_default(subscriber, || {
            refit_if_drifted(target, Mode::Fullscreen, report.applied)
        });

        let refit = refit.expect("a scale-factor drift must be corrected");
        assert_eq!(refit.source, Source::Fitted);
        assert_eq!(refit.applied, 1.0);
        assert_eq!(target.scale_factor(), 1.0, "the fit is applied again");

        let text = capture.text();
        assert!(text.contains("reported=2.25"), "drift logged: {text}");
        assert!(text.contains("applied=1"), "re-fit logged: {text}");
    }
}
