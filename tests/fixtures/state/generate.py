#!/usr/bin/env python3
"""Generate a release's machine state using its own serializers, offline.

Usage: python3 tests/fixtures/state/generate.py v0.0.7-deck
Only scratch checkouts beneath this worker's .scratch are changed.
"""
from pathlib import Path
import os
import subprocess
import sys

root = Path(__file__).resolve().parents[3]
tag = sys.argv[1]
if tag not in {"v0.0.5-screens.1", "v0.0.6-local", "v0.0.7-deck"}:
    raise SystemExit("Review the scenario against the new tag before adding it to this generator")
scratch = root / ".scratch" / tag
if not scratch.exists():
    subprocess.run(["git", "worktree", "add", "--detach", str(scratch), tag], cwd=root, check=True)
source_path = "crates/ddl/src/services.rs"
original = subprocess.check_output(["git", "show", f"{tag}:{source_path}"], cwd=root).decode()
scenario = Path(__file__).with_name("generate.rs").read_text()
if tag == "v0.0.5-screens.1":
    # This tag predates spec 026 and has no custom-setup type or serializer.
    start = scenario.index("        SetupStore::new(&data).save")
    end = scenario.index("    }\n}", start)
    scenario = scenario[:start] + scenario[end:]
(scratch / source_path).write_text(original + "\n" + scenario)
env = dict(os.environ, DDL_FIXTURE_OUTPUT=str(Path(__file__).parent / tag), CARGO_TARGET_DIR=str(root / "target"))
subprocess.run(["cargo", "test", "--manifest-path", str(scratch / "Cargo.toml"), "-p", "ddl", "--bin", "ddl", "services::release_state_fixture::generate_release_state_fixture", "--", "--exact", "--nocapture"], cwd=root, env=env, check=True)
