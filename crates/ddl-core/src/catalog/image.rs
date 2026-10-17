//! Bounded, header-only image inspection for catalog screenshots (spec 023, R1/R2).
//!
//! Catalog lint must prove that a referenced screenshot is a real PNG, JPEG or
//! WebP within the size and dimension bounds **without** decoding pixels, so a
//! malformed or hostile file can never allocate or expand in CI. Only the magic
//! bytes and the dimension fields of each container are read, with every offset
//! checked; no third-party image decoder is pulled into `ddl-core` (D-005).
//!
//! Formats accepted here mirror what the Slint UI can display: PNG, JPEG and
//! WebP. The committed assets are small JPEGs (spec 023, R1).

use std::fmt;

/// The largest screenshot file the catalog accepts, in bytes. Spec 023 R1 asks
/// for "small (aim ≤ 250 KB)"; the hard lint bound is 256 KiB so a rounded
/// commit still passes.
pub const MAX_SCREENSHOT_BYTES: u64 = 256 * 1024;

/// The smallest card-sized screenshot the catalog accepts (spec 023, R1/R2).
pub const MIN_SCREENSHOT_WIDTH: u32 = 640;
pub const MIN_SCREENSHOT_HEIGHT: u32 = 360;

/// Hard upper bound on `width * height`, so a decompression-bomb header is
/// rejected even though no pixels are decoded.
pub const MAX_SCREENSHOT_PIXELS: u64 = 4096 * 4096;

/// A recognisable image container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

impl ImageFormat {
    /// The lowercase extension this format is committed with.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
        }
    }
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Webp => "WebP",
        })
    }
}

/// Why a file is not an accepted screenshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageError {
    /// Not a PNG, JPEG or WebP by its magic bytes.
    UnknownFormat,
    /// The container was identified but its dimensions could not be read.
    Malformed,
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFormat => f.write_str("not a PNG, JPEG or WebP image"),
            Self::Malformed => f.write_str("malformed image header; dimensions unreadable"),
        }
    }
}

impl std::error::Error for ImageError {}

/// Reads the format and pixel dimensions of `bytes` from its header only.
///
/// # Errors
///
/// [`ImageError::UnknownFormat`] when the magic bytes match no accepted
/// container, [`ImageError::Malformed`] when the header is truncated or its
/// dimensions are zero.
pub fn image_info(bytes: &[u8]) -> Result<(ImageFormat, u32, u32), ImageError> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return png_dims(bytes).map(|(w, h)| (ImageFormat::Png, w, h));
    }
    if bytes.starts_with(b"\xff\xd8") {
        return jpeg_dims(bytes).map(|(w, h)| (ImageFormat::Jpeg, w, h));
    }
    if bytes.len() >= 12 && bytes[..4] == *b"RIFF" && bytes[8..12] == *b"WEBP" {
        return webp_dims(bytes).map(|(w, h)| (ImageFormat::Webp, w, h));
    }
    Err(ImageError::UnknownFormat)
}

/// Whether `text` is a screenshot file name DDL accepts as a bundled asset.
#[must_use]
pub fn accepted_asset_extension(text: &str) -> bool {
    matches!(
        text.rsplit('.')
            .next()
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp")
    )
}

fn be_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let slot = bytes.get(at..at + 4)?;
    Some(u32::from_be_bytes(slot.try_into().ok()?))
}

/// PNG: signature (8) + IHDR length (4) + `IHDR` (4) + width (4) + height (4).
fn png_dims(bytes: &[u8]) -> Result<(u32, u32), ImageError> {
    if bytes.get(12..16) != Some(b"IHDR".as_slice()) || be_u32(bytes, 8) != Some(13) {
        return Err(ImageError::Malformed);
    }
    let width = be_u32(bytes, 16).ok_or(ImageError::Malformed)?;
    let height = be_u32(bytes, 20).ok_or(ImageError::Malformed)?;
    nonzero(width, height)
}

/// JPEG: walk the marker segments to the first Start-Of-Frame, which carries
/// the height and width. Stop at Start-Of-Scan/End-Of-Image.
fn jpeg_dims(bytes: &[u8]) -> Result<(u32, u32), ImageError> {
    /// Markers that introduce a frame header (exclude DHT `C4`, JPG `C8`, DAC `CC`).
    fn is_sof(marker: u8) -> bool {
        matches!(
            marker,
            0xC0 | 0xC1
                | 0xC2
                | 0xC3
                | 0xC5
                | 0xC6
                | 0xC7
                | 0xC9
                | 0xCA
                | 0xCB
                | 0xCD
                | 0xCE
                | 0xCF
        )
    }
    let mut at = 2;
    while at + 4 <= bytes.len() {
        if bytes[at] != 0xFF {
            return Err(ImageError::Malformed);
        }
        // Skip fill bytes: a marker code is the last 0xFF before a non-FF byte.
        let mut marker = bytes[at + 1];
        while marker == 0xFF && at + 2 < bytes.len() {
            at += 1;
            marker = bytes[at + 1];
        }
        if marker == 0xD8 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            at += 2; // standalone marker, no payload
            continue;
        }
        if marker == 0xD9 || marker == 0xDA {
            break; // end of image / start of scan: no frame header found
        }
        let length = usize::from(u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]));
        if length < 2 || at + 2 + length > bytes.len() {
            return Err(ImageError::Malformed);
        }
        if is_sof(marker) {
            let height = u32::from(u16::from_be_bytes([bytes[at + 5], bytes[at + 6]]));
            let width = u32::from(u16::from_be_bytes([bytes[at + 7], bytes[at + 8]]));
            return nonzero(width, height);
        }
        at += 2 + length;
    }
    Err(ImageError::Malformed)
}

