---
id: OD-LEDGER-019
type: decision
title: Abandoning ends a claim and declining ends an item, so one verb cannot do both
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - concurrency
  - history
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-LEDGER-006
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
  - target: OD-LEDGER-014
    type: relates-to
  - target: OD-LEDGER-018
    type: relates-to
---

# Abandoning ends a claim and declining ends an item, so one verb cannot do both

## Question

`ItemState::Declined { reason }` is declared, is counted terminal beside `Done` by
`ItemState::Is_Finished`, is filtered by `nomos work list --state declined`, and is produced
by nothing. The one item in it, `P3-RESTORE`, was written into the file by hand.

So when an item turns out not to be work — the successor already landed it, or its own
governing record refuses its last clause by name — the only verb that touches it is
`abandon`, and `abandon` returns it to `Ready`. The board then offers it again.

`P10-DECLINE-VERB` asks which of two things closes that: a verb that ends the item, or
`abandon` widened with a flag saying which way the release ends. If it is a new verb, two
further questions have to be answered rather than left to whichever path the first caller
takes — whether declining records who did it, and what it does to an item somebody is
holding.

## What Was Measured

The board at `80f679a`: 107 items, 20 abandonments across 19 items. Every abandonment
classified by the state its item is in now, which is the outcome the release produced rather
than a reading of its prose.

| Abandonments | Item's state now | What the return to `Ready` produced |
|---|---|---|
| 18, on 17 items | `Done` | the work was still wanted; somebody took it and finished it |
| 2, on 2 items | `Ready` | the work was not wanted; the board is still offering it |

**Abandon's return to `Ready` is correct for 18 of its 20 recorded uses.** The verb is not
broken and is not what this record changes. What the board has no verb for is the other two,
and they are not a weaker case of the same thing — they are a different fact reached through
the only door that was open.

The two are `P10-REQUIRABLE-DECLARED` and `P10-DERIVED-FACT`. The first is the one where the
cost was paid in full, and it is the only subject on the board that has been claimed after it
was superseded:

| unix | event |
|---|---|
| 1786380919 | `claude-requirable-set-w33` abandons `P10-REQUIRABLE-SET`: premise measured too pessimistic, territory cannot be widened under claim, re-authored as `P10-REQUIRABLE-DECLARED` |
| 1786381395 | `claude-requirable-declared-w34` abandons `P10-REQUIRABLE-DECLARED`: its own new check measured the premise false, re-authored as `P10-REQUIRABLE-FLOOR`. **The item returns to `Ready`** |
| 1786382075 | `P10-REQUIRABLE-FLOOR` finishes, predicate exit 0, gate exit 0 |
| 1786382440 | `claude-requirable-declared-w62` abandons `P10-REQUIRABLE-DECLARED` a second time, having checked its four clauses one by one and found three landed by the successor and the fourth refused by name in the alternatives table of `OD-PROJECT-002`. **The item returns to `Ready`** |

1,045 seconds separate the abandonment that superseded the item from the end of the run that
took it anyway, and 365 of those seconds were spent after the successor was already verified
`Done`. Nothing was written to the tree under that second claim, because there was nothing
to write. Its reason ends:

> This should be Declined, and there is no verb that can decline it.

That is the measurement. One subject became three items, and a session's whole run went on
establishing that the second of them was already finished — a fact the board had, in the
abandonment reason, and could not show in the column anybody reads before claiming.

The state census that explains why, taken across the workspace excluding test modules:

| `ItemState` variant | Written by |
|---|---|
| `Ready` | `work.rs:277` at `add`, `item.rs:450` at construction, `exclusion.rs:253` at `abandon` |
| `Claimed` | `store.rs:934` at `claim` |
| `Done` | `exclusion.rs:248` at `finish` |
| `Blocked` | nothing |
| `Declined` | nothing |

Two of five states cannot be produced, and both are still read: `list --state` filters on
them, `Is_Finished` counts `Declined` terminal, and `store.rs:563` validates that a `Blocked`
item carries its reason. The ledger can describe two conditions it cannot enter.

## The Decision

### 1. Abandoning ends a claim; declining ends an item

