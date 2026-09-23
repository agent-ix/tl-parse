"""Fault controls for V4 report credit and checked seed admission."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import run_v4_campaign as campaign


class V4CampaignTests(unittest.TestCase):
    def test_exact_engine_count_is_required_for_clean_credit(self) -> None:
        self.assertEqual(
            campaign.classify(0, b"#100 DONE cov: 42\n", 100, []),
            ("bounded_no_crash", 100, "run_budget"),
        )
        for raw in (b"", b"#99 DONE cov: 42\n", b"#100 NEW cov: 42\n"):
            self.assertEqual(campaign.classify(0, raw, 100, [])[0], "incomplete")
        self.assertEqual(campaign.classify(1, b"#100 DONE\n", 100, [])[0],
                         "engine_failed_or_unconfirmed_crash")

    def test_crash_artifact_cannot_be_reported_as_clean(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "crash-seed"
            artifact.write_bytes(b"bad input")
            self.assertEqual(
                campaign.classify(0, b"#100 DONE\n", 100, [artifact])[0],
                "crash_requires_minimization_replay",
            )

    def test_checked_corpus_has_real_seed_inputs(self) -> None:
        package = campaign.tomllib.loads((campaign.ROOT / "Cargo.toml").read_text())[
            "package"]["name"]
        self.assertGreaterEqual(len(campaign.corpus_manifest(campaign.TARGETS[package])), 3)


if __name__ == "__main__":
    unittest.main()
