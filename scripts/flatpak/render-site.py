#!/usr/bin/env python3
"""Render the lean project page without touching the signed Flatpak repository.

Only Python's standard library is required. Publication supplies the version,
publish date, application ID and site base; asset sources stay in their existing
repository homes so their provenance and licenses remain attached.
"""
import argparse
import hashlib
from datetime import date
from html import escape
from pathlib import Path
import re
import shutil
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[2]
SITE = ROOT / "packaging/flatpak/site"
ASSETS = {
    "style.css": SITE / "style.css",
    "emblem-small.svg": ROOT / "crates/ddl/ui/assets/logo/emblem-small.svg",
    "horizontal.svg": ROOT / "crates/ddl/ui/assets/logo/horizontal.svg",
    "icon.png": ROOT / "crates/ddl/ui/assets/logo/icon.png",
    "hero.png": ROOT / "docs/design/logo/steam/hero.png",
    "social-preview.png": ROOT / "docs/design/logo/social-preview.png",
    "Barlow-Regular.ttf": ROOT / "crates/ddl/ui/assets/Barlow-Regular.ttf",
    "PirataOne-Regular.ttf": ROOT / "crates/ddl/ui/assets/PirataOne-Regular.ttf",
    "Barlow-OFL.txt": ROOT / "crates/ddl/ui/assets/Barlow-OFL.txt",
    "PirataOne-OFL.txt": ROOT / "crates/ddl/ui/assets/PirataOne-OFL.txt",
    **{f"{name}.png": ROOT / f"docs/screenshots/{name}.png"
       for name in ("home", "discover", "play-setup", "mapset-detail", "audio-settings")},
}


def render(output, version, published, app_id, site_base):
    """Copy only page-owned assets; leave ref, key, repo and Git state intact."""
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?", version):
        raise ValueError("version must be the published semantic version")
    date.fromisoformat(published)
    if not re.fullmatch(r"[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+){2,}", app_id):
        raise ValueError("invalid Flatpak application ID")
    site_base = site_base.rstrip("/")
    url = urlsplit(site_base)
    if url.scheme != "https" or not url.netloc or url.query or url.fragment or url.username:
        raise ValueError("site base must be an absolute HTTPS URL without query or fragment")
    values = dict(VERSION=version, DATE=published, APP_ID=app_id, SITE_BASE=site_base,
                  STYLE_HASH=hashlib.sha256(ASSETS["style.css"].read_bytes()).hexdigest()[:16],
                  ICON_HASH=hashlib.sha256(ASSETS["icon.png"].read_bytes()).hexdigest()[:16])
    values.update({name.removesuffix(".png").replace("-", "_").upper() + "_HASH":
                   hashlib.sha256(source.read_bytes()).hexdigest()[:16]
                   for name, source in ASSETS.items() if source.parent == ROOT / "docs/screenshots"})
    pages = {}
    for target, source in {"index.html": SITE / "index.html",
                           "assets/credits.html": SITE / "credits.html"}.items():
        html = source.read_text()
        for key, value in values.items():
            html = html.replace(f"@@{key}@@", escape(value, quote=True))
        if re.search(r"@@[A-Z_]+@@", html):
            raise ValueError("unresolved page template token")
        pages[target] = html
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    # Both branch publishers retain the apex domain on every refresh.
    (output / "CNAME").write_text("doomdecklauncher.com\n")
    assets = output / "assets"
    assets.mkdir(exist_ok=True)
    for name, source in ASSETS.items():
        shutil.copyfile(source, assets / name)
    for target, html in pages.items():
        (output / target).write_text(html)


def refresh(output, app_id, site_base):
    """Refresh page assets while retaining the already published app release."""
    page = Path(output) / "index.html"
    if page.stat().st_size > 1024 * 1024:
        raise ValueError("published page exceeds metadata size limit")
    releases = re.findall(
        r'<p class="release">.*?v([0-9][A-Za-z0-9.+-]*)\s*·\s*Published\s*'
        r'<time datetime="([^"]+)">', page.read_text(), re.DOTALL)
    if len(releases) != 1:
        raise ValueError("expected exactly one published release; publish Flatpak first")
    version, published = releases[0]
    render(output, version, published, app_id, site_base)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--version")
    parser.add_argument("--refresh", action="store_true",
                        help="retain the app release from the existing published page")
    parser.add_argument("--date")
    parser.add_argument("--app-id", required=True)
    parser.add_argument("--site-base", required=True)
    args = parser.parse_args()
    if args.refresh:
        if args.version or args.date:
            parser.error("--refresh cannot be combined with --version/--date")
        refresh(args.output, args.app_id, args.site_base)
    else:
        if not args.version or not args.date:
            parser.error("--version and --date are required unless --refresh is used")
        render(args.output, args.version, args.date, args.app_id, args.site_base)


if __name__ == "__main__":
    main()
