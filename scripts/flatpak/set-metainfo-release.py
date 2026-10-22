#!/usr/bin/env python3
"""Put the workspace version first in Flatpak AppStream metadata (spec 099)."""

import argparse
from datetime import date
from pathlib import Path
import tomllib
import xml.etree.ElementTree as ET


METAINFO = "packaging/flatpak/com.doomdecklauncher.DoomDeckLauncher.metainfo.xml"


def set_release(root, release_date):
    if date.fromisoformat(release_date).isoformat() != release_date:
        raise ValueError("release date must use YYYY-MM-DD")
    root = Path(root)
    with (root / "Cargo.toml").open("rb") as manifest:
        version = tomllib.load(manifest)["workspace"]["package"]["version"]
    if not isinstance(version, str) or not version:
        raise ValueError("workspace version must be a nonempty string")

    path = root / METAINFO
    tree = ET.parse(path)
    releases = tree.getroot().find("releases")
    if releases is None:
        raise ValueError("metainfo must contain a releases element")
    matching = [r for r in releases.findall("release") if r.get("version") == version]
    release = matching[0] if matching else ET.Element("release", version=version)
    for existing in matching:
        releases.remove(existing)
    release.set("date", release_date)
    release.tail = "\n    " if len(releases) else "\n  "
    releases.text = "\n    "
    releases.insert(0, release)
    tree.write(path, encoding="UTF-8", xml_declaration=True)
    # Keep the repository's final newline convention across repeated writes.
    with path.open("ab") as output:
        output.write(b"\n")
    return version


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--date", required=True, help="UTC build date, YYYY-MM-DD")
    args = parser.parse_args()
    print(f"AppStream release: {set_release(args.root, args.date)} ({args.date})")
