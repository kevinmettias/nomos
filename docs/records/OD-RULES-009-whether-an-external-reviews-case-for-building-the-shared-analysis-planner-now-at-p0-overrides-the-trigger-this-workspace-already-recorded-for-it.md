---
id: OD-RULES-009
type: decision
title: Whether an external review's case for building the shared analysis planner now, at P0, overrides the trigger this workspace already recorded for it
status: accepted
version: 5
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
  - target: OD-CAPABILITY-010
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

## Amendment: A Third Round Of The Same Review, Re-Checked Against Real Growth Since The Last Amendment

A later round of the same external review repeated its original recommendation almost
verbatim — build the generalized planner now, at P0, calling `Run`'s hand-written call list
this workspace's biggest architectural problem — without citing new evidence the workspace
had not already measured. Rather than re-arguing a case this record and `OD-GATE-017` already
settled, this amendment does what the Status section already invited: checks the three
remaining named triggers against real growth that landed since the amendment above, at this
amendment's own HEAD (`e9f171b`, `P14-RULES-010-TOOL-PROVIDER-DENY-3`).

**"Participation varying by request" — still only the one axis `OD-GATE-017` already
covers.** A sixth rule, `Check_Dependency_Policy` (`crates/rules/nomos-rules/src/policy.rs:39`,
`OD-RULES-010`), joined `Run` since the last amendment, wired through the identical `Wants`
gate every rule before it uses (`crates/orchestration/nomos-check-orchestration/src/run.rs:172-
178, 250-252`). It varies by request through the same `selected: &[RuleId]` parameter
`OD-GATE-017` already built, not a second, different axis of variance. Nothing new to fire
here beyond what is already answered.

**"A materialization step measured to cost real, wasted work" — a third real-cost step
arrived, and it is not wasted.** `Materialize_Policy` (`run.rs:172`) launches its own
subprocess through `nomos-lang-rust-deny`, the identical shape `Materialize_Dependencies`
(`cargo metadata`) and `Materialize_Lint` (`cargo clippy`) already have — `run.rs`'s own
comment states plainly that skipping it "skips its own subprocess launch entirely" (`run.rs:
143`). It is gated by the same `Wants(selected, DEPENDENCY_POLICY)` check every real-cost step
already uses. Three real-cost materializations now exist, all three already avoid their own
cost when unselected, and none is shown recomputing a fact a sibling rule already produced.
This is data that the hand-written, per-rule gate continues to scale to a growing population of
real-cost steps without waste, not evidence it has started to fail.

**"A fifth rule's required-capability shape reconverging with an existing family" — the
population diverged further, not less.** `nomos-cap-dependency-policy`
(`crates/capabilities/nomos-cap-dependency-policy/src/contract.rs`) is a fifth, structurally
distinct capability crate — its own `Capability_Contract`, its own `FactVariant`/`Assurance`/
`IncrementalGranularity` ceiling (`SemanticallyResolved`/`Sound`/`Sound`/`WholeWorkspace`,
`contract.rs:50-53`) — not a second offer against `nomos.cap.dependency.edges` or any existing
contract. Five capability families now exist (syntax, dependency-edges, controlflow, lint,
dependency-policy) where this record's first version measured three. The role-surface rule
(`Check_Declared_Role_Matches_Surface`) this record's first amendment already found
non-reconverging remains exactly as it was: unwired into `Run`, no capability contract, reading
plain committed files.

**All three remain unfired, with fresh evidence pointing further away from firing, not
toward it.** A sixth rule joined using the pattern under review, cleanly, using mechanism
already built for exactly this. A third subprocess-launching materialization step arrived and
is already covered by the same selection gate with no measured waste. A fifth capability
family was created rather than converging with a fourth. This record's decline continues to
hold on the evidence, not merely on precedent — each trigger got a genuine chance to fire
since the last check and did not.

## Amendment: The Fourth Trigger's Named Event Finally Occurred, And Its Own Wording Already Says What It Means

