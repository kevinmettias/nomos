# Nomos 0.1 validation evidence

Measured 2026-10-04 on Windows, Rust 1.88.0, against Nomos `d7588252` plus the
MCP acceptance tests introduced with this evidence. This is a dated measurement,
not a release declaration or a second work board. `nomos work list` remains the
work authority.

## A client can inspect, decline, commit and verify a correction

From the repository root:

```powershell
cargo test --locked -p nomos-mcp --test release_workflow
```

Both conversations run the built `nomos-mcp` binary over actual stdin/stdout.
The first initializes, lists tools, judges a deliberately false mirror claim,
checks its summary, location, applicability and evidence class, and asks for its
explanation. The default correction call stages a preview and leaves the source
byte-identical. Rechecking without sending commit is the worked decline case:
the finding remains and the source remains unchanged. A subsequent explicit
commit changes the workspace snapshot and writes precisely the expected source;
reanalysis finds no blocking finding for that rule.

The second sends malformed JSON, an unknown tool and an invalid commit argument.
Every refusal preserves the source, and the same server still answers a valid
judgment. Response waits are bounded; each conversation reaps its process even
when an assertion fails. These are constructed adversarial fixtures, not a claim
that every repository can be repaired automatically.

## Real source populations

```powershell
$env:NOMOS_RUST_CORPUS = 'F:\repos\xvpe\crates'
cargo test --locked -p nomos-integration-tests --test analysis_slice scale:: -- --nocapture --test-threads=1
cargo test --locked -p nomos-integration-tests --test calibration -- --nocapture
nomos gate run --root F:\repos\kwb --rule completeness-mirror --rule no-trailing-whitespace --sarif kwb.sarif
```

The XVPE crates run passed four tests in 102.58 seconds, excluding compilation.
It walked 14,083 actual source files. The parser produced 14,081 syntax facts,
explicitly refused two files, and produced 3,424 directory rollups with one
reported degraded rollup. Unchanged reanalysis materialized zero syntax facts
and zero rollups, reusing all 14,081 successful syntax facts and 3,424 rollups.
The weaker scanner covered all 14,083 files, with no degraded rollup. These are
fact-materialization measurements; they do not establish that unchanged analysis
performs no filesystem reads or subprocess work.

The complete composed-rule calibration passed against the `hex` 0.4.3 excerpt
in `tests/integration/fixtures/third-party`. Its provenance document spells out
which upstream production source was kept and which test/feature sections were
trimmed, and distinguishes the added policy declarations from upstream material.
This is third-party production code, but it is an excerpt, not a complete
independent repository checkout.

The two-rule CLI run over the actual KWB repository reported one advisory:
`TOOLS` in `crates/host/kwb-mcp/src/lib.rs` declares no completeness mirror. It
reported no blocking finding and exited zero. This result is scoped to the two
selected rules; it does not claim KWB passes the full Nomos rule set. SARIF is the
inspectable structured output, preserving the selected finding's location and
rule information.

The initial full XVPE-root scale attempt was cancelled. Its test-harness walk
skips `target` and `.git`, but follows directory junctions in local task artifacts,
which exposed Cargo's source cache. No result from that attempt is reported as
real-code acceptance. The measured run deliberately selects the actual `crates`
source tree and records that scope.

## Observed stale-file protection

```powershell
cargo test --locked -p nomos-correction-orchestration
```

All 66 correction tests passed. The three added regressions cover a competing
edit, a removed target and an unchanged target. A filesystem strategy introduces
the competing state between the first read and commit's reread. Changed or
unreadable content is refused before either model publication or file replacement.
Removing that guard made the competing-edit test fail with exit 101; the guard
was restored byte-identically before the completion gates and commit.

## Limits of this evidence

This does not yet certify the whole 0.1 claim. In particular, a disk reread is
not an atomic compare-and-replace with other writers, and model stage/validate
is not a repository build or test run over a staged checkout. Individual cold,
unchanged and one-file-change latency measurements, affected-rule work counts,
and a complete external-repository correction demonstration require additional
acceptance evidence. The owner-only criterion of personally tracing, breaking,
debugging and defending the showcased subsystems cannot be satisfied by an AI
agent reporting its own tests.
