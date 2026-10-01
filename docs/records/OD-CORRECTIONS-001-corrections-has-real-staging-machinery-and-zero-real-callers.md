---
id: OD-CORRECTIONS-001
type: decision
title: Corrections has real staging machinery and zero real callers
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - corrections
  - roadmap
  - workspace
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
---

# Corrections has real staging machinery and zero real callers

## Question

`ARC-ROADMAP-001` names corrections as a near-term-tier item, and the user's standing
roadmap override names it as unblocked work, not still waiting for a trigger. Nothing in
this workspace names what corrections' first real increment actually is, or even what
state the crate is really in. Building the wrong first piece here is the same expensive
mistake `OD-WORKFLOW-001` corrected twice over for the workflow tier -- so this record
checks the real code and the real corpus before naming anything, rather than after.

## What Was Measured

Grepped directly: nothing outside `crates/corrections/nomos-corrections` itself references
`nomos_corrections` anywhere in this workspace, except
`tests/integration/tests/determinism/{declarations,domains,productions}.rs`, which check
only that `CorrectionStaging` declares a determinism strategy -- not real usage. Zero real
callers, matching the "corrections product workflow" gap the roadmap override names.

Read in full before drafting any code -- `candidate.rs`, `plan.rs`, `staged.rs`,
`validated.rs`, `committed.rs`, `determinism.rs` -- rather than assumed from the crate
existing: it is not empty scaffolding.

- `CorrectionPlan::New` already refuses two candidates that touch the same path --
  `COR-003`'s "multiple corrections shall be planned as a compatible set", read from the
  v14 corpus's `COR-001`..`COR-013` family (`volume-05.7-3-corrections-and-convergence`).
- `StagedPlan::Of` already refuses a candidate whose declared prior content does not match
  what the live workspace holds at that path.
- `ValidatedPlan::Commit` and `CommittedPlan::Rollback` already submit through and reverse
  through `nomos_workspace::Workspace`'s one real door, and every step
  (`staged::Assert_Not_Moved`, shared by all three) refuses if the workspace moved since
  the previous one.

This is a real, tested implementation of `COR-005`'s first half: "After staging, the
system shall compile/test as required, rerun affected rules, compare state signatures,
and commit or roll back" -- the stage/validate/commit-or-refuse half, not the
rerun-and-compare half.

## The Finding

At original acceptance: **what was missing was not infrastructure this item should build.
It was a real trigger -- and none existed then.** See the amendments below for what
changed.

Checked against `COR-005`'s second half: `ValidatedPlan::Commit` does not call back into
`nomos-check-orchestration` or `nomos-rules` at all, so nothing reruns affected rules or
compares state signatures after a commit. Checked against `COR-006` (oscillation,
divergence, stall detection): that needs a caller loop, and there is no caller. Checked
against `COR-001`/`COR-010` (classifying a candidate as mechanical, proposed, agent, or
interactive) and `COR-011`..`COR-013` (ranking, arbitration, objective weights): none of
it exists, and the crate's own module doc says this is deliberate, not an oversight this
item is filling in -- "No agent and no model backend appears anywhere in this crate ...
what a candidate would do is decided before this crate exists to run it." This crate is
scoped to the mechanical executor half by its own design.

Checked against the one real candidate for "what decides what to fix": this workspace's
four shipped rules (`Check_Completeness_Mirrors`, `Check_Naming_Convention`,
`Check_Dependency_Direction`, `Check_Unread_Reaches_A_Finding`). None produces a violation
with a safe, judgment-free fix today. `Check_Naming_Convention`'s own violation -- a
function name against `README.md`'s `Pascal_Snake_Case` convention -- would need every
call site renamed in step to stay correct, a real refactor requiring semantic
understanding this workspace has no infrastructure for, not a mechanical edit. The other
three rules' violations (an unmirrored doc claim, a dependency pointed the wrong way, an
unread value reaching a finding) are no more mechanical. `OD-PACKAGE-008` already measured
the same absence from a different angle, about `RulePackage`'s manifest fields: "the
complete absence ... of any real instance of ... a correction and suppression contract ...
with zero real shape to build a manifest field against anywhere in the workspace today."

