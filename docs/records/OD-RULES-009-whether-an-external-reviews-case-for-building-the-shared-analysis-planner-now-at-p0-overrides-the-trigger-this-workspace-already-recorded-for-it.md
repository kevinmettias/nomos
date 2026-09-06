---
id: OD-RULES-009
type: decision
title: Whether an external review's case for building the shared analysis planner now, at P0, overrides the trigger this workspace already recorded for it
status: accepted
version: 7
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
  - target: OD-GATE-020
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

## Amendment: A Sixth Round, Checked Against A Registry That Stopped Absorbing And A Materialization Step That Stopped Being Free

A sixth round of the same external review cited the measured divergence between `Run`'s
composed rule count and `nomos-gate-orchestration`'s registry as new evidence, rather than
repeating the P0 framing unchanged. Rather than re-arguing what five prior rounds already
settled, this amendment checks that measurement directly, and checks the two triggers still
unfired at the fifth amendment against the growth since.

**Trigger 1 is not reopened, but this record's own most-cited supporting number was assumed
stable and was not.** The fifth amendment's closing argument rested in part on "the list `Run`
and `gate-orchestration`'s `RuleRegistry` maintain has not grown by thirty-one entries; it has
grown by one" — true at that amendment's checkpoint, where both stood at eight. `OD-GATE-020`,
decided in the same session as this amendment, measured the same pair at current HEAD: `Run`
composes fifty-six rules; the registry still offers eight. The two lists this record has cited
since its first amendment as evidence of "mechanism already built ... absorbing" have, since the
last check, diverged by forty-eight rather than staying level. This does not reopen trigger 1 —
that trigger was about `Run` itself gaining a selector, which it has and keeps — but it corrects
the premise this record's own text used as supporting evidence. `OD-GATE-020` applied
`OD-GATE-011`'s own legitimate-exception test to the pair directly and found it fails the test's
first condition (no shared derivation from one named external authority), so `OD-GATE-019`'s two
prior corrections narrowed a real instance of `OD-GATE-011`'s named defect class without ever
making it a legitimate exception to it.

**Trigger 3 has its first real, if narrow, instance.** This record's own "efficiency case"
claim — "no rule in this workspace recomputes a fact another rule already produced" — no longer
holds without qualification. `run_context.rs::Materialize_Capabilities` (`run_context.rs:
232-247`) now runs `Materialize_Naming_Policy_Section`, `Materialize_Limits_Policy_Section`,
`Materialize_Scripting_Policy_Section`, `Materialize_Goals_Policy_Section` and
`Materialize_Words_Policy_Section` in sequence within one call, each gated on its own rule
selection per `OD-GATE-017`'s mechanism, and each — verified directly — independently reads and
parses the identical `standards.json` (`nomos-repo-standards`, `-limits`, `-scripting`, `-goals`
and `-words`'s own `Discover_Workspace` functions, each with its own `Read_To_String`/
`serde_json::from_str`). A normal full `nomos check` run, selecting all five families' rules,
launches five reads and five parses of one file in one invocation. This is not the exact shape
the trigger's own wording anticipated — five *different* facts rather than one fact recomputed —
but it is the same underlying inefficiency the shared-fact-DAG argument targets, and it is real,
measured waste where the fifth amendment found none. `P33-RULES-019-RECORD`, filed independently
in the same session, already carries this instance as its own decision item; this amendment does
not re-decide it, and notes only that the trigger's "not yet observed" status from the third and
fifth amendments no longer holds.

**Trigger 4's population diverged further, not less, and the divergence is now real rather than
scaffolding.** Ten capability contract crates now exist under `crates/capabilities/`
(`nomos-cap-controlflow`, `-dependency`, `-dependency-policy`, `-goals-policy`, `-limits-policy`,
`-lint`, `-naming-policy`, `-scripting-policy`, `-syntax`, `-words-policy`), against five at the
fifth amendment's checkpoint and the sixth crate (`nomos-cap-naming-policy`) that amendment found
was "scaffolding one commit ahead of the question this record tracks... no provider and no rule
reads it yet." All five of the new ones now have a real provider and at least one real, wired
rule reading them — `Materialize_Capabilities`'s own five new sections above are exactly that
wiring. None of the five is a second offer against an existing contract; each is `OD-RULES-011`'s
own family, a structurally distinct `nomos.cap.*.policy` contract in its own crate. This
continues the pattern every round but the fourth has found; the fourth's own reconverging
instance (`Check_Cross_Language_Correspondence`) remains the only rule this record has found
reusing an existing family rather than adding one.

