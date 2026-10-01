---
id: OD-LEDGER-005
type: decision
title: A listing labels an item with the refusal claiming would give, not with its state field
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - enforcement
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-LEDGER-004
    type: relates-to
---

# A listing labels an item with the refusal claiming would give, not with its state field

## Question

`work list` printed `ready` for items nothing could claim. Is the enforcement missing, or is
the report wrong?

## The Report Was Wrong

`depends_on` is enforced. `Claim` walks an item's dependencies and returns
`ClaimRefusal::DependencyUnmet`, retryable, exit 3. Territory is enforced too. Neither was
the defect.

`List` read `item.state` and printed the word it found there. `Ready` means *nobody holds
this*, and every reader takes it to mean *I can take this*. Those are different claims, and
they came apart whenever a dependency was unfinished or somebody held overlapping ground.

Measured on 2026-08-09, four times. The widest: one claim on P10-FIRST-CHECK refused all
eight remaining items, and the column called all eight `ready` throughout. An agent picking
work off that column burns a round trip per item and then learns to distrust the column,
which is worse than the round trip.

## One Implementation, Two Callers

The obvious repair — teach `List` when an item is claimable — is the wrong one. Two
implementations of one rule is how they come to disagree, and a listing that disagreed with
claiming would be worse than one that said too little: the first is a wrong answer, the
second is only an incomplete one.

So the refusal logic moved out of `Claim` into `Claim_Refusal(document, item, now)`, which
returns the first reason a claim would be refused, in the order `Claim` refuses in. `Claim`
is now one of its callers and the listing is the other. There is no second guard, and there
is no way for the column to drift from the behaviour it describes, because the column *is*
the behaviour.

The extraction is behaviour-preserving: `Claim`'s refusals were unchanged and every existing
test of them still passes, which is the condition the item set.

## The Words

`ready` now means claimable now. An item that is not gets the reason instead:

| label | what it means | retryable |
| --- | --- | --- |
| `waiting` | a dependency is unfinished | yes — finishing it resolves this |
| `held` | somebody holds overlapping territory | yes — waiting resolves this |
| `snagged` | independence could not be established | no — somebody must close a modelling gap |

`waiting` rather than `blocked`, because `Blocked` is already a state an author sets by hand
with a `Blocker` explaining why. Reusing the word would have merged a judgement somebody made
with a fact the ledger computed.

`snagged` exists because `ClaimRefusal::UnknownIndependence` is not retryable and reporting it
as `ready` would send an agent to discover that by being refused — which is the same defect
this record is about, in the one case where waiting does not help.

Filtering follows the column: `--state ready` no longer answers with items nothing can claim.
That was the query the defect actually cost round trips on.

## Scope

The item asked only about dependencies. Territory is reported too, because it falls out of
the same call — `Claim_Refusal` returns whichever refusal comes first, and suppressing half
of its answer to stay inside the item's wording would mean deliberately printing `ready` for
something the function had just said was held. The territory case is also the one that was
measured widest.

## The Guard

`crates/host/nomos-cli/tests/list_tells_the_truth.rs`, six tests against the real binary
through `NOMOS_WORK_DIR`, because the defect was never in the exclusion logic and a test that
called a function would have been testing the half that already worked.

**Confirmed red.** Reverting the label to the state field fails exactly three of the six —
the unfinished dependency, the held ground, and the `--state ready` filter — while the three
controls stay green. That split is the evidence: the guards fire on the defect and not on
everything.

The controls are what stop the repair overshooting. An item with no dependency is still
`ready`; an item on free ground is still `ready`; and — the sharp one — an item whose
dependency is *present and satisfied* is still `ready`, which a label driven by "does this
item have a `depends_on` entry" would get wrong while passing the other two.

## Status

Accepted. The column now says what claiming will do, because it asks the same function.
