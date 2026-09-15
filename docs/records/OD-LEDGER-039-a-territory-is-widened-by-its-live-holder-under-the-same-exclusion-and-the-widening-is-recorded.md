---
id: OD-LEDGER-039
type: decision
title: A territory is widened by its live holder under the same exclusion, and the widening is recorded
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - territory
relations:
  - target: OD-LEDGER-008
    type: relates-to
  - target: OD-LEDGER-012
    type: relates-to
  - target: OD-LEDGER-015
    type: relates-to
  - target: OD-LEDGER-019
    type: relates-to
  - target: OD-LEDGER-024
    type: relates-to
  - target: OD-LEDGER-038
    type: relates-to
---

# A territory is widened by its live holder under the same exclusion, and the widening is recorded

## Question

An item's territory is authored before the change it reserves has been attempted. It is
therefore a prediction, and a prediction is sometimes wrong by one file.

Until this record the ledger had no proportionate answer. Territory is immutable once claimed
and there was no verb to enlarge it, so a holder who found their reservation short by one
legitimate path had exactly one route: abandon the claim, decline the item, re-author it with
the missing path, and re-author every dependent the decline stranded, because a declined
dependency never becomes `Done` and `OD-LEDGER-020` correctly calls its dependents `stranded`
forever.

The immutability is not the defect and must survive. A holder who could rewrite their own
reservation could take a path a peer is actively holding, and the exclusion check the board
exists for would never be asked. What is wrong is that the refusal was total rather than
proportionate: the ledger answered a request to add one path by ending the item.

There is a second loss and it is the one that compounds. The board cannot say how often a
predicted cone escapes its reservation. A decline stores a holder, a timestamp and free
prose, so the number is not derivable and any stated one would be a figure nobody could
check. A widening refused into a decline destroys the evidence at the moment it is produced.

## What was measured

**One item, three declines, in one session on 2026-09-14.** The subject was a shared
eligibility threshold in the specification crates.

The first escape was a crate root. The helper the item's own rule had to call sat in a private
module, so no other crate could name it, and the file carrying the module declaration was not
reserved. The second was two fixtures whose bodies the change made vacuous: leaving them would
have left two tests passing for a reason that was no longer the one they claimed, and editing
them was outside the reservation. Neither file was reachable by grepping what the predicate
reads. Both appeared only when the change was run. The third attempt reserved what an actual
build and test run named, and held.

The cost was not the editing. It was five ledger transitions and a dependent re-authored
twice, for work whose only defect was that one file it legitimately had to touch could not be
predicted before the attempt.

**A second cause, and the author was not wrong at all.** The decline of `P11-AMEND-TERRITORY`
records a reservation that was correct when authored and became false afterwards: `ef1c93f`,
*Give every substrate type its own file*, moved the type whose wording that item's `done_when`
required it to change. The same decline states the population rather than the instance — three
refactor commits split 49 files into 99, and eight open items reserved a file that was split.
No prediction discipline reaches that case, because there was no prediction error.

**The count of items whose terms rather than whose territory were wrong is different, and
deliberately not addressed here.** Thirteen declines on this board cite the item's own wording;
about eight are genuinely work that was right and terms that were not. Every one of those was
a `Ready`, unclaimed item, where re-authoring costs a board entry and nothing else. The
expensive case is the claimed one, which is this record's.

## The decision

### 1. A live holder may enlarge the territory of an item they hold

`nomos work widen --item <id> --holder <name> --territory <path> …`, a fifth verb that changes
the board.

It **only ever adds**. There is no argument that can express a replacement territory, and that
is the point rather than an omission: dropping a path drops the `done_when` clause that path
carried, so narrowing must not be one typo away from a holder who meant to add one file. A
path the territory already reserves contributes nothing, is not recorded as added, and is
reported as such — compared after `Normalize_Path`, so two spellings of one path are one path
here exactly as they are to `Territory::Intersect`.

### 2. Authorization is a live lease, not a matching name

A widening is refused when the item has no claim, when the item has ended, when the caller is
not the holder, and — the case that needs stating — when the holder's lease has run out.

A lapsed claim stops excluding. `store/refusal.rs` already records that for takeovers: another
item may since have been claimed over exactly the files this one reserves. A lapsed holder
enlarging a reservation that currently excludes nobody is that hazard reached through a new
verb, and their name still matching is precisely what would make it look permissible. The
remedy named is `takeover`, which is the verb that makes the claim live again.

### 3. The exclusion is the claim's own, re-run against the board as it stands

The enlarged territory is checked by `Held_Ground` — the same function a claim is refused by,
called and not copied, so a widening cannot come to a different answer about contention than a
claim would. The check is run over the territory the item *would have*: the target is cloned,
the widening is applied to the clone, and the result is handed to the check. Asking instead
whether each added path collides would be a second notion of what a widening produces, free to
disagree with the one that applies it.

It is re-run at widen time and never inherited from the claim. Independence was established
when the claim was granted and the board has moved since.

### 4. Deciding and writing are one mutation under one lock acquisition

