#!/usr/bin/env python3
"""Re-derive the clean-room attribution digests from the source Cargo resolved.

This is a domain provenance gate, not evidence machinery, and it survives the
shared-assurance migration deliberately. Reviewers of PR #6 called
`docs/ATTRIBUTION.md` "the first artifact in this program that lets a reviewer
check a clean-room claim instead of reading an assertion about it" — and what
makes that true is this script, which re-hashes the real files out of the real
resolved checkout instead of comparing the document to itself.

Two tables are checked, because after the tl-syntax repin there are two facts:

  * the AUTHORSHIP BASIS at 740182f1, which is historical and does not move. Its
    digests cannot be re-derived from the resolved source once the crate compiles
    against a different revision, so they are checked for presence and shape and
    are pinned in this file. Rewriting them to match whatever is currently
    resolved is precisely the failure mode this gate exists to prevent.
  * the COMPILED REVISION, which is whatever `Cargo.toml` resolves today. Its
    digests ARE re-derived, from the checkout Cargo actually used.

The compiled revision is read from `src/lib.rs`'s `TL_SYNTAX_REVISION` rather
than hard-coded here, so a repin cannot leave this gate asserting a revision the
crate is not built from. The previous version of this file hard-coded it, which
made the constant and the gate two places to update and one to forget.

Exit status: 0 when both tables hold, 1 when one does not, 2 on a usage or
environment error.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
ROW = re.compile(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|$", re.MULTILINE)
REVISION = re.compile(r'TL_SYNTAX_REVISION: &str = "([0-9a-f]{40})"')

CENSUS = {"src/syntax.rs", "src/document.rs", "LICENSE-MIT", "LICENSE-APACHE"}

# The authorship basis. Historical, and pinned here so it cannot be quietly
# rewritten to whatever the current pin resolves to.
AUTHORSHIP_REVISION = "740182f13b84858008d6f176f75136737d405c1b"
AUTHORSHIP_DIGESTS = {
    "src/syntax.rs": "04e6a46e697444df8e6764dd0e5e5227b1271199ffc0e9d24f77720c979eb14e",
    "src/document.rs": "f97005479f1f12511f1fceb2f9a85b94b482170e606c5735758e11aa2e4580f2",
    "LICENSE-MIT": "97ead12ddb151fc37ffb1c623ab42b9814e21629dee252ff23dc7205f1df9f05",
    "LICENSE-APACHE": "62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a",
}


class AttributionError(RuntimeError):
    """The attribution claim could not be checked."""


def compiled_revision() -> str:
    found = REVISION.search((ROOT / "src" / "lib.rs").read_text(encoding="utf-8"))
    if found is None:
        raise AttributionError("src/lib.rs declares no TL_SYNTAX_REVISION")
    return found.group(1)


def resolved_source_root(revision: str) -> Path:
    try:
        metadata = json.loads(
            subprocess.run(
                ["cargo", "metadata", "--format-version", "1", "--all-features"],
                cwd=ROOT,
                check=True,
                capture_output=True,
                text=True,
            ).stdout
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise AttributionError(f"cargo metadata could not be read: {error}") from error
    package = next(
        (item for item in metadata["packages"] if item["name"] == "tl-syntax"), None
    )
    if package is None:
        raise AttributionError("tl-syntax is not in the resolved dependency graph")
    source = package.get("source") or ""
    if not source.endswith(f"#{revision}"):
        raise AttributionError(
            f"the resolved tl-syntax source is {source}, which is not the compiled "
            f"revision {revision} that src/lib.rs declares"
        )
    return Path(package["manifest_path"]).parent


def tables(document: str) -> list[dict[str, str]]:
    """Split the document's digest tables in order of appearance."""
    found: list[dict[str, str]] = []
    current: dict[str, str] = {}
    previous_end = None
    for match in ROW.finditer(document):
        if previous_end is not None and document[previous_end : match.start()].strip():
            # A gap containing prose means a new table started.
            found.append(current)
            current = {}
        current[match.group(1)] = match.group(2)
        previous_end = match.end()
    if current:
        found.append(current)
    return found


def build_report() -> dict[str, Any]:
    revision = compiled_revision()
    document = (ROOT / "docs" / "ATTRIBUTION.md").read_text(encoding="utf-8")
    found = tables(document)
    if len(found) != 2:
        raise AttributionError(
            f"docs/ATTRIBUTION.md carries {len(found)} digest table(s); the authorship "
            "basis and the compiled revision are two different facts and both must be "
            "recorded"
        )
    authorship, compiled = found

    problems: list[str] = []

    if set(authorship) != CENSUS:
        problems.append(f"authorship table census is {sorted(authorship)}")
    for name, digest in AUTHORSHIP_DIGESTS.items():
        if authorship.get(name) != digest:
            problems.append(
                f"authorship digest for {name} is {authorship.get(name)}, "
                f"which is not the recorded {digest}"
            )
    if AUTHORSHIP_REVISION not in document:
        problems.append(f"the authorship revision {AUTHORSHIP_REVISION} is not named")
    if revision not in document:
        problems.append(f"the compiled revision {revision} is not named")

    if set(compiled) != CENSUS:
        problems.append(f"compiled table census is {sorted(compiled)}")

    # The compiled table is the one that gets re-derived, from the checkout
    # Cargo actually resolved rather than from anything this repository stores.
    source_root = resolved_source_root(revision)
    rederived: dict[str, str] = {}
    for name in sorted(CENSUS):
        path = source_root / name
        if not path.is_file():
            problems.append(f"{name} is absent from the resolved tl-syntax checkout")
            continue
        observed = hashlib.sha256(path.read_bytes()).hexdigest()
        rederived[name] = observed
        if compiled.get(name) != observed:
            problems.append(
                f"compiled digest for {name} is {compiled.get(name)}, "
                f"but the resolved source hashes to {observed}"
            )

    return {
        "schemaVersion": "tl-parse.attribution-report/v1",
        "authorship_revision": AUTHORSHIP_REVISION,
        "compiled_revision": revision,
        "resolved_source_root": str(source_root),
        "rederived_digests": rederived,
        "problems": problems,
        "matched": not problems,
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--json", action="store_true")
    arguments = parser.parse_args(argv[1:])
    try:
        report = build_report()
    except AttributionError as error:
        print(str(error), file=sys.stderr)
        return 2
    if arguments.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        for problem in report["problems"]:
            print(problem, file=sys.stderr)
        if report["matched"]:
            print(
                "clean-room attribution holds: authorship basis "
                f"{report['authorship_revision'][:8]} recorded, compiled revision "
                f"{report['compiled_revision'][:8]} re-derived from the resolved source"
            )
    return 0 if report["matched"] else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
