#!/usr/bin/env python3
"""Disposable-worktree behavior tests for the evidence collector boundary."""

from __future__ import annotations

import hashlib
import json
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
    "Makefile",
    "scripts/collect_evidence.sh",
    "scripts/run_cargo_toolchain.py",
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
        fake_rustup = Path(bin_directory) / "rustup"
        fake_rustup.write_text(
            "#!/usr/bin/bash\n"
            "set -eu\n"
            "target_dir=''\n"
            "previous=''\n"
            "for argument in \"$@\"; do\n"
            "  if [[ \"$previous\" == --target-dir ]]; then target_dir=\"$argument\"; fi\n"
            "  previous=\"$argument\"\n"
            "done\n"
            "if [[ -z \"$target_dir\" ]]; then target_dir=\"$PWD/fuzz/target\"; fi\n"
            "/usr/bin/mkdir -p \"$target_dir\"\n"
            "/usr/bin/printf '%s\\n' \"$*\" >\"$target_dir/invocation.txt\"\n",
            encoding="utf-8",
        )
        fake_rustup.chmod(0o755)
        lock_path = checkout / "tools.lock"
        lock = json.loads(lock_path.read_text(encoding="utf-8"))
        identity = lock["profiles"][PROFILE]["tools"]["rustup"]
        identity["path"] = str(fake_rustup)
        identity["sha256"] = hashlib.sha256(fake_rustup.read_bytes()).hexdigest()
        lock_path.write_text(json.dumps(lock, indent=2) + "\n", encoding="utf-8")
        subprocess.run(["/usr/bin/git", "add", "tools.lock"], cwd=checkout, check=True)
        subprocess.run(
            [
                "/usr/bin/git", "-c", "user.name=Collector Policy Test",
                "-c", "user.email=policy@example.invalid", "commit", "-qm",
                "pin fake rustup for target-placement probe",
            ],
            cwd=checkout,
            check=True,
        )
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
        target_dir = Path(target) / "fuzz"
        invocation = (target_dir / "invocation.txt").read_text(encoding="utf-8")
        assert invocation.strip() == (
            f"run nightly cargo fuzz build parser --target-dir {target_dir}"
        )
        assert not (checkout / "target").exists()
        assert not (checkout / "fuzz" / "target").exists()
        assert subprocess.run(
            ["/usr/bin/git", "status", "--porcelain", "--untracked-files=all"],
            cwd=checkout,
            check=True,
            capture_output=True,
            text=True,
        ).stdout == "", "cargo-fuzz probe dirtied the candidate worktree"


def assert_collector_option_errors() -> None:
    with disposable_worktree() as checkout:
        cases = (
            (["--tool-profile"], "requires a reviewed profile name"),
            (["--not-a-collector-option"], "unknown collector option"),
            (["evidence/one", "evidence/two"], "only one evidence destination"),
        )
        for arguments, message in cases:
            result = subprocess.run(
                ["/usr/bin/bash", "scripts/collect_evidence.sh", *arguments],
                cwd=checkout,
                check=False,
                capture_output=True,
                text=True,
            )
            assert result.returncode == 2 and message in result.stderr, (
                f"collector option contract failed for {arguments}: {result.stderr}"
            )


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy assertions", file=sys.stderr)
        return 2
    installed_trap = "trap cleanup_staging EXIT"
    assert_fault_cleanup(installed_trap, installed_trap + "\nexit 91", 91)
    retained = '# The candidate cannot already carry an AA-001 record for itself.'
    assert_fault_cleanup(retained, "exit 92\n" + retained, 92)
    assert_fuzz_target_is_external()
    assert_collector_option_errors()
    print("collector clean-environment, cleanup, and target placement behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
