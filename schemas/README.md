# Frozen evidence schemas

These two files are **frozen, not live**. Nothing validates against them, and no
script in this repository may reference them.

| File | SHA-256 |
|---|---|
| `tl-parse-evidence-input-v1.schema.json` | `1e6683eb04231f18d1ad4c6bf95f1b039798374f3d26483c7fe25333bf6669d2` |
| `tl-parse-evidence-manifest-v1.schema.json` | `0717400327650c92e4311fdf2f47720f969906c207ec595e3d3d31ef937a80f9` |

They were the schemas of the local evidence framework the shared-assurance
migration removed. They are kept because all twelve retained envelopes under
`evidence/` name them by path and by SHA-256, and those bytes are immutable —
deleting the schemas would break a reference inside a record this repository is
required to leave untouched.

`tests/shared_assurance.rs` asserts both digests and censuses the whole source
tree — every directory except `evidence/`, `target/` and the assurance
virtualenv — to prove nothing validates against them.

Three files name them on purpose and are allow-listed by that census: the test
itself (which pins the digests), this README (which documents the freeze), and
`assurance/change-assurance.json` (which states the preservation constraint).
Anything else naming them fails TC-028.
