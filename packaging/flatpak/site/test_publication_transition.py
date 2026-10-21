"""Bounded OSTree layout fixtures for the one-time application-ID transition."""
import importlib.util
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import textwrap
import unittest

from test_render import APP_ID, ROOT

HELPER = ROOT / "scripts/flatpak/retire-old-app-ref.py"
spec = importlib.util.spec_from_file_location("retire_ref", HELPER)
retirement = importlib.util.module_from_spec(spec)
spec.loader.exec_module(retirement)


class TransitionTests(unittest.TestCase):
    def test_cleanup_cli_removes_only_exact_ref_and_is_idempotent(self):
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            obsolete = "refs/heads/" + retirement.OBSOLETE_REF
            fixtures = {obsolete: b"a" * 64 + b"\n"}
            for name in (f"refs/heads/app/{APP_ID}/x86_64/stable",
                         obsolete + "-candidate",
                         obsolete.replace("x86_64", "aarch64"),
                         obsolete.replace("/stable", "/beta"),
                         obsolete.replace("heads/", "remotes/upstream/"),
                         "refs/heads/runtime/com.example.Runtime/x86_64/stable",
                         "refs/heads/app/com.example.Other/x86_64/stable",
                         "refs/heads/appstream/x86_64", "summary", "summary.sig",
                         "objects/aa/commit", ".git/refs/heads/main", ".git/refs/tags/v1"):
                fixtures[name] = name.encode()
            for name, data in fixtures.items():
                target = repo / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
            for _ in range(2):
                subprocess.run([sys.executable, str(HELPER), str(repo)], check=True,
                               capture_output=True, text=True)
                remaining = {str(p.relative_to(repo)): p.read_bytes()
                             for p in repo.rglob("*") if p.is_file()}
                self.assertEqual(remaining, {name: data for name, data in fixtures.items()
                                             if name != obsolete})

    def test_symlink_paths_and_non_file_refs_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            outside = root / "outside"
            outside.mkdir()
            sentinel = outside / "stable"
            sentinel.write_bytes(b"untouched")
            repo = root / "repo"
            ref = repo / "refs/heads" / retirement.OBSOLETE_REF
            ref.parent.mkdir(parents=True)
            ref.symlink_to(sentinel)
            with self.assertRaises(ValueError):
                retirement.retire(repo)
            ref.unlink()
            ref.parent.rmdir()
            ref.parent.symlink_to(outside, target_is_directory=True)
            with self.assertRaises(ValueError):
                retirement.retire(repo)
            self.assertEqual(sentinel.read_bytes(), b"untouched")
            ref.parent.unlink()
            ref.mkdir(parents=True)
            with self.assertRaises(ValueError):
                retirement.retire(repo)

    def test_cleanup_precedes_metadata_generation_and_homepage_is_canonical(self):
        workflow = (ROOT / ".github/workflows/flatpak-remote.yml").read_text()
        export = workflow.index("          flatpak build-export ")
        retire = workflow.index('          python3 scripts/flatpak/retire-old-app-ref.py "$site/repo"')
        update = workflow.index("          flatpak build-update-repo ")
        self.assertLess(export, retire)
        self.assertLess(retire, update)
        metadata = workflow[update:workflow.index('          du -sh "$site/repo"', update)]
        self.assertIn('--homepage="${SITE_BASE}/"', metadata)
        self.assertIn('--gpg-sign="$GPG_FINGERPRINT"', metadata)
        self.assertIn('--gpg-homedir="$GNUPGHOME"', metadata)

    def test_smoke_listing_requires_new_id_and_rejects_obsolete_id(self):
        workflow = (ROOT / ".github/workflows/flatpak-remote.yml").read_text()
        check = textwrap.dedent(re.search(
            r'          flatpak --user remote-ls --app --columns=application ddl-smoke.*?\n          fi',
            workflow, re.DOTALL).group())
        old_id = retirement.OBSOLETE_REF.split("/")[1]
        with tempfile.TemporaryDirectory() as directory:
            apps = Path(directory) / "apps.txt"
            for listed, succeeds in (([APP_ID, "com.example.Other"], True),
                                     ([APP_ID, old_id], False), ([old_id], False), ([], False)):
                apps.write_text("\n".join(listed) + "\n")
                result = subprocess.run(
                    ["bash", "-euo", "pipefail", "-c",
                     'flatpak() { cat "$FIXTURE_APPS"; }\n' + check],
                    env={**os.environ, "APP_ID": APP_ID, "RUNNER_TEMP": directory,
                         "FIXTURE_APPS": str(apps)}, capture_output=True, text=True)
                self.assertEqual(result.returncode == 0, succeeds, result.stdout + result.stderr)
