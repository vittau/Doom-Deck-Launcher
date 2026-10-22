#!/usr/bin/env python3
"""Regenerate the committed WebP derivatives used by the project website.

Website delivery only: the shared README/source PNGs stay untouched, and the
renderer copies these files as they are. cwebp (libwebp) is a build-time tool
used by whoever changes a source image, never a runtime or CI dependency; the
committed derivatives and ``manifest.json`` are what publication and tests read.

    python3 scripts/flatpak/generate-site-images.py           # rewrite derivatives
    python3 scripts/flatpak/generate-site-images.py --check   # verify, no encoder

The manifest pins the encoder version, quality, dimensions and both hashes, so a
replaced source PNG without regenerated derivatives fails the offline tests.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
IMAGES = ROOT / "packaging/flatpak/site/images"
MANIFEST = IMAGES / "manifest.json"
FORMAT = 1
# Source asset name (as published by the renderer) -> website delivery plan.
PLAN = {
    "hero.png": {
        "source": "docs/design/logo/steam/hero.png",
        "widths": (960, 1920, 2560),
        "quality": 80,
    },
    **{
        f"{name}.png": {
            "source": f"docs/screenshots/{name}.png",
            "widths": (640, 960, 1280),
            "quality": 88,
        }
        for name in ("discover", "home", "play-setup", "mapset-detail", "audio-settings")
    },
}


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def png_size(path):
    """Read width and height from the PNG IHDR chunk."""
    data = Path(path).read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        raise ValueError(f"{path}: expected a PNG with an IHDR chunk")
    return int.from_bytes(data[16:20], "big"), int.from_bytes(data[20:24], "big")


def webp_size(path):
    """Read width and height from a VP8, VP8L or VP8X WebP file."""
    data = Path(path).read_bytes()
    if data[:4] != b"RIFF" or data[8:12] != b"WEBP":
        raise ValueError(f"{path}: expected a WebP file")
    chunk = data[12:16]
    if chunk == b"VP8X":
        width = int.from_bytes(data[24:27], "little") + 1
        height = int.from_bytes(data[27:30], "little") + 1
        return width, height
    if chunk == b"VP8 ":
        if data[23:26] != b"\x9d\x01\x2a":
            raise ValueError(f"{path}: unexpected VP8 frame header")
        width = int.from_bytes(data[26:28], "little") & 0x3FFF
        height = int.from_bytes(data[28:30], "little") & 0x3FFF
        return width, height
    if chunk == b"VP8L":
        if data[20] != 0x2F:
            raise ValueError(f"{path}: unexpected VP8L signature")
        bits = int.from_bytes(data[21:25], "little")
        return (bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1
    raise ValueError(f"{path}: unsupported WebP chunk {chunk!r}")


def planned_derivatives():
    for name, plan in PLAN.items():
        width, height = png_size(ROOT / plan["source"])
        for target_width in plan["widths"]:
            if target_width > width:
                raise ValueError(f"{name}: {target_width}px exceeds the {width}px source")
            yield name, plan, target_width, round(height * target_width / width)


def encoder_version():
    result = subprocess.run(["cwebp", "-version"], check=True, capture_output=True, text=True)
    return " / ".join(line.strip() for line in result.stdout.splitlines() if line.strip())


def generate():
    IMAGES.mkdir(parents=True, exist_ok=True)
    sources, derivatives = {}, {}
    for name, plan, width, height in planned_derivatives():
        source = ROOT / plan["source"]
        target = IMAGES / f"{name.removesuffix('.png')}-{width}.webp"
        subprocess.run(["cwebp", "-quiet", "-q", str(plan["quality"]),
                        "-resize", str(width), "0", str(source), "-o", str(target)],
                       check=True)
        if webp_size(target) != (width, height):
            raise ValueError(f"{target.name}: encoder produced {webp_size(target)}, expected {(width, height)}")
        sources[name] = {"path": plan["source"], "sha256": sha256(source),
                         "width": png_size(source)[0], "height": png_size(source)[1]}
        derivatives[target.name] = {
            "source": name,
            "width": width,
            "height": height,
            "quality": plan["quality"],
            "saver": "WebP (cwebp)",
            "sha256": sha256(target),
            "bytes": target.stat().st_size,
        }
        print(f"{target.relative_to(ROOT)}\t{width}x{height}\t{target.stat().st_size} bytes")
    manifest = {
        "format": FORMAT,
        "generator": "scripts/flatpak/generate-site-images.py",
        "encoder": encoder_version(),
        "source_pngs_are_shared_and_unmodified": True,
        "sources": sources,
        "derivatives": derivatives,
    }
    MANIFEST.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    keep = set(derivatives)
    for stale in IMAGES.glob("*.webp"):
        if stale.name not in keep:
            stale.unlink()
            print(f"removed stale {stale.name}")


def check():
    """Verify the committed manifest without running the encoder."""
    manifest = json.loads(MANIFEST.read_text())
    if manifest.get("format") != FORMAT:
        raise ValueError("unsupported image manifest format; regenerate the derivatives")
    problems = []
    expected = {f"{name.removesuffix('.png')}-{width}.webp"
                for name, _, width, _ in planned_derivatives()}
    for name, source in manifest["sources"].items():
        path = ROOT / source["path"]
        if not path.is_file():
            problems.append(f"{name}: missing source {source['path']}")
            continue
        if sha256(path) != source["sha256"] or png_size(path) != (source["width"], source["height"]):
            problems.append(f"{name}: source {source['path']} changed; rerun the generator")
    for name, entry in manifest["derivatives"].items():
        path = IMAGES / name
        if not path.is_file():
            problems.append(f"{name}: missing derivative; rerun the generator")
            continue
        if sha256(path) != entry["sha256"]:
            problems.append(f"{name}: bytes do not match the manifest")
        if webp_size(path) != (entry["width"], entry["height"]):
            problems.append(f"{name}: dimensions {webp_size(path)} do not match the manifest")
        if entry["source"] not in manifest["sources"]:
            problems.append(f"{name}: unknown source {entry['source']}")
    for name in sorted(expected - set(manifest["derivatives"])):
        problems.append(f"{name}: planned derivative is missing from the manifest")
    for name in sorted(set(manifest["derivatives"]) - expected):
        problems.append(f"{name}: derivative is no longer planned")
    if problems:
        print("\n".join(problems), file=sys.stderr)
        return 1
    total = sum(entry["bytes"] for entry in manifest["derivatives"].values())
    print(f"PASS: {len(manifest['derivatives'])} WebP derivatives, {total} bytes, "
          f"encoder {manifest['encoder']}")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                        help="verify the committed files without invoking cwebp")
    args = parser.parse_args()
    return check() if args.check else (generate() or 0)


if __name__ == "__main__":
    raise SystemExit(main())
