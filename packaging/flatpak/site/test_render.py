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
APP_ID = "com.doomdecklauncher.DoomDeckLauncher"
BASE = "https://doomdecklauncher.com"


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

    def test_images_carry_alt_text_without_duplicate_accessible_names(self):
        """Bing's site scan counts alt="" as a missing attribute, so every image is
        described; the ones that only repeat adjacent text stay out of the
        accessibility tree instead of being announced twice."""
        text = self.render()
        imgs = re.findall(r"<img\b[^>]*>", text)
        self.assertEqual(len(imgs), 10)
        for img in imgs:
            alt = re.search(r'\balt="([^"]*)"', img)
            self.assertIsNotNone(alt, f"missing alt: {img}")
            self.assertTrue(alt.group(1).strip(), f"empty alt: {img}")
        header = re.search(r"<header\b.*?</header>", text, re.DOTALL).group()
        self.assertIn('aria-label="DoomDeckLauncher home"', header)
        wordmark = re.search(r'<span class="brand-wordmark" aria-hidden="true">\s*<img\b[^>]*'
                             r'alt="DoomDeckLauncher wordmark"', header, re.DOTALL)
        self.assertIsNotNone(wordmark, "wordmark alt hidden from assistive technology")
        hero = re.search(r'<div class="hero-art" aria-hidden="true">.*?'
                         r'alt="DoomDeckLauncher’s Rose Compass emblem in the window', text, re.DOTALL)
        self.assertIsNotNone(hero, "hero artwork alt inside the hidden hero-art layer")
        footer = re.search(r"<footer\b.*", text, re.DOTALL).group()
        emblem = re.search(r'<img\b[^>]*emblem-small\.svg[^>]*>', footer).group()
        self.assertIn('alt="DoomDeckLauncher emblem"', emblem)
        self.assertIn('aria-hidden="true"', emblem)
        self.assertIn('alt="Buy Me a Coffee"', re.search(r'<img\b[^>]*bmc-logo[^>]*>', text).group())

    def test_assets_local_links_fragments_and_css_resolve(self):
        self.render()
        # These are signed/publication artifacts supplied by the existing workflow,
        # never created or rewritten by the page renderer.
        (self.out / "ddl.flatpakrepo").touch()
        (self.out / f"{APP_ID}.flatpakref").touch()
        for page in (self.out / "index.html", self.out / "assets/credits.html"):
            text = page.read_text()
            doc = Document(text)
            ids = [a["id"] for _, a in doc.elements if "id" in a]
            self.assertEqual(len(ids), len(set(ids)), "duplicate HTML IDs")
            for tag, attrs in doc.elements:
                if tag == "img":
                    self.assertIn("alt", attrs)
                    self.assertTrue(attrs["alt"].strip(), "empty alt attribute")
                    self.assertIn("width", attrs)
                    self.assertIn("height", attrs)
                for key in ("src", "href"):
                    value = attrs.get(key, "")
                    url = urlsplit(value)
                    page_url = f"{BASE}/{page.relative_to(self.out)}"
                    resolved = urlsplit(urljoin(page_url, value))
                    if resolved.netloc != urlsplit(BASE).netloc:
                        continue
                    if url.path:
                        local = self.out / unquote(resolved.path).lstrip("/")
                        if resolved.path.endswith("/"):
                            local /= "index.html"
                        self.assertTrue(local.is_file(), resolved.path)
                        continue
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

    def test_donation_button_is_local_and_matches_owner_branding(self):
        text = self.render()
        doc = Document(text)
        # No third-party script, stylesheet or font request for the widget.
        for forbidden in ("cdnjs.buymeacoffee.com", "button.prod.min.js",
                          'data-name="bmc-button"', "fonts.googleapis.com"):
            self.assertNotIn(forbidden, text)
        buttons = [a for tag, a in doc.elements
                   if tag == "a" and a.get("class") == "bmc-btn"]
        self.assertEqual(len(buttons), 1)
        button = buttons[0]
        self.assertEqual(button["href"], "https://buymeacoffee.com/vittau")
        self.assertEqual(button["target"], "_blank")
        self.assertEqual(button["rel"], "noopener")
        self.assertIn("Donate", text[text.index('class="bmc-btn"'):])
        logos = [a for tag, a in doc.elements if tag == "img" and a.get("class") == "bmc-logo"]
        self.assertEqual(len(logos), 1)
        digest = hashlib.sha256((self.out / "assets/bmc-logo.svg").read_bytes()).hexdigest()[:16]
        self.assertEqual(logos[0]["src"], f"assets/bmc-logo.svg?v={digest}")
        svg = (self.out / "assets/bmc-logo.svg").read_text()
        self.assertIn('viewBox="0 0 884 1279"', svg)
        self.assertIn('fill="#ffffff"', svg, "white cup from the owner configuration")
        self.assertIn('fill="#000000"', svg, "owner outline colour")
        font = (self.out / "assets/Cookie-Regular.woff2").read_bytes()
        self.assertEqual(font[:4], b"wOF2")
        self.assertIn("SIL Open Font License",
                      (self.out / "assets/Cookie-OFL.txt").read_text())
        # Spec 083 R7: one compact widget in the hero, below the release line and
        # outside the header, while installation stays the primary action.
        header = re.search(r'<header class="site-header.*?</header>', text, re.DOTALL).group()
        self.assertNotIn("coffee-widget", header)
        self.assertNotIn("buymeacoffee", header)
        self.assertEqual(text.count('class="coffee-widget"'), 1)
        self.assertEqual(text.count("https://buymeacoffee.com/vittau"), 1)
        self.assertLess(text.index('<p class="release">'), text.index('class="coffee-widget"'))
        self.assertLess(text.index('class="coffee-widget"'), text.index('class="quiet-link"'))
        self.assertLess(text.index('class="button primary"'), text.index('class="coffee-widget"'))
        # A plain link: the donation CTA works with every script blocked.
        without_scripts = re.sub(r"(?is)<script\b.*?</script>", "", text)
        self.assertIn('href="https://buymeacoffee.com/vittau"', without_scripts)

    def test_stylesheet_url_tracks_css_content(self):
        text = self.render()
        digest = hashlib.sha256((self.out / "assets/style.css").read_bytes()).hexdigest()[:16]
        doc = Document(text)
        styles = [a["href"] for tag, a in doc.elements
                  if tag == "link" and a.get("rel") == "stylesheet"]
        self.assertEqual(styles, [f"assets/style.css?v={digest}"])

    def test_google_tag_loads_and_configures_once_per_page(self):
        self.render()
        tag_id = "G-BTYE5G8WCM"
        tag_url = f"https://www.googletagmanager.com/gtag/js?id={tag_id}"
        for page, marker in (("index.html", "Install on Steam Deck"),
                             ("assets/credits.html", "Asset credits")):
            with self.subTest(page=page):
                text = (self.out / page).read_text()
                scripts = [attrs for tag, attrs in Document(text).elements if tag == "script"]
                google = [attrs for attrs in scripts if attrs.get("src") == tag_url]
                self.assertEqual(len(google), 1, "exactly one Google tag script per page")
                self.assertIn("async", google[0])
                self.assertEqual(text.count("<!-- Google tag (gtag.js)"), 1)
                self.assertEqual(text.count(tag_id), 2, "script URL plus one config call")
                self.assertEqual(text.count("window.dataLayer = window.dataLayer || []"), 1)
                self.assertEqual(text.count("gtag('js', new Date())"), 1)
                self.assertEqual(text.count(f"gtag('config', '{tag_id}')"), 1)
                self.assertLess(text.index(tag_url), text.index("</head>"))
                # Spec 082: one website property, no other IDs, identifiers, URL data or events;
                # spec 085 adds exactly one named event, on the landing page only.
                self.assertEqual(set(re.findall(r"G-[A-Z0-9]{6,}", text)), {tag_id})
                for forbidden in ("gtag('set'", "user_id", "page_location", "send_to"):
                    self.assertNotIn(forbidden, text)
                self.assertEqual(text.count("gtag('event'"),
                                 1 if page == "index.html" else 0)
                # Tracking is the only JavaScript on the legal page, and content never
                # depends on it: the page text survives with every script removed.
                if page == "assets/credits.html":
                    self.assertEqual(len(scripts), 2)
                without_scripts = re.sub(r"(?is)<script\b.*?</script>", "", text)
                self.assertIn(marker, without_scripts)

    def test_donate_click_sends_one_named_event_without_parameters(self):
        """Spec 085: the Donate link reports one named event, and nothing else changes."""
        text = self.render()
        call = "gtag('event', 'donate_click')"
        self.assertEqual(text.count("gtag('event'"), 1)
        self.assertEqual(text.count(call), 1)
        # The event name is the whole payload: no parameters, identifiers or destination.
        self.assertNotRegex(text, r"gtag\('event',[^)]*\{")
        for forbidden in ("user_id", "page_location", "send_to", "link_url"):
            self.assertNotIn(forbidden, text[text.index(call):])
        # The anchor keeps its navigation contract untouched.
        buttons = [a for tag, a in Document(text).elements
                   if tag == "a" and a.get("class") == "bmc-btn"]
        self.assertEqual(len(buttons), 1)
        self.assertEqual(buttons[0]["id"], "donate-link")
        self.assertEqual(buttons[0]["href"], "https://buymeacoffee.com/vittau")
        self.assertEqual(buttons[0]["target"], "_blank")
        self.assertEqual(buttons[0]["rel"], "noopener")
        # The handler is the page's own plain script: it guards a blocked tag and never
        # cancels or delays the click, so the link still works without analytics.
        handler = re.search(r"(?is)<script>\s*\(function \(\) \{.*?</script>", text).group()
        self.assertIn("document.getElementById('donate-link')", handler)
        self.assertIn("addEventListener('click'", handler)
        self.assertIn("typeof gtag === 'function'", handler)
        self.assertNotIn("preventDefault", handler)
        self.assertNotIn("googletagmanager", handler)
        self.assertLess(text.index('id="donate-link"'), text.index(call))
        self.assertLess(text.index(call), text.index("</body>"))
        # Only the landing page measures the action.
        for page in ("assets/credits.html", "404.html"):
            with self.subTest(page=page):
                other = (self.out / page).read_text()
                self.assertNotIn("gtag('event'", other)
                self.assertNotIn("donate_click", other)

    def test_favicon_provenance_root_and_cache_version_on_both_pages(self):
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
