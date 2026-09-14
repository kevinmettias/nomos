---
id: OD-GATE-030
type: decision
title: A baseline entry is evidence of an occurrence at a snapshot, and continuity is not claimed until it can be shown
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - enforcement
  - adoption
relations:
  - target: OD-GATE-015
    type: affects
  - target: OD-GATE-022
    type: relates-to
  - target: OD-GATE-028
    type: relates-to
---

# A baseline entry is evidence of an occurrence at a snapshot, and continuity is not claimed until it can be shown

## Question

`BaselinePolicy` matches a `Finding` by `rule` and `subject` against a declared list, and a
match cannot fail the build. Fixing the finding does not remove the entry. So the same
violation, returning later at the same subject, re-matches and is tolerated again — silently,
with no finding, no report, and no way for a reader to know it happened.

That makes a baseline entry behave as a **perpetual exemption for a `rule`/`subject` pair**.
What a repository meant by adopting a baseline was something narrower: *this occurrence existed
when we started*. The two readings agree until a finding is fixed and comes back, and then they
disagree exactly where it matters.

Adoption is the whole point of a baseline. A repository that adopts Nomos on existing debt and
then tightens over time must be able to do so without either treating old debt as newly
introduced or hiding a regression behind the tolerance it granted for a different occurrence.
The first is handled — baselined findings sit in their own bucket and `Compare_Gate_Runs`
reports a `Baselined → Blocking` move with both ends. The second is not.

## What Was Measured

Checked 2026-09-14, and the measurement changed what this record could decide rather than
confirming a plan.

**A finding carries no occurrence identity.** `Finding` is `rule`, `subject`, `subject_name`,
`applicability`, `evidence`, `gate`, `summary`, `locations`. Nothing distinguishes *this*
occurrence of a violation from another occurrence of the same violation at the same subject.

**A baseline entry carries no snapshot.** `BaselineDebt` is `rule`, `subject`, `rationale`. It
does not record what tree it was taken against, so "existed at the baseline" is implied by the
entry's existence rather than stated by it.

**Nothing keeps a run history.** `OD-GATE-022-A` decided a compare caller re-derives both runs
in one process, builds no run-history store and serializes no `GateRunResult`, and that
cross-process comparison is deferred until a real caller needs it. No store exists.

**Therefore continuity is unprovable today.** A single run can see that a finding matches a
baseline entry. It cannot see whether that finding persisted since the baseline or was fixed
and returned, because the evidence that would distinguish them — the states in between — is not
kept anywhere.

## Decision

**A baseline entry is evidence that an occurrence existed at a named snapshot. It is not a
standing exemption for a `rule`/`subject` pair.**

**And a run does not claim continuity it cannot show.** Where a finding matches a baseline entry
and nothing establishes whether it persisted or recurred, the honest report is that continuity
is **undetermined** — not that this is persistent debt. Reporting persistent debt for a finding
that may have been reintroduced is precisely the hiding failure a gate exists to prevent, and
it would be a claim made by the absence of evidence rather than by any.

This is the same discipline `Applicability` already applies one layer down, where
`MissingCapability` and `ProviderUnavailable` are kept apart from a clean result so that
"nothing could look" never reads as "nothing was wrong". An undetermined continuity is that
state for a baselined finding.

**A line-age or diff-age heuristic is refused, not merely unbuilt.** "New code" as a proxy for
"new finding" answers a different question and answers it approximately, at the one point where
a repository is relying on the gate to be exact about what it is tolerating. The concept this
needs is a **finding's lifecycle across snapshots**, which fits what this workspace already has
— content-addressed subjects, run identities, a fact store — better than source geometry does.

## What Would Make Continuity Provable

Named so a later reader knows what is missing rather than re-deriving it, and in the order the
parts would arrive.

1. **A snapshot identity on the entry.** A baseline entry says which tree state it was taken
   against, so "existed at B" is recorded rather than inferred. This is the part that can be
   built today, and `BaselineDebt`'s own doc already names the order: the key appears on the
   declared entry first, and the domain type grows a field to carry it.
2. **An occurrence identity on the finding.** Enough to say that the occurrence seen now is the
   one seen at B, rather than another occurrence of the same rule at the same subject.
3. **A history between them.** Some record of the states in between, which is the run history
   `OD-GATE-022-A` deferred. Without it, 1 and 2 establish that an occurrence matching B's is
   present now, and still not that it never left.

Only the third makes `PersistentDebt` and `Reintroduced` separable. The first two narrow the
undetermined state; they do not close it, and a record that implied otherwise would be setting
up the next reader to over-claim.

## What This Does Not Decide

**Scope by source geometry.** `BaselineDebt`'s own doc defers scoping by revision and source
shape until a real caller needs one, and nothing here reopens it.

**Whether the undetermined state blocks.** That is a policy question — a repository might
reasonably treat undetermined continuity as tolerable during adoption and as blocking once it
has tightened — and it belongs with the other policy choices a `nomos-gate.json` carries, not
in the meaning of a baseline entry.

**When the run history gets built.** `OD-GATE-022-A` owns that deferral. This record adds a
second consumer to the case for one, and does not schedule it.
