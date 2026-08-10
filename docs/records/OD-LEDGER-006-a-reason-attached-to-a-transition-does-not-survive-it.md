---
id: OD-LEDGER-006
type: decision
title: A reason attached to a transition does not survive it, and a reason attached to a state does
status: accepted
version: 2
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

*(Amended at version 2. This paragraph said the claim is "still on the item", which stopped
being the whole truth once an item could be taken over: something has to replace it.
`OD-LEDGER-012` moves it to `LedgerItem::displaced` rather than dropping it, and moves the
claim itself rather than a summary of it — so what this paragraph protects is unchanged and
only where it points has moved. No `reason` field is created for a lapse there either, for the
reason given above: nobody was present to write one. See the amendment note below.)*

## What This Found And Did Not Fix

A lapse does not make the item takeable again.

*(Amended at version 2. That sentence is superseded: a lapse still does not make an item
**claimable**, and `claim` still refuses it, but `OD-LEDGER-012` makes it **takeable** by a verb
of its own, `nomos work takeover`. The rest of this section is the measurement that opened
`P10-LAPSE-TAKEOVER` and is kept as it was written. See the amendment note below.)*

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

*(Amended at version 2. That decision is now made, in `OD-LEDGER-012`, and it is neither of the
two shapes this paragraph anticipated: a lapse does not return the item to `Ready`, and what it
costs an agent that is merely slow is bounded by that agent's own lease rather than by a policy.
The test did exactly what this paragraph says it was for — it is renamed and its refusal
assertion re-pointed, and the assertion that the lapsed claim was not replaced is kept word for
word. See the amendment note below.)*

## Consequences

Every item written before this field existed has no abandonments, and `#[serde(default)]`
leaves it that way. That is a fact about those items rather than something to backfill.

The reason is prose, and prose is not queryable. `Blocker` is a typed enum precisely
because storing "why" as a sentence made it impossible to ask how much of the backlog was
waiting on a person. The same objection applies here and is accepted rather than answered:
an abandonment is a narrative addressed to the next agent, and the shapes worth counting
are not yet known. If they turn out to be countable, they can be typed later against real
examples instead of guessed at now.

## Amendment, Version 2

`P10-LAPSE-TAKEOVER` reopened this record deliberately, which its own `done_when` required of
it: *"`OD-LEDGER-006` is reopened deliberately or it is not touched."* `OD-LEDGER-012` carries
the new decision and this note records what moved here.

**What changed is where a lapse's evidence lives once something displaces it.** This record
decided that the evidence of stopped work must survive, and it left a lapse's evidence in the
one place that needed no arranging: the claim, still on the item. A takeover needs that slot, so
the claim moves to `LedgerItem::displaced` — a list, oldest first, holding each displaced claim
exactly as it stood. `LedgerItem::Replace_Lapsed_Claim` performs the move and the install as one
operation, which is this record's own `ReleaseOutcome::Record_On` technique applied to the field
it created: an implementation free to spell one half of a rule at its call site eventually
spells one half.

**What did not change is the rule that put it there.** A lapse still gets no synthesized
reason, because there is still nobody who gave one. `Abandonment::reason` remains the words the
holder gave, `displaced` has no `reason` field to fill, and the criterion this record
established — a transition's evidence survives only where something on the item is given the job
of holding it, and only what somebody actually knew at the time is held — is what chose the
shape of the new field rather than being weakened by it. `OD-LEDGER-012` states that criterion
in this record's terms and applies it to a transition nobody was present for.

**What was found here and deferred is closed.** The section above measured that a lapsed item
was unreachable by anybody and said so rather than fixing it, on the ground that the fix was a
different decision. That was correct and the deferral held for exactly as long as it should
have: the decision is made in a record of its own, and the behaviour this record's test declined
to pin is now pinned by that record's tests.

The decision this record is *about* is untouched. A reason attached to a state survives and one
attached to a transition does not, and that is why this record is amended in place rather than
superseded — a `superseded_by` edge would tell a later reader the whole of it is dead.

## Status

Accepted. Implemented in `nomos-ledger` and reported by `nomos work show`.

Amended at version 2 by `OD-LEDGER-012`, which makes a lapsed item recoverable and needed
somewhere to put the claim it replaces.
