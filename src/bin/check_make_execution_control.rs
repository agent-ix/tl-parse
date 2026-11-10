//! Parse-time guard for the Makefile (agent-ix/tl-parse#11): a `-` recipe
//! prefix, `.IGNORE`, or a `SHELL := /usr/bin/true` assignment can make
//! `make ci` exit 0 having executed nothing. This is a static scan of the
//! Makefile's own text for the constructs that neuter Make's execution
//! controls. The Makefile invokes it at parse time through `$(shell ...)` and
//! `$(error ...)`, before any target runs, so it re-reads the file from disk
//! and an append anywhere below is still caught.
//!
//! Not a collector: it retains no evidence, seals nothing, and asserts nothing
//! beyond "does this text appear in this Makefile".
//!
//! Usage: `check_make_execution_control MAKEFILE...`; exit 1 on any finding.

use std::{env, fs, process::ExitCode};

type Check = fn(&str) -> bool;

const VARS: [&str; 4] = ["SHELL", ".SHELLFLAGS", "MAKE", "MAKEFLAGS"];

/// Everything outside a recipe line (one starting with a tab) after the first
/// unescaped `#` is a Make comment and not live syntax; a recipe line is left
/// untouched because `#` there is shell text. Line count is preserved.
fn strip_non_recipe_comments(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| {
            if line.starts_with('\t') {
                return line.to_owned();
            }
            let mut out = String::new();
            let mut chars = line.chars();
            while let Some(c) = chars.next() {
                match c {
                    '\\' => {
                        out.push(c);
                        if let Some(next) = chars.next() {
                            out.push(next);
                        }
                    }
                    '#' => break,
                    _ => out.push(c),
                }
            }
            out
        })
        .collect()
}

fn is_word_end(rest: &str) -> bool {
    rest.chars().next().is_none_or(char::is_whitespace)
}

/// `(^|[:%\s])VAR\s*(:=|::=|+=|?=|!=|=)`.
fn assigns(line: &str) -> bool {
    VARS.iter().any(|var| {
        line.match_indices(var).any(|(at, _)| {
            let before_ok = line[..at]
                .chars()
                .next_back()
                .is_none_or(|c| c == ':' || c == '%' || c.is_whitespace());
            let after = line[at + var.len()..].trim_start();
            before_ok
                && [":=", "::=", "+=", "?=", "!=", "="]
                    .iter()
                    .any(|op| after.starts_with(op))
        })
    })
}

/// `^\s*define\s+VAR(\s|$)`.
fn defines(line: &str) -> bool {
    let Some(rest) = line.trim_start().strip_prefix("define") else {
        return false;
    };
    rest.starts_with(char::is_whitespace)
        && VARS
            .iter()
            .any(|v| rest.trim_start().strip_prefix(v).is_some_and(is_word_end))
}

/// `^\s*(export|override|unexport|private)\s+([[:alnum:]_,\s]*\s)?VAR(\s|$|=)`.
fn exports(line: &str) -> bool {
    let trimmed = line.trim_start();
    ["export", "override", "unexport", "private"]
        .iter()
        .filter_map(|k| trimmed.strip_prefix(k))
        .filter(|rest| rest.starts_with(char::is_whitespace))
        .any(|rest| {
            let rest = rest.trim_start();
            VARS.iter().any(|var| {
                rest.match_indices(var).any(|(at, _)| {
                    let head = &rest[..at];
                    let prefix_ok = head
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == ',' || c.is_whitespace())
                        && head.chars().next_back().is_none_or(char::is_whitespace);
                    let tail = &rest[at + var.len()..];
                    prefix_ok && (is_word_end(tail) || tail.starts_with('='))
                })
            })
        })
}

fn includes(line: &str) -> bool {
    let t = line.trim_start();
    ["-include", "sinclude", "include"].iter().any(|k| {
        t.strip_prefix(k)
            .is_some_and(|r| r.starts_with(char::is_whitespace))
    })
}

fn special_target(line: &str) -> bool {
    let t = line.trim_start();
    [".IGNORE", ".SILENT", ".ONESHELL", ".DEFAULT"]
        .iter()
        .any(|k| {
            t.strip_prefix(k)
                .is_some_and(|r| r.trim_start().starts_with(':'))
        })
}

/// A `-` anywhere in the `@`/`+`/`-` run right after the recipe tab.
fn dash_prefix(line: &str) -> bool {
    line.strip_prefix('\t').is_some_and(|rest| {
        rest.chars()
            .take_while(|c| matches!(c, '@' | '+' | '-'))
            .any(|c| c == '-')
    })
}

/// `|| true` / `|| :` not followed by more identifier text.
fn swallows_failure(line: &str) -> bool {
    line.match_indices("||").any(|(at, _)| {
        let rest = line[at + 2..].trim_start();
        ["true", ":"].iter().any(|word| {
            rest.strip_prefix(word).is_some_and(|t| {
                t.chars()
                    .next()
                    .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
            })
        })
    })
}

