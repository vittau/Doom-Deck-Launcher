#!/usr/bin/env python3
"""Render the lean project page without touching the signed Flatpak repository.

Only Python's standard library is required. Publication supplies the version,
publish date, application ID and site base; asset sources stay in their existing
repository homes so their provenance and licenses remain attached.
"""
import argparse
import configparser
import hashlib
from datetime import date
from html import escape
import json
from pathlib import Path
import re
import shutil
from urllib.parse import urlsplit
from xml.sax.saxutils import escape as xml_escape

ROOT = Path(__file__).resolve().parents[2]
SITE = ROOT / "packaging/flatpak/site"
IMAGES = SITE / "images"
IMAGE_MANIFEST = IMAGES / "manifest.json"
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
    # Local donation button: official cup glyph and the Cookie typeface, so the
    # widget needs no third-party script, stylesheet or font request.
    "bmc-logo.svg": SITE / "bmc-logo.svg",
    "Cookie-Regular.woff2": SITE / "fonts/Cookie-Regular.woff2",
    "Cookie-OFL.txt": SITE / "fonts/Cookie-OFL.txt",
    **{f"{name}.png": ROOT / f"docs/screenshots/{name}.png"
       for name in ("home", "discover", "play-setup", "mapset-detail", "audio-settings")},
}


def structured_data(site_base, app_id):
    """Machine-readable product facts, all of them also visible on the page.

    No rating, review, author or organization is asserted because the project
    has none of those recorded, and no rich result is promised.
    """
    return {
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "WebSite",
                "@id": f"{site_base}/#website",
                "name": "DoomDeckLauncher",
                "alternateName": "DDL",
                "url": f"{site_base}/",
                "inLanguage": "en",
            },
            {
                "@type": "SoftwareApplication",
                "@id": f"{site_base}/#application",
                "name": "DoomDeckLauncher",
                "url": f"{site_base}/",
                "applicationCategory": "GameApplication",
                "operatingSystem": "Linux",
                "license": "https://spdx.org/licenses/GPL-3.0-or-later.html",
                "isAccessibleForFree": True,
                "offers": {"@type": "Offer", "price": 0, "priceCurrency": "USD"},
                "downloadUrl": f"{site_base}/{app_id}.flatpakref",
                "softwareRequirements": (
                    "A Steam Deck and legally owned DOOM base-game files (IWADs). "
                    "DoomDeckLauncher never bundles or downloads commercial base games."
                ),
            },
        ],
    }


def json_ld(site_base, app_id):
    """Serialize JSON-LD for a raw <script> element without allowing markup breaks."""
    text = json.dumps(structured_data(site_base, app_id), indent=2, ensure_ascii=False)
    return text.replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026")


def robots_txt(site_base):
    """Policy-neutral crawl policy: only the OSTree object tree is excluded."""
    return ("# DoomDeckLauncher project website\n"
            "User-agent: *\n"
            "Allow: /\n"
            "Disallow: /repo/\n"
            "\n"
            f"Sitemap: {site_base}/sitemap.xml\n")


def sitemap_xml(site_base):
    """The two indexable pages; no lastmod without real modification provenance."""
    urls = (f"{site_base}/", f"{site_base}/assets/credits.html")
    entries = "".join(f"  <url>\n    <loc>{xml_escape(url)}</loc>\n  </url>\n" for url in urls)
    return ('<?xml version="1.0" encoding="UTF-8"?>\n'
            '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
            f"{entries}</urlset>\n")


def image_derivatives():
    """Load the committed WebP manifest; the offline tests check it for staleness."""
    try:
        manifest = json.loads(IMAGE_MANIFEST.read_text())
    except (OSError, ValueError) as error:
        raise ValueError(f"cannot read the image manifest: {error}") from error
    if manifest.get("format") != 1:
        raise ValueError("unsupported image manifest format")
    derivatives = manifest.get("derivatives") or {}
    for name, entry in derivatives.items():
        if not (IMAGES / name).is_file():
            raise ValueError(f"missing committed image derivative {name}")
        if entry.get("source") not in ASSETS:
            raise ValueError(f"unknown image source for {name}")
    return derivatives


