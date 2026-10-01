---
id: OD-GATE-003
type: decision
title: A judging group's no-vacuous-success guarantee is declared once, not argued per module
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - vacuity
  - cli
  - contracts
relations:
  - target: OD-GATE-001
    type: relates-to
  - target: OD-GATE-002
    type: relates-to
  - target: OD-GATE-004
    type: relates-to
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-COMPLETENESS-004
    type: relates-to
---

# A judging group's no-vacuous-success guarantee is declared once, not argued per module

## Question

`nomos check` cannot report a clean run over nothing. An empty walk exits
`ExitCode::Vacuous`, `6`, and `check.rs`'s own doc comment names the sibling defect it exists
against: `check-standards-tree /nonexistent` walked nothing, found nothing and reported
CLEAN. `nomos spec` has the same shape under a different name — an identifier the store
never got because a corpus was never configured exits `ExitCode::Absent` rather than reading
as a plain "not found" — and `corpus.rs`'s own doc comment names the same defect one level up:
"a read command that prints an empty table for a corpus it never had is the same defect
wearing a different hat."

Both guards are real, both are tested, and both are entirely local. `check::ExitCode` is
private to `check.rs`'s own module group; `spec::ExitCode` is private to `spec.rs`'s. Nothing
in `nomos-contracts` — the crate every judging group already depends on — makes a success
value unconstructible without evidence that something was examined, so nothing stops a third
judging group from being wired up with an `Ok` that a caller can reach over an empty subject
set, the same way `check` could before `OD-RULES-001` and `spec` could before `corpus.rs` was
written. Each module argues its own placement well, in its own doc comment. A cross-command
guarantee decided in one module's prose is a decision the next author does not encounter,
because the next author reads their own module and not the sibling's.

Two shapes of answer were on the table. Either caller-side placement stands, and something
makes the next judging group inherit the guard instead of re-deriving it from scratch — a
shared entry point, a contract test over every group that can print a verdict, or a named,
accepted cost if it stays purely per-command. Or the guarantee moves into a value in
`nomos-contracts` that a judging command cannot construct an `Ok` from without having judged
something.

## The Decision

**Caller-side placement stands. `check::sources::Walked`, `check::facts::Nothing_Materialized`
and `spec::reporting::Absent_Or` are unchanged — they still decide it, because `check.rs`'s
own argument for that is correct and this record does not relitigate it: "did I see a
plausible amount of the world" is a question only the caller that chose the tree, or the
identifier, can answer. What changes is that the decision now lives in one place a third
group cannot avoid finding.**

`crates/host/nomos-cli/src/vacuity.rs` is that place. It declares `Group`, the closed set of
things `main.rs` dispatches to (`Work`, `Spec`, `Check`, `Request`), and `Stance_Of`, a match
over `Group` with no wildcard arm that says, for each one, whether it is `Guarded` — walks or
looks up a caller-chosen subject set and refuses `Ok` when it is empty, naming the function
that does the refusing — or `NotApplicable`, with the reason named rather than left blank.
`main.rs`'s own dispatch was rewired to route through this module's `Named` lookup rather
than comparing strings inline, so a fifth group has to be spelled in `vacuity::NAMES` before
`main.rs` can reach it at all, and a `Group` variant with no `Stance_Of` arm fails the build
at that match. `#[cfg(test)]` tests in the same module do not stop at the declaration: for
each `Guarded` group they drive the real `Run` function over a subject deliberately emptied —
an empty directory for `check`, a record identifier the store never held with no corpus
configured for `spec` — and assert the exit code is not that group's `Ok`.

`work` and `request` are declared `NotApplicable`. `work` reports the ledger's own state; an
empty board is a true empty board, not a broken read of one that has entries. `request`
always names exactly one submission the caller wrote; there is no caller-chosen subject set
for it to have walked and found empty the way a checked-out tree or a queried record can be.
Neither claim is asserted by a test — there is no principled way to synthesize "an empty
ledger-coordination request" the way an empty directory or a missing record can be
synthesized — so both rest on the review a change to either group already gets, the same way
`OD-GATE-013`'s per-enum exception rests on review rather than on a check.

**The type-level alternative — a value in `nomos-contracts` a judging command cannot
construct `Ok` from without evidence — was not taken, and the reason is `nomos-contracts`'
own admission test.** `OD-CONTRACTS-001` admits a type to band 0 only when it crosses a
subsystem, process or plugin boundary and the parties on both sides need one stable shared
representation of it: *would a peer that never compiles this crate — a knowledge service in
another language, a client in TypeScript, a platform in another Rust workspace — be unable to
agree with us without it?* An exit code `nomos-cli`'s `main` produces is not part of that
protocol. It is what happens at a process boundary — this binary talking to a shell, to CI —
that none of band 0's other peers ever cross. Forcing every peer that compiles
`nomos-contracts` to carry a "was something judged" type so that one binary's two command
groups can share it would be exactly the kind of addition `OD-CONTRACTS-001` exists to
refuse: important to this crate, general in shape, and not something the other side of the
boundary needs to agree about.

