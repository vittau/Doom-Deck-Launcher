"""Discovery files, structured data and image delivery contracts (spec 083)."""
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import re
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from urllib.parse import urlsplit
import xml.etree.ElementTree as ET

from test_render import APP_ID, BASE, ROOT, Document, renderer

IMAGES = ROOT / "packaging/flatpak/site/images"
MANIFEST = IMAGES / "manifest.json"
SITEMAP_NS = {"s": "http://www.sitemaps.org/schemas/sitemap/0.9"}
GENERATOR_SPEC = importlib.util.spec_from_file_location(
    "generate_site_images", ROOT / "scripts/flatpak/generate-site-images.py")
images = importlib.util.module_from_spec(GENERATOR_SPEC)
GENERATOR_SPEC.loader.exec_module(images)
SCREENSHOTS = ("discover", "home", "play-setup", "mapset-detail", "audio-settings")


def text_content(html):
    """Visible text only: markup, script bodies and entities resolved, whitespace collapsed."""
    from html import unescape
    without_script = re.sub(r"(?is)<script\b.*?</script>", " ", html)
    without_style = re.sub(r"(?is)<style\b.*?</style>", " ", without_script)
    return re.sub(r"\s+", " ", unescape(re.sub(r"(?s)<[^>]+>", " ", without_style))).strip()


