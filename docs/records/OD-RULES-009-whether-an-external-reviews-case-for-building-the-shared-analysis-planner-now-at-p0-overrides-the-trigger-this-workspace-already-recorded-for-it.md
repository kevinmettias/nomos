---
id: OD-RULES-009
type: decision
title: Whether an external review's case for building the shared analysis planner now, at P0, overrides the trigger this workspace already recorded for it
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - rules
  - orchestration
  - gate
  - architecture
  - planning
relations:
  - target: OD-HOST-004
    type: relates-to
  - target: OD-GATE-014
    type: relates-to
  - target: OD-RULES-007
    type: relates-to
  - target: OD-RULES-008
    type: relates-to
  - target: OD-GATE-011
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
---

# Whether an external review's case for building the shared analysis planner now, at P0, overrides the trigger this workspace already recorded for it

## Question

An external architecture review (read against this workspace rather than accepted on read)
names `nomos-check-orchestration::run::Run`'s hand-written `Materialize_X`/`Check_Y` call
list, and `nomos-gate-orchestration`'s separately-composed `RuleRegistry`, as this
workspace's biggest architectural problem, and recommends — at P0, ahead of any further rule
work — building a generalized `RunPlanner`: rules declare capability requirements, a planner
collects and deduplicates them into a shared fact DAG, resolves providers, and both `check`
and `gate` execute through the same planned object.

This is not a new question to this workspace. `OD-RULES-007` and `OD-RULES-008` already
name "the shared demand planner `ARC-ROADMAP-001` still leaves for later" and give it a
specific, checkable trigger. Whether this review's argument is new evidence that fires that
trigger, or a restatement of a case the workspace's own records already considered, was
unmeasured before this record.

## What Was Measured

**The trigger already on record has not fired.** `nomos-lang-rust/../facts.rs`'s own doc
comment on `Materialize_Dependencies` states composing a second hardcoded materialization
step is "`OD-HOST-004`'s 'composition, not choice' again, not a case for the shared demand
planner `ARC-ROADMAP-001` still leaves for later." `OD-RULES-008`, checking a third
candidate materialization step against the same question, sharpened this into a criterion:
"`Run` ... still has no selector of any kind (`OD-GATE-014`, still open), so nothing is
asking to materialize *less* than everything — the planner's trigger is selection creating
unread work." Verified directly against the real code at this record's own HEAD, not
assumed: `nomos-check-orchestration::run::Run` (`crates/orchestration/nomos-check-
orchestration/src/run.rs`) now calls four rules —
`Check_Completeness_Mirrors`, `Check_Naming_Convention`, `Check_Dependency_Direction`,
`Check_Unread_Reaches_A_Finding` — unconditionally, over every source it is handed, with no
field on `Run` or its caller that varies the set per invocation. `OD-GATE-014` is still
open: `.github/workflows/gate.yml`'s `Rules` step calls `gate run`, but that caller judges
everything, unconditionally, the identical shape it has always had. No selector exists
anywhere in this workspace. The trigger `OD-RULES-008` named — selection creating unread
work — has not fired.

**A fourth rule already joined using the pattern the review says to stop using, and nothing
broke.** The review's underlying commit is `e6c41e1`, three rules, before
`Check_Unread_Reaches_A_Finding` was wired in. `P13-CONTROLFLOW-REACHABILITY-WIRE` (commit
`7672b65`) added it as a fourth hand-written call in `Run` and a fourth `RuleOffer` in
`gate-orchestration`'s registry, the identical shape the first three already had. This is
direct evidence bearing on the review's own stated stopping point ("I would not add a
fourth/fifth/sixth rule using the existing direct-call pattern"): a fourth rule was added
using exactly that pattern, under `OD-HOST-004`'s own criterion (unconditional participation
is composition, not a selection problem, regardless of count), and the workspace's own
`crate::tests` in `gate-orchestration` — which pins the registry's offers against what `Run`
actually calls — passed without needing a planner to keep the two lists honest.

