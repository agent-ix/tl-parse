#!/usr/bin/env python3
"""Summarize two Criterion runs without promoting an unmatched run to a pass.

The baseline archive must use the identical copied harness and inputs. This
script only reads Criterion's existing measurements; it never runs a benchmark.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CASES = (
    "bounded_small",
    "past_small",
    "infinite_small",
    "infinite_fairness",
    "bounded_near_node_cap",
    "past_near_node_cap",
    "infinite_near_node_cap",
)
INPUTS = ROOT / "benches" / "inputs"
THRESHOLD = 0.20


def command(*args: str) -> str:
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, check=True)
    return result.stdout.strip()


def optional_command(*args: str) -> str | None:
    try:
        return command(*args)
    except (OSError, subprocess.CalledProcessError):
        return None


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def hardware_model() -> dict[str, str | None]:
    raw = optional_command("system_profiler", "SPHardwareDataType", "-json")
    if raw is None:
        return {"chip": None, "model": None, "memory": None, "processor_layout": None}
    try:
        overview = json.loads(raw)["SPHardwareDataType"][0]
    except (ValueError, KeyError, IndexError, TypeError):
        return {"chip": None, "model": None, "memory": None, "processor_layout": None}
    # Keep only performance-relevant fields; hardware serials and device IDs
    # in the unfiltered system report must not enter a retained artifact.
    return {
        "chip": overview.get("chip_type"),
        "model": overview.get("machine_model"),
        "memory": overview.get("physical_memory"),
        "processor_layout": overview.get("number_processors"),
    }


def checked_inputs() -> dict[str, str]:
    expected = {}
    for line in (INPUTS / "SHA256SUMS").read_text().splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([a-z0-9-]+\.txt)", line)
        if match is None:
            raise ValueError(f"invalid benchmark digest row: {line!r}")
        expected[match.group(2)] = match.group(1)
    actual = {path.name: digest(path) for path in INPUTS.glob("*.txt")}
    if not expected or expected != actual:
        raise ValueError(f"benchmark input manifest disagrees: {expected=} {actual=}")
    return actual


def criterion_run(path: Path) -> dict:
    sample = json.loads((path / "sample.json").read_text())
    estimates = json.loads((path / "estimates.json").read_text())
    iterations = sample["iters"]
    times = sample["times"]
    if len(iterations) != 20 or len(times) != 20 or any(value <= 0 for value in iterations):
        raise ValueError(f"expected 20 nonvacuous samples at {path}")
    normalized = [round(total / count, 6) for total, count in zip(times, iterations, strict=True)]
    median = estimates["median"]
    deviation = estimates["std_dev"]["point_estimate"]
    return {
        "sample_count": len(normalized),
        "sample_ns_per_iteration": normalized,
        "median_ns": median["point_estimate"],
        "median_95pct_ci_ns": [
            median["confidence_interval"]["lower_bound"],
            median["confidence_interval"]["upper_bound"],
        ],
        "stddev_ns": deviation,
        "variance_ns2": deviation * deviation,
    }


def row(directory: Path, name: str, baseline_name: str) -> dict:
    case = directory / "parser_roundtrip" / name
    baseline = criterion_run(case / baseline_name)
    current = criterion_run(case / "new")
    change = json.loads((case / "change" / "estimates.json").read_text())["median"]
    low = change["confidence_interval"]["lower_bound"]
    high = change["confidence_interval"]["upper_bound"]
    point = change["point_estimate"]
    if low > THRESHOLD:
        disposition = "repeat_required_above_threshold"
    elif high > THRESHOLD:
        disposition = "inconclusive_threshold_overlap"
    else:
        disposition = "below_20pct_threshold"
    return {
        "case": name,
        "baseline": baseline,
        "candidate": current,
        "median_change_ratio": point,
        "median_change_95pct_ci_ratio": [low, high],
        "disposition": disposition,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--initial-criterion-dir", type=Path, required=True)
    parser.add_argument("--criterion-dir", type=Path, required=True)
    parser.add_argument("--baseline-name", required=True)
    parser.add_argument("--baseline-commit", required=True)
    parser.add_argument("--candidate-commit", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    head = command("git", "rev-parse", "HEAD")
    if command("git", "status", "--porcelain"):
        raise ValueError("candidate worktree must be clean before reporting")
    if head != args.candidate_commit:
        for path in (
            "src",
            "benches/parser_roundtrip.rs",
            "benches/inputs",
            "Cargo.toml",
            "Cargo.lock",
        ):
            measured = command("git", "rev-parse", f"{args.candidate_commit}:{path}")
            current = command("git", "rev-parse", f"HEAD:{path}")
            if measured != current:
                raise ValueError(
                    f"candidate HEAD {head} changes measured {path} from {args.candidate_commit}"
                )
    if command("git", "rev-parse", args.baseline_commit) != args.baseline_commit:
        raise ValueError("baseline commit must be an exact full commit ID")
    inputs = checked_inputs()
    cases = []
    for name in CASES:
        initial = row(args.initial_criterion_dir, name, args.baseline_name)
        repeat = row(args.criterion_dir, name, args.baseline_name)
        states = (initial["disposition"], repeat["disposition"])
        if states == ("repeat_required_above_threshold",) * 2:
            disposition = "confirmed_above_20pct_threshold"
        elif "repeat_required_above_threshold" in states:
            disposition = "one_run_spike_not_confirmed"
        elif "inconclusive_threshold_overlap" in states:
            disposition = "inconclusive_threshold_overlap"
        else:
            disposition = "below_20pct_threshold_in_both_runs"
        cases.append(
            {
                "case": name,
                "initial_paired_run": initial,
                "repeat_paired_run": repeat,
                "disposition": disposition,
            }
        )
    confirmed = [case["case"] for case in cases if case["disposition"] == "confirmed_above_20pct_threshold"]
    spikes = [case["case"] for case in cases if case["disposition"] == "one_run_spike_not_confirmed"]
    uncertain = [case["case"] for case in cases if case["disposition"] == "inconclusive_threshold_overlap"]
    if confirmed:
        conclusion = f"Repeat-confirmed >20% parser roundtrip regressions require findings: {confirmed}."
    elif uncertain:
        conclusion = f"Threshold-overlapping parser roundtrip cases remain inconclusive: {uncertain}."
    elif spikes:
        conclusion = f"One-run >20% spikes were not reproduced: {spikes}; no repeat-confirmed regression is established."
    else:
        conclusion = "Both paired runs remain below the 20% parser roundtrip regression threshold."
    conclusion += " The host was not thermally controlled; other TL performance lanes are outside this report."
    manifest = (ROOT / "Cargo.toml").read_text()
    syntax_pin = re.search(r'tl-syntax = \{[^\n]*rev = "([0-9a-f]{40})"', manifest)
    if syntax_pin is None:
        raise ValueError("exact tl-syntax pin is absent")
    report = {
        "schema": "tl-parse.criterion-paired-report/v1",
        "scope": "parser_format_roundtrip_only",
        "baseline": {
            "parser_commit": args.baseline_commit,
            "parser_src_tree": command("git", "rev-parse", f"{args.baseline_commit}:src"),
            "construction": "git archive of parser commit with identical copied candidate harness, inputs, and Criterion dev dependency metadata",
        },
        "candidate": {
            "parser_commit": args.candidate_commit,
            "parser_src_tree": command("git", "rev-parse", f"{args.candidate_commit}:src"),
            "report_generator_commit": head,
            "measured_code_and_benchmark_inputs_match_generator_head": True,
        },
        "shared": {
            "syntax_pin": syntax_pin.group(1),
            "harness_sha256": digest(ROOT / "benches" / "parser_roundtrip.rs"),
            "input_sha256": inputs,
            "criterion_version": "0.5.1",
            "release_profile": "Cargo [profile.release], lto=thin, codegen-units=1",
            "criterion_config": {"samples": 20, "warmup_ms": 500, "measurement_ms": 1000},
            "threshold": "median regression >20% with 95% change CI entirely above 20% requires a repeated run",
            "comparison": "two consecutive paired baseline/candidate runs in one local session and shared target directory",
            "measurement_host": {
                "machine": platform.machine(),
                "system": platform.platform(),
                "kernel": optional_command("uname", "-a"),
                "cpu": optional_command("sysctl", "-n", "machdep.cpu.brand_string"),
                "logical_cpus": optional_command("sysctl", "-n", "hw.logicalcpu"),
                "memory_bytes": optional_command("sysctl", "-n", "hw.memsize"),
                "hardware_model": hardware_model(),
                "rustc": command("rustc", "-Vv"),
                "cargo": command("cargo", "-V"),
            },
        },
        "cases": cases,
        "conclusion": conclusion,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