class DiscoveryTests(unittest.TestCase):
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

    def structured_data(self, text):
        blocks = re.findall(r'<script type="application/ld\+json">(.*?)</script>', text, re.DOTALL)
        self.assertEqual(len(blocks), 1, "exactly one JSON-LD block")
        return json.loads(blocks[0])

    def test_robots_is_policy_neutral_and_declares_the_canonical_sitemap(self):
        self.render()
        robots = (self.out / "robots.txt").read_text()
        self.assertEqual(robots, (
            "# DoomDeckLauncher project website\n"
            "User-agent: *\n"
            "Allow: /\n"
            "Disallow: /repo/\n"
            "\n"
            f"Sitemap: {BASE}/sitemap.xml\n"))
        # No crawler is singled out: the policy is the wildcard group only.
        self.assertEqual(robots.count("User-agent:"), 1)
        self.assertEqual(robots.count("Disallow:"), 1)
        for named in ("GPTBot", "ClaudeBot", "PerplexityBot", "Google-Extended",
                      "OAI-SearchBot", "CCBot", "bingbot", "Googlebot"):
            self.assertNotIn(named, robots)
        # The file follows SITE_BASE rather than a baked-in host.
        self.render(site_base="https://example.com/project")
        moved = (self.out / "robots.txt").read_text()
        self.assertIn("Sitemap: https://example.com/project/sitemap.xml\n", moved)
        self.assertNotIn(BASE, moved)

    def test_sitemap_lists_the_homepage_and_credits_without_invented_lastmod(self):
        self.render()
        text = (self.out / "sitemap.xml").read_text()
        self.assertTrue(text.startswith('<?xml version="1.0" encoding="UTF-8"?>\n'))
        root = ET.fromstring(text)
        self.assertEqual(root.tag, f"{{{SITEMAP_NS['s']}}}urlset")
        urls = root.findall("s:url", SITEMAP_NS)
        locs = [url.findtext("s:loc", namespaces=SITEMAP_NS) for url in urls]
        self.assertEqual(locs, [f"{BASE}/", f"{BASE}/assets/credits.html"])
        for url in urls:
            self.assertEqual(len(list(url)), 1, "only <loc>; no optional lastmod is invented")
            self.assertIsNone(url.find("s:lastmod", SITEMAP_NS))
        # Each listed URL is a real rendered page, and both stay indexable.
        for loc in locs:
            relative = urlsplit(loc).path.lstrip("/") or "index.html"
            page = self.out / relative
            self.assertTrue(page.is_file(), loc)
            self.assertNotRegex(page.read_text(), r'(?i)<meta[^>]+name="robots"[^>]+noindex')

    def test_structured_data_uses_only_facts_visible_on_the_page(self):
        text = self.render()
        data = self.structured_data(text)
        nodes = {node["@type"]: node for node in data["@graph"]}
        self.assertEqual(set(nodes), {"WebSite", "SoftwareApplication"})
        site = nodes["WebSite"]
        self.assertEqual(site["name"], "DoomDeckLauncher")
        self.assertEqual(site["alternateName"], "DDL")
        self.assertEqual(site["url"], f"{BASE}/")
        app = nodes["SoftwareApplication"]
        self.assertEqual(app["name"], "DoomDeckLauncher")
        self.assertEqual(app["applicationCategory"], "GameApplication")
        self.assertEqual(app["operatingSystem"], "Linux")
        self.assertEqual(app["offers"], {"@type": "Offer", "price": 0,
                                        "priceCurrency": "USD"})
        self.assertTrue(app["isAccessibleForFree"])
        self.assertEqual(app["license"], "https://spdx.org/licenses/GPL-3.0-or-later.html")
        self.assertEqual(app["downloadUrl"], f"{BASE}/{APP_ID}.flatpakref")
        visible = text_content(text)
        for claim in ("a Steam Deck and legally owned DOOM base-game files (IWADs)",
                      "never bundles or downloads commercial base games",
                      "GPL-3.0-or-later", "DSDA-Doom", "free, open-source"):
            self.assertIn(claim, visible)
        # The requirement sentence stays centered on Steam Deck and owned IWADs.
        self.assertEqual(app["softwareRequirements"],
                         "A Steam Deck and legally owned DOOM base-game files (IWADs). "
                         "DoomDeckLauncher never bundles or downloads commercial base games.")
        self.assertNotIn("other Linux", json.dumps(data))
        # Truth guard: no ratings, reviews, author, publisher or organization.
        for node in data["@graph"]:
            for forbidden in ("aggregateRating", "ratingValue", "review", "author",
                              "publisher", "organization", "brand"):
                self.assertNotIn(forbidden, node)
        self.assertNotIn("Windows", json.dumps(nodes))
        self.assertNotIn("macOS", json.dumps(nodes))
        self.assertNotIn("Windows", app["operatingSystem"])
        # The canonical URL agrees across metadata and structured data.
        canonical = [a["href"] for tag, a in Document(text).elements
                     if tag == "link" and a.get("rel") == "canonical"]
        self.assertEqual(canonical, [site["url"]])

    def test_structured_data_survives_unsafe_generated_values(self):
        text = self.render(site_base='https://example.com/a"quoted</script>&x')
        data = self.structured_data(text)
        self.assertEqual(data["@graph"][0]["url"], 'https://example.com/a"quoted</script>&x/')
        block = re.search(r'<script type="application/ld\+json">(.*?)</script>', text,
                          re.DOTALL).group(1)
        self.assertNotIn("</script>", block)
        # Only the four real script closures remain (Google tag loader, inline
        # tag config, JSON-LD and the donation embed).
        self.assertEqual(text.count("</script>"), 4)
        self.assertIn("\\u003c/script\\u003e", text)
        # HTML attributes keep their entity escaping.
        self.assertIn('https://example.com/a&quot;quoted', text)

    def test_social_metadata_names_site_locale_and_image_type(self):
        text = self.render()
        doc = Document(text)
        meta = {}
        for tag, attrs in doc.elements:
            if tag == "meta":
                meta[attrs.get("property") or attrs.get("name")] = attrs.get("content")
        self.assertEqual(meta["og:site_name"], "DoomDeckLauncher")
        self.assertEqual(meta["og:locale"], "en_US")
        self.assertEqual(meta["og:image:type"], "image/png")
        social = self.out / "assets/social-preview.png"
        self.assertEqual(social.read_bytes()[:8], b"\x89PNG\r\n\x1a\n")
        self.assertEqual((int.from_bytes(social.read_bytes()[16:20], "big"),
                          int.from_bytes(social.read_bytes()[20:24], "big")), (1280, 640))
        self.assertEqual(meta["og:image:width"], "1280")
        self.assertEqual(meta["og:image:height"], "640")
        self.assertIn("DoomDeckLauncher", meta["twitter:title"])
        self.assertIn("Steam Deck", meta["twitter:title"])
        self.assertIn("free, open-source", meta["twitter:description"])
        for key in ("og:title", "og:description", "og:url", "og:image"):
            self.assertTrue(meta.get(key), key)

    def test_credits_page_has_description_and_self_canonical_and_stays_indexable(self):
        self.render()
        text = (self.out / "assets/credits.html").read_text()
        doc = Document(text)
        meta = {attrs.get("name"): attrs.get("content") for tag, attrs in doc.elements
                if tag == "meta" and attrs.get("name")}
        self.assertIn("Credits and licenses", meta["description"])
        self.assertGreaterEqual(len(meta["description"]), 80)
        self.assertLessEqual(len(meta["description"]), 200)
        canonical = [a["href"] for tag, a in doc.elements
                     if tag == "link" and a.get("rel") == "canonical"]
        self.assertEqual(canonical, [f"{BASE}/assets/credits.html"])
        self.assertNotIn("noindex", text)
        sitemap = ET.fromstring((self.out / "sitemap.xml").read_text())
        self.assertIn(canonical[0], [loc.text for loc in sitemap.iter(
            f"{{{SITEMAP_NS['s']}}}loc")])

    def test_branded_404_is_small_noindex_and_self_contained(self):
        self.render()
        text = (self.out / "404.html").read_text()
        self.assertLess(len(text.encode()), 5000)
        self.assertIn('<meta name="robots" content="noindex, follow">', text)
        self.assertNotIn("<script", text)
        doc = Document(text)
        links = {a.get("href") for tag, a in doc.elements if tag == "a"}
        self.assertIn(f"{BASE}/", links)
        self.assertIn(f"{BASE}/assets/credits.html", links)
        styles = [a["href"] for tag, a in doc.elements
                  if tag == "link" and a.get("rel") == "stylesheet"]
        digest = hashlib.sha256((self.out / "assets/style.css").read_bytes()).hexdigest()[:16]
        # Absolute URLs: GitHub Pages keeps the missing path in the address bar.
        self.assertEqual(styles, [f"{BASE}/assets/style.css?v={digest}"])

    def test_webp_derivatives_are_current_and_cache_versioned(self):
        self.render()
        with contextlib.redirect_stdout(io.StringIO()) as report:
            self.assertEqual(images.check(), 0)
        self.assertIn("WebP derivatives", report.getvalue())
        manifest = json.loads(MANIFEST.read_text())
        for name, source in manifest["sources"].items():
            path = ROOT / source["path"]
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), source["sha256"], name)
            self.assertEqual(images.png_size(path), (source["width"], source["height"]))
            # The shared README/source PNG is copied unchanged by the renderer.
            self.assertEqual((self.out / "assets" / name).read_bytes(), path.read_bytes(), name)
        for name, entry in manifest["derivatives"].items():
            path = IMAGES / name
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), entry["sha256"], name)
            self.assertEqual(images.webp_size(path), (entry["width"], entry["height"]), name)
            self.assertEqual(path.stat().st_size, entry["bytes"], name)
            self.assertIn(entry["source"], manifest["sources"])
            self.assertLessEqual(entry["width"], manifest["sources"][entry["source"]]["width"])
        self.assertEqual(set(manifest["derivatives"]),
                         {f"{name}-{width}.webp" for name in ("hero", *SCREENSHOTS)
                          for width in ((960, 1920, 2560) if name == "hero" else (640, 960, 1280))})

    def test_stale_derivative_manifest_is_rejected(self):
        manifest = json.loads(MANIFEST.read_text())
        manifest["sources"]["hero.png"]["sha256"] = "0" * 64
        manifest["derivatives"]["hero-960.webp"]["sha256"] = "1" * 64
        with tempfile.TemporaryDirectory() as directory:
            stale = Path(directory) / "manifest.json"
            stale.write_text(json.dumps(manifest))
            with patch.object(images, "MANIFEST", stale), \
                    contextlib.redirect_stdout(io.StringIO()), \
                    contextlib.redirect_stderr(io.StringIO()) as errors:
                self.assertEqual(images.check(), 1)
        self.assertIn("rerun the generator", errors.getvalue())

    def test_picture_elements_deliver_responsive_webp_with_png_fallbacks(self):
        text = self.render()
        doc = Document(text)
        manifest = json.loads(MANIFEST.read_text())
        checked = set()
        for tag, attrs in doc.elements:
            srcset = attrs.get("srcset")
            if not srcset:
                continue
            self.assertEqual(tag, "source")
            self.assertEqual(attrs.get("type"), "image/webp")
            self.assertTrue(attrs.get("sizes"))
            for candidate in srcset.split(","):
                url, descriptor = candidate.strip().split()
                self.assertTrue(descriptor.endswith("w"))
                path, _, query = url.partition("?")
                local = self.out / path
                self.assertTrue(local.is_file(), path)
                digest = hashlib.sha256(local.read_bytes()).hexdigest()[:16]
                self.assertEqual(query, f"v={digest}", path)
                checked.add(local.name)
        self.assertEqual(checked, set(manifest["derivatives"]))
        for tag, attrs in doc.elements:
            if tag != "img" or ".png?v=" not in attrs.get("src", ""):
                continue
            path = attrs["src"].partition("?")[0]
            self.assertEqual(attrs["width"], str(images.png_size(self.out / path)[0]))
            self.assertEqual(attrs["height"], str(images.png_size(self.out / path)[1]))
            self.assertTrue(attrs.get("alt") is not None)
        # Above the fold: the hero artwork and its launcher preview load eagerly
        # with high priority; the four below-fold screenshots stay lazy.
        eager = [(attrs["src"], attrs.get("fetchpriority")) for tag, attrs in doc.elements
                 if tag == "img" and attrs.get("fetchpriority") == "high"]
        self.assertEqual(len(eager), 2, eager)
        self.assertTrue(all(path.startswith("assets/hero.png") or path.startswith("assets/discover.png")
                            for path, _ in eager))
        lazy = [attrs["src"] for tag, attrs in doc.elements
                if tag == "img" and attrs.get("loading") == "lazy"]
        self.assertEqual(len(lazy), 4)
        self.assertTrue(all(attrs.get("decoding") == "async" for tag, attrs in doc.elements
                            if tag == "img" and attrs.get("loading") == "lazy"))
        # Full-size anchors still open the untouched PNG originals.
        anchors = [attrs["href"].partition("?")[0] for tag, attrs in doc.elements
                   if tag == "a" and ".png?v=" in attrs.get("href", "")]
        for path in anchors:
            self.assertTrue((self.out / path).is_file(), path)
        self.assertEqual(len(anchors), 5)

    def test_page_only_allowlist_accepts_discovery_and_derivative_files(self):
        workflow = (ROOT / ".github/workflows/project-site.yml").read_text()
        pattern = re.search(r'case "\$path" in\n\s+(index\.html\|[^\n]+)\) ;;', workflow).group(1)
        script = ('while IFS= read -r path; do\n'
                  f'  case "$path" in {pattern}) ;;\n'
                  '  *) echo "rejected: $path"; exit 1 ;;\n'
                  '  esac\n'
                  'done\n')
        accepted = ("index.html\nrobots.txt\nsitemap.xml\n404.html\nCNAME\n"
                    "assets/home.webp\nassets/hero-1920.webp\n")
        result = subprocess.run(["bash", "-c", script], input=accepted, text=True,
                                capture_output=True)
        self.assertEqual(result.returncode, 0, result.stdout)
        for rejected in ("repo/summary", ".nojekyll", "ddl.flatpakrepo",
                         f"{APP_ID}.flatpakref", "robots.txt.bak"):
            result = subprocess.run(["bash", "-c", script], input=rejected + "\n",
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 1, rejected)

    def test_questions_and_product_description_do_not_depend_on_javascript(self):
        text = self.render()
        without_scripts = re.sub(r"(?is)<script\b.*?</script>", "", text)
        for phrase in ("Questions, answered.", "What is DoomDeckLauncher?",
                       "Is it free, and what is the license?", "Which source port does it use?",
                       "How do installation and updates work?", "Is it made for Steam Deck?",
                       "No commercial base game is bundled or downloaded"):
            self.assertIn(phrase, without_scripts)

    def test_refresh_regenerates_discovery_files_and_keeps_release_metadata(self):
        self.render(version="0.0.9-focus", published="2026-10-20")
        (self.out / "robots.txt").write_text("stale")
        (self.out / "sitemap.xml").write_text("stale")
        renderer.refresh(self.out, APP_ID, BASE)
        self.assertEqual((self.out / "robots.txt").read_text(),
                         renderer.robots_txt(BASE))
        self.assertEqual((self.out / "sitemap.xml").read_text(),
                         renderer.sitemap_xml(BASE))
        self.assertIn("v0.0.9-focus", (self.out / "index.html").read_text())


if __name__ == "__main__":
    unittest.main()
