# Retained evidence

Run `bash scripts/collect_evidence.sh` from a clean repository root. Each run
creates a revision-and-UTC-time-scoped directory, refuses overwrite, and
retains command stdout, stderr, status, identities, limitations, a canonical
`quire.derivation-evidence/v1` envelope, a post-seal validation summary, and an
external SHA-256 manifest.

`evidence/ANCHORS` binds each retained outer manifest from the reviewed source
tree. `make verify-evidence` checks this second-level anchor before rechecking
the outer manifest, every manifest-listed artifact, and the derived collection
summary. `evidence/RETRACTIONS.json` explicitly identifies superseded records;
they remain immutable history but are not active qualification evidence.

Set `PGM01_SCHEMA` and `PGM01_VALIDATOR` to the exact merged PGM-01 Draft 7
schema and validator. The validator is run with the source-locked Python;
`PGM01_PYTHON` overrides are refused. Missing external gates are status 125 and
non-conclusive, never a pass. The exact finalized envelope is revalidated
before the separate summary is written, so it never self-attests.

The verifier checks append-only behavior relative to the Git history presented
by the checkout. Remote branch protection and review history, not a locally
restatable digest, provide resistance to branch re-rooting.

Evidence informs a pending human source-release decision. It neither proves
all bounded strings safe nor approves, validates, qualifies, accredits,
certifies, publishes, or releases a consuming project.