A fourth round of the same external review repeated its P0 case for the general planner
again. Rather than re-arguing what the three prior rounds already settled, this amendment
checks the one trigger this record's own text has never yet seen occur: "a fifth rule's
required-capability shape reconverging with an existing one, rather than adding a fourth
distinct family."

**`Check_Cross_Language_Correspondence` (`OD-CAPABILITY-010`) is that event, for the first
time since this record began tracking.** Every prior rule this record has checked — through
five capability families across three amendments — diverged: each added a structurally
distinct family (syntax, dependency-edges, controlflow, lint, dependency-policy). This one
does not. Its own module doc (`crates/rules/nomos-rules/src/crosslang.rs:1-11`) states
plainly: "No new capability: both sides are the same fact, read twice." It reads
`nomos.cap.syntax.items` — the same fact `Check_Completeness_Mirrors` and
`Check_Naming_Convention` already consume — over a subject pair a declared correspondence
names, and needs no new `Materialize_X` step in `run.rs`: "every source's
`nomos.cap.syntax.items` fact is already written by `Materialize_Syntax` before any rule
runs." The named event has occurred, literally, for the first time.

**This trigger's own text already says what a fired instance of it means, and it is not what
firing the other three meant.** This record's first amendment already read the trigger's
accompanying reasoning correctly without needing to say so explicitly: "a converging
population is weaker evidence for a general planner than a diverging one." A rule reconverging
with an existing capability family is the *cheap* case — no new fact, no new materialization
step, no new capability crate — and Check_Cross_Language_Correspondence's own zero-new-anything
shape is direct, first-hand confirmation of exactly that claim, not a counterexample to it. The
other three named triggers each describe a cost or a demand growing (selection creating unread
work, participation varying, a materialization step wasting real work); this one describes the
opposite — a rule arriving that costs *nothing new* to add. Naming its occurrence a reason to
build a planner would invert the trigger's own stated logic: the planner's case rests on
managing growing, diverging complexity, and this event is evidence the population is not
straining under that growth, not evidence that it is.

**The other three triggers remain exactly as the third amendment found them.** No selection
axis beyond `OD-GATE-017`'s has appeared; no materialization step has been measured wasting
real work; participation still varies only by the one mechanism already built.

**All four named triggers have now been checked against a real, occurred instance at least
once, and none supports building the planner now.** The first fired and was answered
narrowly (`OD-GATE-017`). The second and third remain unfired. The fourth has now fired,
literally, and its own accompanying reasoning — read correctly rather than merely quoted —
argues against the planner it might, misread, appear to argue for. This is the strongest
disconfirmation this record has recorded yet: not absence of evidence, but a real instance
whose own shape demonstrates the hand-written approach absorbing a reconverging rule at zero
marginal cost.

## Amendment: A Fifth Round Of The Same Review, Checked Against A Rule Population That Nearly Quintupled Without Touching Any Named Trigger

A fifth round of the same external review repeated its P0 case for the general planner again,
against this record's own citation, `nomos-check-orchestration::run::Run`'s hand-written
`Materialize_X`/`Check_Y` list, now read at a HEAD (`521cd4fa`) where that list has had four
amendments' worth of real growth to strain it. Rather than re-arguing what four prior rounds
already settled, this amendment checks the remaining triggers against the largest single burst
of rule growth this record has yet measured.

