---
id: OD-GATE-030
type: decision
title: A baseline entry is evidence of an occurrence at a snapshot, and continuity is not claimed until it can be shown
status: accepted
version: 2
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

They disagree along a second axis too, and it is the one that can be settled. The entry says
nothing about **how many** occurrences were adopted, so it tolerates however many the scope
comes to hold. A repository that adopted one violation and later writes four more has them
tolerated by the entry it wrote for the first, and neither the run nor the report says the debt
grew.

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

### Remeasured before the first increment was built

Version 1 named a snapshot identity on the entry as the part that could be built today. Costed
against the built binary rather than by reading, on the same day, it closes nothing.

**A file's subject is the digest of its normalized path, not of its content.** So a baseline
entry survives every content change by construction — which is deliberate, and is why a
tolerated entry is not lost to an unrelated edit above it. A tree digest on the entry could
therefore establish continuity only while the tree is byte-identical to adoption: true until the
first commit after adopting, and false forever after.

**Meanwhile the unbounded tolerance is reachable in four steps.** Adopt one entry for one
violation; fix that violation; write five new ones at the same subject; run the gate. It reports
five baselined and exits clean. Every one of the five was written after adoption, and the report
attributes all five to debt that existed before it.

**The useful half of "existed at a snapshot" is *how much*, not *which tree*.** A quantity is
decidable from one run, and it is what the reachable defect is about.

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

### Baseline tolerance is bounded by the quantity that was adopted

**An adopted baseline records the maximum occurrence population accepted for its `rule`/`subject`
scope. A later run may tolerate no more than that quantity, and an observed population above the
accepted count is a baseline expansion the baseline must not hide.**

The argument is a counting one and nothing else. Where a scope accepted `N` occurrences and a
run observes `M` greater than `N`, at least `M - N` of the observed occurrences cannot belong to
the population that was adopted. Nothing has to be matched to an earlier occurrence for that to
hold, no revision has to be diffed, and nothing has to be remembered between runs. It is exact
where the refused heuristic above is approximate, and it is stable under the edit that made
occurrence-scoped matching unsafe: moving a line changes no count.

**The count proves nothing about continuity.** `M` at most `N` is not evidence that the adopted
occurrences persisted. A scope that accepted five and observes five is equally consistent with
the same five persisting and with all five having been fixed while five different violations
appeared, and no reading of the two numbers separates those histories. Occurrence continuity,
persistence and reintroduction remain exactly as unresolved as this record's first version left
them, and they remain so until historical identity evidence exists. A reader who takes this
bound for partial reintroduction detection has read the record backwards: what is bounded is
capacity, not history.

**No occurrence inside an exceeded population is attributed.** Where `M` exceeds `N`, a run knows
that the population is over its allowance by `M - N` and does not know *which* of the `M`
occurrences are the adopted ones. Sorting them by any stable key and calling the first `N`
historical would manufacture precisely the evidence the paragraph above says nobody has, and
would hand the decision to the key: under an occurrence identity that takes locations, a
reformatting commit would change which occurrences a repository is said to have adopted. The
honest report is about the population — what it accepted, what it observed, and the excess. An
interface that must place every finding somewhere places the whole affected population in a
state of its own, rather than splitting it into some findings said to be old and others said to
be new.

**An entry naming no count tolerates without limit, and is reported as doing so.** Every entry
authored before this decision names none, and neither silent reading of that is acceptable:
reading it as one occurrence blocks a build over debt the repository did adopt, and reading it
as unlimited without remark preserves the defect this decision exists to remove. So an entry
with no count keeps the meaning it was written under and is reported as unbounded — a statement
an author can act on, and not a quantity anybody invented on their behalf. A count that *is*
written must be positive: an entry accepting zero occurrences tolerates nothing, which is what
declining to write the entry already does.

## What Would Make Continuity Provable

Named so a later reader knows what is missing rather than re-deriving it, and in the order the
parts would arrive.

1. **A quantity on the entry.** A baseline entry says how many occurrences its scope accepted, so
   "this much existed at adoption" is recorded rather than implied by the entry's existence. This
   is the part that can be built today, and `BaselineDebt`'s own doc already names the order: the
   key appears on the declared entry first, and the domain type grows a field to carry it.

   Version 1 named a *snapshot identity* here — the tree state the entry was taken against — and
   the remeasurement above retired it. It is not deferred for cost; it is the wrong artifact,
   because a path-addressed entry outlives every tree it could be compared against. This step
   bounds capacity and establishes no continuity, which is why it is first: it is the only one of
   the three that closes anything on its own.

2. **An occurrence identity that survives a revision.** Enough to say that the occurrence seen now
   is the one seen at adoption, rather than another occurrence of the same rule at the same
   subject. `FindingOccurrenceId` exists and is deliberately not this: it is scoped to one pinned
   result, it takes the finding's locations as the only occurrence-discriminating material there
   is, and its own documentation refuses its use for matching a tolerance across revisions,
   because an identity that moves when a line moves would expire every tolerated entry on the
   next reformatting commit.

3. **A history between them.** Some record of the states in between, which is the run history
   `OD-GATE-022-A` deferred. Without it, 1 and 2 establish that an occurrence matching adoption's
   is present now, and still not that it never left.

   When it arrives it names its states with `IdentityTransitionKind`, which this workspace
   already publishes: `ExactContinuity`, `Recreated` — whose own doc says it is "not continuity,
   and it must not be reported as such" — `Unresolved`, and an `Is_History_Preserving` rule that
   already decides a tolerance must not survive a recreation. Measured 2026-09-14: no gate code
   reads it. A second `Persistent`/`Reintroduced`/`Moved` vocabulary minted in gate code beside
   that one would be the duplicated authority this repository files records about.

Only the third makes persistence and reintroduction separable. The first bounds how much may be
tolerated and settles nothing about history; the second would narrow the undetermined state and
still not close it. A record that implied otherwise would be setting up the next reader to
over-claim.

## What This Does Not Decide

**Scope by source geometry.** `BaselineDebt`'s own doc defers scoping by revision and source
shape until a real caller needs one, and nothing here reopens it.

**Whether the undetermined state blocks.** That is a policy question — a repository might
reasonably treat undetermined continuity as tolerable during adoption and as blocking once it
has tightened — and it belongs with the other policy choices a `nomos-gate.json` carries, not
in the meaning of a baseline entry.

**When the run history gets built.** `OD-GATE-022-A` owns that deferral. This record adds a
second consumer to the case for one, and does not schedule it.

**How a declared entry addresses a finding that is not subjected to a file.** `DeclaredSuppression`
and `DeclaredDebt` both compute their subject from a path, so a rule whose findings are subjected
to a qualified name can be named by neither — measured 2026-09-14 at fourteen rule sites outside
test code, including every rule in the naming family. That is an addressing-model gap in the
configuration language, it applies to suppression identically, and it is not a question about
baseline capacity. It belongs to its own item.
