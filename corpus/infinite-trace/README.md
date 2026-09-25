# Infinite-trace parser fixtures

`manifest.json` is a hand-reviewed oracle for TL-208. Byte spans are half-open
UTF-8 offsets into each fixture file excluding its final newline. The source
files are retained with a final newline for readable diffs; the parser input
is the file with exactly that newline removed. `SHA256SUMS` pins all bytes.

The expected output was written from the v4 grammar and counted source bytes,
not copied from tl-parse. TL-14 makes the replay executable. A fixed point
test may canonicalize whitespace, but must preserve each owner graph and
fairness root.
