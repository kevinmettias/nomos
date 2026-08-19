---
id: OD-GATE-014
type: decision
title: Whether Gate's ScopeSelector and RuleSelector are built now, or wait for a caller that needs to select less than everything
status: open
version: 2
authority: canonical-normative-record
tags:
  - gate
  - orchestration
  - rules
  - architecture
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
  - target: D-135
    type: relates-to
---

# Whether Gate's ScopeSelector and RuleSelector are built now, or wait for a caller that needs to select less than everything

## Question

`ARC-ROADMAP-001` names the product-level `Gate` object's first two configurable policy
types as `ScopeSelector` and `RuleSelector` — what an end-user repository would set to pick
which files and which rules a gate evaluates. `nomos-gate-orchestration`'s first increment
gave `Gate` its own crate and its `Plan` a real `RuleRegistry` consumer, but deliberately left
both selector types unbuilt: `GateCommand::root` is "accepted and carried, not read," and
`GatePlan` reports every rule the registry holds, unconditionally. The open question is
whether that second increment — real `ScopeSelector`/`RuleSelector` types that actually
filter `GatePlan` — is built next because `ARC-ROADMAP-001` already named the types, or
waits for a real caller that needs to select fewer than all rules or all files.

## Current Position

`OD-HOST-004` already decided the immediately adjacent case: `nomos-check-orchestration::
Run` "stays hand-written for as long as every rule it calls runs unconditionally, on every
invocation," and flips only "the first time a rule... is meant to run for *some* check
invocations and not others." Verified directly against the real code, not assumed:
`Run` (`crates/orchestration/nomos-check-orchestration/src/run.rs`) now calls three shipped
rules, `Check_Completeness_Mirrors`, `Check_Naming_Convention` and (since `P13-DEPENDENCY-
WIRE-1`) `Check_Dependency_Direction`, unconditionally over every source it is handed. Only
one language provider, `nomos-lang-rust`, exists, so no per-language variance is
demonstrated either — nothing in this workspace today wants to run a subset of rules or a
subset of scope. This repository's own use of the rule layer (`OD-GATE-004`'s CI step)
always wants all three rules over everything it is given.

`ARC-ROADMAP-001` itself disclaims settling this: its "What This Record Does Not Do" section
states plainly that it "does not order the near-term tier internally. Which of those items is
built next is a separate judgment, informed by this record but not fixed by it." Naming
`ScopeSelector`/`RuleSelector` as the first two policy types a Gate would carry is not the
same as naming them the next increment to build — the same distinction `OD-ANALYSIS-004`
drew between a program-semantics capability's worked shapes and picking one to build first
(`OD-ANALYSIS-007`).

Building either selector type now, on the strength of `ARC-ROADMAP-001`'s naming alone, would
fix a field shape — glob include/exclude for scope, allow-list or deny-list of `RuleId` for
rules, or something else — before any real caller exists to hold that shape to account. This
is the same mistake `D-135` already named elsewhere: inferring a whole configuration surface
from a wish rather than a demonstrated concrete need, the identical reasoning `OD-PACKAGE-006`
and `OD-PACKAGE-008` already applied to provider and rule registration, and `OD-HOST-004`
already applied to this exact crate family's selection question.

This is deliberately not treated as covered by the user's 2026-08-18 direction to prioritize
language/rule plugin registration infrastructure ahead of this repository's own
wait-for-a-second-instance default. That override is about the registration seam — how a
provider or rule package joins a registry independently — not about runtime selection policy
for an already-registered rule set. `Gate`'s selectors do not gate whether a rule package can
register; `RuleRegistry::Offer` already does that unconditionally, for any count of rules.
Nothing about `ScopeSelector`/`RuleSelector` blocks independent plugin or package development,
the same distinction that kept `OD-RULES-005`, `OD-RULES-006` and `OD-CAPABILITY-007` as open
questions rather than builds despite arising from the same prioritized work.

## `run` Arrived: What Changed And What Did Not

`P13-GATE-RUN-FIRST-INCREMENT-3` gave `Gate` a real `run` verb, closing part of what this
record was waiting on. Verified directly against the real code, not assumed: `nomos gate
run` (`crates/host/nomos-cli/src/gate/run.rs`) walks the tree exactly as `nomos check`
does, calls `nomos_check_orchestration::Run` unconditionally over everything it finds, and
reduces the result through `nomos_gate_orchestration::Disposition` — no scope or rule
filtering happens anywhere in that path, the same "unconditionally, over everything it is
handed" shape `Run` itself has always had. `run`'s real shape is exactly what this record
already predicted it would be: nothing here surprised the prediction, only confirmed it.

`run` still has no real caller. Verified directly against `.github/workflows/gate.yml`, not
assumed: the `Rules` step — `OD-GATE-004`'s own CI step — still invokes `cargo run --quiet
-p nomos-cli --bin nomos -- check --root .`. `nomos check` directly, not `nomos gate run` or
`nomos gate plan`. This repository's own enforcement does not go through `Gate` at all yet.

## What Would Decide It

The risk `run`'s absence posed — building either selector type against a hypothetical `run`
whose real shape might not match what got built — is gone: that risk is what this record's
second, independent trigger named, and it has fired. It has not resolved the record's first
and primary trigger, which remains exactly as unmet as before: a real caller that needs to
evaluate fewer than all registered rules, or fewer than every file a root contains — this
repository's own CI wanting to run only one rule over a subset of paths, or a second,
genuinely different repository or configuration wanting a different rule set than this one.
Nothing calls `gate run` at all yet, so no such caller exists to name either selector's real
shape. `run` existing only means that shape, whenever a caller arrives to want it, can now be
checked against a real reduction pipeline instead of a hypothetical one — the same way
`Check_Naming_Convention`'s arrival gave `OD-RULES-006` a second rule to compare rationale
against instead of one.

## Status

Open. `Gate` has a real `run` verb now (`P13-GATE-RUN-FIRST-INCREMENT-3`,
`P13-GATE-014-015-RUN-TRIGGER-FIRED`), which closes the risk of shaping a selector around a
`run` that did not yet exist. It still has no real caller: `OD-GATE-004`'s CI step invokes
`nomos check` directly, not `Gate`, and no consuming repository has adopted `Gate`. Revisit
when `gate run` gains a real caller that needs to evaluate fewer than every registered rule
over fewer than every file — this repository's own CI switching to it and wanting a subset,
or a second repository wanting a different rule set.