**Trigger 2 remains exactly as unfired as every prior round found it.** `run_context.rs`'s only
participation gate is still `Is_Rule_Selected` against `selected: &[RuleId]` (`run_context.rs:
667`); `Check_Declared_Role_Matches_Surface`, the rule the first amendment already found additive
and unwired, still has zero references in `run_context.rs` — unchanged since that amendment, now
three rounds later.

**Neither new finding argues for building the `RunPlanner` this record has declined five times,
and the workspace's own response to each new instance continues to be the narrow one this record
has documented from its first amendment on.** Both new pieces of evidence — the registry
divergence and the repo-policy read duplication — already have their own filed decision items
(`OD-GATE-020`, accepted; `P33-RULES-019-RECORD`, ready) rather than a generalized mechanism
proposed to cover both at once. That is itself data bearing on the review's own repeated
argument: six rounds in, every real problem this workspace has found under the review's general
banner has been real, and every one has been fixed, or is being decided, narrowly, by an item
scoped to the actual instance, not by the abstraction the review keeps proposing ahead of one. A
planner remains undeclined-against in principle; it remains, six rounds in, still short of a
trigger that asks for it specifically rather than for a narrower fix at the site where growth
actually happened.

## Amendment: A Seventh Round, Naming The Same Mapping OD-GATE-017 Already Named And Finding The Registry Gap Closed Rather Than Open

A seventh round of the same external review named `run_context.rs`'s `Materialize_*_Section`
functions as its own highest-priority finding, proposing that each section's gate — a hardcoded
chain of `Is_Rule_Selected(selected, X) || Is_Rule_Selected(selected, Y) || ...` naming specific
`RuleId`s by hand — be replaced by a lookup against `RuleDescriptor.requires`, so a new rule
needs only a `DESCRIPTORS` entry rather than a second hand-written one in this crate. Checked
directly against the tree at this amendment's checkpoint rather than assumed novel: this is the
identical shape `OD-GATE-017` already named and declined to build — "a fifth rule still needs
its own hand-written entry in both places" — restated against nine sections instead of four, not
new evidence bearing on any of the four triggers this record tracks.

**Trigger 1 is not reopened, and the gap `OD-GATE-020` measured is now closed rather than merely
decided narrowly.** `nomos-gate-orchestration::composition::Registered`
(`crates/orchestration/nomos-gate-orchestration/src/composition.rs:90-108`) no longer builds its
registry from a hand-written list at all: it loops over `nomos_rules::DESCRIPTORS` directly,
`Offer`-ing every descriptor, and calls `nomos_check_orchestration::Resolve_Rules` against
`Declared_Rules()` and `Composed_Rules()` first so a contradiction refuses before the registry is
built. `Test_Registered_Should_Offer_Every_Composed_Rule` pins the result against `Composed_Rules`
directly rather than a written-out identifier list, and `nomos-check-orchestration`'s own
`Test_Every_Composed_Rule_Should_Be_Declared_And_Nothing_Else` pins `Declared_Rules()` against
`Composed_Rules()` the same way. Both pass, verified directly, at `DESCRIPTORS.len() == 66`,
matching `run_context.rs`'s own `RULE_COUNT`. The registry-versus-`Run` divergence this record's
fifth amendment mis-stated as converging and the sixth amendment found had widened to forty-eight
is gone, not narrowed — closed by exactly the kind of derived, tested pairing `OD-GATE-011`'s
legitimate-exception test asks for, built for rule *identity* rather than for the materialization
question this amendment's own review round re-raised.

**Trigger 2 remains unfired, checked directly.** Every `selected` parameter in
`run_context.rs` is still typed `&[RuleId]`, and no second selection axis exists anywhere in
`nomos-check-orchestration` or `nomos-rules` — no `ScopeSelector`, no per-request parameter
beyond the one `OD-GATE-017` built. Seven rounds in, participation still varies along exactly one
axis.

**Trigger 3 is decided, not merely tracked.** `P33-RULES-019-RECORD` — "whether the five
repo-policy providers owe `standards.json` one shared read, or independent parsing stays the
accepted cost of one-capability-one-provider" — is `declined`. Independent parsing is the
accepted cost; this record's own prior Status line describing it as "not decided here" is now
stale and is corrected below.

**Trigger 4 has not diverged further.** `crates/capabilities/` still holds exactly the ten crates
the sixth amendment counted, unchanged.

**The population this record measures for absorption has grown again, and is absorbing better,
not worse.** `nomos-rules` now exports eighty-five `Check_*` functions (`grep -c "^pub fn Check_"`
against every file under `crates/rules/nomos-rules/src/`), against `DESCRIPTORS`,
`Declared_Rules`, `Registered` and `Composed_Rules` holding steady at sixty-six, all four
synchronized and tested rather than independently maintained. Nineteen rules sit additive and
unwired — smaller a share of the whole (nineteen of eighty-five) than the thirty-one of
thirty-nine the fifth amendment found, the opposite of what a straining hand-written surface
would show.

