#!/usr/bin/env python3
"""Disposable-worktree behavior tests for the evidence collector boundary."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
from contextlib import contextmanager
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PROFILE = "peter-linux-x86_64-v1"
OVERLAY = (
    "scripts/collect_evidence.sh",
    "scripts/tool_identity.py",
    "tools.lock",
)


@contextmanager
def disposable_worktree():
    with tempfile.TemporaryDirectory(prefix="tl-parse-collector-") as directory:
        checkout = Path(directory) / "checkout"
        subprocess.run(
            ["/usr/bin/git", "worktree", "add", "--detach", "-q", str(checkout), "HEAD"],
            cwd=ROOT,
            check=True,
        )
        try:
            for relative in OVERLAY:
                destination = checkout / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / relative, destination)
            subprocess.run(["/usr/bin/git", "add", *OVERLAY], cwd=checkout, check=True)
            subprocess.run(
                [
                    "/usr/bin/git",
                    "-c",
                    "user.name=Collector Policy Test",
                    "-c",
                    "user.email=policy@example.invalid",
                    "commit",
                    "--allow-empty",
                    "-qm",
                    "overlay collector policy under test",
                ],
                cwd=checkout,
                check=True,
            )
            yield checkout
        finally:
            subprocess.run(
                ["/usr/bin/git", "worktree", "remove", "--force", str(checkout)],
                cwd=ROOT,
                check=True,
            )


def commit_fault(checkout: Path, marker: str, replacement: str) -> None:
    collector = checkout / "scripts" / "collect_evidence.sh"
    source = collector.read_text(encoding="utf-8")
    assert source.count(marker) == 1, f"collector injection marker drifted: {marker}"
    collector.write_text(source.replace(marker, replacement, 1), encoding="utf-8")
    subprocess.run(["/usr/bin/git", "add", str(collector)], cwd=checkout, check=True)
    subprocess.run(
        [
            "/usr/bin/git",
            "-c",
            "user.name=Collector Policy Test",
            "-c",
            "user.email=policy@example.invalid",
            "commit",
            "-qm",
            "inject collector failure",
        ],
        cwd=checkout,
        check=True,
    )


def assert_fault_cleanup(marker: str, replacement: str, expected: int) -> None:
    with disposable_worktree() as checkout:
        anchors = (checkout / "evidence" / "ANCHORS").read_bytes()
        assurance = (checkout / "spec" / "assurance" / "AA-001.md").read_bytes()
        commit_fault(checkout, marker, replacement)
        destination = "evidence/collector-policy-probe"
        result = subprocess.run(
            [
                "/usr/bin/bash",
                "scripts/collect_evidence.sh",
                "--tool-profile",
                PROFILE,
                destination,
            ],
            cwd=checkout,
            check=False,
            capture_output=True,
            text=True,
        )
        assert result.returncode == expected, (
            f"collector fault returned {result.returncode}, expected {expected}: {result.stderr}"
        )
        assert not list(checkout.glob(".tl-parse-evidence-stage.*")), (
            "failed collection left a staging directory"
        )
        assert not (checkout / destination).exists(), "failed collection published evidence"
        assert (checkout / "evidence" / "ANCHORS").read_bytes() == anchors
        assert (checkout / "spec" / "assurance" / "AA-001.md").read_bytes() == assurance


def assert_fuzz_target_is_external() -> None:
    with disposable_worktree() as checkout, tempfile.TemporaryDirectory(
        prefix="tl-parse-fuzz-bin-"
    ) as bin_directory, tempfile.TemporaryDirectory(prefix="tl-parse-fuzz-target-") as target:
        fake_cargo = Path(bin_directory) / "cargo"
        fake_cargo.write_text(
            "#!/usr/bin/bash\n"
            "set -eu\n"
            "/usr/bin/mkdir -p \"$CARGO_TARGET_DIR/fuzz\"\n"
            "/usr/bin/printf '%s\\n' \"$*\" >\"$CARGO_TARGET_DIR/fuzz/invocation.txt\"\n",
            encoding="utf-8",
        )
        fake_cargo.chmod(0o755)
        environment = {
            "PATH": f"{bin_directory}:/usr/bin",
            "HOME": str(Path(target) / "home"),
            "CARGO_TARGET_DIR": target,
            "LANG": "C.UTF-8",
            "LC_ALL": "C.UTF-8",
            "TL_PARSE_TOOL_PROFILE": PROFILE,
        }
        result = subprocess.run(
            ["/usr/bin/make", "--no-print-directory", "fuzz-build"],
            cwd=checkout,
            env=environment,
            check=False,
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0, result.stderr
        invocation = (Path(target) / "fuzz" / "invocation.txt").read_text(encoding="utf-8")
        assert invocation.strip() == "+nightly fuzz build parser"
        assert not (checkout / "target").exists()
        assert not (checkout / "fuzz" / "target").exists()
        assert subprocess.run(
            ["/usr/bin/git", "status", "--porcelain", "--untracked-files=all"],
            cwd=checkout,
            check=True,
            capture_output=True,
            text=True,
        ).stdout == "", "cargo-fuzz probe dirtied the candidate worktree"


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy assertions", file=sys.stderr)
        return 2
    staging = 'staging_root="$(/usr/bin/mktemp -d -p . .tl-parse-evidence-stage.XXXXXX)"'
    assert_fault_cleanup(staging, "exit 91\n" + staging, 91)
    retained = '# The candidate cannot already carry an AA-001 record for itself.'
    assert_fault_cleanup(retained, "exit 92\n" + retained, 92)
    assert_fuzz_target_is_external()
    print("collector clean-environment, cleanup, and target placement behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
