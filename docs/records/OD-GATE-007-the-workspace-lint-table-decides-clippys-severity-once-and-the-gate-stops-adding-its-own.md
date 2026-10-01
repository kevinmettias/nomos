---
id: OD-GATE-007
type: decision
title: The workspace lint table decides clippy's severity once, and the gate stops adding its own
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - lint
  - determinism
relations:
  - target: OD-GATE-006
    type: relates-to
  - target: OD-LEDGER-003
    type: relates-to
  - target: OD-GATE-004
    type: relates-to
---

# The workspace lint table decides clippy's severity once, and the gate stops adding its own

## Question

`Cargo.toml`'s `[workspace.lints.clippy]` states a severity per lint: `clippy::all` and
`clippy::pedantic` sit at `warn`, and `float_cmp`, `unwrap_used`, `arithmetic_side_effects`
and `indexing_slicing` sit at `deny`, because a panic on the analysis path is a determinism
defect rather than a bug — a replay must reach the same panic at the same step.

`.github/workflows/gate.yml`'s `Lint` step ran `cargo clippy --workspace --all-targets --
-D warnings`. `-D warnings` promotes every remaining warning to an error, so it did not add a
new rule — it erased the distinction the table had just drawn. `clippy::all` and
`clippy::pedantic`, `warn` in the table, were hard failures in CI exactly like the four `deny`
lints, and nothing said so. A contributor reading `Cargo.toml` to learn what blocks a merge
learned something false: the stricter of the two rules was the one written down nowhere as
policy. `OD-GATE-006` named this and deferred it — "The workspace lint table in `Cargo.toml`
and the gate's `-D warnings` are two authorities for one severity policy, and both of them
run — that is a disagreement between executors rather than an absent one" — because both
authorities *executed*, which was a different defect than the one that record was about.

## Decision

**`Cargo.toml`'s `[workspace.lints.clippy]` table is the one place clippy's severity is
stated. The gate's `Lint` step runs `cargo clippy --workspace --all-targets`, unchanged
otherwise, with no promotion flag of its own.**

Chosen over promoting the table to match the gate — setting `all` and `pedantic` to `deny`
so `-D warnings` becomes redundant there too — because that direction reaches further than
this disagreement. `-D warnings` is not scoped to `[workspace.lints.clippy]`; it promotes
every warning the invocation produces, including `[workspace.lints.rust]`'s `unsafe_code`
(`warn`) and any ordinary rustc warning clippy surfaces along the way. Matching the gate's
severity inside the clippy table would still have left those outside it silently governed by
a flag this record was never written to examine. Removing the flag instead collapses onto the
table that already states a reason for each of its own choices, and it removes exactly the
authority this record is about — nothing wider.

The table's declared severities do not change. `float_cmp`, `unwrap_used`,
`arithmetic_side_effects` and `indexing_slicing` are still `deny`, so a bare `cargo clippy` —
on a contributor's machine or in the gate — still fails on them with no extra flag, unchanged
by this decision. `clippy::all` and `clippy::pedantic` are still `warn`, and that is now true
everywhere rather than true only until `-D warnings` was added: a contributor's own machine
and the gate agree, for the first time, about what actually blocks a merge.

## What This Costs

**Named rather than discovered.** Measured against the working tree this record was written
in: `cargo clippy --workspace --all-targets -- -D warnings` (the arrangement being replaced)
exits 0 with no warnings. `cargo clippy --workspace --all-targets` (the arrangement this
record adopts) also exits 0 with no warnings — the same command, run without the flag, over
the same tree. So today the two arrangements reject the identical, empty set: nothing that
passed the old gate would have failed it, and nothing new passes the new one that failed the
old one.

The cost is forward-looking rather than measured. Before this decision, a `clippy::pedantic`
finding anywhere in the workspace failed the gate — CI rejected it whether or not `Cargo.toml`
said so. After this decision, the same finding only warns in CI, exactly as it already only
warned on a contributor's own machine; it stops a merge only if somebody reads the warning and
acts on it, or if the finding also trips one of the four `deny` lints. That is a real loss of
a backstop this repository had, silently, until this record. It is accepted because the
backstop was never a *stated* policy — `Cargo.toml` never claimed pedantic findings would
block a merge — and restoring it by raising the table's severity was rejected above for
reaching into `unsafe_code` and plain rustc warnings this record does not examine. A future
record is free to raise `clippy::pedantic` to `deny` on its own stated grounds; this one does
not do it by accident, through a flag scoped to something else.

**`unsafe_code` loses the same promotion, unexamined.** `[workspace.lints.rust]`'s
`unsafe_code = "warn"` was also promoted to a hard failure in CI by the same flag, for the
same unstated reason. Removing `-D warnings` removes that promotion too. This record does not
raise `unsafe_code` to `deny` — that table, and that lint, are outside what `Cargo.toml`'s
`[workspace.lints.clippy]` comment and this item's territory describe — but it is named here
rather than left for the next reader to find by noticing a workflow diff with no record
behind it.

## Consequences

`.github/workflows/gate.yml`'s `Lint` step's `run:` line drops `-- -D warnings`. The step's own
comment, and `Cargo.toml`'s `[workspace.lints.clippy]` header comment, both point here instead
of each re-explaining why.

`README.md`'s `## Running the gate` section stops reproducing the gate's exact commands; it
routes to `.github/workflows/gate.yml` instead, which is the one place `OD-LEDGER-003` already
derives `work finish`'s lint step from. The paragraph naming `unwrap_used`, `indexing_slicing`,
`arithmetic_side_effects` and `float_cmp` as denied stays, and stays true — `Cargo.toml`'s
table did not change on that axis.

`nomos_ledger::Derive_Step(&Workflow(), LINT_STEP)` yields an argv one flag shorter than
before: `cargo clippy --workspace --all-targets`, no trailing `-- -D warnings`.
`Test_The_Derived_Lint_Step_Should_Still_Be_Clippy`
(`crates/host/nomos-cli/tests/gate_step/lint_step.rs`) and
`Test_This_Repository_Gate_Should_Still_Yield_A_Lint_Step`
(`crates/substrate/nomos-ledger/tests/gate_covers_finish/repository_gate.rs`) are updated to
assert clippy still runs and `-D` no longer does, rather than asserting the flag this record
removes.

## What Holds It

The two tests named above, run over the repository's real workflow rather than a fixture — the
same shape `OD-LEDGER-003` already used to catch a scripted or renamed `Lint` step.
`crates/substrate/nomos-ledger/src/gate.rs`'s own unit tests use a self-contained fixture
workflow string and are untouched by this decision; they test `Derive_Step`'s parsing, not
this repository's severity policy.

## What This Record Does Not Decide

It does not decide `[workspace.lints.rust]`'s severities, including `unsafe_code`. It does not
raise `clippy::pedantic` to `deny` to recover the backstop the cost section above names — that
is a choice with its own cost, for a later record to make on its own stated grounds, not a side
effect of this one.