**Neither this round's proposal nor its supporting evidence fires a trigger this record does not
already track, and the one number that changed moved the wrong direction for the review's own
argument.** Seven rounds in, the pattern the sixth amendment named continues: real problems this
workspace has found under the review's general banner are fixed or decided narrowly at the site
where growth actually happened — trigger 1's gap by generic derivation scoped to rule identity,
trigger 3 by a decision that independent parsing is fine — never by the standing abstraction the
review keeps proposing ahead of a trigger that asks for it.

## Amendment: An Eighth Round, Whose Architectural Case Is Unchanged And Whose Real Contribution Was Finding Stale Prose By Accident

An eighth round of the same external review restates this record's central recommendation once
more -- build the generalized analysis planner at P0, ahead of further rule work -- and adds
four adjacent claims: that `RulePackage` ownership is inverted, that workflow dispatch is
vendor-coupled, that the capability-contract crates are over-fragmented, and that the project
should stop feature work for a consolidation phase. It also raises three questions this record
does not reach, which became `OD-HOST-011`, `OD-HOST-012` and `OD-AGENT-004` rather than
amendments here.

### The seventh amendment made a measurement error, and this one corrects it

The seventh amendment stated, under trigger 2: "no second selection axis exists anywhere in
`nomos-check-orchestration` or `nomos-rules` -- no `ScopeSelector`, no per-request parameter
beyond the one `OD-GATE-017` built."

**The parenthetical was false when it was written.**
`crates/orchestration/nomos-gate-orchestration/src/policy/scope_selector.rs` exists, is a real
`ScopeSelector` with `include`/`exclude` prefix lists and an `Is_In_Scope` predicate, is a field
on `GateCommand`, and its file was last touched on 2026-08-30 by `P24-REGRESSION-ORCHESTRATION`
-- a week before the seventh amendment was committed on 2026-09-06. `gate_plan.rs` dates it
further back still, to `P13-GATE-014-SCOPE-RULE-`. It was not absent; it was not looked for
outside the two crates the sentence named.

**And it is reached by real callers narrowing real work, not carried and ignored.**
`nomos-cli::gate::parsing` turns repeated `--include`/`--exclude` arguments straight into
`ScopeSelector`'s two lists; `nomos-api-transport::gate_parameters` parses the same selector off
the wire. `crates/host/nomos-cli/tests/gate_orchestration_seam.rs` pins the ordering that
matters: `Run_Gate` "scopes the walk before ever calling `nomos_check_orchestration::Run`", and
a scope admitting nothing reports the same `ExitCode::Vacuous` an empty walk does. So the work
removed is removed ahead of the judging seam rather than filtered out of its results afterward.

**The trigger it was supporting is nevertheless still unfired, for a better reason than the one
given.** `ScopeSelector` narrows *sources*, not rules: `gate_environment::Scoped_Sources`
filters a `Vec<SourceFile>` before `Run` is ever called, and every selected rule still runs
over every source that survives. No rule runs for some invocations and not others, which is
trigger 2's actual wording ("a rule meant to run for *some* check invocations and not others").
The narrow claim -- that no second axis exists *inside* `nomos-check-orchestration` -- is true.
The bare claim that no `ScopeSelector` exists was not.

This cuts the way this record already goes, and more sharply than the sentence it replaces. A
real second selection axis was added to this workspace, by a real caller, and it was
implemented as one `filter` over a vector ahead of the judging seam. That is a second axis
arriving *without* a planner underneath it, which is stronger evidence for this record's
standing decline than the absence the seventh amendment claimed.

### The other three triggers, measured at this round

**Trigger 1 (selection creating unread work)** fired once and was answered narrowly by
`OD-GATE-017`; unchanged. The registry-versus-`Run` divergence the sixth amendment found is
still closed by derivation from `DESCRIPTORS`.

**Trigger 3 (a materialization step costing real wasted work)** is decided, not tracked:
`P33-RULES-019-RECORD` declined a shared `standards.json` read, and independent parsing is the
accepted cost.

**Trigger 4 (a diverging capability population)** moved by exactly one, and the seventh
amendment's count of it is now stale. `crates/capabilities/` holds **eleven** crates, not the
ten it recorded: `nomos-cap-requirement-trace` joined via
`P42-REQUIREMENT-TRACE-STALENESS-RULE-2`. One crate in a family of ten is not divergence
resuming, and `OD-PACKAGE-015` already governs whether that family's boundaries are earned at
all.

