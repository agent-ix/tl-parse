#!/usr/bin/env python3
"""Mutation tests for the local-CI failure-propagation policy."""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "check_failure_propagation", ROOT / "scripts" / "check_failure_propagation.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy assertions", file=sys.stderr)
        return 2
    original = (ROOT / "Makefile").read_text(encoding="utf-8")
    assert MODULE.inspect_makefile(ROOT / "Makefile") == []
    mutations = [
        original.replace("\tcargo clippy", "\t-cargo clippy", 1),
        original.replace("\tcargo test", "\tcargo test || true", 1),
        original.replace(
            "\t/usr/bin/python3 scripts/check_checksum_manifest.py fuzz/corpus/parser",
            "\t/usr/bin/python3 scripts/check_checksum_manifest.py fuzz/corpus/parser || :",
            1,
        ),
        original + "\n.IGNORE:\n",
        original + "\n.SILENT:\n",
        original + "\nMAKEFLAGS += -i\n",
        original + "\nexport MAKEFLAGS = -i\n",
        original + "\noverride MAKEFLAGS = -i\n",
        original + "\nunexport MAKEFLAGS = -i\n",
        original.replace("\tcargo test --all-targets --all-features", "\tcargo test --all-targets --all-features &", 1),
        original.replace(
            "\t/usr/bin/python3 scripts/run_local_ci.py --include-verify",
            "\t/usr/bin/python3 scripts/run_local_ci.py",
            1,
        ),
        original.replace(
            "\t/usr/bin/python3 scripts/run_local_ci.py\n",
            "\t/usr/bin/python3 scripts/run_local_ci.py --include-verify\n",
            1,
        ),
    ]
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for index, text in enumerate(mutations):
            makefile = root / f"Makefile.{index}"
            makefile.write_text(text, encoding="utf-8")
            assert MODULE.inspect_makefile(makefile), f"mutation {index} escaped inspection"
            actual = subprocess.run(
                [sys.executable, str(ROOT / "scripts" / "check_failure_propagation.py"),
                 "--makefile", str(makefile), "--static-only"],
                cwd=ROOT, check=False, capture_output=True,
            )
            assert actual.returncode != 0, f"checker exit contract accepted mutation {index}"

        ignored = root / "ignored"
        (ignored / "tests").mkdir(parents=True)
        (ignored / "tests" / "disabled.rs").write_text(
            "#[test]\n#[cfg_attr(all(), ignore)]\nfn disabled() {}\n", encoding="utf-8"
        )
        assert MODULE.inspect_ignored_tests(ignored), "cfg_attr(ignore) escaped inspection"
        (ignored / "tests" / "disabled.rs").write_text(
            '#[serde(rename = "ignore")]\nstruct Wire;\n', encoding="utf-8"
        )
        assert MODULE.inspect_ignored_tests(ignored) == [], "serde rename caused a false positive"
        helper = ignored / "helpers"
        helper.mkdir()
        (helper / "hidden.rs").write_text("#[test]\n#[ignore]\nfn hidden() {}\n", encoding="utf-8")
        assert MODULE.inspect_ignored_tests(ignored), "ignored included helper escaped inspection"

        hidden = root / "hidden.mk"
        hidden.write_text(
            original.replace("cargo +1.75.0 check --all-targets --all-features", "$(MSRV_CHECK)", 1)
            + "\nMSRV_CHECK = cargo +1.75.0 check --all-targets --all-features || true\n",
            encoding="utf-8",
        )
        assert MODULE.inspect_expanded_recipes(hidden, ROOT), "expanded shell control escaped"

        swallowed = root / "swallowed.mk"
        swallowed.write_text(
            original.replace(
                "\t/usr/bin/python3 scripts/check_checksum_manifest.py fuzz/corpus/parser",
                "\t/usr/bin/python3 scripts/check_checksum_manifest.py fuzz/corpus/parser || :",
                1,
            ),
            encoding="utf-8",
        )
        assert MODULE.probe_command_positions(swallowed), "non-first command swallow escaped"

    clean_env = dict(os.environ)
    clean_env.pop("MAKEFLAGS", None)
    for arguments in (
        ["-i", "ci"], ["-t", "ci"], ["--eval=.IGNORE:", "ci"],
        ["ci", "CARGO=true"], ["ci", "PYTHON=true"],
    ):
        result = subprocess.run(
            ["make", *arguments], cwd=ROOT, check=False,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=clean_env,
        )
        assert result.returncode != 0, f"make {' '.join(arguments)} produced false success"
    assert MODULE.probe_command_positions(ROOT / "Makefile") == []
    profile_name, lock_profile, locked_tools = MODULE.tool_identity.load_lock()
    unavailable, mismatches = MODULE.tool_identity.verify_live(
        profile_name, lock_profile, locked_tools
    )
    assert unavailable == [] and mismatches == []
    qualified_environment = MODULE.tool_identity.qualified_environment(
        profile_name, lock_profile, locked_tools
    )
    assert qualified_environment["CARGO_TARGET_DIR"] == "/home/peter/.cargo-target"
    for name, identity in locked_tools.items():
        for option, field in (("--tool-path", "path"), ("--tool-sha256", "sha256")):
            result = subprocess.run(
                ["python3", "scripts/tool_identity.py", option, name],
                cwd=ROOT, check=False, capture_output=True, text=True,
                env=qualified_environment,
            )
            assert result.returncode == 0 and result.stdout.strip() == identity[field], (
                f"clean-environment tool identity lookup failed for {name} {field}"
            )
    forged_tools = {name: dict(identity) for name, identity in locked_tools.items()}
    forged_tools["cargo"]["sha256"] = "0" * 64
    assert MODULE.tool_identity.verify_live(profile_name, lock_profile, forged_tools)[1], (
        "a mismatched mandatory-tool digest escaped qualification"
    )
    with tempfile.TemporaryDirectory() as directory:
        fixture = Path(directory)
        lock = json.loads((ROOT / "tools.lock").read_text(encoding="utf-8"))
        alias_profile = json.loads(json.dumps(lock["profiles"][profile_name]))
        alias = fixture / "cargo"
        shutil.copyfile(locked_tools["cargo"]["path"], alias)
        alias.chmod(0o755)
        alias_profile["tools"]["cargo"]["path"] = str(alias)
        lock["profiles"]["byte-identical-alias-v1"] = alias_profile
        lock_path = fixture / "tools.lock"
        lock_path.write_text(json.dumps(lock), encoding="utf-8")
        selected, selected_profile, selected_tools = MODULE.tool_identity.load_lock(
            lock_path, "byte-identical-alias-v1"
        )
        assert selected == "byte-identical-alias-v1"
        assert MODULE.tool_identity.verify_live(
            selected, selected_profile, selected_tools
        ) == ([], []), "a declared byte-identical path alias was rejected"
        alias.write_bytes(b"different executable bytes\n")
        assert MODULE.tool_identity.verify_live(
            selected, selected_profile, selected_tools
        )[1], "different bytes at a declared alias did not fail closed"
        try:
            MODULE.tool_identity.load_lock(lock_path, "undeclared-alias-v1")
        except ValueError:
            pass
        else:
            raise AssertionError("an undeclared qualification profile was selected")
    with tempfile.TemporaryDirectory() as directory:
        shim = Path(directory)
        for name in ("cargo", "python3"):
            path = shim / name
            path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            path.chmod(0o755)
        hostile = dict(clean_env)
        hostile["HOME"] = directory
        hostile["PATH"] = f"{directory}:{hostile['PATH']}"
        result = subprocess.run(
            ["/usr/bin/make", "ci"], cwd=ROOT, check=False,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=hostile,
        )
        assert result.returncode != 0, "HOME/PATH-shadowed tools bypassed local CI"
    print("failure-propagation policy behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
