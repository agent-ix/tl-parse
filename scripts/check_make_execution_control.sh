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
  exit 2
fi

violations=0

check_pattern() {
  local description="$1" pattern="$2" file="$3"
  local hits
  hits=$(grep -nE "$pattern" "$file" || true)
  if [[ -n "$hits" ]]; then
    while IFS= read -r hit; do
      echo "execution-control guard: ${file}:${hit%%:*}: ${description}" >&2
    done <<< "$hits"
    violations=1
  fi
}

for file in "$@"; do
  [[ -f "$file" ]] || continue

  # SHELL / .SHELLFLAGS / MAKE / MAKEFLAGS assignment, global or target-scoped
  # (`target: VAR = val`, multi-target, pattern-rule, or `define` forms), with
  # any of the seven assignment operators (=, :=, ::=, +=, ?=, !=, and the
  # bare `define ... endef` block form).
  check_pattern \
    "assigns SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS" \
    '(^|[:%[:space:]])(SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)[[:space:]]*(:=|::=|\+=|\?=|!=|=)' \
    "$file"
  check_pattern \
    "defines SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS via define/endef" \
    '^[[:space:]]*define[[:space:]]+(SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)([[:space:]]|$)' \
    "$file"

  # export / override / unexport / private of the same four variables.
  check_pattern \
    "export/override/unexport/private of SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS" \
    '^[[:space:]]*(export|override|unexport|private)[[:space:]]+([[:alnum:]_,[:space:]]*[[:space:]])?(SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)([[:space:]]|$|=)' \
    "$file"

  # $(eval ...) can synthesize any of the above at parse time.
  check_pattern \
    'uses $(eval ...)' \
    '\$\(eval[[:space:]]' \
    "$file"

  # include / -include / sinclude can pull in a file that does any of the above.
  check_pattern \
    "uses include/-include/sinclude" \
    '^[[:space:]]*(-include|sinclude|include)[[:space:]]' \
    "$file"

  # Special targets that make every recipe (or output) report success without
  # having run, or that hide the fact that it did not.
  check_pattern \
    "declares .IGNORE/.SILENT/.ONESHELL/.DEFAULT" \
    '^[[:space:]]*\.(IGNORE|SILENT|ONESHELL|DEFAULT)[[:space:]]*:' \
    "$file"

  # The `-` recipe-line prefix suppresses a nonzero exit from that line.
  check_pattern \
    "recipe line uses the '-' error-suppressing prefix" \
    $'^\t-' \
    "$file"

  # `|| true` (or `|| :`) swallows a nonzero exit inside a recipe line.
  check_pattern \
    "recipe line swallows failure with '|| true' or '|| :'" \
    '\|\|[[:space:]]*(true|:)([[:space:]]|$)' \
    "$file"
done

exit "$violations"
