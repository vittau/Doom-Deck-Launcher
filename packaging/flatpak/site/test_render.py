"""Offline acceptance checks: python3 -m unittest discover -s packaging/flatpak/site."""
import importlib.util
import hashlib
from html.parser import HTMLParser
from pathlib import Path
import re
import tempfile
import unittest
from unittest.mock import patch
from urllib.parse import unquote, urljoin, urlsplit

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("render_site", ROOT / "scripts/flatpak/render-site.py")
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)
APP_ID = "io.github.vittau.Doom-Deck-Launcher"
BASE = "https://www.vitormach.dev/Doom-Deck-Launcher"


class Document(HTMLParser):
    def __init__(self, text):
        super().__init__(convert_charrefs=True)
        self.elements = []
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        self.elements.append((tag, dict(attrs)))


class RenderTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.out = Path(self.temp.name)

    def render(self, **overrides):
        args = dict(output=self.out, version="1.2.3-rc.4", published="2026-10-21",
                    app_id=APP_ID, site_base=BASE)
        args.update(overrides)
        renderer.render(**args)
        return (self.out / "index.html").read_text()

    def test_publish_values_and_download_contract(self):
        text = self.render()
        self.assertNotRegex(text, r"@@[A-Z_]+@@")
        self.assertIn("v1.2.3-rc.4", text)
        doc = Document(text)
        downloads = [attrs for tag, attrs in doc.elements if tag == "a" and "download" in attrs]
        self.assertEqual(len(downloads), 2)
        self.assertTrue(all(a["href"] == f"{BASE}/{APP_ID}.flatpakref" for a in downloads))
        self.assertIn(("time", {"datetime": "2026-10-21"}), doc.elements)
        self.assertIn(("link", {"rel": "canonical", "href": f"{BASE}/"}), doc.elements)
        self.assertTrue(any(tag == "meta" and attrs.get("property") == "og:image"
                            and attrs["content"] == f"{BASE}/assets/social-preview.png"
                            for tag, attrs in doc.elements))

    def test_assets_local_links_fragments_and_css_resolve(self):
        self.render()
        # These are signed/publication artifacts supplied by the existing workflow,
        # never created or rewritten by the page renderer.
        (self.out / "ddl.flatpakrepo").touch()
        for page in (self.out / "index.html", self.out / "assets/credits.html"):
            text = page.read_text()
            doc = Document(text)
            ids = [a["id"] for _, a in doc.elements if "id" in a]
            self.assertEqual(len(ids), len(set(ids)), "duplicate HTML IDs")
            for tag, attrs in doc.elements:
                if tag == "img":
                    self.assertIn("alt", attrs)
                    self.assertIn("width", attrs)
                    self.assertIn("height", attrs)
                for key in ("src", "href"):
                    url = urlsplit(attrs.get(key, ""))
                    if url.scheme or url.netloc:
                        continue
                    if url.path:
                        self.assertTrue((page.parent / unquote(url.path)).is_file(), url.path)
                    elif url.fragment:
                        self.assertIn(url.fragment, ids)
        css = (self.out / "assets/style.css").read_text()
        for url in re.findall(r"url\(['\"]?([^)'\"]+)", css):
            self.assertTrue((self.out / "assets" / url).is_file(), url)
        self.assertIn(":focus-visible", css)
        self.assertIn("prefers-reduced-motion", css)

    def test_render_preserves_signed_artifacts_and_repository(self):
        artifacts = {f"{APP_ID}.flatpakref": b"signed ref bytes\x00\xff",
                     "ddl.flatpakrepo": b"repository descriptor",
                     "repo/summary": b"signed summary", "repo/summary.sig": b"signature",
                     ".git/config": b"git config", ".nojekyll": b""}
        for name, data in artifacts.items():
            path = self.out / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        self.render()
        self.render(version="2.0.0", published="2026-10-22")
        for name, data in artifacts.items():
            self.assertEqual((self.out / name).read_bytes(), data, name)

    def test_reject_invalid_publish_inputs_before_writing(self):
        for args in ({"version": "<script>"}, {"published": "2026-02-30"},
                     {"app_id": "../../wrong"}, {"site_base": "http://example.com"},
                     {"site_base": f"{BASE}?wrong=true"}):
            with self.subTest(args=args), self.assertRaises(ValueError):
                self.render(**args)
        self.assertEqual(list(self.out.iterdir()), [])

    def test_refresh_preserves_published_release_and_signed_files(self):
        text = self.render(version="0.0.9-focus", published="2026-10-20")
        artifact = self.out / "repo/summary.sig"
        artifact.parent.mkdir()
        artifact.write_bytes(b"signed summary")
        # Simulate an older page layout without changing its release metadata.
        (self.out / "index.html").write_text(text.replace("Your DOOM.", "Old heading"))
        renderer.refresh(self.out, APP_ID, BASE)
        result = (self.out / "index.html").read_text()
        self.assertIn("Your DOOM.", result)
        self.assertIn("v0.0.9-focus", result)
        self.assertIn('datetime="2026-10-20"', result)
        self.assertEqual(artifact.read_bytes(), b"signed summary")

    def test_refresh_rejects_missing_ambiguous_or_invalid_release(self):
        original = self.render()
        for text in ("no release", original + original,
                     original.replace('datetime="2026-10-21"', 'datetime="invalid"')):
            with self.subTest(text=text[:30]):
                (self.out / "index.html").write_text(text)
                with self.assertRaises(ValueError):
                    renderer.refresh(self.out, APP_ID, BASE)
                self.assertEqual((self.out / "index.html").read_text(), text)

    def test_official_donation_embed_matches_owner_configuration(self):
        doc = Document(self.render())
        embeds = [a for tag, a in doc.elements
                  if tag == "script" and a.get("data-name") == "bmc-button"]
        self.assertEqual(len(embeds), 1)
        embed = embeds[0]
        self.assertEqual(embed["src"], "https://cdnjs.buymeacoffee.com/1.0.0/button.prod.min.js")
        for key, value in {"slug": "vittau", "font": "Cookie", "text": "Donate",
                           "color": "#FFDD00", "font-color": "#000000",
                           "outline-color": "#000000", "coffee-color": "#ffffff"}.items():
            self.assertEqual(embed[f"data-{key}"], value)
        self.assertTrue(any(tag == "a" and a.get("href") == "https://buymeacoffee.com/vittau"
                            for tag, a in doc.elements))

    def test_stylesheet_url_tracks_css_content(self):
        text = self.render()
        digest = hashlib.sha256((self.out / "assets/style.css").read_bytes()).hexdigest()[:16]
        doc = Document(text)
        styles = [a["href"] for tag, a in doc.elements
                  if tag == "link" and a.get("rel") == "stylesheet"]
        self.assertEqual(styles, [f"assets/style.css?v={digest}"])

    def test_favicon_provenance_subpath_and_cache_version_on_both_pages(self):
        self.render()
        source = ROOT / "crates/ddl/ui/assets/logo/icon.png"
        self.assertEqual((self.out / "assets/icon.png").read_bytes(), source.read_bytes())
        digest = hashlib.sha256(source.read_bytes()).hexdigest()[:16]
        for page in ("index.html", "assets/credits.html"):
            with self.subTest(page=page):
                text = (self.out / page).read_text()
                self.assertNotRegex(text, r"@@[A-Z_]+@@")
                icons = [a for tag, a in Document(text).elements
                         if tag == "link" and "icon" in a.get("rel", "").split()]
                self.assertEqual(len(icons), 1)
                self.assertEqual(icons[0]["type"], "image/png")
                self.assertEqual(icons[0]["sizes"], "256x256")
                self.assertEqual(urljoin(f"{BASE}/{page}", icons[0]["href"]),
                                 f"{BASE}/assets/icon.png?v={digest}")
        # A replacement approved export must change the URL on both pages,
        # including through a page-only refresh that retains release metadata.
        replacement = ROOT / "docs/design/logo/steam/icon-512.png"
        with patch.dict(renderer.ASSETS, {"icon.png": replacement}):
            renderer.refresh(self.out, APP_ID, BASE)
        changed = hashlib.sha256(replacement.read_bytes()).hexdigest()[:16]
        self.assertNotEqual(changed, digest)
        for page in ("index.html", "assets/credits.html"):
            self.assertIn(f"icon.png?v={changed}", (self.out / page).read_text())
        self.assertIn("v1.2.3-rc.4", (self.out / "index.html").read_text())

    def test_current_screenshots_shared_with_readme_and_cache_versioned(self):
        doc = Document(self.render())
        readme = (ROOT / "README.md").read_text()
        for name in ("home", "discover", "play-setup", "mapset-detail", "audio-settings"):
            with self.subTest(name=name):
                source = ROOT / f"docs/screenshots/{name}.png"
                data = source.read_bytes()
                self.assertEqual((self.out / f"assets/{name}.png").read_bytes(), data)
                self.assertIn(f"docs/screenshots/{name}.png", readme)
                self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
                self.assertEqual(data[16:24], (1280).to_bytes(4, "big") + (800).to_bytes(4, "big"))
                digest = hashlib.sha256(data).hexdigest()[:16]
                expected = f"{BASE}/assets/{name}.png?v={digest}"
                for tag, key in (("img", "src"), ("a", "href")):
                    links = [urljoin(f"{BASE}/", a[key]) for t, a in doc.elements
                             if t == tag and urlsplit(a.get(key, "")).path == f"assets/{name}.png"]
                    self.assertEqual(links, [expected])
        with patch.dict(renderer.ASSETS, {"home.png": ROOT / "docs/screenshots/discover.png"}):
            updated = self.render()
        changed = hashlib.sha256((ROOT / "docs/screenshots/discover.png").read_bytes()).hexdigest()[:16]
        self.assertIn(f"assets/home.png?v={changed}", updated)

    def test_header_reuses_app_ddl_lockup_with_product_subtitle(self):
        text = self.render()
        brand = re.search(r'<a class="brand".*?</a', text, re.DOTALL).group()
        self.assertIn('aria-label="DoomDeckLauncher home"', brand)
        self.assertIn('src="assets/horizontal.svg"', brand)
        self.assertIn('class="brand-subtitle">DoomDeckLauncher</span>', brand)
        self.assertEqual((self.out / "assets/horizontal.svg").read_bytes(),
                         (ROOT / "crates/ddl/ui/assets/logo/horizontal.svg").read_bytes())

    def test_url_attribute_is_escaped(self):
        text = self.render(site_base='https://example.com/a"quoted')
        self.assertIn('https://example.com/a&quot;quoted/', text)
        self.assertNotIn('href="https://example.com/a"quoted/', text)


if __name__ == "__main__":
    unittest.main()
