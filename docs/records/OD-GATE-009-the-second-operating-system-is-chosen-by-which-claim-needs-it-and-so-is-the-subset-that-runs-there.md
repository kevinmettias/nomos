---
id: OD-GATE-009
type: decision
title: The second operating system is chosen by which claim needs it, and so is the subset that runs there
status: closed
version: 2
authority: canonical-normative-record
tags:
  - gate
  - ci
  - enforcement
  - determinism
relations:
  - target: OD-GATE-006
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
  - target: OD-DETERMINISM-001
    type: relates-to
  - target: OD-DETERMINISM-002
    type: relates-to
---

# The second operating system is chosen by which claim needs it, and so is the subset that runs there

## Question

`OD-GATE-006` decided that a declaration of enforcement is incomplete until something executes
it, *in every environment the claim needs*, and named four instances. This record carries the
fourth and the only one where the missing environment is a host rather than a step.

The gate ran one job on `ubuntu-latest`. This repository is developed on Windows — every
measurement in its ledger carries an `F:\repos\nomos` path — so the two environments that matter
were the one CI checked and the one nobody checked, and the second is where the code is written.

Two questions follow and they have different answers. **Which** second host, and **what** runs
there. The first is easy to answer badly, by taking whatever the runner catalogue offers. The
second is easy to answer badly in the opposite direction, by running everything twice: a gate
that costs double to re-answer questions with no host in them is a gate somebody will later
narrow under time pressure, and narrowing it then is done without the reasoning that chose it.

## What Was Measured

All of it on Windows, in a detached worktree at `41de751`, before any of it was decided.

| Measurement | Result |
|---|---|
| `cfg(windows)`, `cfg(unix)`, `cfg(target_os = …)` in the workspace | **none, anywhere** |
| the nine candidate packages, `cargo test --no-fail-fast` | 458 passed, 0 failed |
| `spec freshness --require diagram-set --require domain-specification` | exit 0, both current |
| `check --root .` | exit 0, 466 files examined, 13 findings, 0 that can fail a build |

The first row decides more than it looks. With no conditional compilation anywhere in the
workspace, a second `clippy` run lints an identical token stream — so the lint step's exclusion
from the Windows leg is a measurement rather than a cost argument, and it stops being true the
day somebody adds the first `cfg`. That sentence is in the workflow beside the step, because the
next author to add a `cfg` will be reading the workflow and not this record.

The remaining rows say the Windows leg is green today. That matters for a reason the item
carrying this was explicit about: a lane nobody has to fix is the corpus-gate shape without the
corpus gate's reason for it, so this leg is added knowing it passes, not hoping.

## The Decision

One job, a matrix over `[ubuntu-latest, windows-latest]`, and each step declares which legs it
runs on.

| Step | Legs | Why |
|---|---|---|
| `checkout`, `Show toolchain` | both | the leg cannot run otherwise |
| `Lint` | Linux | no `cfg` in the workspace, so the second run lints the same tokens |
| `Test` (`--workspace`) | Linux | the whole suite once; the subset that has a host in it runs opposite |
| `Determinism` | Windows | the seven crates carrying a `determinism.rs`, plus `nomos-spec-store` |
| `Boundaries` | both | surface snapshots are produced by walking source files |
| `Required projections` | both | a content hash over rendered bytes, which is the cross-host claim |
| `Rules` | Linux | judges code content, measured identical on Windows |
| `Install cargo-deny`, `Supply chain` | Linux | read `Cargo.lock` and a policy file, neither of which has a host |
| `Corpus gates` | Linux | reports an absence that is absent from both runners for one reason |

`nomos-spec-store` is on the Windows leg for a different reason from the determinism crates. It
owns the embedded-record carriage-return test that `.gitattributes` names as its **only** defence
against a class it describes in full — "content hashes are computed over file bytes and the
governing records are embedded with `include_str!`", so a clone with `core.autocrlf=true` would
change every block hash and "the same commit would validate on one machine and not on another".
A line ending is the one input that cannot be checked on the host that cannot produce it.

## Why Windows Rather Than Whatever Was Available

Because a claim in this tree depends on it and names it. `.gitattributes` is not a lint
preference; it is a defence, written as prose, against a specific failure of a specific pair of
hosts, and its own text says the defence is one test. Line endings differ across exactly this
pair. The rest of the class this repository is exposed to — path separators, directory iteration
order, filename case — differs on the same axis, and the six crates carrying a `determinism.rs`
make claims of the form "the same input yields the same output" over inputs of exactly that kind.

A third host would have to be justified the same way, by a claim that needs it, and none in this
tree does today.

## `fail-fast: false` Is Not `continue-on-error`, And The Distinction Is The Whole Point

