#!/usr/bin/env python3
"""Run synthetic allocation/time/size qualification using only Python stdlib."""
import argparse
import csv
from datetime import datetime, timezone
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import statistics
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, CARGO_BUILD_JOBS="1", RUST_TEST_THREADS="1")

def run(args, **kwargs):
    return subprocess.run(args, cwd=ROOT, env=ENV, text=True, check=True, capture_output=True, **kwargs)

def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--record", action="store_true", help="replace checked-in host baseline intentionally")
    group.add_argument("--check-baseline", action="store_true", help="require matching host/toolchain and enforce timing/size budgets")
    args = parser.parse_args()
    artifacts = ROOT / "target" / "qualification"
    artifacts.mkdir(parents=True, exist_ok=True)
    built = run(["cargo", "build", "--locked", "--release", "--bench", "qualification", "--features", "reversible", "--message-format=json"])
    executable = next(json.loads(line)["executable"] for line in built.stdout.splitlines() if json.loads(line).get("reason") == "compiler-artifact" and json.loads(line).get("executable") and json.loads(line)["target"]["name"] == "qualification")
    rows = []
    memory = []
    for repeat in range(3):
        command = [executable]
        if platform.system() == "Darwin":
            command = ["/usr/bin/time", "-l", executable]
        result = run(command)
        reader = csv.DictReader(io.StringIO(result.stdout))
        for row in reader:
            row["repetition"] = repeat + 1
            rows.append(row)
        match = re.search(r"(\d+)\s+maximum resident set size", result.stderr)
        memory.append(int(match.group(1)) if match else None)
    sizes = {}
    for name, features in [("default", []), ("reversible", ["--features", "reversible"])]:
        ENV["CARGO_TARGET_DIR"] = str(artifacts / name)
        run(["cargo", "build", "--locked", "--release", "--example", "size_probe", "--no-default-features", *features])
        probe = artifacts / name / "release" / "examples" / ("size_probe.exe" if platform.system() == "Windows" else "size_probe")
        run([str(probe)])
        sizes[name] = probe.stat().st_size
    ENV.pop("CARGO_TARGET_DIR", None)
    machine = {
        "os": platform.system(), "os_release": platform.release(), "architecture": platform.machine(),
        "rustc": run(["rustc", "--version"]).stdout.strip(),
        "cargo": run(["cargo", "--version"]).stdout.strip(),
    }
    if platform.system() == "Darwin":
        machine["cpu"] = run(["sysctl", "-n", "machdep.cpu.brand_string"]).stdout.strip()
        machine["memory_bytes"] = int(run(["sysctl", "-n", "hw.memsize"]).stdout)
    grouped = {}
    for row in rows:
        key = "/".join(row[name] for name in ["workload", "mode", "stage"])
        grouped.setdefault(key, []).append(int(row["ns_per_op"]))
    if not grouped or any(len(values) != 3 for values in grouped.values()):
        raise SystemExit("incomplete workload repetitions: require three samples per workload/stage")
    medians = {key: statistics.median(values) for key, values in grouped.items()}
    summary = {
        "observation_date": datetime.now(timezone.utc).date().isoformat(), "machine": machine,
        "build": "release; LTO=true; codegen-units=1; no strip; allocator instrumentation only in bench",
        "repetitions": 3, "ns_per_op_medians": medians,
        "peak_rss_bytes": memory,
        "probe_binary_bytes": sizes, "feature_delta_bytes": sizes["reversible"] - sizes["default"],
        "thresholds": {"same_host_time_multiplier": 3, "time_noise_floor_ns": 1000, "binary_growth_fraction": 0.1, "binary_noise_floor_bytes": 16384},
    }
    source_files = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "scripts/qualify.py", ROOT / "benches/qualification.rs", ROOT / "benches/support/counting_allocator.rs", ROOT / "examples/size_probe.rs", *sorted((ROOT / "src").glob("*.rs"))]
    summary["source_files"] = [str(path.relative_to(ROOT)) for path in source_files]
    summary["base_git_revision"] = run(["git", "rev-parse", "HEAD"]).stdout.strip()
    summary["source_digest_sha256"] = hashlib.sha256(b"".join(path.read_bytes() for path in source_files)).hexdigest()
    destination = ROOT / "qualification" if args.record else artifacts
    destination.mkdir(parents=True, exist_ok=True)
    with (destination / "baseline.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
        writer.writeheader(); writer.writerows(rows)
    (destination / "baseline.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    if args.check_baseline:
        baseline = json.loads((ROOT / "qualification" / "baseline.json").read_text())
        if baseline["machine"] != machine:
            raise SystemExit("baseline host/toolchain mismatch: record a reviewed comparable baseline first")
        if baseline["repetitions"] != 3:
            raise SystemExit("baseline repetition count changed: review a comparable baseline first")
        budget = baseline["thresholds"]
        if medians.keys() != baseline["ns_per_op_medians"].keys():
            raise SystemExit("workload/stage coverage changed: review and record a complete baseline first")
        for key, median in medians.items():
            limit = max(baseline["ns_per_op_medians"][key] * budget["same_host_time_multiplier"], budget["time_noise_floor_ns"])
            if median > limit:
                raise SystemExit("same-host runtime regression gate failed: " + key)
        for name, size in sizes.items():
            allowed = baseline["probe_binary_bytes"][name] + max(int(baseline["probe_binary_bytes"][name] * budget["binary_growth_fraction"]), budget["binary_noise_floor_bytes"])
            if size > allowed:
                raise SystemExit("comparable binary size gate failed: " + name)
    print("qualification passed: 3 repetitions, allocation invariants, probe binaries, synthetic reports")
    print(json.dumps({"binary_bytes": sizes, "feature_delta": summary["feature_delta_bytes"], "peak_rss_bytes": memory}))

if __name__ == "__main__":
    main()
