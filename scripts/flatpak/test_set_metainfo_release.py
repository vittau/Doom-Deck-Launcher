"""Offline tests for AppStream release synchronization (standard library only)."""

import importlib.util
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET


spec = importlib.util.spec_from_file_location(
    "set_metainfo_release", Path(__file__).with_name("set-metainfo-release.py")
)
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)


class SetMetainfoReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "Cargo.toml").write_text(
            '[workspace.package]\nversion = "0.0.12"\n', encoding="utf-8"
        )
        self.path = self.root / helper.METAINFO
        self.path.parent.mkdir(parents=True)
        self.path.write_text(
            '<component><name>A &amp; B</name><releases>\n'
            '  <release version="0.0.11" date="2026-10-21">'
            '<description><p>Previous &amp; preserved.</p></description></release>\n'
            '  <release version="0.0.9" date="2026-10-20"/>\n'
            '</releases></component>\n', encoding="utf-8"
        )

    def releases(self):
        return ET.parse(self.path).getroot().findall("releases/release")

    def test_prepends_workspace_version_and_preserves_history_and_other_metadata(self):
        previous = [ET.tostring(r).strip() for r in self.releases()]
        self.assertEqual(helper.set_release(self.root, "2026-10-22"), "0.0.12")
        releases = self.releases()
        self.assertEqual(releases[0].attrib, {"version": "0.0.12", "date": "2026-10-22"})
        self.assertEqual([ET.tostring(r).strip() for r in releases[1:]], previous)
        self.assertEqual(ET.parse(self.path).getroot().findtext("name"), "A & B")

    def test_repeated_run_is_byte_identical_and_updates_date_without_duplicates(self):
        helper.set_release(self.root, "2026-10-22")
        first = self.path.read_bytes()
        helper.set_release(self.root, "2026-10-22")
        self.assertEqual(self.path.read_bytes(), first)
        helper.set_release(self.root, "2026-10-23")
        self.assertEqual(len(self.releases()), 3)
        self.assertEqual(self.releases()[0].get("date"), "2026-10-23")

    def test_promotes_existing_release_without_losing_its_description(self):
        (self.root / "Cargo.toml").write_text(
            '[workspace.package]\nversion = "0.0.9"\n', encoding="utf-8"
        )
        tree = ET.parse(self.path)
        release = tree.getroot().findall("releases/release")[1]
        ET.SubElement(ET.SubElement(release, "description"), "p").text = "Keep me."
        tree.write(self.path)
        helper.set_release(self.root, "2026-10-22")
        releases = self.releases()
        self.assertEqual([r.get("version") for r in releases], ["0.0.9", "0.0.11"])
        self.assertEqual(releases[0].findtext("description/p"), "Keep me.")

    def test_invalid_dates_leave_file_unchanged(self):
        original = self.path.read_bytes()
        for invalid in ["2026-02-30", "20261022", "today"]:
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                helper.set_release(self.root, invalid)
            self.assertEqual(self.path.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