They differ in subject, not in degree. `abandon` is a statement about a *claim*: this holder
stopped. The item is untouched by it — it goes back on the board because the work is still
wanted, and the 18 measured cases are that working. `decline` is a statement about the
*item*: this will not be done by anybody. Nothing about a holder is being said.

So the reason each carries answers a different question. Abandon's reason answers *why did
you stop*, and the next claimant reads it as context. Decline's reason answers *why is this
not work*, and there is no next claimant. Merging them makes an item's history a list of
sentences whose question the reader has to guess.

### 2. It is a verb, not a flag on `abandon`, and the measurement decides it rather than taste

Three independent reasons, in the order they close the question.

**Eighteen of twenty.** A flag on `abandon` puts a decision in front of every release,
including the eighteen where the answer has never been anything but the default. The
frequent case pays for the rare one, and the rare one is not made any more visible: a flag
somebody forgot leaves the item exactly where it is today.

**The precondition contradicts the purpose.** `abandon` requires an active claim by the named
holder: `FileLedger::Release` refuses an unclaimed item as `NotClaimable { state:
"unclaimed" }` and one held by somebody else as `HeldBy`. Both items that need declining are
unclaimed and have been since the runs that superseded them released. Reaching
them through a widened `abandon` would mean claiming work nobody intends to do in order to
record that nobody intends to do it, and the claim itself excludes territory for as long as
it is held.

**The two writes are not the same kind of write.** `item.rs` already draws this, above
`Abandonment`: `Declined` carries its reason because it is part of a *state*, and states are
what get written down, whereas an abandonment is a *transition* that leaves nothing behind
unless a field on the item is given the job of holding it. A single verb switching between
those on a flag is one verb with two return types wearing one name.

### 3. `decline` takes a holder, and who and when survive on the item

`OD-LEDGER-006` decided the general form — a transition is not persisted; whatever it carried
is gone unless something on the item holds it — and this is that rule applied to the new
transition rather than a second answer to it.

`ItemState::Declined { reason }` keeps its reason unchanged. The reason is the state's own
content and the thing that makes the state unreachable reasonlessly through the type, which
is the property `item.rs` built it for. What a state cannot carry is *who* and *when*,
because those are facts about a transition and a state is not one.

So `LedgerItem` gains `declined`, holding the holder and the timestamp, and **not** the
reason. Two copies of one fact is the defect this repository has an operating hazard about at
document scale and would be no better at field scale; a reader asking why goes to the state,
and a reader asking who goes to the item, and neither can be told two different things.

`P3-RESTORE` reads as declined with nobody named, which is exactly true: it was declined
before a verb existed and there is no holder to attribute it to. That is the same treatment
`abandoned` and `displaced` got when they arrived — a fact about the items written before the
field, rather than something to backfill.

### 4. A live claim refuses; `Done` and `Declined` conflict

The third question, answered rather than left to the first caller.

**An item with an active claim is refused, retryably, naming the holder.** The remedy is two
commands from the person who has the facts: `abandon`, then `decline`. `OD-LEDGER-001` is
why. Territory is declared and not enforced, so the ledger's answer *is* the exclusion — and
ending a live claim from outside would be the one transition where the board overrules the
only party who knows whether the work is still running. Retryable is the honest code: the
claim will lapse or be released, and the caller should come back rather than fetch a person.

**An item already `Done` is a conflict.** Declining it would put prose where a verification
record's verdict is, and `finish` exists precisely so that a `Done` item cannot claim a check
that never ran.

**An item already `Declined` is a conflict, and the refusal reports the reason it carries**
rather than replacing it. A second decline is either a duplicate or a disagreement, and both
are for a person.

Every one of these names the item and what stopped it, per `OD-LEDGER-014`, and every one
lands on an exit code the README's table already defines. No new code is introduced.

### 5. The schema version moves, and that is the guard working

`declined` is a key on a serialized item, and
`Test_A_Field_Added_To_An_Item_Should_Raise_The_Schema_Version` counts those keys, so
`SCHEMA_VERSION` goes to 3. This is not incidental: `OD-LEDGER-008` chose deny-unknown-fields
over a discretionary bump exactly so that a forgotten bump could only degrade a message and
never cost a field, and the counting test is what makes the bump non-discretionary.

