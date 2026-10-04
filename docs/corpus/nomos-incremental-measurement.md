# Nomos 0.1 incremental check measurement

Measured 2026-10-04 on Windows, in an unoptimized debug test binary, through
`nomos_check_orchestration::Run_Reassessing` with the standard providers. This is
acceptance evidence for a selected check path, not a latency promise for all rules.

```powershell
$env:NOMOS_RELEASE_CORPUS = 'F:\repos\xvpe\crates'
cargo test --locked --offline -p nomos-integration-tests --test release_incremental -- --ignored --nocapture
cargo test --locked --offline -p nomos-check-orchestration --lib Test_Run_Reassessing -- --nocapture
```

The explicitly ignored integration test fails when its configured corpus is
missing, unreadable or empty. Ordinary test runs visibly report it as ignored;
they are not evidence this measurement ran.

## Real source, public counters

The input was the actual XVPE `crates` tree, 14,083 recognized source files.
The checkout reported HEAD `ee27e3ae` and contained uncommitted work. These
numbers describe the captured source population, not a pristine revision.
The harness freezes source bytes once and changes one file only in memory:
`apps/demos/xvpe-demo-black-hole/src/lib.rs`. It appends a constant declaring a
missing completeness mirror, so the edit has an observable judgment consequence.
No source file is written to disk.

The two selected rules are `completeness-mirror` and
`requirement-trace-staleness`. Elapsed time brackets only the public check call;
compilation, corpus walking and preparation are excluded. Each row is one
measurement, with no distribution or timing threshold inferred from it.

| Run | Elapsed milliseconds | New fact materializations | Findings |
|---|---:|---:|---:|
| Cold | 8,784.109 | 14,082 | 149 |
| Unchanged, resident workspace/store/cache | 1,564.558 | 0 | 149 |
| One file edited, resident state | 2,182.457 | 1 | 150 |
| Fresh state over the identical edited inputs | 9,473.941 | 14,082 | 150 |

The edited and fresh findings are asserted equal, including their order and full
values. The missing mirror must occur in the edited judgment and be absent from
the cold judgment. Unchanged findings must equal cold findings, and the edit
must materialize a nonzero strict subset of cold work. This rules out an
unconditionally silent cache passing as an incremental implementation.

The materialization counter is the public `MemoryFactStore::Materializations()`.
It counts filed facts, not all parsing, input hashing, filesystem reads or
subprocess work. An unchanged run still has measurable work and latency. Two
syntax refusals in this corpus are exposed by the existing scale test rather
than silently counted as successfully parsed source.

## Rule invocation evidence, checked independently

`RuleReassessmentCache::Recorded()` is test-only and crate-private. The real
corpus integration test cannot observe it; none of the table's numbers is
presented as a rule-invocation count.

The separate existing `Test_Run_Reassessing` unit tests directly inspect that
private counter using two real selected rule closures and a small fixture.
Cold records two invocations. An unchanged run records no additional invocation.
Editing one source records exactly one additional invocation: the syntax-based
completeness rule runs again, while the workspace requirement-trace rule whose
fact did not change stays cached. This is an assertion about those selected
closures and that fixture, not a measured all-rule invocation count over XVPE.

## Remaining acceptance work

This evidence does not cover every capability, compiler/tool invocation, a CLI
process retaining state across runs, an optimized build, a complete independent
repository or repeated latency sampling. Correction safety and the owner's
personal mastery exercises have their own acceptance obligations. No complete
0.1 release claim follows from this measurement alone.
