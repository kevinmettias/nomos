---
id: OD-LEDGER-006
type: decision
title: A reason attached to a transition does not survive it, and a reason attached to a state does
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - verification
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-SPEC-005
    type: relates-to
---

# A reason attached to a transition does not survive it, and a reason attached to a state does

## Question

`nomos work abandon` has always refused without `--reason`. `ReleaseOutcome::Abandoned`
has always carried `reason: String` non-optionally — the same technique the doc comment
directly above it praises the finished arm for, where carrying the `VerificationRecord` in
the variant is what makes a finished item structurally impossible without evidence.

Then `FileLedger::Release` matched it with `{ .. }` and set the state and nothing else, in
the arm adjacent to the one writing `verified = Some(record)`. One match, two arms, one
keeping its evidence and one discarding it.

So the question is not whether an abandonment should be recorded. Every layer above the
store already behaved as though it were. The question is why one of the two reasons this
ledger asks for survives and the other does not.

## What The Difference Actually Is

`ItemState::Declined` carries a reason, and its doc comment says why: so an item cannot be
declined reasonlessly. That reason survives every write, and nobody had to arrange it.

It survives because it is part of a **state**. States are what a ledger persists. A
transition is not persisted at all — it happens, it produces a new state, and whatever it
was carrying is gone unless something on the item was given the job of holding it. Nothing
had been.

That is the whole of the defect, and it generalizes past this field: any evidence a
transition carries needs somewhere on the record to land, or the type system's insistence
on carrying it buys nothing beyond the function call. The signature made the reason
impossible to omit and could not make it impossible to drop.

## Why It Inverted The Crate's Own Intent

`item.rs` says a lapsed claim does not release itself, and stays visible "so that a person
can see the work was abandoned rather than never started". The crate root says the same.
That is a deliberate decision to keep evidence of work that stopped.

Against it: an agent that died holding a claim left that claim behind, visible. An agent
that stopped deliberately — having been compelled to write down why — left nothing at all.
The claim went to `None`, the item went back to `Ready`, and afterwards the item did not
say it had ever been held.

The well-behaved path was the lossy one. That is worse than an oversight, because it
selects against the behaviour the ledger asks for: the more correctly an agent shut down,
the less of it survived.

Observed on 2026-08-09 releasing `P9-ONE-DIRECTION`. The reason given said where the work
had reached and why it stopped, and it survives only in the message of commit `a749f9a` —
the half-recorded shape `OD-SPEC-005` exists to name.

## The Decision

An abandonment is recorded on the item it happened to: who stopped, why, and when.

**It is a list, not a field.** An item abandoned twice was abandoned twice. Keeping only
the most recent would discard the earlier reason, which is this same defect one scale down.

**Both arms of a release are written once**, in `ReleaseOutcome::Record_On`, next to the
type that carries the evidence — the reason `Refusal_From` exists in the same crate. This
is not defensive: the defect *was* an implementation spelling the rule out for itself and
spelling one arm of it correctly. A second `ExclusionLedger` — the run-scoped and
session-scoped instances this crate's scope section anticipates — would have been free to
reproduce it exactly.

**It is reported, not merely stored.** `nomos work show` reports one item, because `list`
is a column per item and cannot carry prose. A record no surface reports is a record only
somebody willing to read the JSON will find, which is most of the way back to not keeping
it.

**Ending the claim is part of recording the abandonment**, rather than left to the caller.
An abandonment is a record of something that stopped; a record that went on excluding
people would be a worse defect than the one it replaced.

## The Lapsed Claim, Settled

A lapsed claim is given **no** synthesized abandonment.

Nobody was there to write a reason. Writing "the lease expired" would be inventing the one
field that exists to hold what a person actually said — filler wearing a record's clothes,
which this repository already refuses elsewhere for placeholder records.

What a lapse leaves is the claim itself, still on the item, which says who held it and
when they stopped. So both paths now leave a record, and they differ by what is genuinely
knowable rather than by which code path ran. That is the asymmetry that should exist.

## What This Found And Did Not Fix

A lapse does not make the item takeable again.

`item.rs` says a lapsed claim "stops excluding, which is what lets the next agent take the
item". The first half is true: `Has_Active_Claim` goes false, so the lapsed claim stops
being counted against *other* items' territory. The second half is not. A lapse leaves
`state` at `Claimed`, and `Claim_Refusal` rejects anything that is not `Ready` before it
ever looks at a lease — so the lapsed item itself can never be re-claimed until somebody
edits the file by hand. `MAXIMUM_LEASE` exists so that "an agent which died at lunchtime
does not hold territory until tomorrow", and for its own item that is not what happens.

Nothing asserted it. The two lapse tests check `Has_Active_Claim` and check that `Validate`
reports no overlap; neither takes the item.

It is left unfixed here deliberately. It is a different defect with a different decision
behind it — whether a lapse should return an item to `Ready`, and what that costs an agent
that is merely slow rather than dead — and it is on the ledger rather than in this
sentence. The test written here asserts the reason and the visibility and stops short of
the claimability, because an assertion either way would pin the behaviour in place before
that decision is made.

## Consequences

Every item written before this field existed has no abandonments, and `#[serde(default)]`
leaves it that way. That is a fact about those items rather than something to backfill.

The reason is prose, and prose is not queryable. `Blocker` is a typed enum precisely
because storing "why" as a sentence made it impossible to ask how much of the backlog was
waiting on a person. The same objection applies here and is accepted rather than answered:
an abandonment is a narrative addressed to the next agent, and the shapes worth counting
are not yet known. If they turn out to be countable, they can be typed later against real
examples instead of guessed at now.

## Status

Accepted. Implemented in `nomos-ledger` and reported by `nomos work show`.
