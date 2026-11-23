import json, os, sys
out = sys.argv[1]
os.makedirs(out, exist_ok=True)
cases = []
def add(cid, cls, src, reason, canonical=None, code=None, locus=None, occurrence=0, span=None,
        catalog="standard", profile="closed", limits=None, fixed_point=False, raw=None):
    path = cid + (".bin" if raw is not None else ".txt")
    data = raw if raw is not None else src.encode("utf-8")
    open(os.path.join(out, path), "wb").write(data)
    case = {"id": cid, "class": cls, "path": path, "catalog": catalog, "profile": profile}
    if limits: case["limits"] = limits
    if canonical is not None: case["expectedCanonical"] = canonical
    if fixed_point: case["expectFixedPoint"] = True
    if code is not None:
        case["expectedCode"] = code
        if span is None:
            b = src.encode("utf-8"); l = locus.encode("utf-8"); i = -1
            for _ in range(occurrence + 1): i = b.index(l, i + 1)
            span = [i, i + len(l)]
        case["expectedSpan"] = span
    case["reason"] = reason
    cases.append(case)

for op in ["<", "<=", ">", ">=", "==", "!="]:
    name = {"<":"lt","<=":"le",">":"gt",">=":"ge","==":"eq","!=":"ne"}[op]
    add(f"valid-op-{name}", "valid-operator", f"(speed{op}5)", f"comparison {op}", canonical=f"(speed {op} 5)")
add("valid-node-add", "valid-node-kind", "(speed+1>0)", "add", canonical="(speed + 1 > 0)")
add("valid-node-sub", "valid-node-kind", "(speed - 1 > 0)", "sub", canonical="(speed - 1 > 0)")
add("valid-node-mul", "valid-node-kind", "(speed * 2 > 0)", "mul", canonical="(speed * 2 > 0)")
add("valid-node-neg", "valid-node-kind", "(-speed > 0)", "neg", canonical="(-speed > 0)")
add("valid-node-neg-literal", "valid-node-kind", "(-(5) < speed)", "neg of a literal stays a node", canonical="(-(5) < speed)")
add("valid-node-abs", "valid-node-kind", "(abs(speed) > 0)", "abs", canonical="(abs(speed) > 0)")
add("valid-node-rescale", "valid-node-kind", "(scale(speed,2) > temp)", "explicit widening", canonical="(scale(speed, 2) > temp)")
add("valid-nested-binary", "valid-node-kind", "(speed + 1 + 2 > 0)", "nested binary parenthesized", canonical="((speed + 1) + 2 > 0)")
add("valid-scale-0", "scale-0", "(speed == 5)", "scale 0 literal", canonical="(speed == 5)")
add("valid-scale-1", "scale-1", "(tenth == 1.5)", "scale 1 literal", canonical="(tenth == 1.5)")
add("valid-scale-2", "scale-2", "(temp == 5.00)", "scale 2 keeps trailing zeros", canonical="(temp == 5.00)")
add("valid-scale-9", "scale-9", "(fine == 0.000000001)", "scale 9 literal", canonical="(fine == 0.000000001)")
add("valid-scale-18", "scale-18", "(micro == 0.000000000000000001)", "scale 18 literal", canonical="(micro == 0.000000000000000001)")
add("valid-skeleton", "valid-operator", "G[0,5]((speed > 0) -> F[1,2](temp < 1.00)) & p0", "temporal skeleton as v2",
    canonical="G[0,5]((speed > 0)->F[1,2](temp < 1.00))&p0")
add("valid-until-release", "valid-operator", "(speed > 0) U[0,2] p0 | (speed > 0) R[1,3] p0", "U and R",
    canonical="(speed > 0)U[0,2]p0|(speed > 0)R[1,3]p0")
add("valid-weak-until", "valid-operator", "(speed > 0) W[0,2] p0", "W lowered by tl-syntax; canonical is the expansion", fixed_point=True)
add("valid-strong-release", "valid-operator", "(speed > 0) M[0,2] p0", "M lowered by tl-syntax; canonical is the expansion", fixed_point=True)
add("valid-dedup", "valid-node-kind", "(speed > 0) & (speed>0)", "structurally identical comparisons share one identity",
    canonical="(speed > 0)&(speed > 0)")
