---
id: OD-ROADMAP-003
type: decision
title: Each of the three pauses OD-ROADMAP-002 set has lapsed on its own stated condition
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - roadmap
  - architecture
  - gate
  - model
  - analysis
relations:
  - target: OD-ROADMAP-002
    type: affects
  - target: OD-ROADMAP-001
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
  - target: OD-RULES-027
    type: relates-to
  - target: OD-GATE-023
    type: relates-to
  - target: OD-PACKAGE-016
    type: relates-to
---

# Each of the three pauses OD-ROADMAP-002 set has lapsed on its own stated condition

## Question

`OD-ROADMAP-002` paused three areas — new model-routing vocabulary, new gate policy
increments, and additional one-off capability orchestration — and bound each pause to a
named successor: "it lapses for each of them when a named successor lands", and, in its own
closing sentence, "if a successor is abandoned rather than built then the pause it justified
lapses with it." It set no date.

Seventeen days after it landed, an external review read against this tree at `bc0aaacf`
lists four things as core gaps: an `ApplicabilityPolicy`, gate-level evidence requirements,
authoring phases and approvals through `nomos-gate.json`, and composing
`nomos-lang-rust-compiler` into a real check run. Each of the four falls inside one of the
three paused areas, and each is closed today by nothing except the pause. Whether the pauses
still hold was unmeasured: nothing on the board and nothing in the records says either that a
successor landed or that one was abandoned, so a session meeting one of the four gaps finds a
standing prohibition with no way to tell whether its condition still exists.

This record measures each pause against its own condition. It answers whether the pause
holds, not whether the work behind it should be done.

## What Was Measured

Taken at `8338c6ec`, against the files named.

### The model-routing pause: its successor landed

The pause waited for "a real routing consumer", and explicitly allowed wiring the existing
vocabulary to one, because "that work reduces the gap this pause exists because of".

That wiring is done, in two items. `P40-MODEL-ROUTING-RESOLVER` built `Resolve_Profile` in
`crates/orchestration/nomos-agent-orchestration/src/profile_resolution.rs`, which resolves a
declared `ModelExecutionProfile` against a caller-supplied `DeclaredTarget` set into the
`DispatchConfig` a dispatch already takes, and answers every selector it cannot resolve with
a `ProfileAbsence` naming the measured absence rather than a default.
`P40-MODEL-ROUTING-DISPATCH-WIRING` made the dispatch go through it. `Selected_Dispatch` in
`backend_selection.rs` attempts a person's named preference against the declared set and
otherwise calls `Resolve_Profile`. It is reached by three consumers: `Dispatched_Agent` in
`crates/orchestration/nomos-workflow-orchestration/src/run.rs`, which passes
`preferred: None` and the platform's declared targets, so a workflow step reaches a backend
only because its profile resolved; `Requested` in `crates/host/nomos-cli/src/agent.rs`, which
declares a family label where, as its own doc records, "the line this replaced returned
`Backend::ClaudeCode` directly"; and `nomos-api`'s `Handle_Agent_Execute` and
`Handle_Agent_Judge_Role`, which take a `BackendSelection` from their caller. `Agent_Body` in
`crates/host/nomos-cli/src/workflow/parsing.rs` turns `--executor` and `--model-backend` into
a `ModelSelector::BackendFamily` the declared set answers, where it used to construct the
`Body` variant for that backend. Every host supplies the declared set from
`Declared_Targets()` in `declared_targets.rs`, which derives it from `Backend::ALL` rather
than listing it beside the enum.

`OD-PACKAGE-016` is the record that decided the resolver's shape, its crate, and the one
addition to vocabulary it authorized, `Backend::Label`. The condition was a real routing
consumer; there are three.

**Lapsed, by the successor landing.**

### The gate-policy pause: half its condition was measured harmless, and the other half was abandoned

The pause waited for two things at once: "for the gate to stop selecting twice and to compile
into one resolved run plan."

*Selecting twice.* `OD-GATE-023`, two days after the pause was set, measured what the double
selection actually was. `Run_Gate` in
`crates/orchestration/nomos-gate-orchestration/src/gate_environment.rs` hands
`command.rules.include` to `nomos_check_orchestration::Run` as its `selected` list, so a
deselected rule's materialization never runs and its finding never exists. `Reduced_Findings`
in `gate_environment/reduction.rs` then filters that same finding list by `rules.Is_Included`
once more, and the plan verb's `Run` in `run.rs` applies the same include list to the
registry's offers. `RuleSelector` in `policy/rule_selector.rs` carries one field, `include`,
so the two applications cannot disagree. `OD-GATE-023` called this "a redundant second
application of one include list, not two mechanisms disagreeing", declined to make it an
item, and declined `P41-GATE-SELECTION-COLLAPSE` because its defect was already fixed by
`OD-GATE-017` and its `done_when` — "a gate compiles its policy into a run plan plus a
result policy" — was the planner under another name. That is still what the tree holds. A
redundancy a record has judged not to be a defect cannot be the condition keeping a product
surface closed: nothing will ever stop it, because nothing needs to.

