#!/usr/bin/env python3
"""Run local CI gates and require a positive transcript census."""

from __future__ import annotations

import os
import subprocess
import sys

import check_failure_propagation as propagation
import finalize_collection


def main() -> int:
    if sys.argv[1:] not in ([], ["--include-verify"]):
        print("usage: run_local_ci.py [--include-verify]", file=sys.stderr)
        return 2
    include_verify = bool(sys.argv[1:])
    targets = sorted(
        propagation.PROBES if include_verify else propagation.COLLECTION_PROBES
    )
    targets.insert(0, propagation.GUARD_TARGET)
    environment = propagation.clean_environment()
    transcript: list[str] = []
    for target in targets:
        result = subprocess.run(
            ["/usr/bin/make", "--no-print-directory", "MAKEFLAGS=", target],
            cwd=propagation.ROOT, check=False, capture_output=True, text=True,
            env=environment,
        )
        sys.stdout.write(result.stdout)
        sys.stderr.write(result.stderr)
        transcript.extend((result.stdout, result.stderr))
        if result.returncode != 0:
            return result.returncode
    combined = "\n".join(transcript)
    if not finalize_collection.positive_ci_census(combined, require_verify=include_verify):
        print("local CI transcript does not satisfy the positive gate census", file=sys.stderr)
        return 1
    print("local CI transcript satisfies the positive gate census")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