The operational consequence is stated here so nobody has to discover it. Every binary copied
before this lands refuses every ledger verb with the store-error code and the sentence naming
both schemas, until it is rebuilt and re-copied. That is the designed behaviour and the
alternative is the one `OD-LEDGER-008` was written after: a writer that does not understand
the document writing it anyway, at exit 0, dropping the field.

### 6. `Blocked` is left unreachable, deliberately and not silently

It is the other state nothing writes. It is outside what `P10-DECLINE-VERB` was asked and it
is not fixed here, and the reason for saying so rather than saying nothing is that a variant
nobody can produce is a promise the type makes and the tool does not keep. `Declined` was that
promise for the whole life of this ledger and cost a session's run to notice. `Blocked` is
still it, `store.rs:563` still validates a reason for it, and the next reader should find that
written down rather than measure it again.

## What Was Considered And Rejected

**`abandon --declined`, a flag on the existing verb.** Refused in decision 2 on three
independent grounds, the first of which is measured: it taxes 18 releases to serve 2, its
precondition requires claiming work in order to say the work is not wanted, and it makes one
verb write both a state and a transition record. It is the cheapest change and it is cheap
because it moves the question rather than answering it.

**Deleting the superseded item from the ledger.** This is what a hand edit would most likely
do, and it destroys the abandonment reason — which for `P10-REQUIRABLE-DECLARED` is the only
written account of why `P10-REQUIRABLE-FLOOR` exists and what its census corrected, and for
`P10-DERIVED-FACT` is the only account of why `P10-DEPENDENT-EDGE` needs the territory it
reserves. `OD-LEDGER-006` exists to stop exactly that loss one scale down.

**A `work edit` verb to narrow the superseded item's territory instead.** It answers a
different question. Both items are correctly scoped for the work they described; the work is
what stopped existing. Narrowing the territory leaves them `Ready` and the board still offers
them.

**Hand-editing the state, which is what happens today.** It writes a value no verb produces
and no transition records, so who declined it and when are gone at the moment of the edit —
the shape `OD-LEDGER-006` already settled — and it does it to shared coordination state that
other sessions are reading, which `OD-LEDGER-018` covers.

**Requiring the item to be claimed before it can be declined, for symmetry with `abandon`.**
Refused on the same measurement that refuses the flag: the two items this exists for are
unclaimed, and a precondition that forces a claim in order to close something makes the verb
unusable for its own motivating case.

## What This Does Not Do

- It does not change `abandon`. Its reason, its list, and its return to `Ready` are correct
  and measured correct, and the 18 cases stay on the path they are on.
- It does not make `Blocked` reachable, and decision 6 says so rather than leaving a reader to
  find it.
- It does not delete or rewrite any superseded item. `P10-REQUIRABLE-DECLARED` and
  `P10-DERIVED-FACT` are declined through the verb, keeping every abandonment reason they
  already carry.
- It does not let the board end work somebody is holding. That refusal is decision 4 and it is
  the point of the verb having a precondition at all.

## Controls

| Weakening | What it produces |
|---|---|
| a flag on `abandon` instead of a verb | a decision in front of 18 releases that never needed one, and a forgotten flag leaves the item exactly where it is today |
| decline without a reason | the state `item.rs` built to be unreachable reasonlessly, reached reasonlessly through the CLI |
| decline recording only the reason, not who and when | `OD-LEDGER-006`'s loss, re-installed on the one transition added after it was decided |
| decline overriding a live claim | the board ending work whose only informed party is the holder, on ground `OD-LEDGER-001` says nothing enforces |
| decline overwriting a `Done` item | prose replacing a verification verdict, which is what `finish` exists to prevent |
| add `declined` without moving `SCHEMA_VERSION` | a stale binary dropping the field at exit 0 — the `OD-LEDGER-008` defect, on the field this record adds |
| leave the state unreachable and hand-edit as needed | measured: one subject, three items, and a whole session run spent re-establishing that the work was already done |

## Status

Closed by `P10-DECLINE-VERB`.