## What This Does Not Do

At original acceptance, it did not build a `CorrectionCandidate` generator, a
classification or ranking layer, `COR-005`'s rerun-and-compare half, or any oscillation
detection, and did not add a `nomos correct` verb or an orchestration crate for
corrections. See the amendments below for what has since been built and what still has
not. It does not reopen `OD-PACKAGE-008` or restate `ARC-ROADMAP-001`.

## What Would Decide The Next Increment

Historical -- superseded by the amendments below, kept for the record of what this record
originally named as its own trigger. Either of:

- **A shipped rule gains a genuinely mechanical fix.** A rule whose violation has exactly
  one safe correction with no semantic judgment required -- the same bar `COR-001`'s
  "mechanical correction" category names -- would have given `CorrectionCandidate::New` a
  real caller and this crate's stage/validate/commit chain a real first exercise.
- **A real driver for agent- or interactive-class corrections exists.** `ARC-ROADMAP-001`
  deferred `ModelBackend`/`AgentExecutor` infrastructure as its own near-term item; once
  either became real, `COR-001`'s other three classes (proposed, agent, interactive) would
  have something to be driven by.

## Amendment: The Wait For A Trigger Is Retired

Added at version 2. `OD-ROADMAP-001` retires waiting for either trigger above. Neither a
shipped rule with a genuinely mechanical fix nor a real agent/interactive driver has
appeared on its own; the standing instruction is to build `CorrectionCandidate` generation,
the classification/ranking layer (`COR-001`, `COR-010` through `COR-013`), `COR-005`'s
rerun-and-compare half, and oscillation detection (`COR-006`) directly from the `COR-*`
corpus family this record already read, rather than continue waiting for a real caller to
appear first. The measurements above -- that none of the four shipped rules has a safely
mechanical fix today, that `ValidatedPlan::Commit` does not rerun rules or compare state
signatures, that no classification layer exists -- stay true and stay the right starting
inventory for what to build; only the conclusion that building must wait for a trigger is
superseded. Where a genuinely mechanical fix is still absent from every shipped rule, a
`CorrectionCandidate` generator's first real exercise can be its own test suite rather than
a live rule violation, the same standing this override extends to the rest of the
AgentExecutor/ModelBackend/RulePackage cluster.

## Amendment: A Real Caller Now Exists, For One Narrower Sub-Case The Blanket Dismissal Above Did Not Distinguish

Added at version 3. Between versions 2 and 3, four more rules shipped --
`Check_Lint_Diagnostics`, `Check_Dependency_Policy`, `Check_Cross_Language_Correspondence`
and `Check_Declared_Role_Matches_Surface` -- bringing the real total to eight. Each was
checked directly against the same bar version 2 already set, "a rule whose violation has
exactly one safe correction with no semantic judgment required":

- `Check_Lint_Diagnostics` relays `cargo clippy`'s own diagnostics 1:1, but this
  workspace's `nomos.cap.lint.diagnostics` payload (`LintDiagnostic`: `level`, `lint`,
  `message`, `file`, `line`) carries no suggested-replacement span -- `cargo clippy`
  itself often knows a mechanical fix, and this build throws that fact away before a
  `Finding` is ever built. Not a shape this item's scope licenses inventing; a real second
  item, if a real caller needs it.
- `Check_Dependency_Policy` requires choosing whether to remove a dependency or add a
  policy allowance -- a judgment, not a mechanical edit.
- `Check_Cross_Language_Correspondence` requires reconciling two providers' declared
  fields -- also a judgment.
- `Check_Declared_Role_Matches_Surface` is `role_surface.rs`'s own first rule that always
  resolves `Applicability::AgentRequired`; its module doc is explicit that the judgment is
  semantic and this crate does not attempt one.

So the general claim above -- no shipped rule's violation has a safe, judgment-free fix --
is unchanged and now checked against eight rules rather than four. What changed is a
narrower reading of one rule this record already had in front of it. "The other three
rules' violations (an unmirrored doc claim, ...) are no more mechanical" was written about
`Check_Completeness_Mirrors`'s violations as a whole, and that blanket verdict conflated
two shapes the rule itself keeps apart (`verdict.rs`'s own `Verdict` function, one branch
each):

