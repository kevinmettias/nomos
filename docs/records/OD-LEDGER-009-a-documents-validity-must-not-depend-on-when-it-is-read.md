---
id: OD-LEDGER-009
type: decision
title: A document's validity must not depend on when it is read, and a lapsed item is not claimable
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-LEDGER-006
    type: relates-to
  - target: OD-LEDGER-005
    type: relates-to
---

# A document's validity must not depend on when it is read, and a lapsed item is not claimable

## Question

`MAXIMUM_LEASE` exists so that an agent which dies holding a claim stops holding territory,
without anybody editing `work/ledger.json` by hand. `item.rs` says so at the definition of
the lapse check.

The lease expiring caused exactly what the lease exists to prevent. Document validation
treated `Claimed` with no *active* claim as a violation, and a lapsed lease is precisely
that state. `Save` refuses to persist an invalid document and every claim saves, so the
moment any lease ran out, every claim on the board failed — including claims on items
sharing no territory with the lapsed one. Reproduced by forcing one lease into the past and
then claiming an item with disjoint territory: refused.

The unit level was right the whole time, and that is what hid it. `Has_Active_Claim`
returns false on a lapsed claim, exclusion honours that, and its tests pass. Validation
refused the document before exclusion was ever consulted, so the correct answer never got
to matter.

## The Decision, In Two Parts

### Validity is a property of the document, not of the clock

**A rule that can turn a valid document invalid by the passage of time is not an invariant,
and does not belong in validation.**

`Claimed` with a lapsed claim is the normal end of an agent that died. It is what the design
*intends* to happen, and calling it corruption made a file that was written valid stop being
valid on its own, with no writer involved.

What remains is the invariant that cannot move: **an item marked `Claimed` records a claim.**
`claim: None` under `Claimed` is a real corruption — nothing can say whose work it is or was
— and no amount of elapsed time produces it.

The one time-dependent rule left is `Overlapping_Claims`, and it is safe in the direction
that matters: as leases lapse its violations only ever *disappear*. A document that
validated cannot stop validating. That asymmetry is the test to apply to the next rule
somebody wants to add here.

### A lapsed item stays `Claimed` and nobody else may take it

The other half of the question, and it is settled deliberately rather than left as a
consequence.

`Has_Lapsed`'s doc comment said a lapse "stops excluding, which is what lets the next agent
take the item". Half of that is true and is what this record restores: it stops excluding
*other* items. The second half was never true — `Claim_Refusal` rejects anything that is not
`Ready` before it ever reaches the lease — and it now stays untrue on purpose.

`Claim` overwrites `claim`. `claim` is the only thing on a lapsed item recording that the
work was ever started, and `OD-LEDGER-006` decided that must survive: it refused to
synthesize an `Abandonment` for a lapse, because `Abandonment::reason` is *the words the
holder gave* and a lapse has none. So a takeover has nowhere to put the claim it replaces,
and giving it one means reopening what a lapse leaves behind — which is a decision
`OD-LEDGER-006` has already made and this item's scope explicitly does not reopen.

Taking over a lapsed item is therefore a **different operation from claiming a free one**,
and it does not exist. `P10-LAPSE-TAKEOVER` is opened for it.

## What A Lapsed Item Looks Like Now

`work list` calls it `lapsed` rather than `claimed`. That is the whole of what makes the
deferral above tolerable: an item whose holder is gone reads as work in progress otherwise,
and it is the one state on this board that needs a person. `claimed` was the second word in
this listing to mean something other than what a reader takes it to mean, after `ready` —
see `OD-LEDGER-005`.

The holder's own recovery is unchanged and is now the whole recovery story. `Renew` and
`Release` match on the holder and never took the validating path, so an agent that comes back
can always rescue its own claim. While the board was bricked that was the only recovery
there was; now it is the only thing a lapse blocks rather than the only thing it permits.

## The Second Defect: Two Causes Wearing One Name

`Claim`, `Renew` and `Release` discarded every load and save error and returned
`ClaimRefusal::NoSuchItem`. An unreadable file, a parse error and an invalid document all
told the operator that their identifier matched nothing — sending them to check a spelling
while the ledger was refusing to be written.

That is why the defect above needed an experiment rather than a reading, and it is the third
instance of the shape `OD-LEDGER-006` named: a reason destroyed by the failure it explains.

`ClaimRefusal::LedgerUnusable` carries what the store said, verbatim, and the CLI maps it to
exit `5` — which the README already defines as "the ledger or its lock could not be used at
all". The distinction between `3` and `5` is the one that repository documents as earning
its own code: an agent told the item is taken picks up something else, and an agent told the
ledger is broken stops and fetches a person. This refusal used to arrive as `4`, after
arriving as the wrong sentence.

## Status

Closed by `P10-LAPSE-BRICKS`.
