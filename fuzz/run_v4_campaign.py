#!/usr/bin/env python3
"""Run one bounded, source-pinned libFuzzer lane and retain its raw evidence."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
import time
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGETS = {
    "tl-syntax": "infinite_wire_decode",
    "tl-parse": "unbounded_parse_roundtrip",
    "tl-rewrite": "infinite_rewrite",
}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command(argv: list[str]) -> str:
    return subprocess.check_output(argv, cwd=ROOT, text=True).strip()


def corpus_manifest(target: str) -> dict[str, str]:
    directory = ROOT / "fuzz" / "corpus" / target
    lines = (directory / "SHA256SUMS").read_text().splitlines()
    seeds: dict[str, str] = {}
    for line in lines:
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.-]+)", line)
        if match is None or match[2] in seeds:
            raise ValueError("malformed or duplicate seed digest entry")
        seeds[match[2]] = match[1]
    actual = {path.name for path in directory.iterdir() if path.is_file()}
    if not seeds or actual != set(seeds) | {"SHA256SUMS"}:
        raise ValueError("undeclared, missing, or empty seed corpus")
    for name, expected in seeds.items():
        if digest((directory / name).read_bytes()) != expected:
            raise ValueError(f"seed digest mismatch: {name}")
    return dict(sorted(seeds.items()))


def classify(exit_code: int, log: bytes, requested: int, artifacts: list[Path]) -> tuple[str, int | None, str]:
    matches = re.findall(rb"(?m)^#(\d+)\s+DONE\b", log)
    actual = int(matches[-1]) if matches else None
    if artifacts:
        return "crash_requires_minimization_replay", actual, "artifact_present"
    if exit_code != 0:
        return "engine_failed_or_unconfirmed_crash", actual, "nonzero_exit_without_artifact"
    if len(matches) != 1 or actual != requested:
        return "incomplete", actual, "missing_or_short_done_count"
    return "bounded_no_crash", actual, "run_budget"


def run(output: Path, runs: int, seed: int, seconds: int) -> int:
    if runs <= 0 or seed <= 0 or seconds <= 0:
        raise ValueError("runs, seed and seconds must be positive")
    if output.exists():
        raise ValueError("output already exists; stale artifacts cannot be reused")
    package = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["name"]
    target = TARGETS[package]
    if command(["git", "status", "--porcelain"]):
        raise ValueError("source worktree must be clean at its measured commit")
    revision = command(["git", "rev-parse", "HEAD"])
    seeds = corpus_manifest(target)
    rustc = command(["rustc", "-Vv"])
    if "nightly" not in rustc:
        raise ValueError("libFuzzer campaign requires the recorded nightly toolchain")
    versions = {
        "rustc": rustc,
        "cargo": command(["cargo", "-V"]),
        "cargo_fuzz": command(["cargo", "fuzz", "-V"]),
    }
    output.mkdir(parents=True)
    artifacts_dir = output / "artifacts"
    artifacts_dir.mkdir()
    with tempfile.TemporaryDirectory(prefix="tl-v4-fuzz-") as temp:
        corpus = Path(temp) / "corpus"
        corpus.mkdir()
        for name in seeds:
            shutil.copyfile(ROOT / "fuzz" / "corpus" / target / name, corpus / name)
        argv = [
            "cargo", "fuzz", "run", "--sanitizer", "address", target, str(corpus), "--",
            f"-runs={runs}", f"-seed={seed}", f"-max_total_time={seconds}",
            "-max_len=4096", f"-artifact_prefix={artifacts_dir}/",
        ]
        started = time.monotonic()
        try:
            completed = subprocess.run(argv, cwd=ROOT, capture_output=True,
                                       timeout=seconds + 300, check=False)
            stdout, stderr, exit_code = completed.stdout, completed.stderr, completed.returncode
        except subprocess.TimeoutExpired as error:
            stdout, stderr, exit_code = error.stdout or b"", error.stderr or b"", 124
        elapsed = time.monotonic() - started
    compressed = {
        "stdout.log.gz": gzip.compress(stdout, mtime=0),
        "stderr.log.gz": gzip.compress(stderr, mtime=0),
    }
    for name, data in compressed.items():
        (output / name).write_bytes(data)
    artifact_paths = sorted(path for path in artifacts_dir.iterdir() if path.is_file())
    status, actual, stop_reason = classify(exit_code, stdout + b"\n" + stderr, runs, artifact_paths)
    report = {
        "schema": "tl-v4.libfuzzer-campaign/v1",
        "crate": package,
        "source_revision": revision,
        "target": target,
        "engine": "libFuzzer",
        "sanitizer": "address",
        "tool_versions": versions,
        "root_lock_sha256": digest((ROOT / "Cargo.lock").read_bytes()),
        "fuzz_lock_sha256": digest((ROOT / "fuzz" / "Cargo.lock").read_bytes()),
        "seed_files_sha256": seeds,
        "starting_corpus_sha256": digest(json.dumps(seeds, sort_keys=True,
                                                    separators=(",", ":")).encode()),
        "command": argv,
        "budget": {"runs": runs, "seed": seed, "seconds": seconds, "max_len": 4096},
        "observed": {"executions": actual, "exit_code": exit_code,
                     "elapsed_seconds": round(elapsed, 3), "stop_reason": stop_reason},
        "raw_output_sha256": {name: digest(data) for name, data in compressed.items()},
        "raw_stream_sha256": {"stdout": digest(stdout), "stderr": digest(stderr)},
        "crash_artifacts_sha256": {path.name: digest(path.read_bytes()) for path in artifact_paths},
        "replay": {"required": bool(artifact_paths), "confirmed": False,
                   "minimized_artifact_sha256": None},
        "status": status,
        "scope": "bounded observation; no four-crate V4 or correctness claim",
    }
    (output / "report.json").write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")
    return 0 if status == "bounded_no_crash" else 1


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--runs", type=int, default=1000)
    parser.add_argument("--seed", type=int, default=181)
    parser.add_argument("--seconds", type=int, default=30)
    args = parser.parse_args()
    raise SystemExit(run(args.output.resolve(), args.runs, args.seed, args.seconds))


if __name__ == "__main__":
    main()
