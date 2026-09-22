#!/usr/bin/env bash
# Reinstated after agent-ix/tl-parse#11: a `-` prefix, `.IGNORE`, or a
# `SHELL := /usr/bin/true` assignment can make `make ci` (or any pure gate
# such as fmt-check, lint, deny, audit-unsafe, rustdoc) exit 0 having executed
# nothing. Quoin's digest binding only catches this for producers whose output
# it retains (the `assurance-inputs` chain); it does not cover a pure gate with
# no retained output. This script is the cheap, collector-free replacement:
# a static scan of the Makefile's own text for the constructs that neuter
# Make's execution controls. It is invoked at PARSE time from the top of the
# Makefile (via $(shell ...) + $(error ...)), before any target runs, so it
# reads the whole file from disk regardless of where in the file a dangerous
# construct is appended.
#
# Not a collector: it retains no evidence, seals nothing, and asserts nothing
# beyond "does this text appear in this Makefile".
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "usage: check_make_execution_control.sh <Makefile> [<Makefile>...]" >&2
  echo "       check_make_execution_control.sh --self-test" >&2
  exit 2
fi

# --self-test reproduces each bypass an independent review found (and each
# control that proves the fix isn't just refusing everything), plus a check
# on the real repository Makefile. It is not part of the parse-time guard
# path Make invokes; it's this script's own regression test, run the same way
# `check_unsafe_comments.sh --update-baseline` is a second mode of that
# script rather than a separate file. `make test-execution-control-guard`
# (and `ci`) run it.
if [[ "${1:-}" == "--self-test" ]]; then
  self_test_dir="$(cd "$(dirname "$0")/.." && pwd)"
  self_test_script="$self_test_dir/scripts/check_make_execution_control.sh"
  self_test_failures=0

  self_test_report() {
    local description="$1" ok="$2"
    if [[ "$ok" == "true" ]]; then
      echo "ok   - $description"
    else
      echo "FAIL - $description"
      self_test_failures=1
    fi
  }

  # Runs the script itself against a fixture Makefile written to a scratch
  # directory, and checks both the exit status and (when given) that stderr
  # names the expected finding.
  expect_scan() {
    local description="$1" fixture="$2" want_violation="$3" want_message="${4:-}"
    local scratch makefile status stderr_output
    scratch=$(mktemp -d)
    makefile="$scratch/Makefile"
    printf '%s' "$fixture" > "$makefile"
    stderr_output=$(bash "$self_test_script" "$makefile" 2>&1 1>/dev/null) && status=0 || status=$?
    rm -rf "$scratch"
    local ok=true
    if [[ "$want_violation" == "true" && "$status" -eq 0 ]]; then
      ok=false
    fi
    if [[ "$want_violation" == "false" && "$status" -ne 0 ]]; then
      ok=false
    fi
    if [[ -n "$want_message" && "$stderr_output" != *"$want_message"* ]]; then
      ok=false
    fi
    self_test_report "$description" "$ok"
  }

  # Runs the real `make` binary against this repository's actual Makefile
  # with the given extra flags, for the MAKEFLAGS-shape cases the script
  # itself never sees (those are the Makefile's own parse-time $(MAKEFLAGS)
  # check, not this script). `help` has no prerequisites, so a run that gets
  # past the guard exits immediately without building anything. MAKEFLAGS is
  # stripped from the child's environment first: Make exports it to every
  # child process, so running this from inside `make test`/`make ci` would
  # otherwise leak the outer invocation's own flags (and jobserver file
  # descriptors) into the nested `make` under test.
  expect_make() {
    local description="$1" want_violation="$2" want_message="$3"
    shift 3
    local status stderr_output
    stderr_output=$(env -u MAKEFLAGS -u MFLAGS make -C "$self_test_dir" "$@" help 2>&1 1>/dev/null) \
      && status=0 || status=$?
    local ok=true
    if [[ "$want_violation" == "true" && "$status" -eq 0 ]]; then
      ok=false
    fi
    if [[ "$want_violation" == "false" && "$status" -ne 0 ]]; then
      ok=false
    fi
    if [[ -n "$want_message" && "$stderr_output" != *"$want_message"* ]]; then
      ok=false
    fi
    self_test_report "$description" "$ok"
  }

  # FND-001 (high, fixed in the Makefile): a bundled -i (GNU Make merges
  # single-letter flags into one no-dash word) used to bypass the exact-word
  # MAKEFLAGS check.
  expect_make "bundled -ik is still caught" true "ignore-errors" -ik
  # FND-001 (high): once a -j jobserver is active, GNU Make reports -i as its
  # own standalone dashed word instead of bundling it; that shape also
  # bypassed the exact-word check.
  expect_make "jobserver-separated -j -i is still caught" true "ignore-errors" -j -i
  # Controls for FND-001: flags that never include -i must not be blocked.
  expect_make "-k alone is not blocked" false "" -k
  expect_make "-j4 alone is not blocked" false "" -j4
  expect_make "no flags is not blocked" false ""

  # FND-002 (high): `|| true` / `|| :` used to require a trailing space or
  # end-of-line, missing the common `cmd || true; more-stuff` shape.
  expect_scan "|| true without trailing boundary is caught" \
    $'all:\n\tcmd || true; echo continuing\n' true "swallows failure"
  expect_scan "|| : without trailing boundary is caught" \
    $'all:\n\tcmd || :; echo continuing\n' true "swallows failure"
  # Control for FND-002: a lookalike word must not false-positive.
  expect_scan "|| truest lookalike is not flagged" \
    $'all:\n\tcmd || truest\n' false

  # FND-003 (medium): the '-' recipe-prefix detector only matched '-' as the
  # first prefix character; GNU Make accepts @/+/- in any order.
  expect_scan "@-command (dash not first) is caught" \
    $'all:\n\t@-command\n' true "error-suppressing prefix"
  expect_scan "+-command (dash not first) is caught" \
    $'all:\n\t+-command\n' true "error-suppressing prefix"
  # Control for FND-003: prefix characters without a dash must not false-positive.
  expect_scan "@command (no dash) is not flagged" \
    $'all:\n\t@command\n' false

  # FND-004 (low): the scanner used to be comment-blind, so prose that merely
  # names a dangerous construct as an example could self-trigger the guard.
  expect_scan "a comment naming SHELL := as an example is not flagged" \
    $'# example: SHELL := /usr/bin/true is dangerous\nall:\n\techo hi\n' false
  # Control for FND-004: a real assignment must still be caught, including
  # alongside a comment naming the same construct.
  expect_scan "a real SHELL assignment is still caught" \
    $'# example: SHELL := /usr/bin/true is dangerous\nall:\n\techo hi\nSHELL := /usr/bin/true\n' \
    true "assigns SHELL"

  # Safety net: the shipped Makefile itself must stay clean.
  expect_scan_repo_makefile() {
    local status
    bash "$self_test_script" "$self_test_dir/Makefile" >/dev/null 2>&1 && status=0 || status=$?
    self_test_report "the real repository Makefile is clean" "$([[ "$status" -eq 0 ]] && echo true || echo false)"
  }
  expect_scan_repo_makefile

  if [[ "$self_test_failures" -eq 0 ]]; then
    echo "check_make_execution_control.sh --self-test: all cases passed"
  fi
  exit "$self_test_failures"
