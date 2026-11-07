# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (C) 2026 Agent-IX
"""tl-parse documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. Values are read from this
repository: the hostile-input corpus manifest, the corpus fixtures, the dialect
revisions in src/lib.rs and the diagnostic codes in src/diagnostic.rs.

The block between the two SHARED markers is identical in every TL repository
(tl-syntax, tl-parse, tl-rewrite, tl-mltl, tl-oracle, quire-mltl). Copy it
verbatim and call tl_stack(here="<crate>") and stack_pipeline(here="<crate>"). The block
between the two SHARED SCENARIO markers is also identical in every TL repository; call
scenario_sequence(here="<crate>").
"""

import json
import re
from pathlib import Path

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

ROOT = Path(__file__).resolve().parents[2]

# One role per kind of thing, fixed across the TL and Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "processing step", "box"),
    ("model", "checked document / view", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ==== SHARED: copy verbatim into sibling figures.py (begin) ===================

TL_CRATES = [
    # (column, row, name, one-line role, dependency note)
    (0, 1, "tl-parse", "text to formula graph"),
    (1, 1, "tl-oracle", "independent reference"),
    (2, 1, "tl-mltl", "evaluates traces"),
    (3, 1, "quire-mltl", "the bridge to Quire"),
    (2, 2, "tl-rewrite", "equivalent rewrites"),
    (3, 2, "quire-observation", "Quire observations"),
]


def tl_stack(here="tl-syntax"):
    """The TL crate family as a dependency graph; `here` is drawn highlighted."""
    f = Figure(
        "tl-stack",
        980,
        356,
        "The TL crates. tl-syntax defines the formula graph, profiles and signal catalog, and "
        "tl-parse, tl-oracle, tl-mltl, tl-rewrite and quire-mltl all depend on it. tl-rewrite "
        "depends on tl-mltl. quire-mltl depends on tl-mltl and quire-observation, and is the "
        "single bridge to Quire. Dashed lines are development-only: tl-mltl and tl-rewrite "
        "test against tl-oracle, and tl-rewrite tests against tl-parse.",
    )
    w, h, step = 192, 56, 262
    rows = {0: 24, 1: 140, 2: 256}
    f.text(3 * step, 244, "QUIRE", "muted", "mono", SMALL)
    f.box(0, rows[0], 980, h, "logic" if here == "tl-syntax" else "ink", "tl-syntax",
          "formula graph · semantic profiles · signal catalog · strict documents")
    where = {}
    for col, row, name, sub in TL_CRATES:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=(name == "quire-observation"))
    for name in ("tl-parse", "tl-oracle", "tl-mltl", "quire-mltl"):
        x, y = where[name]
        f.line([(x + w / 2, rows[0] + h), (x + w / 2, y - 3)], "model")
    f.note(where["tl-parse"][0] + w / 2 + 10, 104, "types", "model")
    mx, my = where["tl-mltl"]
    rx, ry = where["tl-rewrite"]
    f.line([(mx + w / 2, my + h), (rx + w / 2, ry - 3)], "model")
    f.note(mx + w / 2 + 10, my + h + 30, "documents", "model")
    qx, qy = where["quire-mltl"]
    f.line([(mx + w, my + h / 2), (qx - 3, qy + h / 2)], "model")
    f.note(mx + w + 8, my + h / 2 - 8, "results", "model")
    ox, oy = where["quire-observation"]
    f.line([(ox + w / 2, oy - 3), (ox + w / 2, qy + h)], "model")
    f.note(ox + w / 2 + 10, qy + h + 30, "observations", "model")
    # development-only edges: tl-oracle and tl-parse feed tests of tl-mltl and tl-rewrite
    px, py = where["tl-parse"]
    zx, zy = where["tl-oracle"]
    f.line([(zx + w, zy + h / 2), (mx - 3, my + h / 2)], "model", dashed=True)
    f.line([(zx + w / 2, zy + h), (zx + w / 2, ry + 18), (rx - 3, ry + 18)], "model", dashed=True)
    f.line([(px + w / 2, py + h), (px + w / 2, ry + 40), (rx - 3, ry + 40)], "model", dashed=True)
    f.legend(0, 350, [("logic", "this repository", "box"), ("ink", "other crate", "box"),
                      ("model", "depends on", "solid"), ("model", "development only", "dash")])
    return f