/// Findings as `(line number, description)`.
pub fn scan(text: &str) -> Vec<(usize, &'static str)> {
    let checks: [(Check, &str); 8] = [
        (assigns, "assigns SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS"),
        (
            defines,
            "defines SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS via define/endef",
        ),
        (
            exports,
            "export/override/unexport/private of SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS",
        ),
        (
            |l| {
                l.contains("$(eval")
                    && l.split("$(eval")
                        .skip(1)
                        .any(|r| r.starts_with(char::is_whitespace))
            },
            "uses $(eval ...)",
        ),
        (includes, "uses include/-include/sinclude"),
        (
            special_target,
            "declares .IGNORE/.SILENT/.ONESHELL/.DEFAULT",
        ),
        (
            dash_prefix,
            "recipe line uses the '-' error-suppressing prefix",
        ),
        (
            swallows_failure,
            "recipe line swallows failure with '|| true' or '|| :'",
        ),
    ];
    let mut findings = Vec::new();
    for (index, line) in strip_non_recipe_comments(text).iter().enumerate() {
        for (check, description) in &checks {
            if check(line) {
                findings.push((index + 1, *description));
            }
        }
    }
    findings
}

fn main() -> ExitCode {
    let files: Vec<String> = env::args().skip(1).collect();
    if files.is_empty() {
        eprintln!("usage: check_make_execution_control MAKEFILE...");
        return ExitCode::from(2);
    }
    let mut violations = false;
    for file in &files {
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        for (line, description) in scan(&text) {
            eprintln!("execution-control guard: {file}:{line}: {description}");
            violations = true;
        }
    }
    ExitCode::from(u8::from(violations))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(text: &str) -> Vec<&'static str> {
        scan(text).into_iter().map(|(_, m)| m).collect()
    }

    // FND-002: `|| true` without a trailing boundary.
    #[test]
    fn or_true_and_or_colon_are_caught_without_a_trailing_space() {
        assert!(messages("all:\n\tcmd || true; echo continuing\n")
            .contains(&"recipe line swallows failure with '|| true' or '|| :'"));
        assert!(messages("all:\n\tcmd || :; echo continuing\n")
            .contains(&"recipe line swallows failure with '|| true' or '|| :'"));
        assert!(messages("all:\n\tcmd || truest\n").is_empty());
    }

    // FND-003: the dash prefix in any position of the prefix run.
    #[test]
    fn a_dash_anywhere_in_the_prefix_run_is_caught() {
        assert!(!messages("all:\n\t@-command\n").is_empty());
        assert!(!messages("all:\n\t+-command\n").is_empty());
        assert!(!messages("all:\n\t-command\n").is_empty());
        assert!(messages("all:\n\t@command\n").is_empty());
    }

    // FND-004: prose naming a construct is not a finding, a real one is.
    #[test]
    fn comments_are_ignored_but_real_assignments_are_caught() {
        let prose = "# example: SHELL := /usr/bin/true is dangerous\nall:\n\techo hi\n";
        assert!(messages(prose).is_empty());
        let real = format!("{prose}SHELL := /usr/bin/true\n");
        assert_eq!(
            scan(&real),
            vec![(4, "assigns SHELL/.SHELLFLAGS/MAKE/MAKEFLAGS")]
        );
    }

    #[test]
    fn every_neutering_construct_is_caught() {
        for text in [
            "SHELL = /bin/true\n",
            "all: .SHELLFLAGS = -c\n",
            "MAKEFLAGS += -i\n",
            "define SHELL\nendef\n",
            "export SHELL\n",
            "override MAKE = true\n",
            "$(eval SHELL := true)\n",
            "include other.mk\n",
            "-include other.mk\n",
            ".IGNORE:\n",
            ".SILENT :\n",
        ] {
            assert!(!messages(text).is_empty(), "{text:?}");
        }
        assert!(
            messages("MAKEFILE_LIST := a\nSHELLCHECK = x\nall:\n\techo $(MAKE) -C x\n").is_empty()
        );
    }

    #[test]
    fn word_boundaries_keep_lookalike_names_from_being_flagged() {
        for text in [
            "define SHELLX\nendef\n",
            "defineSHELL\n",
            "export SHELLX\n",
            "export XSHELL\n",
            "export a=b SHELL\n",
            "XSHELL = x\n",
            "x := $(evaluate)\n",
        ] {
            assert!(messages(text).is_empty(), "{text:?}");
        }
    }

    #[test]
    fn assignment_prefixes_and_export_lists_are_recognised() {
        for text in [
            "all:SHELL = x\n",
            "%SHELL = x\n",
            "all: SHELL=x\n",
            "export A,B SHELL\n",
            "export A_B,C SHELL\n",
            "export A1 SHELL=x\n",
            "define\tSHELL\nendef\n",
        ] {
            assert!(!messages(text).is_empty(), "{text:?}");
        }
    }

    #[test]
    fn an_escaped_hash_does_not_start_a_comment() {
        assert!(!messages("FOO = a\\#b\nSHELL := x\n").is_empty());
        assert!(!messages("FOO = a\\# SHELL := x\n").is_empty());
        assert!(messages("FOO = a # SHELL := x\n").is_empty());
    }

    // Safety net: the shipped Makefile itself stays clean.
    #[test]
    fn the_repository_makefile_is_clean() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Makefile");
        let text = fs::read_to_string(path).unwrap();
        assert_eq!(scan(&text), Vec::new());
    }
}