Load, holder check, enlargement, exclusion, history append, validation and write happen inside
a single acquisition of the cross-process lock, through the same guarded path the fourth
board-changing verb uses. `OD-LEDGER-015` is the record of what the three verbs before it cost
by deciding outside it.

Sharing the exclusion function is necessary and is not the property. Two widenings running at
once can each observe a board on which their own added paths are free and jointly produce the
overlap that check exists to prevent, and nothing about sharing the function prevents it.

### 5. The enlargement is recorded, not merged in silently

`LedgerItem::widened` keeps, per widening, which paths were added, by which holder, at what
moment. The added paths rather than the territory afterwards, which the item already carries:
what cannot be recovered from the item is which paths this particular widening contributed.

A list and not a single row, for the reason `abandoned` and `displaced` are lists. An item
widened twice was widened twice, and coalescing two rows into one would leave the territory
correct while destroying the difference between one bad prediction and a reservation that was
never a serious attempt.

This is the measurement. How often a predicted cone escapes, and by how much, is answerable
from these rows and from nothing else on the board.

### 6. The compatibility of the new field has two directions and two mechanisms

Stated separately because attributing either to the other is how a guard comes to rest on
something that does not hold it.

An **older build meeting a newer file** is stopped by `#[serde(deny_unknown_fields)]` on
`LedgerItem`, which refuses the parse on a key it does not know. This is mechanical and applies
to any field ever added.

A **newer build meeting an older file** is stopped by the *absence* of `#[serde(default)]` on
the new field. That is what forces the board to be migrated rather than silently read, and it
is why the field carries none: an item written before this verb existed may well have been
widened by the only means there was, which was to decline it and re-author it with more paths.
Reading such a row as never widened would be a claim about history the document cannot support.
`kind` and `origin` carry none for the same reason.

`SCHEMA_VERSION` does neither. It is consulted *after* a parse has already failed and decides
only whether the operator reads `Unrecognized` or `Malformed`. `OD-LEDGER-008` chose that
deliberately: a field once arrived without the number moving, so a guard resting on the bump
would have reported clean on the next instance of the defect it was built for. A forgotten bump
costs a sentence and cannot cost a field.

## What this record does not do

**It does not infer, validate or propose a dependency cone.** An author still predicts the
territory and still measures it. The recorded widenings are the only evidence that would ever
justify changing that, and there are none yet. A mechanism that computed the cone would be a
different decision resting on data this one is built to produce.

**It is not a licence to reserve loosely.** A claim is the author's best pre-work reservation
and a widening is an evidence-backed repair for the case where execution disproves that
prediction. It is not the first move of an incremental discovery of territory, and neither this
record nor the verb's own help text may read as an invitation to treat it as one. The
distinction is what keeps the history worth having: a board on which widening is routine has
lost the measurement rather than gained a convenience, because an escape rate says nothing
unless the reservations it is measured against were genuine attempts to get the territory
right.

**It does not make an item's terms editable.** `why`, `done_when`, the predicate and the
dependencies are unchanged by this verb, and the evidence for a general edit is weaker and
cheaper — see *What was measured*. A general mutable-item surface is not what a repeated
expensive failure justifies.

**It does not report a reservation that decayed under a peer's refactor.** The second measured
cause is real and its repair is detection, not another mutation: `work audit` says nothing
today about a reserved path that is no longer in the tree. That is filed separately.

## What was considered and rejected

| Alternative | What it produces |
|---|---|
| let the holder write a whole new territory | narrowing one typo away, and a dropped path drops the `done_when` clause it carried |
| skip the exclusion check for the item's own holder | the one refusal the verb has to earn, removed |
| check each added path instead of the enlarged territory | a second notion of what a widening produces, free to disagree with the one that applies it |
| decide under the lock and write after releasing it | two concurrent widenings each see a safe board and jointly create the overlap |
| authorize on the holder's name alone | a lapsed holder enlarges a reservation that currently excludes nobody, onto ground a peer may hold |
| union the paths without recording the widening | the territory stays correct and the measurement is destroyed |
| coalesce repeated widenings into one row | one bad prediction becomes indistinguishable from a reservation that was never serious |
| give the field `#[serde(default)]` | every item predating the verb reads as never widened, which the document cannot support |
| rest the stale-writer guard on the schema bump | a field once arrived without the bump; the guard reports clean on the next instance |
| infer the replacement territory and widen automatically | a cone computed from data this record exists to start collecting |

## Controls

| Weakening | What it produces |
|---|---|
| refuse every widening | the item is safe and the verb is pointless; the two granting tests are the control |
| grant every widening | territory stops excluding, which is the whole of what the board does |
| record the territory after instead of the paths added | a second shape that can drift from `Territory`, and the escape no longer recoverable |
| drop the lapse refusal | authorization survives the lease that granted it |
| report the request instead of what was added | a caller is told a path landed that was already there |

## Status

Accepted. Decisions 1 through 6 are closed by the commit that publishes this record; the
detection of a reservation that decayed under a peer's refactor is a separate filed item and
not a clause of this one.
