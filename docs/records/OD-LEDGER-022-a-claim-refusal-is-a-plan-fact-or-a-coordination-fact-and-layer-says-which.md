---
id: OD-LEDGER-022
type: decision
title: A claim refusal is a plan fact or a coordination fact, and Layer says which
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - dispatch
  - readiness
relations:
  - target: ARC-HARNESS-001
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
  - target: OD-LEDGER-005
    type: relates-to
  - target: OD-LEDGER-020
    type: relates-to
---

# A claim refusal is a plan fact or a coordination fact, and Layer says which

## Question

`ClaimRefusal` has ten variants, and two of the questions they answer are categorically
different. `DependencyUnmet` says a dependency is unfinished: a fact about the plan, true no
matter who asks or when. `HeldBy` says another session holds overlapping territory: a fact
about this moment, false again the instant a lease lapses. `ARC-HARNESS-001`'s sentence names
the two owners — "the scheduler decides what should run, coordination decides whether it can
run now" — and a refusal is always an answer to one of those two questions, never both.

Before this record, that distinction existed in exactly one place: `nomos-cli`'s
`Refusal_Label`, a match arm that turns four specific variants into the words `waiting`,
`stranded`, `held` and `lapsed`, and everything else into `snagged`. It is a correct mapping —
`OD-LEDGER-005` and `OD-LEDGER-020` are the records of the two times it was fixed — but it is
the *only* place the mapping exists. `ClaimRefusal` itself offers `Describe` (a sentence) and
`Is_Retryable` (a bool that answers a third, orthogonal question — see below), and neither
lets a caller other than this one CLI ask which of the two questions a refusal answered.

The cost is concrete for a design already on the table. `ARC-HARNESS-001` assigns "readiness,
ordering and dispatch mechanics" to coordination and leaves "what should run" to a scheduler
above the board, under a one-way rule: coordination may withhold work the plan calls ready,
and coordination alone may never make ready what the plan calls not ready. That rule cannot be
stated, let alone tested, against a model where both halves of it are strings a CLI happens to
print — a caller wanting to ask "is this plan-ready" has to attempt a claim and parse prose, or
reimplement the CLI's match arm and drift from it the next time `ClaimRefusal` grows a variant.
`P10-REQUIRABLE-DECLARED` is the closest thing to a lived instance: it sat `Ready` while
carrying two abandonments explaining its clauses had already landed, and `Ready` — "not
claimed, not done, not blocked, not declined" — was never a readiness answer strong enough to
say so.

### Retryability is not this distinction

`Is_Retryable` looks like it could stand in for the split and does not. `DependencyUnmet` is a
plan fact and is retryable — the plan itself can change, when the dependency finishes.
`StillHeld` is a coordination fact and is retryable — the lease is released or it lapses. The
two axes are independent, and a caller that read `Is_Retryable` as the readiness answer would
tell a scheduler `DependencyUnmet` "might resolve, so it's not a plan question", which is
exactly backwards.

## What Changed

`ClaimRefusal::Layer(&self) -> RefusalLayer`, `crates/substrate/nomos-ledger/src/claim/refusal.rs`.

`RefusalLayer` is a two-variant enum: `Readiness` and `Dispatch`, named after `ARC-HARNESS-001`'s
own two owners rather than invented vocabulary. `Layer` is an exhaustive match with no
catch-all arm, so a variant `ClaimRefusal` grows in the future must be classified here before
the crate compiles — the same guard `OD-CONTRACTS-002` put on `Applicability`'s three
predicates, applied to this type's two.

The classification, and why each variant sits where it does:

| Variant | Layer | Why |
|---|---|---|
| `NotClaimable` | Readiness | The item's own state — `Done`, `Blocked`, `Declined` — is a codebase fact independent of who is asking. |
| `DependencyUnmet` | Readiness | Explicitly the record's motivating example: unfinished work, true for everybody. |
| `DependencyDeclined` | Readiness | `OD-LEDGER-020`'s dead end — a plan fact that will never resolve on its own. |
| `NoSuchItem` | Readiness | Not being on the board at all is the plan's answer, not coordination's. |
| `HeldBy` | Dispatch | Another session's live territory — `ARC-HARNESS-001` names territory and leases as coordination's exact subject matter. |
| `UnknownIndependence` | Dispatch | Whether two *live claims'* territories can be shown disjoint is a question about the current board, not the plan. |
| `LeaseTooLong` | Dispatch | A lease ceiling is coordination's own policy on this attempt's request, not a fact about the item. |
| `Lapsed` | Dispatch | A dead holder is a fact about a specific claim's history, resolved by a coordination operation (`takeover`), not by the plan changing. |
| `StillHeld` | Dispatch | The item's own live claim, ending which is coordination's business. |
| `LedgerUnusable` | Dispatch | The store *is* the coordination mechanism; being unable to reach it says nothing about the plan. |

`Is_Readiness` and `Is_Dispatch` are the predicate forms of `Layer`, the shape
`Applicability::Requires_Agent` already used beside its own classification (`OD-CONTRACTS-002`):
a caller that needs one side of a two-way question is not made to match on the enum itself.

