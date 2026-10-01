---
id: OD-LEDGER-020
type: decision
title: A declined dependency is a dead end, and a dependent may not be told to wait
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - dependencies
relations:
  - target: OD-LEDGER-005
    type: relates-to
  - target: OD-LEDGER-014
    type: relates-to
  - target: OD-LEDGER-019
    type: relates-to
---

# A declined dependency is a dead end, and a dependent may not be told to wait

## Question

`nomos work list` and `claim` both refuse an item whose `depends_on` names an unfinished
item with `ClaimRefusal::DependencyUnmet`, labelled `waiting` and reported retryable. The
refusal and the label are correct for exactly the case they were built for: a dependency
that has not finished yet, where finishing it is what resolves the refusal.

`ItemState::Declined` is a third possible state for that dependency — one `OD-LEDGER-019`
made reachable — and it is not covered by that case. A declined item is terminal by
`ItemState::Is_Finished`, and finishing it is not merely unlikely, it is precisely what will
not happen: the item is not going to be done by anybody. `Unmet_Dependency` compared the
dependency's state against `Done` and reported everything else identically, so a declined
dependency and an in-progress one produced the same refusal, the same `waiting` label and
the same `Is_Retryable() == true`.

## What Was Measured

The board carries a live instance, not a hypothetical one. `P10-EDGE-CONSTRAINTS` depends on
`P10-REQUEST-LAYOUT`, and `P10-REQUEST-LAYOUT` was declined — superseded by
`P10-SUBMISSION-LAYOUT`, itself now `Done` — with its own declination reason naming the
consequence directly:

> One consequence is left behind deliberately rather than hidden: `P10-EDGE-CONSTRAINTS`
> depends on this item and is now stranded, because a declined dependency is still reported
> as `DependencyUnmet`, which `Is_Retryable` calls retryable. That is a dead end reported as
> a queue, it is the first time a declined item has had a dependent, and it has its own item.

`nomos work list` reads `P10-EDGE-CONSTRAINTS` as `waiting` today, on that dependency, and
the advice the label carries — retry later, `P10-REQUEST-LAYOUT` will finish — is false. It
will not finish; it is closed. The only way this resolves is a person or a session reading
the dependency's own history and re-pointing the edge, and nothing on the board says so.

## The Decision

### 1. A declined dependency refuses through its own `ClaimRefusal` arm

`ClaimRefusal::DependencyDeclined { item, dependency, state }` is added beside
`DependencyUnmet`, carrying the same three fields for the same reason `OD-LEDGER-014` already
settled for the sibling arm: the dependent, the dependency, and a bounded rendering of what
stopped it. `store::refusal::Unmet_Dependency` checks `ItemState::Declined` before the
generic not-`Done` comparison, so a declined dependency is caught before it reaches the arm
built for a merely unfinished one.

### 2. It is not retryable, and the label says so

`ClaimRefusal::Is_Retryable` reports `false` for the new arm, by omission from its `matches!`
list rather than by a new negative case — the same shape the type already uses for every
other non-retryable arm. `nomos work list` and `work audit` share one function,
`Listing_Label`/`Refusal_Label`, per `OD-LEDGER-005`'s reason for there being only one
implementation: `stranded` is the word, chosen because it is the word the measured instance's
own declination reason already used to describe the fact, and because `waiting`, `held` and
`snagged` are all already spoken for by refusals whose actual advice differs from this one's.

### 3. The refusal names the remedy, not just the dead end

`OD-LEDGER-014`'s discipline — the refused item is never the grammatical subject, so the
sentence composes under a caller that has already named its subject — is followed, and the
sentence goes further than `DependencyUnmet`'s does: it states what the caller is to do next,
because "not retryable" without a next step is a dead end the reader has to solve themselves.
No verb re-points an existing `depends_on` edge, so the only remedy is `nomos work decline`
on the dependent followed by re-authoring it against a dependency that can still finish. The
sentence says exactly that.

### 4. The dependency's state is rendered bounded, not with `Debug`

`Unmet_Dependency` used to build its `state` string by formatting both the target and the
comparison value with `{:?}` and comparing strings. `ItemState::Declined` carries its whole
reason in the variant, and `Debug` prints all of it — `ItemState::Describe` already exists
for exactly this, cut to the reason's first line, and this record's fix uses it for both
arms rather than only the new one. A comparison by `PartialEq` replaces the string
comparison it stood in for; the two were never anything but a roundabout way of asking the
same question the derived trait already answers.

### 5. `P10-EDGE-CONSTRAINTS` is re-pointed, not merely diagnosed

The item that motivated this record is not left as a citation. `P10-SUBMISSION-LAYOUT`, the
dependency `P10-REQUEST-LAYOUT`'s own declination named as its successor, is confirmed `Done`.
`P10-EDGE-CONSTRAINTS` is declined and re-authored, depending on `P10-SUBMISSION-LAYOUT`
instead of the declined `P10-REQUEST-LAYOUT`, with its territory, `why` and `done_when`
otherwise unchanged — the work it describes was correctly scoped throughout; only the edge
that stranded it was wrong.

## What Was Considered And Rejected

**Routing a declined dependency through the existing `NotClaimable` variant.** Checked and
found to reproduce the defect: `Listing_Label` only reads `NotClaimable` for the *refused
item's own* state, and for every other subject it falls through to `Refusal_Label`, whose
`NotClaimable` arm names a *different* item's state as its own — a declined dependency routed
this way would print the dependent's own `Ready` state, restoring exactly the `waiting`-that-
lies-by-omission failure this record exists to close, one layer further hidden.

**Reusing `DependencyUnmet` with a `declined: bool` field instead of a new arm.** Rejected on
the same grounds `OD-LEDGER-019` decided the analogous question for `abandon`/`decline`: the
two facts answer different questions — one says wait, the other says stop waiting — and a
single arm branching on a field is one refusal wearing two meanings, discoverable only by a
caller that reads the field rather than matches the type.

**Adding a `work re-point` verb that edits `depends_on` in place.** Out of scope for the
defect this record closes, which is that the refusal lies about retryability — not that the
remedy takes two commands instead of one. Worth having on its own merits, not decided here.

## Controls

| Weakening | What it produces |
|---|---|
| leave `DependencyUnmet` covering both cases | the measured defect: a dead end reported as a retryable queue |
| route through `NotClaimable` | the dependent's own state printed as the reason, reproducing the bug one layer hidden |
| a `declined: bool` field on the existing arm | one refusal answering two different questions, `OD-LEDGER-019`'s shape recurring |
| `Debug` instead of `ItemState::Describe` for the rendered state | a multi-paragraph decline reason as the body of a one-line refusal |
| leave `P10-EDGE-CONSTRAINTS` pointed at the declined item | the record diagnosing the defect without closing the instance that motivated it |

## Status

Closed by `P10-DECLINED-DEPENDENCY-4`.
