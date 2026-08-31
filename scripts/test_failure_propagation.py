#!/usr/bin/env python3
"""Mutation tests for the local-CI failure-propagation policy."""

from __future__ import annotations

import importlib.util
import os
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
            "\tpython3 scripts/check_checksum_manifest.py fuzz/corpus/parser",
            "\tpython3 scripts/check_checksum_manifest.py fuzz/corpus/parser || :",
            1,
        ),
        original + "\n.IGNORE:\n",
        original + "\n.SILENT:\n",
        original + "\nMAKEFLAGS += -i\n",
        original.replace("ci: ", "ci: fabricated ", 1),
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
                "\tpython3 scripts/check_checksum_manifest.py fuzz/corpus/parser",
                "\tpython3 scripts/check_checksum_manifest.py fuzz/corpus/parser || :",
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
    print("failure-propagation policy behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