fi

violations=0
scratch=""
cleanup() {
  # Under `set -e`, a failing last command inside an EXIT trap overrides the
  # script's actual exit code (even one set by an explicit `exit N` before the
  # trap ran) — `[[ -n "$scratch" ]]` returns 1 once scratch is unset, which
  # would silently turn a real violation (exit 1) into exit 0, or a clean run
  # (exit 0) into exit 1. Always end the trap with an explicit success so it
  # never has a say in the reported exit status.
  if [[ -n "$scratch" ]]; then
    rm -f "$scratch"
  fi
  return 0
}
trap cleanup EXIT

# A recipe line (one starting with a tab, the default .RECIPEPREFIX) is left
# untouched: '#' there is shell text, not a Make comment, and could itself be
# the dangerous construct. Everywhere else, Make comments start at the first
# un-escaped '#' and everything after it is not live Makefile syntax — this
# repository's own header prose names SHELL/.SHELLFLAGS/.IGNORE/etc. as
# examples, and without stripping those comments first, editing that prose
# (e.g. removing a backtick) can self-trigger the guard on its own
# explanation of itself rather than on anything the Makefile actually does
# (FND-004). Line count is preserved 1:1 so reported line numbers still
# point at the original file.
strip_non_recipe_comments() {
  awk '
    /^\t/ { print; next }
    {
      line = $0
      out = ""
      i = 1
      n = length(line)
      while (i <= n) {
        c = substr(line, i, 1)
        if (c == "\\" && i < n) {
          out = out c substr(line, i + 1, 1)
          i += 2
          continue
        }
        if (c == "#") break
        out = out c
        i++
      }
      print out
    }
  ' "$1"
}

