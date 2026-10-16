#!/usr/bin/env python3
"""Orchestrator health check for Orca worker terminals (see docs/orchestration.md).

Usage: worker_health.py NAME=TERMINAL_HANDLE [...]
Every 300 s prints one line: per worker, working/IDLE (+UNCHANGED when the screen did not move),
context/cost from the TUI footer, error or permission-prompt signatures; then Codex account
rate-limit usage read from ~/.codex/sessions. Meant to run under a Claude Code Monitor.
"""
import json, subprocess, sys, time, hashlib, re, glob, os
WORKERS = dict(a.split("=", 1) for a in sys.argv[1:])  # name=handle
ERR = re.compile(r"(error|usage limit|rate limit|update available|HTTP [45]\d\d|Session location unavailable|Provider request failed|exceeded|panicked|not supported|Allow OpenCode|Permission required|Do you want to|approve|enter submit)", re.I)
WORK = re.compile(r"(esc interrupt|esc to interrupt|• Working)")
last = {}
CTX = re.compile(r"(Context \d+% used|[\d.]+K \(\d+%\) · \$[\d.]+)")
def codex_limits():
    files = sorted(glob.glob(os.path.expanduser("~/.codex/sessions/*/*/*/*.jsonl")), key=os.path.getmtime)[-5:]
    best = None
    for f in files:
        try:
            for line in open(f, errors="ignore"):
                if '"rate_limits"' in line:
                    o = json.loads(line); rl = None
                    def find(x):
                        nonlocal rl
                        if isinstance(x, dict):
                            if "rate_limits" in x and isinstance(x["rate_limits"], dict): rl = x["rate_limits"]
                            for v in x.values(): find(v)
                        elif isinstance(x, list):
                            for v in x: find(v)
                    find(o)
                    if rl and rl.get("primary"): best = (os.path.getmtime(f), rl)
        except Exception: pass
    if not best: return "codex-limits:?"
    rl = best[1]; p = rl.get("primary") or {}; q = rl.get("secondary") or {}
    return f"codex 5h {p.get('used_percent')}% / week {q.get('used_percent')}%"
while True:
    parts = []
    for name, h in WORKERS.items():
        try:
            out = subprocess.run(["orca", "terminal", "read", "--terminal", h, "--json"], capture_output=True, text=True, timeout=60).stdout
            t = json.loads(out)["result"]["terminal"]
            tail = t.get("tail") or []
            if isinstance(tail, str): tail = tail.splitlines()
            tail = [l for l in tail if l.strip()][-25:]
        except Exception as e:
            parts.append(f"{name}:READ_FAIL({type(e).__name__})"); continue
        blob = "\n".join(tail)
        digest = hashlib.sha1(blob.encode()).hexdigest()
        state = "working" if WORK.search("\n".join(tail[-6:])) else "IDLE"
        if last.get(name) == digest: state += "+UNCHANGED"
        last[name] = digest
        errs = sorted({m.group(1).lower() for l in tail[-12:] if "Tip:" not in l for m in [ERR.search(l)] if m})
        ctx = ""
        for l in reversed(tail):
            m = CTX.search(l)
            if m: ctx = m.group(1).replace("Context ", "ctx "); break
        parts.append(f"{name}:{state}" + (f"({ctx})" if ctx else "") + (f"[{','.join(errs)}]" if errs else ""))
    print(time.strftime("%H:%M"), " | ".join(parts), "||", codex_limits(), flush=True)
    time.sleep(300)