**The population absorbed better again.** `nomos-rules` exports **eighty-six** `Check_*`
functions against `DESCRIPTORS`, `Declared_Rules`, `Registered` and `Composed_Rules` holding
together at **seventy** -- `run_context.rs`'s `RULE_COUNT` reads 70, against the 66 the seventh
amendment measured. Sixteen of eighty-six sit additive and unwired, down from nineteen of
eighty-five. Four rules were composed in since the last round, and the four synchronized
authorities stayed synchronized without anything being hand-maintained into agreement.

### What this round's five overlapping claims are worth

All five are restatements. The planner case is this record's own subject, eight rounds running.
The `RulePackage` claim describes `declared_rules.rs` accurately -- one `PackageVersion` for the
set, `AlwaysSupported` across the board, `NO_STATED_FLOOR` capability requirements -- but reads
as concealment what that module states in its own doc comments as deliberate, each with its
reason and one with an openly named gap (`OD-PACKAGE-008`'s structurally-partial rule, for which
`RuleDescriptor` carries no field). The vendor-coupling claim describes `Body` accurately and is
answered at the site by `OD-EXECUTOR-001`, `OD-EXECUTOR-004` and `OD-PACKAGE-013`, whose shared
trigger -- a second instance of either role -- has not fired: the population is one
`AgentExecutor` and one `ModelBackend`. The crate-granularity claim proposes a boundary test
`OD-PACKAGE-015` already states, including the isolation clause the review presents as missing.
The consolidation freeze is the recommendation `OD-ROADMAP-001` has now declined five times.

### What the round actually produced, which was not an architectural argument

Its sixth point asserted that source-tree walking is duplicated between `nomos-cli` and
`nomos-api` because "the platform port does not expose directory enumeration", and advised
against adding a listing operation to `nomos-platform`.

**The duplication is real and the stated reason was false.** `OD-PLATFORM-002` gave `FileSystem`
a `Read_Directory` on 2026-09-05, one day before the review. The reviewer did not invent the
premise: it was read out of this workspace's own module documentation, which had restated it at
fifteen sites across eight crates and one test crate, plus `OD-HOST-001`, `OD-HOST-002` and
`OD-LEDGER-025`, none of which had been updated. `nomos-surface-provenance::discovery` quoted
the false sentence out of another module as its own stated authority. Two sites enumerated the
port's operations by name and were wrong about its shape rather than about one operation.

So the eighth round's contribution to this workspace was to act as an unintentional detector for
`OD-GATE-011`'s defect class, and to demonstrate its cost in the most legible form available: an
outside reader, reasoning carefully from committed documentation, reached a recommendation that
was wrong because the documentation was. `P72-STALE-PLATFORM-DIRECTORY-CLAIM` and its two
follow-ups corrected all fifteen sites and all three records. `OD-AGENT-004` takes up whether
restating a reason rather than routing to it is the underlying defect.

**Neither the round's proposal nor its evidence fires a trigger this record does not already
track.** Eight rounds in, the pattern holds without exception: what this workspace finds under
the review's banner gets fixed at the site where the problem actually is -- a stale clause, a
divergent registry, a duplicated parse -- and never by the standing abstraction the review keeps
proposing ahead of a trigger asking for it. The decline stands.

## Status

Accepted. This record's first named trigger fired and was addressed by `OD-GATE-017`; the fourth
fired once, via `Check_Cross_Language_Correspondence`, arguing against the planner on its own
terms. The third trigger fired once, narrowly, via the repo-policy family's duplicated
`standards.json` reads, and is now decided: `P33-RULES-019-RECORD` declined a shared read,
leaving independent parsing as the accepted cost. The second remains unfired through eight
rounds, but not for the reason the seventh amendment gave: a real `ScopeSelector` has existed in
`nomos-gate-orchestration` since well before that amendment denied one, and narrows *sources*
ahead of `Run` rather than varying which rules participate — a second axis that arrived without
a planner underneath it, which the eighth amendment corrects and reads as evidence for this
decline rather than against it. The registry-versus-`Run` divergence `OD-GATE-020` measured is closed, not merely decided
narrowly: `nomos-gate-orchestration::composition::Registered` derives from `nomos_rules::
DESCRIPTORS` directly and is pinned against `Composed_Rules` by a real test, the same shape
`Declared_Rules()` already used against `Composed_Rules()` on the `nomos-check-orchestration`
side. Revisit if participation varies by a second axis, or if a *diverging* rule population
resumes growing the hand-written materialization surface past a point future evidence shows it
stops absorbing cleanly, or if a future round's proposal names a trigger this record does not
already track rather than restating one already found unfired.