*One resolved run plan.* That is the run planner, and `OD-RULES-009` has declined it across
eight rounds of the same external review. Its 2026-09-14 amendment did more than decline
again: it measured the path end to end after `P102` deleted the surface it had been watching,
and found that "a planner today would schedule an ordering that does not exist, over a choice
that has no alternatives, using cache state nothing consults. That is speculative
infrastructure, not deferred infrastructure." `OD-ROADMAP-002` named exactly that case in its
last sentence — a successor abandoned rather than built — and by its own terms the pause
lapses with it.

The board had already crossed this pause on the same evidence. `OD-GATE-023` re-authored
`P40-GATE-PHASES-APPROVALS-2` "on its own stated condition having been met", and
`P40-GATE-PHASES-APPROVALS-5` landed `gate_phase.rs` — `GatePhase`, `PhaseThreshold`,
`PhaseApproval` — on 2026-09-06, verified 2026-09-07. Phases, thresholds and approvals are the
three increments `OD-ROADMAP-002` named as paused. They were built two days after the pause
was set, on a measured decision, and no record said the pause had lapsed. This record says
so.

**Lapsed, by the successor being abandoned.**

### The capability-orchestration pause: its successor was abandoned and its cost no longer exists

The pause waited for the run planner, the same artifact, and its stated cost was specific:
"another branch of the form 'if this rule was selected, call this materializer'", each one "a
line the planner migration must later delete".

The branch no longer exists to extend.
`P102-MATERIALIZATION-RESTATES-THE-DEMAND-DESCRIPTORS-ALREADY-DECLARE` replaced every
hand-written guard of that form with `Demanded_Families` in
`crates/orchestration/nomos-check-orchestration/src/run_context/capabilities.rs`: the union
of `RuleDescriptor::requires` over the selected rules, read from `nomos_rules::DESCRIPTORS`
(`crates/rules/nomos-rules/src/rule_descriptor.rs`). Each section asks
`demanded.contains(&RequiredFact::X)` and nothing else; the eight repository-declared policy
families are one row each in `Policy_Families()` and one arm each in
`Materialize_Policy_Family`, whose match is total over `RequiredFact`, so a new family is a
compile error until it is placed.
`Test_A_Rule_Selected_Alone_Should_Demand_The_Family_It_Declares` and
`Test_The_Demand_Should_Be_The_Union_Of_What_The_Selected_Rules_Declare` in
`capabilities/demand_tests.rs` pin the derivation. `OD-RULES-027` made the same move one axis
over, deriving the composed rule set from `DESCRIPTORS`, and `nomos-gate-orchestration`'s
`Registered` in `composition.rs` does the same for the registry, pinned by
`Test_Registered_Should_Offer_Every_Composed_Rule`.

So the thing the pause protected — a growing hand-written mapping the planner would later
have to delete — was deleted by a derivation rather than by a planner, and the planner was
found speculative in the same fortnight. A new materialization section today is a declared
row, not a branch.

**Lapsed, by the successor being abandoned, and the cost the pause was priced against gone.**

### The four gaps the review names, measured

- **`ApplicabilityPolicy`** is named in `ARC-ROADMAP-001`'s list of what a gate object
  configures and in `crates/orchestration/nomos-gate-orchestration/src/lib.rs`'s own doc as
  one of two elements "nothing in this workspace defines". No type of that name exists.
  `P41-APPLICABILITY-IN-THE-PLAN` was declined by `OD-GATE-023` because it asked for
  applicability *in the plan*, which needs the planner; a policy that does not need the plan
  was never authored.
- **Evidence requirements.** `nomos_contracts::Finding` carries an `EvidenceClass`
  (`crates/contracts/nomos-contracts/src/reporting/finding/evidence_class.rs`);
  `GatePolicyFile` in `policy/gate_policy_file.rs` has four fields — `suppressions`,
  `baseline`, `adoption`, `coverage` — and none reads it. The same `lib.rs` doc names this as
  the second undefined element.
- **Phases and approvals through `nomos-gate.json`.** The types exist and `GateCommand`
  carries `phases` and `approvals`, but `Resolve_Gate_Policy` reads neither from the file,
  and `crates/host/nomos-cli/src/gate/parsing.rs` passes `phases: Vec::new()`. A phase can be
  constructed and cannot be authored.