def llms_txt(site_base):
    """Short, accurate site summary for LLM agents; no ranking promise implied."""
    return (
        "# DoomDeckLauncher (DDL)\n"
        "\n"
        "> DoomDeckLauncher is a free, open-source, controller-first launcher for\n"
        "> community DOOM mapsets on Steam Deck, powered by DSDA-Doom. It never\n"
        "> bundles or downloads commercial base games; you bring your own legally\n"
        "> owned DOOM base-game files (IWADs).\n"
        "\n"
        "## Main pages\n"
        "\n"
        f"- [DoomDeckLauncher]({site_base}/): product overview, features, install\n"
        "  steps, requirements and questions answered.\n"
        f"- [Asset credits]({site_base}/assets/credits.html): artwork, font and\n"
        "  community screenshot provenance.\n"
        "\n"
        "## Project documentation\n"
        "\n"
        "- [Steam Deck guide](https://github.com/vittau/Doom-Deck-Launcher/blob/main/docs/steam-deck-guide.md):\n"
        "  installation, updates and controller setup.\n"
        "- [Source code](https://github.com/vittau/Doom-Deck-Launcher): GPL-3.0-or-later\n"
        "  repository and contribution guide.\n"
        "- [Flatpak research](https://github.com/vittau/Doom-Deck-Launcher/blob/main/docs/research/flatpak.md):\n"
        "  upstream evidence for the signed, self-hosted remote.\n"
        "\n"
        "## Notes\n"
        "\n"
        "- The signed Flatpak is an experimental, self-hosted channel, not a\n"
        "  Flathub listing.\n"
        "- Requirements: a Steam Deck and legally owned DOOM base-game files\n"
        "  (IWADs). DSDA-Doom is downloaded separately from its official upstream\n"
        "  release inside the launcher.\n"
        "- No Windows or macOS builds are published.\n"
        "- License: GPL-3.0-or-later.\n"
    )


