# Retained evidence

Run `bash scripts/collect_evidence.sh` from a clean repository root. Each run
creates a revision-and-UTC-time-scoped directory, refuses overwrite, and
retains command stdout, stderr, status, identities, limitations, a canonical
`quire.derivation-evidence/v1` envelope, a post-seal validation summary, and an
external SHA-256 manifest.

Set `PGM01_SCHEMA` and `PGM01_VALIDATOR` to the exact merged PGM-01 Draft 7
schema and validator. Set `PGM01_PYTHON` when the validator uses a dedicated
interpreter. Missing external gates are status 125 and non-conclusive, never a
pass. The exact finalized envelope is revalidated before the separate summary
is written, so it never self-attests.

Evidence informs a pending human source-release decision. It neither proves
all bounded strings safe nor approves, validates, qualifies, accredits,
certifies, publishes, or releases a consuming project.
