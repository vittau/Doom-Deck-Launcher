//! Spec 048: production vector frame seams at Deck scale and 1920×1080 fit.
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, PlatformError, WindowAdapter, WindowEvent};
use slint::{ComponentHandle, PhysicalSize, Rgb8Pixel, SharedPixelBuffer};
use std::rc::Rc;

slint::slint! {
    import { VectorFrame } from "../ui/assets/ember/frame.slint";
    export component Preview inherits Window {
        in property <bool> home: false;
        in property <image> reference;
        width: root.home ? 1280px : 1000px; height: root.home ? 800px : 600px; background: #120c0d;
        if root.home: Image { x: 0px; y: 0px; width: parent.width; height: parent.height; source: root.reference; image-fit: fill; }
        if root.home: VectorFrame { x: 32px; y: 222px; width: 800px; height: 494px; }
        if root.home: VectorFrame { x: 848px; y: 222px; width: 400px; height: 494px; }
        if !root.home: VectorFrame { x: 32px; y: 32px; width: 784px; height: 240px; }
        if !root.home: VectorFrame { x: 32px; y: 304px; width: 384px; height: 264px; }
    }
}
struct Offscreen(Rc<MinimalSoftwareWindow>);
impl Platform for Offscreen {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.0.clone())
    }
}
#[test]
fn vector_frame_design_preview() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Offscreen(window.clone()))).unwrap();
    let ui = Preview::new().unwrap();
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vector-frames");
    std::fs::create_dir_all(&out).unwrap();
    for (scale, name) in [(1.0, "design-1x.png"), (1.35, "design-fit.png")] {
        window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale,
        });
        let (w, h) = ((1000.0 * scale) as u32, (600.0 * scale) as u32);
        window.set_size(PhysicalSize::new(w, h));
        ui.show().unwrap();
        window.request_redraw();
        let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(w, h);
        assert!(window.draw_if_needed(|renderer| {
            renderer.render(buffer.make_mut_slice(), w as usize);
        }));
        for panel in [(32., 32., 784., 240.), (32., 304., 384., 264.)] {
            assert_seams(&buffer, scale, panel);
        }
        if scale == 1.0 {
            let mut crop = SharedPixelBuffer::<Rgb8Pixel>::new(256, 256);
            for y in 0..256usize {
                for x in 0..256usize {
                    crop.make_mut_slice()[y * 256 + x] =
                        buffer.as_slice()[(32 + y / 4) * w as usize + 32 + x / 4];
                }
            }
            save(&out.join("corner-4x.png"), &crop);
        }
        save(&out.join(name), &buffer);
    }
    // Design approval mockup: the existing headless Home image, with only its
    // frame perimeters replaced by the same software-rendered candidate.
    ui.set_home(true);
    ui.set_reference(
        slint::Image::load_from_path(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/screenshots/home.png"),
        )
        .unwrap(),
    );
    window.dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: 1.0 });
    window.set_size(PhysicalSize::new(1280, 800));
    window.request_redraw();
    let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(1280, 800);
    assert!(window.draw_if_needed(|renderer| {
        renderer.render(buffer.make_mut_slice(), 1280);
    }));
    save(&out.join("home-design.png"), &buffer);
}

fn save(path: &std::path::Path, buffer: &SharedPixelBuffer<Rgb8Pixel>) {
    let (w, h) = (buffer.width(), buffer.height());
    let mut encoder = png::Encoder::new(std::fs::File::create(path).unwrap(), w, h);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(buffer.as_bytes())
        .unwrap();
}

/// Compare every pixel in the two adjoining rows/columns, not a golden image.
/// Constant longitudinal profiles must meet exactly, including at fractional fit.
fn assert_seams(
    buffer: &SharedPixelBuffer<Rgb8Pixel>,
    scale: f32,
    (x, y, w, h): (f32, f32, f32, f32),
) {
    let p = |v: f32| (v * scale).round() as usize;
    let pixel = |x, y| buffer.as_slice()[y * buffer.width() as usize + x];
    // Corner/rail joints, plus the edges of every ornament ground.
    for seam in [x + 28., x + w - 28., x + (w - 96.) / 2., x + (w + 96.) / 2.] {
        for start in [y, y + h - 28.] {
            for row in p(start)..p(start + 28.) {
                assert_eq!(
                    pixel(p(seam) - 1, row),
                    pixel(p(seam), row),
                    "horizontal rail seam at {seam}, row {row}, scale {scale}"
                );
            }
        }
    }
    for seam in [y + 28., y + h - 28., y + (h - 48.) / 2., y + (h + 48.) / 2.] {
        for start in [x, x + w - 28.] {
            for col in p(start)..p(start + 28.) {
                assert_eq!(
                    pixel(col, p(seam) - 1),
                    pixel(col, p(seam)),
                    "vertical rail seam at {seam}, column {col}, scale {scale}"
                );
            }
        }
    }
}
