"""Offline domain, exported identity and branch publication contracts (spec 079)."""
import configparser
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import textwrap
import unittest
import xml.etree.ElementTree as ET

from test_render import APP_ID, BASE, ROOT, renderer


class IdentityTests(unittest.TestCase):
    def test_exported_package_names_and_metadata_agree(self):
        package = ROOT / "packaging/flatpak"
        manifest = (package / f"{APP_ID}.yml").read_text()
        self.assertIn(f"app-id: {APP_ID}\n", manifest)
        self.assertIn("command: ddl\n", manifest)
        for suffix, destination in (("desktop", "applications"),
                                    ("metainfo.xml", "metainfo")):
            name = f"{APP_ID}.{suffix}"
            self.assertTrue((package / name).is_file())
            self.assertIn(f"packaging/flatpak/{name}", manifest)
            self.assertIn(f"share/{destination}/{name}", manifest)
        for suffix in ("png", "svg"):
            self.assertIn(f"apps/{APP_ID}.{suffix}", manifest)
        desktop = configparser.ConfigParser(interpolation=None)
        desktop.read(package / f"{APP_ID}.desktop")
        self.assertEqual(desktop["Desktop Entry"]["Exec"], "ddl")
        self.assertEqual(desktop["Desktop Entry"]["Icon"], APP_ID)
        metadata = ET.parse(package / f"{APP_ID}.metainfo.xml").getroot()
        self.assertEqual(metadata.findtext("id"), APP_ID)
        self.assertEqual(metadata.findtext("launchable"), f"{APP_ID}.desktop")
        self.assertEqual(metadata.findtext("url[@type='homepage']"), f"{BASE}/")
        self.assertEqual(metadata.findtext("url[@type='vcs-browser']"),
                         "https://github.com/vittau/Doom-Deck-Launcher")
        for name in ("flatpak", "flatpak-remote"):
            workflow = (ROOT / f".github/workflows/{name}.yml").read_text()
            self.assertIn(f"MANIFEST: packaging/flatpak/{APP_ID}.yml\n", workflow)
            self.assertIn(f"APP_ID: {APP_ID}\n", workflow)

    def test_both_publishers_keep_domain_and_shared_lock(self):
        for name in ("project-site", "flatpak-remote"):
            workflow = (ROOT / f".github/workflows/{name}.yml").read_text()
            self.assertIn(f"APP_ID: {APP_ID}\n", workflow)
            self.assertIn(f"SITE_BASE: {BASE}\n", workflow)
            self.assertIn("group: flatpak-remote\n", workflow)
            self.assertIn("cancel-in-progress: false", workflow)
            self.assertIn("scripts/flatpak/render-site.py", workflow)
        page = (ROOT / ".github/workflows/project-site.yml").read_text()
        self.assertIn("index.html|robots.txt|sitemap.xml|llms.txt|404.html|assets/*|CNAME)", page)

    def test_signed_cleanup_retains_cname_and_new_descriptor(self):
        workflow = (ROOT / ".github/workflows/flatpak-remote.yml").read_text()
        cleanup = re.search(r'          find "\$site" -maxdepth 1 -type f .*?-delete',
                            workflow, re.DOTALL).group()
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            renderer.render(out, "0.0.11", "2026-10-21", APP_ID, BASE)
            for name in (f"{APP_ID}.flatpakref", "ddl.flatpakrepo", ".nojekyll", "obsolete"):
                (out / name).touch()
            subprocess.run(["bash", "-eu", "-c", textwrap.dedent(cleanup)], check=True,
                           env={**os.environ, "site": directory, "APP_ID": APP_ID})
            self.assertEqual((out / "CNAME").read_text(), "doomdecklauncher.com\n")
            self.assertTrue((out / f"{APP_ID}.flatpakref").is_file())
            self.assertFalse((out / "obsolete").exists())
            # Discovery files and the branded 404 are page outputs, not strays.
            for kept in ("robots.txt", "sitemap.xml", "llms.txt", "404.html"):
                self.assertTrue((out / kept).is_file(), kept)

    def test_descriptor_templates_use_canonical_root_and_existing_key(self):
        workflow = (ROOT / ".github/workflows/flatpak-remote.yml").read_text()
        descriptors = re.search(r'          cat > "\$site/\$\{APP_ID\}.flatpakref".*?\n          EOF',
                                workflow, re.DOTALL).group()
        descriptors += "\n" + re.search(r'          cat > "\$site/ddl.flatpakrepo".*?\n          EOF',
                                        workflow, re.DOTALL).group()
        with tempfile.TemporaryDirectory() as directory:
            subprocess.run(["bash", "-eu", "-c", textwrap.dedent(descriptors)], check=True,
                           env={**os.environ, "site": directory, "APP_ID": APP_ID,
                                "SITE_BASE": BASE, "repo_url": f"{BASE}/repo/",
                                "public_key": "existing-public-key-fixture"})
            for name, section in ((f"{APP_ID}.flatpakref", "Flatpak Ref"),
                                  ("ddl.flatpakrepo", "Flatpak Repo")):
                config = configparser.ConfigParser(interpolation=None)
                config.read(Path(directory) / name)
                self.assertEqual(config[section]["Url"], f"{BASE}/repo/")
                self.assertEqual(config[section]["Homepage"], f"{BASE}/")
                self.assertEqual(config[section]["GPGKey"], "existing-public-key-fixture")
                if section == "Flatpak Ref":
                    self.assertEqual(config[section]["Name"], APP_ID)
                    self.assertEqual(config[section]["Branch"], "stable")
            self.assertEqual(config[section]["DefaultBranch"], "stable")

    def test_refresh_creates_preserves_and_corrects_root_cname(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            renderer.render(out, "0.0.11", "2026-10-21", APP_ID, BASE)
            for existing in (None, "doomdecklauncher.com\n", "unrelated.example\n"):
                if existing is None:
                    (out / "CNAME").unlink()
                else:
                    (out / "CNAME").write_text(existing)
                renderer.refresh(out, APP_ID, BASE)
                self.assertEqual((out / "CNAME").read_text(), "doomdecklauncher.com\n")

    def test_bootstrap_defers_without_mutation_then_accepts_new_channel(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            renderer.render(out, "0.0.11", "2026-10-21", APP_ID, BASE)
            (out / "repo").mkdir()
            (out / "repo/summary").write_bytes(b"signed repository")
            (out / "repo/summary.sig").write_bytes(b"signature")
            descriptor = out / f"{APP_ID}.flatpakref"
            valid = (f"[Flatpak Ref]\nName={APP_ID}\nBranch=stable\n"
                     f"Url={BASE}/repo/\nGPGKey=fixture-public-key\n")
            for content in (None, valid.replace(APP_ID, "com.example.Other"),
                            valid.replace(f"{BASE}/repo/", "https://example.com/repo/"),
                            valid.replace("Branch=stable", "Branch=master"),
                            valid.replace("GPGKey=fixture-public-key", "GPGKey="),
                            "not an ini file", valid + valid, "x" * (64 * 1024 + 1)):
                if content is not None:
                    descriptor.write_text(content)
                before = {p.relative_to(out): p.read_bytes() for p in out.rglob("*") if p.is_file()}
                self.assertFalse(renderer.publication_ready(out, APP_ID, BASE))
                self.assertEqual(before, {p.relative_to(out): p.read_bytes()
                                          for p in out.rglob("*") if p.is_file()})
            descriptor.write_text(valid)
            self.assertTrue(renderer.publication_ready(out, APP_ID, BASE))
            (out / "repo/summary.sig").unlink()
            self.assertFalse(renderer.publication_ready(out, APP_ID, BASE))

    def test_bootstrap_cli_and_workflow_gate_all_publish_actions(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([sys.executable, str(ROOT / "scripts/flatpak/render-site.py"),
                                     "--check-publication", "--output", directory,
                                     "--app-id", APP_ID, "--site-base", BASE], check=False)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(list(Path(directory).iterdir()), [])
        workflow = (ROOT / ".github/workflows/project-site.yml").read_text()
        steps = workflow.split("      - name: ")[1:]
        gated = [step for step in steps if "if: steps.publication.outputs.ready == 'true'" in step]
        self.assertEqual(len(gated), 6)
        self.assertIn("Page refresh deferred:", workflow)