## What This Costs

**The caller-side answer covers less than the type-level one would have.** A library
consumer that calls `check::Run` or `spec::Run` directly, never going through `main`, gets
exactly the guard each module's own code already gave it — nothing here adds a second
refusal inside either `Run`. A type-level guarantee would have covered that caller too,
because it would not be possible to *have* an `Ok` value anywhere, library call or process
exit, without having gone through the construction that proves something was judged. This
record does not buy that; it buys the guarantee at the one boundary `.github/workflows/gate.yml`
can actually observe, which is `OD-GATE-004`'s boundary, and stops there.

**The inheritance is compile-and-test enforced only for the step of adding a `Stance_Of` arm
once a `Group` variant exists — not for the step of adding the `Group` variant in the first
place.** A future author wiring a fifth `main.rs` group still has to remember to add it to
`vacuity::Group` and `vacuity::NAMES` before the exhaustiveness check engages at all; nothing
makes *that* step itself fail to compile. What fails is everything downstream of forgetting
being caught early: a `Group` variant with no `Stance_Of` arm fails `cargo test` and the
gate's `Lint` step (`cargo clippy --workspace --all-targets`, which compiles `#[cfg(test)]`
code), and a `Stance::Guarded` claim the real `Run` does not honour fails the behavioural test
next to it. That is a real narrowing from "argued in a doc comment nobody but that module's
reader sees" to "caught the first time anyone runs the suite", but it is not a guarantee that
survives a group added and never wired through this module's dispatch table at all.

**`Stance` and `Stance_Of` are `#[cfg(test)]`-only.** Nothing at runtime consults a group's
declared stance; the exhaustiveness this module exists for is a static property, checked on
every `cargo test` and every `Lint` step, and carrying it into the shipped binary would be
dead code the compiler is right to flag. The consequence is that a plain
`cargo build --release --bin nomos` — the build `AGENTS.md` asks an agent to run before using
the binary for ledger verbs — does not exercise this check at all; only `--all-targets` and
`test` invocations do.

## Consequences

`crates/host/nomos-cli/src/vacuity.rs` is now where a reader goes to find every group's
stance in one place, and `check.rs`'s doc comment was trimmed to point there instead of
re-arguing where the guard belongs. Neither `check.rs`'s guard code nor `spec.rs`'s changed
shape or behaviour. `nomos-contracts` gained nothing: no new module, no new public type, no
change to its surface snapshot.

## What This Does Not Do

- It does not build `nomos-gates`, `nomos-findings` or `nomos-applicability`. `OD-LEDGER-002`
  already records that those crates do not exist; this record answers what replaced the
  guarantee they were meant to carry in their absence, which is the gap that record left open.
- It does not add a second judging command. The mechanism is exercised today by exactly the
  two groups that already had a vacuity concept, `check` and `spec` — a third is not required
  to close this, and the record does not invent one to demonstrate the pattern further than
  the workspace currently needs it demonstrated.
- It does not change any exit code's number or meaning. `check::ExitCode::Vacuous` is still
  `6`; `spec::ExitCode::Absent` is still `6`; `work` and `request` still have no code in that
  position.
- It does not make `work` or `request`'s "not applicable" claim mechanically checked. That gap
  is named above rather than closed, because no principled empty subject exists to drive their
  `Run` functions against.

## Controls

| Alternative | Why not |
|---|---|
| move the guard into a `Verdict`/`Judged` type in `nomos-contracts` | fails `OD-CONTRACTS-001`'s admission test — an exit-code-shaped concept is process-local, not something a peer that never compiles this crate needs to agree about |
| leave the guarantee as two independent doc comments | the defect this record exists to close: the next author reads their own module and never encounters the sibling's argument |
| a `Group` variant added to `vacuity.rs` with no `Stance_Of` arm | the compiler: `non-exhaustive patterns` at `Stance_Of`'s match |
| a fifth `main.rs` group dispatched without a `vacuity::NAMES` entry | `main.rs` cannot route to it — `vacuity::Named` returns `None` and the group falls through to `Usage()` |
| a `Stance::Guarded` claim the real `Run` does not honour | `Test_Check_Should_Refuse_Ok_Over_An_Empty_Tree` / `Test_Spec_Should_Refuse_Ok_Over_A_Record_The_Store_Never_Had` in `vacuity.rs` go red |
| declare `work` or `request` `Guarded` | no empty subject exists to drive them over, so the declaration would be untested prose wearing a test file's name |
