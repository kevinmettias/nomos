---
id: OD-GATE-023
type: decision
title: The gate cluster's dependency on the planner was mostly inherited, and each item is disposed on its own measured claim
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - ledger
  - architecture
  - selection
relations:
  - target: OD-RULES-009
    type: relates-to
  - target: OD-GATE-017
    type: relates-to
  - target: OD-GATE-014
    type: relates-to
  - target: OD-AGENT-004
    type: relates-to
---

# The gate cluster's dependency on the planner was mostly inherited, and each item is disposed on its own measured claim

## Question

`P41-RUN-PLANNER` was declined under `OD-RULES-009`, which has refused the generalized
analysis planner across eight rounds of the same external review. The decline stranded seven
items in one write. Five of them are the gate cluster: `P41-APPLICABILITY-IN-THE-PLAN`,
`P41-GATE-SELECTION-COLLAPSE`, and behind that last one `P41-GATE-PLAN-IS-A-PLAN`,
`P41-COLLAPSE-TRANSITIONAL-LAYERS` and `P40-GATE-PHASES-APPROVALS-2`.

Leaving them stranded answers nothing, and declining them wholesale would assume the
dependency edge was load-bearing in every case. Whether each item actually needs a planner
was unmeasured, and a dependency edge is not evidence — it is what somebody believed when the
item was written, which for this cluster was before `OD-GATE-014`'s selectors and
`OD-GATE-017` existed.

## What Was Measured

Each item's own stated claim, checked against the tree at `cff9df40`.

**`P41-GATE-SELECTION-COLLAPSE` — overtaken.** Its claim is that "a rule can be executed and
then discarded" because gate and check each decide what runs. That stopped being true at
`OD-GATE-017`. `gate_environment::Judged_Sources` is now called with
`selected: &command.rules.include`, so the gate's own selection is what
`nomos_check_orchestration::Run` computes; a selected-out rule's materialization does not run
and its finding never exists. `RuleSelector`'s own documentation states this. What survives is
`Reduced_Findings` re-applying `Is_Included` to findings that `Run` already restricted to the
same list — a redundant second application of one include list, not two mechanisms disagreeing.
`RuleSelector` carries no `exclude`, so there is no path by which the two can diverge.

Its `done_when` is the deciding half: it requires that "a gate compiles its policy into a run
plan plus a result policy." That is the planner, named differently.

**`P41-GATE-PLAN-IS-A-PLAN` — still true, and planner-independent.** `GatePlan` holds exactly
one field, `rules: Vec<RuleOffer>`, and its own doc says it "does not vary by
`GateCommand::root`, `scope` or `rules`." So the verb still reports the registry while wearing
the name of a plan, exactly as the item says. But making it vary needs no demand resolution:
`ScopeSelector` and `RuleSelector` both already exist and `Run_Gate` already consults them.
The dependency was inherited from a sibling, not required by the work.

**`P41-COLLAPSE-TRANSITIONAL-LAYERS` — mostly unevidenced, and its remainder is governed
elsewhere.** Its claim is that an old mechanism, a transitional one and a current one are all
present and reachable in the gate crate. Searching that crate's root for superseded or
unreachable mechanisms finds none; the one candidate was the post-hoc filter above, which is
redundant rather than superseded. What is real is the other half of its `done_when` — module
documentation describing history rather than the current design — and that is now
`OD-AGENT-004`'s subject, decided there on a measured population.

**`P40-GATE-PHASES-APPROVALS-2` — its own sequencing reason is satisfied.** The item states
why it waits: "a phase is a partition of the selected rule set and partitioning a set that is
resolved in two places would inherit that ambiguity." Selection is resolved in one place as of
`OD-GATE-017`. The condition the item set for itself is met.

**`P41-APPLICABILITY-IN-THE-PLAN` — load-bearing, and the only one.** It asks that
applicability be "resolved as part of the plan rather than during judgment," driving execution
and coverage from one resolution. Its territory is a file named `applicability_plan.rs`. There
is no version of this that does not need the plan, and `OD-RULES-009` declines the plan.

## The Decision

**Each item is disposed on its own claim, not on the edge it inherited.**

- **`P41-GATE-SELECTION-COLLAPSE` is declined.** Its defect was fixed by `OD-GATE-017` and its
  `done_when` asks for the artifact `OD-RULES-009` refuses.
- **`P41-COLLAPSE-TRANSITIONAL-LAYERS` is declined**, superseded by `OD-AGENT-004` for the
  half that is real and unevidenced for the half that is not.
- **`P41-APPLICABILITY-IN-THE-PLAN` is declined.** Its dependency was genuinely load-bearing,
  so a decline is the honest terminal state rather than a stranding. It revives with the
  planner or not at all, and `OD-RULES-009`'s "What Would Decide It" already names what would
  do that.
- **`P41-GATE-PLAN-IS-A-PLAN` is re-authored without the planner dependency**, carrying the
  measurement that freed it.
- **`P40-GATE-PHASES-APPROVALS-2` is re-authored without it too**, on its own stated condition
  having been met.

**A sixth item, outside the gate cluster, is declined for a different reason.**
`P41-RUN-STOPS-NAMING-EVERY-RULE` was the seventh stranded item and its complaint is intact —
`run_context.rs` still writes out seventy `ComposedRule` entries, and `Composed_Rules()`
derives from that array rather than from `DESCRIPTORS`. It is declined not because the
complaint is stale but because the *question* it raises now belongs to
`P74-RUN-ARRAY-DERIVATION-DISPOSITION`: whether `Run` can follow the derivation
`composition::Registered` already made, or whether needing each entry's capability slice makes
that derivation the planner. An item cannot both pose a question and be the work that follows
from either answer. If that decision says the derivation is available, it authors the
implementation item; if it says the derivation is the planner, nothing should have been on the
board at all.

**The redundant post-hoc filter is not made an item.** Re-applying an include list that `Run`
already honored costs one pass over a finding vector and cannot produce a wrong answer while
`RuleSelector` has no `exclude`. Filing it would be tidying, and the same restraint
`OD-RULES-009` applies to the planner applies to its leftovers.

## What This Says About Declining A Root

A decline propagates to every dependent at once, and the dependents do not share an answer:
of five here, one edge was load-bearing, one item was already fixed, one was superseded by a
different record, and two were waiting on nothing.
`P73-DECISION-BEFORE-CODE-STRANDS-ITS-DEPENDENTS-2`, still open at this record's writing,
names the mirror shape — an item ended honestly while leaving others unreachable — and this
cluster is its second instance from the other direction. **A decline of a root owes the same
measurement a claim does: each dependent checked against the tree, not against the edge.**

## What Would Decide It Differently

- **`RuleSelector` gaining an `exclude`.** The two selections could then disagree, and
  `P41-GATE-SELECTION-COLLAPSE`'s original defect would be real again rather than redundant.
- **The planner arriving** on any of `OD-RULES-009`'s named triggers, which revives
  `P41-APPLICABILITY-IN-THE-PLAN` on its own terms.
- **A superseded mechanism actually found reachable** in the gate crate, which would restore
  `P41-COLLAPSE-TRANSITIONAL-LAYERS`'s code half.

## Status

Accepted. Five items measured individually at `cff9df40`; three declined, two re-authored free
of an inherited edge, and nothing left stranded behind `P41-RUN-PLANNER`.