**The rule population grew from eight to thirty-nine, and the growth landed almost entirely
outside anything `Run` reads.** Counted directly against the tree at this amendment's last
checkpoint (`e4e262cf`) and again at `521cd4fa`: `nomos-rules` held exactly eight `Check_*`
functions at the fourth amendment; it holds thirty-nine now — a fifth-round increase this
record has not previously seen in one span. But `run_context.rs` (the file `run.rs` was split
into since the last amendment; same mechanism, new name) still wires exactly eight of them into
its `Wants`-gated match arms: `Check_Completeness_Mirrors`, `Check_Naming_Convention`,
`Check_Dependency_Direction`, `Check_Every_Member_Declares_A_Band`, `Check_Lint_Diagnostics`,
`Check_Dependency_Policy`, `Check_Unread_Reaches_A_Finding`, `Check_Cross_Language_
Correspondence`. The other thirty-one — `Check_No_Trailing_Whitespace`, `Check_Todo_Format`,
`Check_Boolean_Predicates`, the Go naming/constant/variable family, the deprecation-marker and
file-size-trigger rules, and every other rule imported since — have, verified by a direct
workspace-wide search, zero references outside `nomos-rules` itself. None is called by `Run`,
none states a `Materialize_X` step, none cites a capability contract. Each is the identical
"additive and unwired" shape this record's own text already established for the fifth rule,
`Check_Declared_Role_Matches_Surface`, now multiplied roughly thirtyfold rather than resolved.

**The one rule that did get wired reconverges again, using the mechanism already built.**
`Check_Every_Member_Declares_A_Band` (`cf76f0de`, `DEPENDENCY_COMPLETENESS`) is `OD-RULES-003`'s
own declared-architecture-vs-observed-fact design applied a second time, reading the identical
`nomos.cap.dependency` fact and `BANDS` table `Check_Dependency_Direction` already reads, gated
by `Is_Rule_Selected(selected, DEPENDENCY_DIRECTION) || Is_Rule_Selected(selected,
DEPENDENCY_COMPLETENESS)` — the same single `selected: &[RuleId]` axis `OD-GATE-017` built,
OR'd, not a second one. Its own commit states plainly: "No new capability, no new provider, no
new contract citation." Trigger 2 (participation varying by a second axis) and trigger 3 (a
materialization step measured wasting real work) remain exactly as unfired as the third and
fourth amendments found them.

**A sixth capability crate landed, and its own commit says it changes nothing yet.**
`nomos-cap-naming-policy` (`521cd4fa`) is a new, structurally distinct capability contract — not
a second offer against an existing one — which would ordinarily bear on trigger 4's diverging-
vs-converging count. But its own commit message forecloses that reading before this record has
to weigh it: "No provider and no rule reads it yet; both are this decision's own next
increments." A capability contract with no rule and no provider is not a rule's required-
capability shape at all, converging or diverging — it is scaffolding one commit ahead of the
question this record tracks, the same status this record already gave the crate at its prior,
uncommitted state. Trigger 4 remains fired only through `Check_Cross_Language_Correspondence`,
as the fourth amendment found, and unextended by this one.

**This is the strongest disconfirming round yet, and for a reason the review's own model does
not have room for.** The review's scaling argument assumes rule growth costs the hand-written
list something — a new `Materialize_X` step, a new selection axis, a new capability crate wired
in — proportional to rule count. This round's real growth mode is one the argument does not
anticipate: bulk import of rule functions that cost the list nothing at all, because they are
not wired into it. Thirty-one of thirty-nine rules now sit in that state. The list `Run` and
`gate-orchestration`'s `RuleRegistry` maintain has not grown by thirty-one entries; it has grown
by one, reconverging, absorbed by mechanism already built. A population multiplying nearly
fivefold while the hand-written surface it is measured against grows by one is not evidence the
hand-written approach is straining — it is a fifth consecutive data point that it is not.

## Status

Accepted. This record's first named trigger fired and was addressed by `OD-GATE-017`, not by
building the `RunPlanner` this record declines. The fourth trigger has now also fired, for the
first time, via `Check_Cross_Language_Correspondence` — and its own stated reasoning, checked
against that real instance, argues against the planner rather than for it. The second and third
triggers remain unfired through a fifth round, this one checked against the largest rule-count
burst this record has yet measured, whose growth landed almost entirely in a bucket — unwired,
zero-cost rule functions — the review's own scaling argument has no room for. Revisit if a
materialization step is measured wasting real work, participation varies by a second axis, an
unwired rule is wired into `Run` in a way that fires trigger 2 or reconverges/diverges under
trigger 4, or a *diverging* rule population resumes growing the hand-written list past a point
future evidence shows it stops absorbing cleanly.
