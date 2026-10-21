#!/usr/bin/env python3
"""Retire only the obsolete DDL x86_64 stable loose OSTree ref (spec 079).

The published branch stores this exact file under repo/refs/heads. Unlinking
it avoids prefix deletion; other refs, commits and Git history are untouched.
Run before regenerating and signing AppStream/summary with build-update-repo.
"""
import argparse
from pathlib import Path

OBSOLETE_REF = "app/io.github.vittau.Doom-Deck-Launcher/x86_64/stable"


def retire(repo):
    repo = Path(repo)
    target = repo / "refs/heads" / OBSOLETE_REF
    candidate = repo
    for part in ("", *target.relative_to(repo).parts):
        candidate = candidate / part
        if candidate.is_symlink():
            raise ValueError("refusing a symlink in the obsolete ref path")
    if not target.exists():
        return False
    if not target.is_file():
        raise ValueError("obsolete ref must be a regular file")
    target.unlink()
    return True


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repo", type=Path)
    args = parser.parse_args()
    print("Retired obsolete app ref" if retire(args.repo) else "Obsolete app ref already absent")