check_pattern() {
  local description="$1" pattern="$2" display_file="$3" scan_file="$4"
  local hits
  hits=$(grep -nE "$pattern" "$scan_file" || true)
  if [[ -n "$hits" ]]; then
    while IFS= read -r hit; do
      echo "execution-control guard: ${display_file}:${hit%%:*}: ${description}" >&2
    done <<< "$hits"
    violations=1
  fi
}

for file in "$@"; do
  [[ -f "$file" ]] || continue

  scratch=$(mktemp)
  strip_non_recipe_comments "$file" > "$scratch"

  # SHELL / .SHELLFLAGS / MAKE / MAKEFLAGS assignment, global or target-scoped
  # (`target: VAR = val`, multi-target, pattern-rule, or `define` forms), with
  # any of the seven assignment operators (=, :=, ::=, +=, ?=, !=, and the
  # bare `define ... endef` block form).
  check_pattern \
    "assigns SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS" \
    '(^|[:%[:space:]])(SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)[[:space:]]*(:=|::=|\+=|\?=|!=|=)' \
    "$file" "$scratch"
  check_pattern \
    "defines SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS via define/endef" \
    '^[[:space:]]*define[[:space:]]+(SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)([[:space:]]|$)' \
    "$file" "$scratch"

  # export / override / unexport / private of the same four variables.
  check_pattern \
    "export/override/unexport/private of SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS" \
    '^[[:space:]]*(export|override|unexport|private)[[:space:]]+([[:alnum:]_,[:space:]]*[[:space:]])?(SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)([[:space:]]|$|=)' \
    "$file" "$scratch"

  # $(eval ...) can synthesize any of the above at parse time.
  check_pattern \
    'uses $(eval ...)' \
    '\$\(eval[[:space:]]' \
    "$file" "$scratch"

  # include / -include / sinclude can pull in a file that does any of the above.
  check_pattern \
    "uses include/-include/sinclude" \
    '^[[:space:]]*(-include|sinclude|include)[[:space:]]' \
    "$file" "$scratch"

  # Special targets that make every recipe (or output) report success without
  # having run, or that hide the fact that it did not.
  check_pattern \
    "declares .IGNORE/.SILENT/.ONESHELL/.DEFAULT" \
    '^[[:space:]]*\.(IGNORE|SILENT|ONESHELL|DEFAULT)[[:space:]]*:' \
    "$file" "$scratch"

  # The recipe-line prefix characters @, +, - can appear in any order and
  # combination right after the tab (GNU Make strips them one at a time,
  # regardless of order, until it hits a character that isn't one of the
  # three); a `-` anywhere in that leading run suppresses a nonzero exit from
  # the line, not only when it is the very first character (FND-003).
  check_pattern \
    "recipe line uses the '-' error-suppressing prefix" \
    $'^\t[@+-]*-[@+-]*' \
    "$file" "$scratch"

  # `|| true` / `|| :` swallows a nonzero exit inside a recipe line. The
  # match must not require a trailing space or end-of-line: `cmd || true;
  # echo continuing` is a common way to write this and has neither (FND-002).
  # Instead require that what follows isn't more identifier text, so this
  # doesn't fire on an unrelated token like `|| truest`.
  check_pattern \
    "recipe line swallows failure with '|| true' or '|| :'" \
    '\|\|[[:space:]]*(true|:)([^[:alnum:]_]|$)' \
    "$file" "$scratch"

  rm -f "$scratch"
  scratch=""
done

exit "$violations"
