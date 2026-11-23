//! Dialects v1..v4 are byte-identical to their pre-v5 state (TL-421, TL-429).
//! UNBUILT, UNTESTED.
//!
//! `tests/fixtures/pre-v5/` holds byte copies, taken from `origin/main` at
//! `50932b7` before any v5/v6 code, of the v1..v4 dialect records, normative
//! documents, attribution record, retained corpora and checked fuzz seeds. No
//! digest literal is pinned (TL program decision 7): digests are recomputed
//! from the retained records. Retained outputs are replayed by the unchanged
//! suites `corpus`, `format`, `versioned_dialects`, `clean_ascii_v2`,
//! `clean_ascii_v3`, `infinite_v4`, `infinite_trace_corpus` and
//! `infinite_loci`, whose expectations these fixtures freeze.

use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tl_parse::{
    dialect_digest, dialect_v2_digest, dialect_v3_digest, ATTRIBUTION_DOCUMENT, DIALECT_DOCUMENT,
    DIALECT_RECORD, DIALECT_V2_DOCUMENT, DIALECT_V2_RECORD, DIALECT_V3_DOCUMENT,
    DIALECT_V3_RECORD, DIALECT_V4_DOCUMENT,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(relative: &str) -> Vec<u8> {
    fs::read(root().join("tests/fixtures/pre-v5").join(relative)).unwrap()
}

fn files_under(directory: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files_under(&path, found);
        } else {
            found.push(path);
        }
    }
}

// Trace: TC-1541, FR-028-AC-2, TC-123, FR-062-AC-1, StR-044-VC-2
#[test]
fn dialect_records_and_digests_equal_their_pre_change_values() {
    for (current, retained, digest) in [
        (DIALECT_RECORD, "records/v1.txt", dialect_digest()),
        (DIALECT_V2_RECORD, "records/v2.txt", dialect_v2_digest()),
        (DIALECT_V3_RECORD, "records/v3.txt", dialect_v3_digest()),
    ] {
        let retained = fixture(retained);
        assert_eq!(current.as_bytes(), retained.as_slice());
        assert_eq!(digest, format!("{:x}", Sha256::digest(&retained)));
    }
}

// Trace: TC-1541, FR-028-AC-2, TC-123, FR-062-AC-1
#[test]
fn dialect_documents_equal_their_pre_change_bytes() {
    for (current, retained) in [
        (DIALECT_DOCUMENT, "files/docs/DIALECT-001-clean-room-mltl-v1.md"),
        (DIALECT_V2_DOCUMENT, "files/docs/DIALECT-002-clean-ascii-v2.md"),
        (DIALECT_V3_DOCUMENT, "files/docs/DIALECT-003-clean-ascii-v3.md"),
        (DIALECT_V4_DOCUMENT, "files/docs/DIALECT-004-clean-ascii-v4.md"),
        (ATTRIBUTION_DOCUMENT, "files/docs/ATTRIBUTION.md"),
    ] {
        assert_eq!(current.as_bytes(), fixture(retained).as_slice(), "{retained}");
    }
}

// Trace: TC-1541, FR-028-AC-2, TC-123, FR-062-AC-1
#[test]
fn retained_corpora_and_seeds_equal_their_pre_change_bytes() {
    let retained_root = root().join("tests/fixtures/pre-v5/files");
    let mut retained = Vec::new();
    files_under(&retained_root, &mut retained);
    let mut compared = 0;
    for path in retained {
        let relative = path.strip_prefix(&retained_root).unwrap();
        if relative.starts_with("docs") {
            continue;
        }
        let current = fs::read(root().join(relative))
            .unwrap_or_else(|error| panic!("{} was removed: {error}", relative.display()));
        assert_eq!(current, fs::read(&path).unwrap(), "{}", relative.display());
        compared += 1;
    }
    // Every pre-change corpus and seed file of origin/main@50932b7.
    assert_eq!(compared, 30);
}
