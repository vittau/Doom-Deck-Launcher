//! The release uses committed exports; verify their actual PNG headers and alpha.
use std::path::Path;

#[test]
fn steam_artwork_and_social_preview_have_contract_dimensions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design/logo");
    for (file, width, height, transparent) in [
        ("steam/icon-256.png", 256, 256, true),
        ("steam/icon-512.png", 512, 512, true),
        ("steam/library-capsule.png", 600, 900, false),
        ("steam/wide-capsule.png", 920, 430, false),
        ("steam/hero.png", 3840, 1240, false),
        ("steam/logo.png", 1280, 720, true),
        ("social-preview.png", 1280, 640, false),
        ("contact-sheet.png", 900, 720, false),
    ] {
        let mut reader = png::Decoder::new(std::io::BufReader::new(
            std::fs::File::open(root.join(file)).unwrap(),
        ))
        .read_info()
        .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        assert_eq!((info.width, info.height), (width, height), "{file}");
        assert_eq!(info.color_type, png::ColorType::Rgba);
        let alpha: Vec<_> = pixels[..info.buffer_size()]
            .chunks_exact(4)
            .map(|p| p[3])
            .collect();
        assert!(alpha.contains(&255), "{file} has visible content");
        if transparent {
            assert_eq!(alpha[0], 0, "{file} has a transparent corner");
            assert!(alpha.contains(&0), "{file} preserves transparency");
        } else {
            assert!(alpha.iter().all(|a| *a == 255), "{file} is opaque");
        }
    }
}

#[test]
fn compiled_svg_masters_match_documented_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in ["emblem", "horizontal", "stacked", "full-name"] {
        let source =
            std::fs::read(root.join(format!("../../docs/design/logo/{name}.svg"))).unwrap();
        assert_eq!(
            source,
            std::fs::read(root.join(format!("ui/assets/logo/{name}.svg"))).unwrap()
        );
    }
    assert_eq!(
        std::fs::read(root.join("ui/assets/logo/icon.png")).unwrap(),
        std::fs::read(root.join("../../docs/design/logo/steam/icon-256.png")).unwrap()
    );
}