The matrix sets `fail-fast: false`. It does not permit a leg to fail: both legs must be green for
the gate to pass. It stops one leg's failure from cancelling the other, which is the difference
between learning "Windows is red and Linux is fine" and learning "something was red".

`continue-on-error` is the excuse, it appears nowhere in this workflow, and
`Test_No_Step_In_The_Gate_Should_Excuse_Itself` refuses it by name along with `|| true` and a
trailing `exit 0`. A red lane here is a finding, and a finding gets an item.

## A Matrix Rather Than A Second Job, For A Reason Discovered Rather Than Chosen

The obvious shape is a second job, and it was written first. It fails a guard this workflow
already carries: a second job needs its own `checkout`, and
`Test_The_Pin_Check_Should_Reject_An_Action_On_A_Tag` measures the pin check against a fixture
built from the real workflow with one action in it, asserting the reported list is exactly one
entry. Two checkouts make it two.

That test lives in `crates/host/nomos-cli/tests/gate_step.rs`, which the item carrying this
record does not reserve, so the choice was between widening a claimed territory mid-item and
finding a shape that fits inside it. One job with one `checkout` fits, and it is not a
compromise: per-step `if:` conditions express the decided split more directly than duplicated
step lists would, because the two legs stay side by side in one place instead of drifting apart
in two.

The guard was right to fire and the assertion should probably become "every action reported" once
some workflow here legitimately runs two. That is not this item's to change.

## What This Record Does Not Decide

It does not decide that the Windows leg's subset is permanent. It decides that the subset is
chosen by which claims have a host in them, and the workflow states the reason beside each step
so the next editor can re-derive it rather than guess.

It does not decide anything about the determinism declarations themselves. `OD-DETERMINISM-001`
and `OD-DETERMINISM-002` govern those, and if this leg ever reddens, the defect lives in the
crate that declares the determinism — which this item does not reserve and must not widen into.

It does not decide that the corpora gap is narrowed. Neither runner has them, `Corpus gates`
reports that on one leg, and a second report of the same absence would read as two measurements.

## What Was Considered And Rejected

**Run the whole workspace on both legs.** Rejected on the item's own terms: the steps that could
differ by platform are not the same set as the steps that are expensive, and the most expensive
steps here — the full suite, clippy, building cargo-deny — are the ones with no host in them.

**A matrix whose extra leg may fail.** Rejected, and it is worth naming because it is what most
repositories do. It converts a gate into a dashboard; nobody fixes a leg nobody has to fix.

**A second job.** Rejected on evidence, above, not on taste.

**macOS as a third leg.** Rejected for now because no claim in this tree names it. Adding a host
because a runner exists is the defaulting this record refuses in its first paragraph.

**Leaving the Windows leg to a contributor's own machine.** That is the status quo this record
ends. It is how `rust-version` stayed false for as long as it did — a configuration everybody
assumed somebody was exercising, which `P11-MSRV-UNCHECKED` measured and nobody was.

## What Holds It

The leg itself holds it, by running rather than by asserting, on every push to `main` and `dev`
and on every pull request.

What does **not** hold it should be said plainly. No test asserts that the matrix still has two
entries in it, so a later editor can delete `windows-latest` and every guard in this repository
stays green. The same is true of the per-step `if:` conditions: nothing checks that the
`Determinism` step still runs somewhere. This record and the comments beside the steps are the
whole of the defence, which is exactly the shape `OD-GATE-006` was written about, one level up.
It is named here rather than quietly left. `P12-UNENFORCED-DECLARATION` proposed the general
check; it was declined, not left open — its territory reserved only two spec-store record
files, with no reach into a Rust source, test or fixture, and could not build the executable
check its own `done_when` demanded. The decline asked for reissue with territory widened to the
check's actual home (a new rule under `nomos-rules`, its `nomos check` wiring, and fixture
territory for positive and negative controls), and no item has reissued it. `OD-GATE-006`'s own
reasoning is why that reissue is not a foregone mechanical follow-up: it refused the same check
"for scope" because there is no natural enumeration of "declarations" to read across a CI
workflow, a ban policy and a compatibility floor, and inventing one would put a second authority
beside the files that already make each claim. The gap `P12-UNENFORCED-DECLARATION` named is
real and still unclosed; whether it is buildable as one general check or four instance-specific
ones is not decided here.

## Status

Closed. `P11-PLATFORM-UNCHECKED` carries it, and it discharges the fourth and last of the
instances `OD-GATE-006` measured. The third, `P11-MSRV-UNCHECKED`, is measured but not landed:
correcting the compatibility floor unmasks clippy findings in files that item does not reserve,
which `P11-COLLAPSIBLE-UNMASKED` carries.