- **`nomos-lang-rust-compiler`** exports a `Provider_Offer`
  (`crates/languages/nomos-lang-rust-compiler/src/guarantee.rs`) and `Check_Copy_Clones`
  (`check.rs`), and no crate under `crates/orchestration` or `crates/host` names it.
  `crates/orchestration/nomos-check-orchestration/src/composition.rs` offers seventeen
  providers into the registry, and this one is not among them.

Each is real, and each sits in an area whose pause has lapsed. The first three are gate
policy increments; the fourth is a provider composition, which the third pause covered as
wiring a capability in.

## The Decision

**All three pauses have lapsed, each on the condition `OD-ROADMAP-002` set for it.** The
model-routing pause lapsed when its successor landed. The gate-policy and
capability-orchestration pauses lapsed when their shared successor was found speculative
rather than deferred, which is abandonment in `OD-ROADMAP-002`'s own terms; the one half of
the gate condition that was not the planner was measured to be a harmless redundancy rather
than a defect.

`OD-ROADMAP-001` governs the three areas again, as it governs everything `OD-ROADMAP-002` did
not pause: a component may be built before a consumer exists.

### The constraint that survives

What replaced the paused mechanisms is a set of declared tables, and the lapse is conditioned
on their staying tables. **A new gate policy family, a new provider composition or a new
materialization section is a declared constant — a `RequiredFact` a rule's descriptor names
and `Demanded_Families` reads, a `Provider_Offer` the composition root registers, a field
`Resolve_Gate_Policy` reads off `nomos-gate.json` — and never a condition that consults
store state, cost or prior materialization.**

This is the line `OD-RULES-027` draws — "the mapping may not grow a *condition*" — and the
line `OD-RULES-009` guards, stated there of `Demanded_Families`: it "reads no store state, no
`Materializations`, no cost and never asks whether a fact is already live". The sentence that
would violate it has the shape *materialize this family unless the store already holds it*,
*offer this provider when the other one would cost more*, or *this phase applies when the
last run's disposition was `Failed`*. The moment an entry reads like that it has stopped
being a declared fact and become the planner, and `OD-RULES-009` is where that has to be
argued rather than slipped in under this record.

The distinction is checkable at the site. A declared row can be added without reading
anything but the row; a condition needs an input the table does not have. That is what keeps
the lapse from becoming a licence to build the planner one branch at a time.

## What This Does Not Do

**It builds nothing.** `ApplicabilityPolicy`, gate-level evidence requirements, phase and
approval authoring through `nomos-gate.json`, and composing `nomos-lang-rust-compiler` into a
real check run are each now unpaused work for an item of its own, with its own territory and
its own measurement. This record removes the prohibition that stood in their way and decides
nothing about their shape.

**It does not reopen `OD-RULES-009`.** The planner stays declined on that record's own
measured reasons. Finding a pause lapsed because the planner was abandoned is the same
finding read from the other side, not a new argument for or against it.

**It does not withdraw `OD-ROADMAP-002`'s distinction.** Building ahead of a consumer and
building ahead of a seam are still different questions. What has changed is that the three
seams it named are no longer in flight: one landed, and two were measured not to be coming.
The rule binds again on a new named successor, per the next section.

**It does not judge the work the board did under the pause.** Phases and approvals landed on
`OD-GATE-023`'s measurement, not on this record; this record makes the board and the records
agree about what that measurement meant.

## What Would Decide It Differently

A pause is reinstated by a successor mechanism actually being built, which is observable:

- **The gate-policy and capability-orchestration pauses return** if any of the three triggers
  `OD-RULES-009`'s 2026-09-14 amendment names fires: a fact family whose production depends
  on another family's output, so that an order exists to get wrong; a capability with two
  installed providers where the choice is not obvious from the requirement alone; or a
  measured cost that makes materializing an unneeded family expensive enough that skipping it
  is worth deciding rather than deriving. Any one of those makes the planner deferred rather
  than speculative again, and work onto `Demanded_Families` or the gate policy reader is once
  more work onto a mechanism being replaced.
- **The model-routing pause returns** if `Resolve_Profile`'s output is made transitional — if
  `ResolvedModelExecution`, which `OD-PACKAGE-016` decision 7 keeps unbuilt, is authorized as
  the resolver's result and `DispatchConfig` becomes the shape being replaced. New routing
  vocabulary would then be built ahead of that seam.
- **The surviving constraint is falsified** the day a row in any of the three tables reads a
  condition. That is not a reinstated pause; it is the planner arriving without a record, and
  the remedy is `OD-RULES-009`, not this one.

## Status

Accepted. `OD-ROADMAP-002` is amended to point here rather than restating any of it, the same
way `OD-ROADMAP-001` points at `OD-ROADMAP-002`. The three areas return to `OD-ROADMAP-001`'s
licence under the one constraint above, and the four gaps the review named are unpaused for
items of their own.