**The review's efficiency case does not describe this codebase.** The review's central
argument for a shared fact DAG is that facts should be "reused across rules" rather than
recomputed. `run.rs` already does this: `Materialize_Syntax`, `Materialize_Dependencies` and
`Materialize_Reachability` each run once per `Run` call, write into one `MemoryFactStore`,
and all four rules read the result through one shared `Reader`. No rule in this workspace
recomputes a fact another rule already produced. The scaling problem the review names — the
*list* of `Materialize_X`/`Check_Y` calls growing linearly with rule and capability count —
is real, but it is the maintenance/duplicated-authority problem `OD-GATE-011` already names
as a defect class, not the recomputation-efficiency problem the review's fact-DAG diagram
argues from.

**The duplicated-authority half is already named, and its own record already declined to
build a checker now.** `OD-GATE-011` (accepted) names "two independent encodings of one
decision" as a defect class from five closed instances in this repository's history — the
same shape `Run`'s hand-written call list and `gate-orchestration`'s separately-composed
`RuleRegistry` share. `OD-GATE-011`'s own "What This Record Does Not Decide" section already
states building a check against this class "is a later item's territory, not this one's."
`gate-orchestration/src/composition.rs`'s own module doc already states its `RuleRegistry`
composition is "that registry's first real consumer... not a change to what `nomos check`
runs," and ships `crate::tests` asserting the two lists agree — the same "a test pins the
pair against each other so drift between the two is still caught" shape `OD-GATE-011`
describes as distinguishing a tracked, tested duplication from an unaccountable one, short of
meeting that record's full three-part legitimate-exception test (the derivation is not yet
from one *named* stated authority at the site, only from a test).

**Real growth exists, and the closest existing precedent for generalizing from it already
declined to.** `OD-PACKAGE-008`, at its own most recent revision (four rules, not two),
measured genuine capability-shape divergence — three structurally distinct
`FactVariant`/`Assurance`/`IncrementalGranularity` families (`nomos_cap_syntax`,
`nomos_cap_dependency`, `nomos_cap_controlflow`), and, for the first time,
`Applicability::PartiallySupported` as a rule's structural output rather than an edge case —
the same "partially supported rules" and "alternate providers" growth the review's own
scaling list names. `OD-PACKAGE-008` measured that growth against the real population and
still declined to scaffold a generalized `RulePackage` manifest, on the ground that roughly a
third of the manifest's contents list has zero real instance anywhere in the workspace yet.
That is a different artifact than a `RunPlanner`, but it is the nearest "does real growth
justify generalizing now" precedent this workspace has, checked against evidence rather than
argued from a wish, and its answer was no.

## Decision

**Declined to elevate to P0 or to build now.** No `RunPlanner` is built. `Run` stays a
hand-written, unconditional list; `gate-orchestration`'s `RuleRegistry` composition stays a
test-pinned second rendering of the same list, per `OD-GATE-011`'s already-accepted cost.

This is not a verdict that a shared analysis planner is the wrong target — `ARC-ROADMAP-
001`'s near-term tier already names "analysis + incremental fact infrastructure" and
"first-class gates" as needed work, and the review's `RunPlanner` shape is broadly compatible
with `OD-CAPABILITY-001`'s existing `Registry::Resolve`, which `OD-HOST-004` already
identifies as the layer that would carry selection if and when it is needed. It is a verdict
that the review's *sequencing* claim — build this now, before adding a fifth rule, ahead of
everything else — restates a case `OD-RULES-007`, `OD-RULES-008`, `OD-HOST-004` and
`OD-GATE-014` already weighed and did not find fired, and that `ARC-ROADMAP-001` itself
explicitly declines to order: "it does not order the near-term tier internally... a separate
judgment, informed by this record but not fixed by it."

## What Would Decide It