`nomos-cli`'s `Refusal_Label` (`crates/host/nomos-cli/src/work/report.rs`) now matches on
`(refusal.Layer(), refusal)` rather than on the variant alone. The four specific words are
unchanged — this does not rename any display string — but they are now grouped under the
layer they answer, and the catch-all that used to be one flat arm is now, structurally, "the
readiness fallback" and "the dispatch fallback" that happen to say the same word today. A
caller who needs the two catch-all cases apart has `Layer` itself; the CLI's word was never
the only way to ask, it was just the only place the question had an answer at all.

### Where the change actually landed, against what the item said

The item this record closes named `crates/host/nomos-cli/src/work.rs` as the CLI's territory.
`work.rs` is the command dispatcher for `nomos work`; the word-producing match arm — what this
record calls `Refusal_Label` — lives in its submodule `crates/host/nomos-cli/src/work/report.rs`,
reached from `work.rs` by `mod report;`. This is the same shape the item's own text had already
found once on the ledger side (`ClaimRefusal` living in `claim/refusal.rs` rather than the
`exclusion.rs` an earlier reading of this item named): a crate split after a territory
description was written moved the code the description meant to point at, one directory level
down, without moving the description. `report.rs` is where the fix had to go, and this record
is where that is written down for the next reader who greps this item's history and finds
`work.rs` empty of the thing they are looking for.

Exposing `RefusalLayer` outside `nomos-ledger` also required adding it to two re-export lists —
`crates/substrate/nomos-ledger/src/claim.rs`'s `pub use refusal::{ClaimRefusal, RefusalLayer};`
and `crates/substrate/nomos-ledger/src/lib.rs`'s `pub use claim::{Claim, ClaimRefusal,
RefusalLayer};` — neither of which is new logic; both are one name added to an existing list a
new public item in an already-exported module always has to join.

## What Holds It

`crates/substrate/nomos-ledger/src/claim/refusal.rs`'s `layer_tests` module, over the whole
refusal set rather than the two variants a caller happens to remember — the shape
`OD-CONTRACTS-002` used for `Applicability`:

- every `Readiness` variant is asserted a plan fact, every `Dispatch` variant a coordination
  fact, by name;
- no refusal answers both `Is_Readiness` and `Is_Dispatch` — the disjointness assertion, run
  over a hand-written universe of one instance per variant;
- the universe itself is matched with no wildcard arm, so an eleventh variant stops the build
  at the list it has to be added to, not silently falling through a catch-all;
- `Layer` and `Is_Retryable` are shown independent on the pair that proves it — `HeldBy`
  (`Dispatch`, retryable) and `DependencyUnmet` (`Readiness`, retryable) — so the axis this
  record adds is not mistaken for the one that was already there.

`tests/contract/surface/nomos-ledger.txt` carries the four new public items
(`RefusalLayer`, its two variants, and `ClaimRefusal::Layer`/`Is_Readiness`/`Is_Dispatch`),
blessed from the crate's own source rather than hand-typed.

`cargo test -p nomos-ledger -p nomos-cli -p nomos-contract-tests` is green except for two
`records_do_not_serialize::snapshot_grain` tests that assert a property of the *live board* —
whether two open items currently widen different crates' APIs independently — and fail
identically on `HEAD` before this change, for board composition this item's territory does not
reach.

## What This Record Does Not Decide

It does not build the scheduler `ARC-HARNESS-001` describes, or wire `Layer` into anything
that dispatches work without a human running `nomos work claim`. It makes the two questions
askable in the type system; deciding *what* asks them is `P11-NEXT-WORK`'s territory, named as
such in `ARC-HARNESS-001`'s own "what this record does not decide" section.

It does not change what any `nomos work` command prints. `Refusal_Label`'s four named words and
its `snagged` fallback are byte-for-byte what they were; only the code that arrives at them
changed shape.

## What Was Considered And Rejected

**A boolean, `Is_Plan_Fact`.** Rejected on the item's own `done_when`: "not closed by a derived
boolean only the CLI consumes." A single bool is one bit narrower than `RefusalLayer` for no
saving — the enum costs nothing extra to match on and leaves room to name a third layer later
without every caller's boolean becoming a lie.

**Splitting `ClaimRefusal` itself into two nested enums** (`ClaimRefusal::Readiness(..)` /
`ClaimRefusal::Dispatch(..)`), so the layers are distinguishable by Rust's own type checker
rather than by a method call. Rejected for this item: every construction site —
`store/refusal.rs`'s `Contested_By`, `Unmet_Dependency`, `Held_Or_Unready`, `Wrong_Verb`,
`store/claiming.rs`, `finish/running.rs` — sits outside this item's territory, and restructuring
the enum's own shape would have to touch every one of them or leave the crate not compiling.
`Layer` gets the same caller-facing answer — "which of two things is this" — without moving
territory that belongs to work this item was not authored to also carry.

**Naming the layers after this codebase's own vocabulary** (`Plan` / `Resource`, `Static` /
`Live`) rather than `ARC-HARNESS-001`'s. Rejected: that record already named the two owners for
this exact seam — "the scheduler decides what should run, coordination decides whether it can
run now" — and a second pair of names for the same distinction is the kind of drift a reader
five records later has to reconcile by hand.