def minify_css(text):
    """Strip CSS comments and redundant whitespace without altering meaning.

    Whitespace inside quotes, parentheses (``calc(100% - 40px)``, ``url(...)``,
    media-query conditions) and descendant selectors is preserved; only the
    spacing around structural tokens is removed. The readable source template
    stays in the repository and this runs at render time.
    """
    out = []
    index, length = 0, len(text)
    quote = None
    parens = 0
    pending_space = False
    while index < length:
        char = text[index]
        if quote:
            out.append(char)
            if char == "\\" and index + 1 < length:
                out.append(text[index + 1])
                index += 2
                continue
            if char == quote:
                quote = None
            index += 1
            continue
        if char in "\"'":
            if pending_space and out and out[-1] not in "{:;,>":
                out.append(" ")
            pending_space = False
            quote = char
            out.append(char)
            index += 1
            continue
        if char == "/" and index + 1 < length and text[index + 1] == "*":
            end = text.find("*/", index + 2)
            if end == -1:
                raise ValueError("unterminated CSS comment")
            index = end + 2
            pending_space = True
            continue
        if char.isspace():
            pending_space = True
            index += 1
            continue
        if char == "(":
            parens += 1
        elif char == ")":
            parens = max(0, parens - 1)
        if pending_space and out:
            if parens == 0 and (char in "{}:;,>" or out[-1] in "{}:;,>"):
                pass
            else:
                out.append(" ")
            pending_space = False
        else:
            pending_space = False
        out.append(char)
        index += 1
    return "".join(out).strip() + "\n"


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
    derivatives = image_derivatives()
    stylesheet = minify_css(ASSETS["style.css"].read_text())
    values = dict(VERSION=version, DATE=published, APP_ID=app_id, SITE_BASE=site_base,
                  STYLE_HASH=hashlib.sha256(stylesheet.encode()).hexdigest()[:16],
                  ICON_HASH=hashlib.sha256(ASSETS["icon.png"].read_bytes()).hexdigest()[:16],
                  HERO_HASH=hashlib.sha256(ASSETS["hero.png"].read_bytes()).hexdigest()[:16],
                  BMC_LOGO_HASH=hashlib.sha256(ASSETS["bmc-logo.svg"].read_bytes()).hexdigest()[:16])
    values.update({name.removesuffix(".png").replace("-", "_").upper() + "_HASH":
                   hashlib.sha256(source.read_bytes()).hexdigest()[:16]
                   for name, source in ASSETS.items() if source.parent == ROOT / "docs/screenshots"})
    values.update({name.removesuffix(".webp").replace("-", "_").upper() + "_HASH":
                   hashlib.sha256((IMAGES / name).read_bytes()).hexdigest()[:16]
                   for name in derivatives})
    raw = {"STRUCTURED_DATA": json_ld(site_base, app_id)}
    pages = {}
    for target, source in {"index.html": SITE / "index.html",
                           "assets/credits.html": SITE / "credits.html",
                           "404.html": SITE / "404.html"}.items():
        html = source.read_text()
        for key, value in values.items():
            html = html.replace(f"@@{key}@@", escape(value, quote=True))
        for key, value in raw.items():
            html = html.replace(f"@@{key}@@", value)
        if re.search(r"@@[A-Z_]+@@", html):
            raise ValueError("unresolved page template token")
        pages[target] = html
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    # Both branch publishers retain the apex domain on every refresh.
    (output / "CNAME").write_text("doomdecklauncher.com\n")
    (output / "robots.txt").write_text(robots_txt(site_base))
    (output / "sitemap.xml").write_text(sitemap_xml(site_base))
    (output / "llms.txt").write_text(llms_txt(site_base))
    assets = output / "assets"
    assets.mkdir(exist_ok=True)
    for name, source in ASSETS.items():
        if name != "style.css":
            shutil.copyfile(source, assets / name)
    for name in derivatives:
        shutil.copyfile(IMAGES / name, assets / name)
    (assets / "style.css").write_text(stylesheet)
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


def publication_ready(output, app_id, site_base):
    """Check bounded descriptor metadata before a page-only deployment.

    This is an ordering guard, not cryptographic signature verification; the
    signed publisher verifies the repository before publishing these files.
    """
    output = Path(output)
    descriptor = output / f"{app_id}.flatpakref"
    try:
        if not descriptor.is_file() or descriptor.stat().st_size > 64 * 1024:
            return False
        if not all((output / name).is_file() for name in
                   ("repo/summary", "repo/summary.sig", "index.html")):
            return False
        config = configparser.ConfigParser(interpolation=None)
        config.read_string(descriptor.read_text())
        ref = config["Flatpak Ref"]
        return (ref.get("Name") == app_id and ref.get("Branch") == "stable"
                and ref.get("Url") == site_base.rstrip("/") + "/repo/"
                and bool(ref.get("GPGKey", "").strip()))
    except (OSError, UnicodeError, configparser.Error, KeyError):
        return False


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--version")
    parser.add_argument("--refresh", action="store_true",
                        help="retain the app release from the existing published page")
    parser.add_argument("--check-publication", action="store_true",
                        help="exit 0 only when a signed publication for this identity exists")
    parser.add_argument("--date")
    parser.add_argument("--app-id", required=True)
    parser.add_argument("--site-base", required=True)
    args = parser.parse_args()
    if args.check_publication:
        if args.refresh or args.version or args.date:
            parser.error("--check-publication cannot be combined with rendering options")
        return 0 if publication_ready(args.output, args.app_id, args.site_base) else 1
    if args.refresh:
        if args.version or args.date:
            parser.error("--refresh cannot be combined with --version/--date")
        refresh(args.output, args.app_id, args.site_base)
    else:
        if not args.version or not args.date:
            parser.error("--version and --date are required unless --refresh is used")
        render(args.output, args.version, args.date, args.app_id, args.site_base)


if __name__ == "__main__":
    raise SystemExit(main())
