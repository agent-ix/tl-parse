# Retained evidence

Run `bash scripts/collect_evidence.sh --tool-profile PROFILE` from a clean
repository root. `PROFILE` must name a reviewed entry in the versioned
`tools.lock`; an omitted argument selects its reviewed `defaultProfile`. Each run
creates a revision-and-UTC-time-scoped directory, refuses overwrite, and
retains command stdout, stderr, status, identities, limitations, a canonical
`quire.derivation-evidence/v1` envelope, a post-seal validation summary, and an
external SHA-256 manifest. Qualification-v2 records created by the current
collector retain the selected profile both as
`qualification-profile.txt` and as `toolProfile` in `collection-input.json`,
then re-derive it against the source revision during verification. Earlier
records that predate the versioned lock are classified as
`unsupported-lock-schema`, not as checked failures or current qualification.
Their checksums and artifact manifests remain mandatory, but once AA-001 names
a fresh active v2 record they are counted as unsupported history rather than
being forced into the retraction registry. An unsupported record cannot itself
be the assurance-bound record and its individual qualification check exits 4.

Profiles bind absolute paths, bytes, `HOME`, and `CARGO_TARGET_DIR`. The current
profile pins the actual stable Cargo/rustc binaries and separately pins rustup
for explicit nightly/MSRV selection. A path
alias is accepted only when that exact alias is declared in the selected
profile and has the declared SHA-256. Different bytes, an undeclared profile,
or a command resolving to a different path fail closed. Before any manual
hosted run, commit and review a runner-specific profile, then select that exact
profile in the workflow-dispatch input. This does not authorize a dispatch or
automatic `push`/`pull_request` triggers.

The source clean-tree check uses Git's complete porcelain census. The checked-in
`.gitignore` excludes `/target`, Python bytecode caches, editor state, and other
local build/runtime residue; those paths are outside the candidate-source claim.

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