- An **admitted gap** (`Verdict::Admitted_Gap`): a universe claims no mirror at all. Fixing
  this means inventing a check name nobody wrote -- exactly the semantic judgment this
  crate has always correctly declined to make on a rule's behalf, and still does; this
  sub-case stays out of scope.
- A **Phantom** (`Verdict::Unresolved_Claim`'s Blocking arm, reached only through
  `Resolution`'s one `EnforcementBreach::Phantom` case): a doc comment names a check that
  provably does not resolve against the real check index. The one safe, judgment-free
  action here is striking that already-false claim, never inventing a true one. It
  requires no understanding of what the *right* check would be, because it does not
  assert one. It cannot break compilation, because a doc comment is not code and nothing
  in this workspace resolves an identifier against one -- unlike `Check_Naming_Convention`'s
  violation, which stays exactly as unsafe as version 2 found it, for exactly the reason
  version 2 gave. And it cannot remove a *true* claim, because `Resolution` only ever
  raises `Phantom` when the claim has already resolved to nothing -- the same asymmetry
  `mirror.rs`'s own module doc already states: "A false claim of coverage is worse than an
  admitted gap."

`nomos correct phantom-mirrors` (`crates/host/nomos-cli/src/correct.rs`) is that caller,
built directly rather than left for a generator's own test suite: it walks a tree, runs
the real `Check_Completeness_Mirrors` rule, and for a real blocking Phantom finding builds
a real `CorrectionCandidate` (`CorrectionClass::Mechanical`, labelled
`MechanicallySafe`/`BehaviorPreserving`), stages and validates it against a real
`nomos_workspace::Workspace` seeded from the file's own real content, and -- only with
`--commit` -- commits it through `ValidatedPlan::Commit` (carrying real `Evidence` at
`EvidenceClass::Derived`, `OD-CORRECTIONS-002`'s own declared-not-judged parameter) and
writes the corrected file to disk. `crates/host/nomos-cli/src/correct/tests.rs`'s
`Test_Committing_Should_Strike_The_Claim_On_Disk_And_Leave_A_Clean_Rerun` proves the whole
path end to end over a real fixture: a real blocking Phantom becomes a real committed
correction, and a second real run over the corrected tree reports the same universe clean.

What this still does not do, matching the honesty this record has held itself to since
version 1: it does not build a general `CorrectionCandidate` generator or classifier over
arbitrary findings -- one real caller for one real finding shape, not "any rule, any
correction". It does not wire `COR-005`'s rerun-and-compare half or `COR-006`'s oscillation
detection onto this new caller; a session on `track-d-workflow-corrections` is building
`COR-005`/`COR-006` (a real `StateSignature` and `Repeated_State_At`) as separate, disjoint
work, unmerged as of this amendment, and this item does not duplicate it. It does not touch
`COR-011`..`COR-013`'s ranking and arbitration layer, since a plan of exactly one candidate
never has an alternative to rank against. And it does not generalize past
`Check_Completeness_Mirrors`'s Phantom case to any other rule; `Check_Naming_Convention`
remains exactly as unmechanizable as this record has always said.

## Status

Accepted. Named corrections' real state precisely at the time, checked against the code and
the `COR-*` corpus family rather than assumed, and the two conditions that would have
decided its next real increment. Amended to version 2 by
`P13-ROADMAP-001-POPULATION-CAUTION-RETIRED`: those conditions are retired via
`OD-ROADMAP-001`, and the deferred pieces are in scope to build now. Amended to version 3 by
`P14-CORRECTIONS-VERTICAL-PHANTOM-MIRROR-FIRST-INCREMENT`: all eight currently shipped
rules re-checked directly, the general finding confirmed unchanged, and one narrower
sub-case the version-2 text had conflated with the rest of `Check_Completeness_Mirrors`'s
violations -- a blocking Phantom finding -- given a real, tested, end-to-end caller.
