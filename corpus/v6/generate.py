import json, os, sys

out = sys.argv[1]
seeds = sys.argv[2]
os.makedirs(out, exist_ok=True)
os.makedirs(seeds, exist_ok=True)
cases = []


def refuse(src, code, locus=None, occurrence=0, span=None):
    if span is None:
        b = src.encode("utf-8")
        l = locus.encode("utf-8")
        i = -1
        for _ in range(occurrence + 1):
            i = b.index(l, i + 1)
        span = [i, i + len(l)]
    return {"code": code, "span": span}


def add(cid, cls, src, reason, ltlf, infinite, limits=None):
    path = cid + ".txt"
    open(os.path.join(out, path), "wb").write(src.encode("utf-8"))
    # Fuzz seed: selector byte (bit 0 = 0 LTLf) then the source.
    open(os.path.join(seeds, cid + ".txt"), "wb").write(b"\x00" + src.encode("utf-8"))
    case = {"id": cid, "class": cls, "path": path}
    if limits:
        case["limits"] = limits
    case["ltlf"] = ltlf
    case["infinite"] = infinite
    case["reason"] = reason
    cases.append(case)


def both(outcome):
    return outcome, outcome


def ok(text):
    return {"canonical": text}


add("empty", "empty", "", "empty source", *both(refuse("", "unexpected_token", span=[0, 0])))
add("whitespace", "whitespace", "   \t\n", "whitespace only", *both(refuse("", "unexpected_token", span=[5, 5])))
add("leading-zero-atom", "leading-zero", "G p01", "leading-zero atom", *both(refuse("G p01", "non_canonical_number", "p01")))
add("u32-overflow-atom", "u32-overflow", "p4294967296", "atom beyond u32", *both(refuse("p4294967296", "integer_overflow", "p4294967296")))
add("non-ascii", "non-ascii", "p0 ∧ p1", "non-ASCII conjunction", *both(refuse("p0 ∧ p1", "unexpected_character", "∧")))
for cid, src, locus in [
    ("interval-future-closed", "F[0,2] p0", "[0,2]"),
    ("interval-globally-open", "G[0,) p0", "[0,)"),
    ("interval-until", "p0 U[1,3] p1", "[1,3]"),
    ("interval-release", "p0 R[0,5] p1", "[0,5]"),
]:
    add(cid, "interval", src, "intervals are spelled in v1 to v4", *both(refuse(src, "interval_not_in_dialect", locus)))
add("bare-next", "next-strength", "X p0", "bare X: refused under LTLf, F[1,1] under infinite",
    refuse("X p0", "next_strength_required", "X"), ok("X p0"))
add("strong-next", "next-strength", "Xs p0", "Xs: LTLf only", ok("Xs p0"), refuse("Xs p0", "next_strength_undefined", "Xs"))
add("weak-next", "next-strength", "Xwp0", "Xw by maximal munch: LTLf only", ok("Xw p0"), refuse("Xwp0", "next_strength_undefined", "Xw"))
add("unbalanced-open", "unbalanced", "(p0 & p1", "missing )", *both(refuse("(p0 & p1", "missing_token", span=[8, 8])))
add("unbalanced-close", "unbalanced", "p0 & p1)", "extra )", *both(refuse("p0 & p1)", "trailing_input", ")")))
add("depth-exact", "depth", "(" * 255 + "p0" + ")" * 255, "nesting at the 256 depth limit", *both(ok("p0")))
add("depth-over", "depth", "(" * 256 + "p0" + ")" * 256, "nesting one past the limit", *both(refuse("", "depth_limit", span=[256, 258])))
add("token-exact", "tokens", "!p0&p1", "exactly max_tokens", *both(ok("!p0 & p1")), limits={"maxTokens": 4})
add("token-over", "tokens", "p0&p1&p2", "max_tokens plus one", *both(refuse("", "token_limit", span=[6, 7])), limits={"maxTokens": 4})
for op in "WM":
    src = f"p0 {op} p1"
    add(f"unsupported-{op}", "unsupported", src, f"derived {op} is outside v6", *both(refuse(src, "unsupported_operator", op)))
for op in "OHY":
    src = f"{op} p0"
    add(f"unsupported-{op}", "unsupported", src, f"past {op} is outside v6", *both(refuse(src, "unsupported_operator", op)))
for op in "ST":
    src = f"p0 {op} p1"
    add(f"unsupported-{op}", "unsupported", src, f"past {op} is outside v6", *both(refuse(src, "unsupported_operator", op)))
add("unknown-prefix", "unknown-prefix", "Xx p0", "Xx is not a keyword", *both(refuse("Xx p0", "unknown_identifier", "Xx")))
add("nul-byte", "nul", "p0\x00", "NUL byte", *both(refuse("p0\x00", "unexpected_character", "\x00")))
add("source-exact", "source", "p0&p1", "exactly max_source_bytes", *both(ok("p0 & p1")), limits={"maxSourceBytes": 5})
add("source-over", "source", "p0&p11", "max_source_bytes plus one", *both(refuse("", "source_limit", span=[5, 6])), limits={"maxSourceBytes": 5})

assert len(cases) == 29, len(cases)
manifest = {"schemaVersion": "tl-parse.corpus/v1", "revision": "tl-parse-corpus/v1", "family": "v6",
            "dialect": "tl-parse.clean-ascii/v6", "cases": cases}
open(os.path.join(out, "manifest.json"), "w").write(json.dumps(manifest, indent=2, ensure_ascii=True) + "\n")
print(len(cases))