PIPELINE_STAGES = [
    ("MLTL text", "your formula", "code", None),
    ("tl-parse", "parse", "logic", "tl-parse"),
    ("tl-rewrite", "rewrite", "logic", "tl-rewrite"),
    ("tl-mltl", "evaluate", "logic", "tl-mltl"),
    ("tl-oracle", "tests only", "logic", "tl-oracle"),
    ("quire-mltl", "map to Quire", "logic", "quire-mltl"),
]


def stack_pipeline(here="tl-syntax"):
    """End-to-end flow: text, parse, rewrite, evaluate with an oracle cross-check, Quire view."""
    f = Figure(
        "pipeline",
        980,
        392,
        "A formula moves from text to a Quire outcome. tl-parse turns MLTL text into a formula "
        "document, tl-rewrite returns a rewritten document, and tl-mltl evaluates it against a "
        "trace. In tests, tl-oracle evaluates the same formula and trace independently and the "
        "verdicts are compared. quire-mltl maps the result to a Boolean Value or a typed "
        "NonValue reason. All documents use the vocabulary of tl-syntax.",
    )
    w, h = 140, 56
    step = (980 - w) / (len(PIPELINE_STAGES) - 1)
    cx = [round(i * step + w / 2) for i in range(len(PIPELINE_STAGES))]
    for i, (name, sub, role, crate) in enumerate(PIPELINE_STAGES):
        if crate is not None:
            role = "logic" if crate == here else "ink"
        f.box(cx[i] - w // 2, 0, w, h, role, name, sub)
        f.line([(cx[i], h + 2), (cx[i], 312)], "faint", dashed=True, arrow=False, weight=1.0)
    msgs = [
        (0, 1, 96, "MLTL text", "code", False),
        (1, 2, 140, "formula document", "model", False),
        (2, 3, 184, "rewritten formula", "model", False),
        (3, 4, 228, "formula + trace", "model", True),
        (4, 3, 262, "reference verdict", "evidence", True),
        (3, 5, 306, "result", "model", False),
    ]
    for a, b, y, label, role, dev in msgs:
        sign = 1 if b > a else -1
        f.line([(cx[a] + 3 * sign, y), (cx[b] - 3 * sign, y)], role, dashed=dev,
               weight=2.4 if not dev else 1.5)
        left = min(cx[a], cx[b]) + 10
        f.note(left, y - 8, label, role)
    f.box(980 - 190, 332, 190, 50, "evidence", "Value or NonValue", "true / false or reason")
    f.line([(cx[5], 312), (cx[5], 329)], "evidence")
    f.box(0, 332, 520, 50, "logic" if here == "tl-syntax" else "ink", "tl-syntax",
          "the formula, profile, signal and document types every step uses")
    return f


# ==== SHARED (end) ============================================================


# ---- repository data ---------------------------------------------------------


# ==== SHARED SCENARIO: copy verbatim into sibling figures.py (begin) ==========

SCENARIO_LANES = [
    # (name, one-word role, step number whose look-here callout sits on this lane)
    ("engineer", "you", None),
    ("tl-parse", "text", 1),
    ("tl-syntax", "signals", 2),
    ("tl-rewrite", "normalize", 3),
    ("tl-mltl", "evaluate", 4),
    ("R2U2", "outside", None),
    ("tl-oracle", "reference", 5),
    ("quire-mltl", "assurance", 6),
]

SCENARIO_LOOK = [
    # (step, crate, what to look at) for the case r2u2-future-deadline-v1
    (1, "tl-parse", "the tree is one F node, interval [0,3], over p2"),
    (2, "tl-syntax", "p2 is bound to signal s2, the signal the requirement waits for"),
    (3, "tl-rewrite", "status Unchanged: no rule applies and no step is recorded"),
    (4, "tl-mltl", "s2 is absent at instants 0 to 3, so false at time 0; R2U2 agrees"),
    (5, "tl-oracle", "a second evaluator reads the same formula and trace and returns false"),
    (6, "quire-mltl", "false becomes Value false only if the observation views allow a value"),
]


def scenario_sequence(here=None):
    """The debugging walkthrough as a sequence diagram; the lane named `here` is highlighted."""
    f = Figure(
        "scenario-sequence",
        980,
        684,
        "A deadline requirement fails on a recorded trace, and the engineer asks why. The engineer "
        "sends F[0,3] p2 to tl-parse and gets a formula document, binds p2 to signal s2 with tl-syntax, "
        "and sends the document to tl-rewrite, which reports Unchanged. tl-mltl evaluates the formula "
        "and trace and returns false at time 0 with a buffer of 4 instants, and R2U2, which runs outside "
        "the crates, returns false at time 0 for the C2PO expression F[0,3](p2). tl-oracle evaluates the "
        "same formula and trace independently and returns false. quire-mltl maps the result to Value false. "
        "Numbered rings mark where to look at each step.",
    )
    w, h = 112, 50
    step = (980 - w) / (len(SCENARIO_LANES) - 1)
    cx = [round(i * step + w / 2) for i in range(len(SCENARIO_LANES))]
    lane = {name: i for i, (name, _, _) in enumerate(SCENARIO_LANES)}
    bottom = 492
    for i, (name, sub, _) in enumerate(SCENARIO_LANES):
        if name == "engineer":
            role = "code"
        elif name == here:
            role = "logic"
        else:
            role = "ink"
        f.box(cx[i] - w // 2, 0, w, h, role, name, sub, dashed=(name == "R2U2"))
        f.line([(cx[i], h + 2), (cx[i], bottom)], "faint", dashed=True, arrow=False, weight=1.0)
    y = 82
    pairs = [
        # (from, to, go label, go role, back label, back role, dashed, ring step)
        ("engineer", "tl-parse", "F[0,3] p2", "code", "formula doc", "model", False, 1),
        ("engineer", "tl-syntax", "signal catalog", "ink", "p2 bound to s2", "model", False, 2),
        ("engineer", "tl-rewrite", "formula doc", "model", "Unchanged", "evidence", False, 3),
        ("engineer", "tl-mltl", "formula + trace", "model", "false at 0, buffer 4", "evidence", False, 4),
        ("tl-mltl", "R2U2", "F[0,3](p2)", "model", "false at 0", "evidence", True, None),
        ("engineer", "tl-oracle", "formula + trace", "model", "false", "evidence", True, 5),
        ("engineer", "quire-mltl", "result", "model", "Value false", "evidence", False, 6),
    ]
    for a, b, go, go_role, back, back_role, dev, ring in pairs:
        ia, ib = lane[a], lane[b]
        f.line([(cx[ia] + 3, y), (cx[ib] - 3, y)], go_role, dashed=dev, weight=1.5 if dev else 2.4)
        f.text(cx[ia] + 12, y - 7, go, go_role, "mono", SMALL)
        yb = y + 26
        f.line([(cx[ib] - 3, yb), (cx[ia] + 3, yb)], back_role, dashed=dev, weight=1.5 if dev else 2.4)
        f.text(cx[ia] + 26, yb - 7, back, back_role, "mono", SMALL)
        if ring:
            rx, ry = cx[ib] + 22, y + 13
            f.dot(rx, ry, 9, "evidence", filled=False)
            f.text(rx, ry + 4, str(ring), "evidence", "mono", SMALL, "middle")
        y += 62
    f.text(0, bottom + 32, "Look here", "muted", "title", TITLE)
    ly = bottom + 56
    for n, crate, what in SCENARIO_LOOK:
        f.dot(9, ly - 4, 9, "evidence", filled=False)
        f.text(9, ly, str(n), "evidence", "mono", SMALL, "middle")
        f.text(30, ly, crate, "logic" if crate == here else "muted", "mono", SMALL)
        f.text(140, ly, what, "ink", "mono", SMALL)
        ly += 22
    return f


# ==== SHARED SCENARIO (end) ===================================================


def load(path):
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def diagnostic_code_count():
    """Variants of DiagnosticCode in src/diagnostic.rs."""
    text = (ROOT / "src/diagnostic.rs").read_text(encoding="utf-8")
    body = text.split("pub enum DiagnosticCode {", 1)[1].split("\n}\n", 1)[0]
    return len(re.findall(r"^\s+[A-Z]\w+,$", body, re.M))


def dialect_count():
    """Dialect revisions declared in src/lib.rs."""
    text = (ROOT / "src/lib.rs").read_text(encoding="utf-8")
    return len(re.findall(r'pub const DIALECT(?:_V\d)?_REVISION: &str', text))


def round_trip_case():
    """The well-formed fixture of the hostile-input corpus: source, profile, canonical text."""
    manifest = load("corpus/v1/manifest.json")
    case = next(c for c in manifest["cases"] if "expectedCanonical" in c)
    source = (ROOT / "corpus/v1" / case["path"]).read_text(encoding="utf-8").strip()
    return source, case["profile"], case["expectedCanonical"]


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: source text and a profile become a document or a diagnostic, and back to text."""
    source, profile, canonical = round_trip_case()
    codes = diagnostic_code_count()
    f = Figure(
        "hero",
        980,
        368,
        f"The source text {source} and a semantic profile, {profile}, go into parse, which "
        f"returns a FormulaDocument or a Diagnostic with one of {codes} stable codes and a byte "
        f"span. format_document turns the document into canonical text, {canonical}. Parsing "
        f"that text again gives the same graph.",
    )
    f.box(0, 24, 184, 62, "code", "source text", source, "corpus/v1")
    f.box(237, 24, 118, 62, "logic", "parse", "dialect v1", "bounded")
    f.box(408, 24, 146, 62, "model", "FormulaDocument", "tl-syntax graph", "byte spans")
    f.box(607, 24, 146, 62, "logic", "format_document", "canonical form", "bounded")
    f.box(808, 24, 172, 62, "code", "canonical text", canonical, "no whitespace")
    f.line([(184, 55), (234, 55)], "code")
    f.note(210, 47, "text", "code", anchor="middle")
    f.line([(355, 55), (405, 55)], "model")
    f.note(381, 47, "graph", "model", anchor="middle")
    f.line([(554, 55), (604, 55)], "model")
    f.note(580, 47, "graph", "model", anchor="middle")
    f.line([(753, 55), (805, 55)], "code")
    f.note(781, 47, "text", "code", anchor="middle")
    f.box(0, 170, 184, 56, "ink", "semantic profile", profile)
    f.line([(184, 198), (296, 198), (296, 89)], "ink")
    f.note(192, 190, "profile", "ink")
    f.box(214, 270, 180, 62, "evidence", "Diagnostic", f"{codes} stable codes", "byte span, no document")
    f.line([(340, 89), (340, 267)], "evidence")
    f.note(348, 190, "refused", "evidence")
    f.box(607, 170, 146, 56, "evidence", "same graph", "equal nodes, root")
    f.line([(481, 89), (481, 198), (604, 198)], "model")
    f.note(542, 190, "graph", "model", anchor="middle")
    f.box(808, 170, 172, 56, "logic", "parse", "canonical text again")
    f.line([(894, 89), (894, 167)], "code")
    f.note(902, 134, "text", "code")
    f.line([(808, 198), (756, 198)], "model")
    f.note(781, 190, "graph", "model", anchor="middle")
    f.legend(0, 360, ROLES)
    return f


def tl_stack_here():
    return tl_stack(here="tl-parse")


def pipeline_here():
    return stack_pipeline(here="tl-parse")


def scenario_sequence_here():
    return scenario_sequence(here="tl-parse")


FIGURES = [hero, tl_stack_here, pipeline_here, scenario_sequence_here]