add("minus-x-5", "minus-forms", "(speed -5 > 0)", "x -5 is one subtraction", canonical="(speed - 5 > 0)")
add("minus-x-minus-5", "minus-forms", "(speed - -5 > 0)", "x - -5 subtracts the literal -5", canonical="(speed - -5 > 0)")
add("minus-neg-x", "minus-forms", "(-speed < 1)", "-x is neg", canonical="(-speed < 1)")
add("paren-p1", "paren-forms", "(p1)", "(p1) is a parenthesised formula", canonical="p1")
add("paren-x", "paren-forms", "(speed)", "(x) at formula position lacks its comparison", code="missing_token", locus=")")
add("reserved-catalog-unreferenced", "reserved-catalog", "(speed > 0)", "catalog with reserved names serves a source that does not reference them",
    catalog="reserved", canonical="(speed > 0)")
add("reserved-catalog-referenced", "reserved-catalog", "(each > 0)", "reserved word as signal name", catalog="reserved",
    code="reserved_identifier", locus="each")
add("reserved-p17", "reserved-signal-names", "(p17 > 0)", "p<digits> as signal name", catalog="reserved",
    code="reserved_identifier", locus="p17")
add("literal-minus-zero", "literal-forms", "(speed > -0)", "-0 is not canonical", code="non_canonical_number", locus="-0")
add("literal-leading-zero", "literal-forms", "(speed > 007)", "leading zero", code="non_canonical_number", locus="007")
add("literal-trailing-dot", "literal-forms", "(speed > 1.)", "missing fraction digits", code="non_canonical_number", locus="1.")
add("literal-leading-dot", "literal-forms", "(temp > .5)", "missing integer digits", code="unexpected_character", locus=".")
add("literal-exponent", "literal-forms", "(speed > 1e5)", "exponent", code="unsupported_operator", locus="1e5")
add("literal-hex", "literal-forms", "(speed > 0x10)", "hexadecimal", code="unsupported_operator", locus="0x10")
add("literal-19-digit", "literal-forms", "(speed > 9999999999999999999)", "i64 overflow", code="integer_overflow", locus="9999999999999999999")
add("literal-scale-19", "literal-forms", "(temp > 0.1234567890123456789)", "scale 19", code="scale_overflow", locus="0.1234567890123456789")
long = "1" + "0" * 4095
add("literal-4096-digit", "literal-forms", f"(speed > {long})", "4,096-digit literal is bounded", code="integer_overflow", locus=long)
add("unicode-minus", "non-ascii", "(speed > −5)", "U+2212 minus", code="unexpected_character", locus="−")
add("fullwidth-digit", "non-ascii", "(speed > ５)", "fullwidth digit", code="unexpected_character", locus="５")
add("nul-byte", "non-ascii", "(speed > \x005)", "NUL", code="unexpected_character", locus="\x00")
add("invalid-utf8", "invalid-utf8", "", "byte-level case: not UTF-8, refused before the parser", raw=b"(speed > \xff)")
add("identifier-255", "identifier-length", "(" + "a"*255 + " > 0)", "255-byte identifier is a catalog name", canonical="(" + "a"*255 + " > 0)")
add("identifier-256", "identifier-length", "(" + "a"*256 + " > 0)", "256-byte identifier exceeds every catalog name", code="unknown_signal", locus="a"*256)
add("chained-comparison", "chained-comparison", "(speed < 1 < 2)", "non-associative", code="non_associative_comparison", locus="< 1 <")
add("single-equals", "single-equals", "(speed = 5)", "= is not ==", code="unexpected_character", locus="=")
add("division", "division-modulo", "(speed / 2 > 0)", "division", code="unsupported_operator", locus="/")
add("modulo", "division-modulo", "(speed % 2 > 0)", "modulo", code="unsupported_operator", locus="%")
for n in (255, 256, 257):
    src = "(" * n + "p1" + ")" * n
    if n == 255:
        add(f"paren-depth-{n}", "nesting", src, "formula nesting at the depth limit", canonical="p1")
    elif n == 256:
        add(f"paren-depth-{n}", "nesting", src, "formula nesting one past the limit", code="depth_limit", span=[256, 258])
    else:
        add(f"paren-depth-{n}", "nesting", src, "formula nesting two past the limit", code="depth_limit", span=[256, 257])
