#!/usr/bin/env python3
"""Report recorded release evidence honestly; never grant publication authority."""
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
status = json.loads((root / "qualification/release-status.json").read_text())
allowed = {"passed", "blocked"}
assert status["gates"] and all(gate["status"] in allowed for gate in status["gates"])
for gate in status["gates"]:
    print(gate["status"] + ": " + gate["name"] + " — " + gate["evidence"])
blocked = any(gate["status"] != "passed" for gate in status["gates"])
print("PUBLIC RELEASE NOT READY" if blocked else "Recorded gates passed; maintainer publication review still required")
raise SystemExit(1 if blocked else 0)