/// WebP: RIFF container; the first chunk after `WEBP` is `VP8 ` (lossy),
/// `VP8L` (lossless) or `VP8X` (extended).
fn webp_dims(bytes: &[u8]) -> Result<(u32, u32), ImageError> {
    let chunk = bytes.get(12..16).ok_or(ImageError::Malformed)?;
    match chunk {
        b"VP8 " => {
            // 3-byte frame tag, then the 3-byte start code, then 14-bit each.
            if bytes.get(23..26) != Some(b"\x9d\x01\x2a".as_slice()) {
                return Err(ImageError::Malformed);
            }
            let raw = bytes.get(26..30).ok_or(ImageError::Malformed)?;
            let width = u32::from(u16::from_le_bytes([raw[0], raw[1]])) & 0x3fff;
            let height = u32::from(u16::from_le_bytes([raw[2], raw[3]])) & 0x3fff;
            nonzero(width, height)
        }
        b"VP8L" => {
            if bytes.get(20) != Some(&0x2f) {
                return Err(ImageError::Malformed);
            }
            let raw = bytes.get(21..25).ok_or(ImageError::Malformed)?;
            let bits = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
            let width = (bits & 0x3fff) + 1;
            let height = ((bits >> 14) & 0x3fff) + 1;
            nonzero(width, height)
        }
        b"VP8X" => {
            let raw = bytes.get(24..30).ok_or(ImageError::Malformed)?;
            let width = u32::from_le_bytes([raw[0], raw[1], raw[2], 0]) + 1;
            let height = u32::from_le_bytes([raw[3], raw[4], raw[5], 0]) + 1;
            nonzero(width, height)
        }
        _ => Err(ImageError::Malformed),
    }
}

fn nonzero(width: u32, height: u32) -> Result<(u32, u32), ImageError> {
    if width == 0 || height == 0 {
        return Err(ImageError::Malformed);
    }
    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal, header-only PNG: signature + IHDR at the documented offsets.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes.extend_from_slice(&[0; 4]);
        bytes
    }

    /// A minimal JPEG: SOI, a skipped APP0 segment, then an SOF0 frame header.
    fn jpeg(width: u16, height: u16) -> Vec<u8> {
        let mut bytes = vec![0xFF, 0xD8];
        bytes.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x04, 0x00, 0x00]); // APP0, length 4
        bytes.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08]); // SOF0, length 17, precision 8
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&[3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1, 0xFF, 0xD9]);
        bytes
    }

    /// A minimal lossy WebP: RIFF/WEBP + `VP8 ` chunk with the fixed start code.
    fn webp(width: u16, height: u16) -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(b"WEBP");
        bytes.extend_from_slice(b"VP8 ");
        bytes.extend_from_slice(&10u32.to_le_bytes()); // chunk size
        bytes.extend_from_slice(&[0, 0, 0]); // frame tag
        bytes.extend_from_slice(b"\x9d\x01\x2a");
        bytes.extend_from_slice(&(width & 0x3fff).to_le_bytes());
        bytes.extend_from_slice(&(height & 0x3fff).to_le_bytes());
        bytes
    }

    #[test]
    fn reads_png_jpeg_and_webp_dimensions() {
        assert_eq!(image_info(&png(640, 360)), Ok((ImageFormat::Png, 640, 360)));
        assert_eq!(
            image_info(&jpeg(1280, 720)),
            Ok((ImageFormat::Jpeg, 1280, 720))
        );
        assert_eq!(
            image_info(&webp(1920, 1080)),
            Ok((ImageFormat::Webp, 1920, 1080))
        );
    }

    #[test]
    fn rejects_unknown_and_truncated_headers() {
        assert_eq!(image_info(b"not an image"), Err(ImageError::UnknownFormat));
        assert_eq!(image_info(b"\x89PNG\r\n\x1a\n"), Err(ImageError::Malformed));
        assert_eq!(image_info(b"\xff\xd8\xff\xc0"), Err(ImageError::Malformed));
        assert_eq!(image_info(&png(0, 360)), Err(ImageError::Malformed));
        let mut gif = b"GIF89a".to_vec();
        gif.extend_from_slice(&[0; 16]);
        assert_eq!(image_info(&gif), Err(ImageError::UnknownFormat));
    }

    #[test]
    fn recognises_accepted_extensions_case_insensitively() {
        for name in ["01.jpg", "shot.JPEG", "card.png", "x.webp"] {
            assert!(accepted_asset_extension(name), "{name}");
        }
        for name in ["01.gif", "card.bmp", "noext", "01.jpg.exe"] {
            assert!(!accepted_asset_extension(name), "{name}");
        }
    }
}