for n in (253, 254, 255):
    src = "(" + "(" * n + "speed" + ")" * n + " > 0)"
    depth = n + 2
    if depth <= 256:
        add(f"arith-depth-{depth}", "nesting", src, "arithmetic nesting within the limit", canonical="(speed > 0)")
    else:
        add(f"arith-depth-{depth}", "nesting", src, "arithmetic nesting past the limit", code="depth_limit", span=[n + 1, n + 6])
add("token-limit-exact", "token-flood", "!p1&p1&p1&p1", "exactly max_tokens tokens", limits={"maxTokens": 8}, canonical="!p1&p1&p1&p1")
add("token-limit-plus-one", "token-flood", "p1&p1&p1&p1&p1", "max_tokens plus one", limits={"maxTokens": 8}, code="token_limit", span=[12, 13])
add("code-source-limit", "diagnostic-code", "p1&p1", "source limit", limits={"maxSourceBytes": 4}, code="source_limit", span=[4, 5])
add("code-node-limit", "diagnostic-code", "p1&p1&p1", "node limit", limits={"maxNodes": 3}, code="node_limit", locus="p1", occurrence=2)
add("code-work-limit", "diagnostic-code", "p1&p1&p1", "work limit", limits={"maxWork": 5}, code="work_limit", span=[0, 2])
add("code-unexpected-token", "diagnostic-code", "p1 & & p1", "operator where an operand is expected", code="unexpected_token", locus="&", occurrence=1)
add("code-missing-token", "diagnostic-code", "(p1 & p1", "missing )", code="missing_token", span=[8, 8])
add("code-trailing-input", "diagnostic-code", "p1 p1", "trailing input", code="trailing_input", locus="p1", occurrence=1)
add("code-profile-mismatch", "diagnostic-code", "p1", "infinite profile is not a v5 profile", profile="infinite", code="profile_mismatch", span=[0, 0])
add("code-unknown-signal", "diagnostic-code", "(nope > 0)", "unknown signal", code="unknown_signal", locus="nope")
add("code-non-numeric-signal", "diagnostic-code", "(door > 0)", "Boolean signal read as a number", code="non_numeric_signal", locus="door")
add("code-scale-mismatch", "diagnostic-code", "(speed + temp > 0)", "scale 0 plus scale 2", code="scale_mismatch", locus="speed + temp")
add("code-scale-overflow", "diagnostic-code", "(micro * temp > 0)", "scale 18 times scale 2", code="scale_overflow", locus="micro * temp")
add("code-rescale-narrowing", "diagnostic-code", "(scale(temp, 1) > tenth)", "narrowing rescale", code="rescale_narrowing", locus="scale(temp, 1)")
add("code-range-overflow", "diagnostic-code", "(big * big > 0)", "derived range exceeds i64", code="range_overflow", locus="big * big")
add("code-unparenthesized", "diagnostic-code", "speed > 0", "bare comparison", code="unparenthesized_comparison", locus="speed >")
add("code-proposition-overflow", "diagnostic-code", "p4294967295 & (speed > 0)", "no fresh identity above u32::MAX",
    code="proposition_overflow", locus="(speed > 0)")
add("code-integer-overflow-interval", "diagnostic-code", "F[0,4294967296]p1", "interval bound beyond u32", code="integer_overflow", locus="4294967296")
add("unsupported-past", "unsupported-temporal", "O[0,1]p1", "past operator", code="unsupported_operator", locus="O")
add("unsupported-next", "unsupported-temporal", "X p1", "next operator", code="unsupported_operator", locus="X")
add("unsupported-unbounded", "unsupported-temporal", "F[0,)p1", "unbounded interval", code="unsupported_operator", locus="[0,)")
add("unsupported-fairness", "unsupported-temporal", "fair {p1}: p1", "fairness envelope", code="unsupported_operator", locus="{")
add("pending-set", "pending-aggregation", "set S", "set declarations land with TL-424", code="unsupported_operator", locus="set")
add("pending-contract", "pending-aggregation", "contract c assume p1 guarantee p1", "contracts land with TL-425", code="unsupported_operator", locus="contract")
manifest = {"schemaVersion": "tl-parse.numeric-corpus/v1", "revision": "tl-parse-corpus/v1",
            "dialect": "tl-parse.clean-ascii/v5", "cases": cases}
open(os.path.join(out, "manifest.json"), "w").write(json.dumps(manifest, indent=2, ensure_ascii=True) + "\n")
print(len(cases))
