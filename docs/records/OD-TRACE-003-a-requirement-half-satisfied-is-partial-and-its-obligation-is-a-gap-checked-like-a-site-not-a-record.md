---
id: OD-TRACE-003
type: decision
title: A requirement half satisfied is Partial, and its obligation is a gap checked like a site, not a governing record
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - traceability
  - requirements
  - corpus
  - completeness
  - verification
relations:
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-TRACE-002
    type: relates-to
  - target: OD-TRACE-005
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
---

# A requirement half satisfied is Partial, and its obligation is a gap checked like a site, not a governing record

## Question

`OD-TRACE-001` named four verdicts and wrote three of them: `Met`, `Diverges`,
`NotBinding`, and `Unassessed` held by the absence of an entry and refused as a written
word. All six entries committed under `tests/contract/requirements` today resolve to one
of those three writable verdicts, so nothing has yet forced the fourth case the corpus
actually produces: a requirement that binds this build and is satisfied at some of the
sites it names and not at others.

That case is none of the three. `Met` overclaims — it is exactly the stale `Met`
`OD-TRACE-001` already names as the guard's blind spot, applied on purpose instead of by
drift. `Diverges` asserts the departure is deliberate and demands a governing record
explaining a decision, and there is no decision to explain: unfinished is not one.
`NotBinding` is false on its face; the requirement does reach this build, which is exactly
why part of it is satisfied. And the fourth, `Unassessed`, is not available by
`OD-TRACE-002`'s own reader: writing it would record that somebody looked and did not look,
which is precisely backwards for a case where somebody looked *hardest* and found the most.

`OD-TRACE-001`'s own finding was that met and unmet have the same shape from outside. Absent
a fourth verdict, this reproduces that property one level in: a requirement somebody
assessed and found partially satisfied would be stored identically to one nobody has opened
— both silently uncounted by `FEWEST_ASSESSMENTS`, the one number that says how much of the
corpus has been examined. That is the defect this record closes, and it is forced by the
first honest partial assessment rather than by any deadline: the pressure at that moment is
to write `Met`.

## The Decision

**A fifth verdict, `Partial`, and it carries an obligation that is not the record the other
two non-`Met` verdicts owe.**

- **`Partial`** — the requirement binds this build and is satisfied at some of the sites
  the entry names and not at others. The entry names at least one `gap`: a `path#symbol`
  site — the identical shape `OD-TRACE-002` gave `site`, and checked by the identical
  predicate — naming where the unsatisfied part of the requirement actually lives.

`Verdict::Owes_A_Record` stays exactly `Diverges | NotBinding`. `Partial` is deliberately
excluded, and that exclusion is the point of this record rather than an oversight. `Diverges`
and `NotBinding` both name a decision — somebody choosing that this build will not do what
the requirement says — and the record is what makes that decision reviewable. A half-built
requirement is not that. Demanding a governing record for it would manufacture a reason
where none exists, which is the reviewer-facing version of the same overclaim `Met` would
make: a record with no real decision behind it reads as one anyway. So `Partial`'s
obligation is a **gap**, not a **record**: the entry says what is not satisfied, at a place
in the workspace, or it is refused.

### Why a gap and not a ledger item

An open ledger item was the other candidate for the obligation, and it is refused here for a
narrower reason than the record was: an item is closed the moment the work lands, and a
`Partial` entry that cites one as its only anchor goes silently unverifiable at exactly the
moment it would need re-reading — the item that would have prompted the re-read is gone.
Nothing in `work/ledger.json` is checked against this registry today, and inventing that
check here would widen this item past a fourth verdict into a second guard. A gap has no
such lifecycle: it is a site, checked the same way every other site here already is, so it
either keeps resolving or it visibly stops. Nothing prevents an entry from naming a `record`
or citing an item in prose alongside its verdict — `Test_Every_Named_Record_Should_Exist_And_Be_Registered`
already reads wider than `OD-TRACE-001` strictly requires, for `Met` as much as for
anything else — but neither is the obligation `Partial` is checked against.

### Why a gap decays the way a site does, on purpose

`sites` already answers "where is this satisfied", checked by resolving `path#symbol`
against the workspace: the guard sees a named site vanish, though it cannot see the code at
a site drift out of meaning what it says — the semantic-drift limit `OD-TRACE-001` already
takes. A `gap` answers the opposite question, "where is this *not* satisfied", with the
identical mechanism and the identical limit: the guard sees a named gap vanish or move.
That is deliberate rather than a reuse of convenience. A prose-only description of what
remains — "auth is not wired up yet" with no site — could describe nothing by the time
anyone reads it again, and nothing would notice. A `gap` that is itself a checked site
cannot do that quietly: if the incomplete code it names is deleted, refactored past
recognition, or finished and removed, `Unresolved_Gaps` reports it exactly the way
`Unresolved_Sites` reports a vanished `site`, and the entry is forced back in front of a
person rather than continuing to describe a state that no longer exists. That closing is not
automatic promotion to `Met` — the same refusal `OD-TRACE-005` already gives a drifted hash,
for the same reason: telling "the gap closed, so this is now Met" from "the gap moved, so
this needs a new one" is reading the code and judging, which stays a human act.

### The obligation is enforced twice, in the shape every other one here already is

`Assert_Complete` refuses a `Partial` entry with no `gap` at read time, the same place
`OD-TRACE-002`'s reader already refuses a `Diverges` entry with no `record`. Separately,
`Partials_With_No_Gap` — mirroring `Divergences_With_No_Record` exactly — is asserted over
the committed set by `Test_Every_Partial_Should_Name_A_Gap`, and a resolution check,
`Unresolved_Gaps`, mirroring `Unresolved_Sites`, is asserted by `Test_Every_Gap_Should_Resolve`.
Both are vacuous today, because no committed entry is `Partial` — the same vacuity
`Test_Every_Divergence_Should_Name_A_Governing_Record` already states plainly rather than
hides, for the identical reason: all six committed entries are `Met` or `Diverges`. Each
vacuous assertion is kept honest by a control that runs the real predicate — not a copy of
it — over a constructed entry: `Test_A_Partial_With_No_Gap_Should_Be_Refused` and
`Test_An_Entry_Naming_A_Vanished_Gap_Should_Be_Reported`, the shape every other assertion in
this suite already has.

## What This Does Not Do

- **It does not assess a 7th requirement.** No entry under `tests/contract/requirements`
  changes; `FEWEST_ASSESSMENTS` stays `4` because the committed set did not grow. Populating
  a real `Partial` entry from a genuine partial audit is a separate item's territory
  (`P12-TRACE-POPULATION`), not this record's.
- **It does not reopen semantic drift.** A `gap` is checked exactly like a `site`: text
  occurring in a file. Code drifting out of actually leaving the requirement unsatisfied,
  while the named gap symbol stays put, is invisible here for the identical reason a stale
  `Met` is — `OD-TRACE-001`'s limit, unchanged.
- **It does not touch the hash machinery `OD-TRACE-005` decided.** A `Partial` entry is
  eligible for the same `hash:` field and the same `Confirmed`/`Drifted`/`Unhashed` reporting
  once that field is built; nothing about a fourth verdict changes what that record already
  settled.
- **It does not make `Unassessed` writable, or add a sixth verdict.** `Partial` fills the
  one gap `OD-TRACE-001`'s three writable verdicts left; it does not reopen the question of
  whether the registry should hold more.

## Status

Closed by `P12-TRACE-PARTIAL`.