Unchanged from the records this one confirms, named together for a reader who arrives at
this question through the review rather than through them:

- **Selection creating unread work** (`OD-RULES-008`'s own trigger): something asks `Run`,
  or `Gate`, to materialize or judge less than everything — `OD-GATE-014`'s `ScopeSelector`/
  `RuleSelector` getting built is the concrete case this would arrive through.
- **Participation varying by request** (`OD-HOST-004`'s trigger): a rule meant to run for
  *some* check invocations and not others, rather than unconditionally on all of them.
- **A materialization step measured to cost real work for a population where not every rule
  needs every fact.** Not yet observed: `Materialize_Reachability`'s tier-1 provider is a
  pure per-file heuristic, the identical cheap shape `Materialize_Syntax` already has, and
  today's four rules partition cleanly across three capability families with no rule waiting
  on a fact only a sibling needs.
- **A fifth rule's required-capability shape reconverging with an existing one**, rather than
  adding a fourth distinct family — `OD-PACKAGE-008`'s own tracked trigger, load-bearing here
  too: a converging population is weaker evidence for a general planner than a diverging one.

## Amendment: The First Named Trigger Fired, And Was Addressed Narrowly

This record's own "What Would Decide It" section named the case exactly: "`OD-GATE-014`'s
`ScopeSelector`/`RuleSelector` getting built is the concrete case this would arrive through."
It has. A real caller, `nomos gate run --rule` (`crates/host/nomos-cli/src/gate/parsing.rs`),
constructs a non-empty `RuleSelector` today, and `OD-GATE-017` measured that `Run` still
computed every rule regardless of it — selection creating unread work, this record's own
first-named trigger, fired precisely as predicted.

**It was addressed narrowly, not by building the `RunPlanner` this record declined.**
`OD-GATE-017` gave `nomos_check_orchestration::Run` a real `&[RuleId]` parameter and a fixed,
hand-written mapping from each of today's four rules to the fact it needs — the identical
"composition, not choice" shape this record's own Decision section already approved staying
with ("`Run` stays a hand-written, unconditional list"), extended to a second axis (whether a
rule's materialization runs at all) rather than replaced by a generic, declared
requirement-to-provider resolution. No rule declares a capability requirement a planner reads;
no fact DAG is deduplicated; no provider is resolved ahead of a hardcoded call list. The
distinction this record already drew — a real caller narrowing what runs, versus a generalized
planner built ahead of any caller needing one — is exactly what separates what `OD-GATE-017`
built from what this record continues to decline.

**The other three named triggers remain unfired, checked directly rather than assumed.** No
rule's required-capability shape has reconverged with an existing family — a fifth rule
(`P14-RULES-005-ROLE-SURFACE-AGENT-REQUIRED`, `Check_Declared_Role_Matches_Surface`) arrived
since this record's first version, but reads plain committed files rather than a `FactReader`,
states no capability contract, and is additive and unwired into `Run` — a structurally new,
diverging family if and when it is ever wired in, not a reconverging one. No materialization
step has been measured to cost real, wasted work for a population where facts are shared and
reused: `run.rs` still writes every fact into one `MemoryFactStore` through one shared `Reader`,
and no rule recomputes a fact another rule already produced. Participation still does not vary
by request beyond the one axis `OD-GATE-017` now covers.

**This record's own decline of the general planner therefore stands, on its remaining,
still-unfired triggers.** One trigger firing and being answered at the scope the evidence
actually supported is not evidence the larger artifact is now due; if anything, `OD-GATE-017`'s
narrow increment satisfying the real caller that existed is itself data that the hand-written
shape continues to scale to a real, evidenced need without a generalized planner underneath it.

## Status

Accepted. This record's first named trigger fired and was addressed by `OD-GATE-017`, not by
building the `RunPlanner` this record declines. Revisit on any of the three remaining triggers
named above, or when `OD-RULES-007`'s own status next changes.
