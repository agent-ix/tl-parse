#!/usr/bin/env python3
"""Compare a cargo-mutants outcomes.json against mutation-thresholds.toml.

Usage: mutation_gate.py [--diff] <mutants.out dir>[,<mutants.out dir>...] [thresholds.toml]

Several comma-separated run directories are merged (a mutant seen in more than
one run counts once, last run wins); this is how a full-core run and a sharded
crate run are combined.

With --diff the run is an `--in-diff` PR run: every non-excluded mutant in the
changed code is judged against `diff_min_kill_percent`, and an empty population
passes (a diff that touches no mutable code has nothing to kill).

Kill rate is caught / (caught + missed + timeout); unviable mutants exercise
no test and count on neither side (the same definition as FR-022). Mutants in
files matched by an `exclude` glob of the thresholds file are ignored, which is
how CI-plumbing modules are kept out of every threshold.

Exit status: 0 every group meets its minimum, 1 a group is below its minimum or
the run is incomplete (an unmutated baseline failure, zero viable mutants in a
group), 2 usage or input error. Python 3.11 or newer (tomllib), stdlib only.
The same script is used in tl-syntax, tl-parse, tl-rewrite and tl-mltl.
"""
import fnmatch
import json
import sys
import tomllib


def main(argv):
    args = argv[1:]
    diff_mode = False
    if args and args[0] == "--diff":
        diff_mode = True
        args = args[1:]
    if len(args) not in (1, 2):
        print(__doc__, file=sys.stderr)
        return 2
    out_dir = args[0]
    thresholds_path = args[1] if len(args) == 2 else "mutation-thresholds.toml"
    try:
        outcomes = {"outcomes": []}
        for directory in out_dir.split(","):
            with open(f"{directory}/outcomes.json", "rb") as handle:
                outcomes["outcomes"].extend(json.load(handle)["outcomes"])
        with open(thresholds_path, "rb") as handle:
            thresholds = tomllib.load(handle)
    except (OSError, ValueError) as error:
        print(f"mutation_gate: {error}", file=sys.stderr)
        return 2

    excludes = thresholds.get("exclude", [])
    groups = thresholds["group"]
    if diff_mode:
        groups = [
            {
                "name": "diff",
                "globs": ["*"],
                "min_kill_percent": thresholds["diff_min_kill_percent"],
                "allow_empty": True,
            }
        ]
    counts = {g["name"]: {"caught": 0, "missed": 0, "timeout": 0, "unviable": 0} for g in groups}
    baseline_ok = True
    merged = {}
    for outcome in outcomes["outcomes"]:
        scenario = outcome["scenario"]
        if scenario == "Baseline":
            baseline_ok = baseline_ok and outcome["summary"] == "Success"
            continue
        mutant = scenario["Mutant"]
        merged[(mutant["file"], mutant["name"])] = outcome
    for outcome in merged.values():
        scenario = outcome["scenario"]
        summary = outcome["summary"]
        path = scenario["Mutant"]["file"]
        if any(fnmatch.fnmatch(path, pattern) for pattern in excludes):
            continue
        key = {
            "CaughtMutant": "caught",
            "MissedMutant": "missed",
            "Timeout": "timeout",
            "Unviable": "unviable",
        }.get(summary)
        if key is None:
            print(f"mutation_gate: unexpected mutant summary {summary!r} for {path}", file=sys.stderr)
            return 1
        for group in groups:
            if any(fnmatch.fnmatch(path, pattern) for pattern in group["globs"]):
                counts[group["name"]][key] += 1

    failed = not baseline_ok
    if not baseline_ok:
        print("FAIL baseline: the unmutated tree did not pass its tests")
    for group in groups:
        c = counts[group["name"]]
        viable = c["caught"] + c["missed"] + c["timeout"]
        minimum = float(group["min_kill_percent"])
        if viable == 0 and group.get("allow_empty"):
            print(f"ok   {group['name']}: no viable mutants in the diff")
            continue
        if viable == 0:
            print(f"FAIL {group['name']}: no viable mutants (empty population)")
            failed = True
            continue
        rate = 100.0 * c["caught"] / viable
        ok = rate >= minimum
        failed |= not ok
        print(
            f"{'ok  ' if ok else 'FAIL'} {group['name']}: kill {rate:.2f}% "
            f"(min {minimum:.2f}%) caught={c['caught']} missed={c['missed']} "
            f"timeout={c['timeout']} unviable={c['unviable']}"
        )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
