---
id: OD-GATE-014
type: decision
title: Whether Gate's ScopeSelector and RuleSelector are built now, or wait for a caller that needs to select less than everything
status: accepted
version: 5
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
`Run` (`crates/orchestration/nomos-check-orchestration/src/run.rs`) now calls four shipped
rules, `Check_Completeness_Mirrors`, `Check_Naming_Convention`, `Check_Dependency_Direction`
(since `P13-DEPENDENCY-WIRE-1`) and `Check_Unread_Reaches_A_Finding` (since
`P13-CONTROLFLOW-REACHABILITY-WIRE`), unconditionally over every source it is handed. Only
one language provider, `nomos-lang-rust`, exists, so no per-language variance is
demonstrated either — nothing in this workspace today wants to run a subset of rules or a
subset of scope. This repository's own use of the rule layer (`OD-GATE-004`'s CI step)
always wants all four rules over everything it is given.

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

## `run` Gets A Real Caller: What Changed And What Did Not

`P13-GATE-RUN-CI-CALLER` gave `run` its first real caller. Verified directly against the
real workflow file, not assumed: `.github/workflows/gate.yml`'s `Rules` step now invokes
`cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .`, not `check`. This
repository's own build now depends on `Gate`'s disposition rather than on `check`'s exit
code taken directly — the deeper condition `OD-GATE-015`'s own trigger names, resolved for
that record by this same change.

It does not resolve this record's own trigger. The caller CI gained judges everything the
walk finds, unconditionally, over the whole tree — `--root .`, unchanged, still written out
explicitly so narrowing it stays visible in a diff. Nothing about the switch from `check` to
`gate run` asks for fewer than every registered rule or fewer than every file; it is the
identical unconditional shape `run` already had, wearing a different name in the step's
`run:` line. This record's own primary trigger — a real caller that needs to evaluate a
subset — remains exactly as unmet as before.

## What Would Decide It

The risk `run`'s absence posed — building either selector type against a hypothetical `run`
whose real shape might not match what got built — is gone: that risk is what this record's
second, independent trigger named, and it has fired. It has not resolved the record's first
and primary trigger, which remains exactly as unmet as before: a real caller that needs to
evaluate fewer than all registered rules, or fewer than every file a root contains — this
repository's own CI wanting to run only one rule over a subset of paths, or a second,
genuinely different repository or configuration wanting a different rule set than this one.
`gate run` now has a caller, but that caller — this repository's own CI — wants exactly what
`run` already gives it: everything. No such caller exists yet to name either selector's real
shape. `run` existing, and now being called, only means that shape, whenever a caller arrives
to want it, can now be checked against a real reduction pipeline instead of a hypothetical
one — the same way `Check_Naming_Convention`'s arrival gave `OD-RULES-006` a second rule to
compare rationale against instead of one.

## Built Under Override, Not By The Primary Trigger Firing

`P13-GATE-014-SCOPE-RULE-SELECTORS` built both types this record was waiting on, but the
primary trigger named above — a real caller that needs to evaluate fewer than every
registered rule over fewer than every file — still never fired. What changed is the same
kind of event the "What Would Decide It" section already distinguished from a real caller
for the plugin-registration case: the user gave a standing, explicit override of this
record's own wait, the same shape as the 2026-08-18 direction this record already declined
to read as covering selector policy. That direction was scoped to registration machinery;
this one is scoped to `ScopeSelector`/`RuleSelector` by name, so the distinction this record
drew between the two does not apply to itself here — the override names this record
directly rather than being read into it by analogy.

Verified directly against the real code, not assumed: `ScopeSelector`
(`crates/orchestration/nomos-gate-orchestration/src/scope_selector.rs`) is `include`/
`exclude` lists compared to `nomos_rules::SourceFile::path` by textual prefix containment —
no glob engine, the same soundness choice `OD-LEDGER-013` already made for ledger territory
over glob-against-glob comparison — consulted by `Run_Gate` before
`nomos_check_orchestration::Run` is called, so an out-of-scope file is never judged at all.
`RuleSelector` (`.../rule_selector.rs`) filters findings by `RuleId` after `Run` returns and
before `Disposition` reduces them: a real narrowing of what can fail a build, but honestly
short of a real narrowing of what runs — `Run` still executes every rule unconditionally
over every source it is handed, and changing that needs a signature change to `Run` itself
across every one of its callers, which this increment does not attempt. Both fields default
to select-everything, so every construction site that predates them, and this repository's
own `gate run --root .` in CI, are unchanged in behavior.

## Status

Accepted. `ScopeSelector` and `RuleSelector` exist and are consulted by a real `run`, closing
what this record asked. Not by the primary trigger this record names — no caller has yet
asked for fewer than every rule over fewer than every file — but by the user's own standing
override, recorded above rather than left to be inferred from a different one. Nothing
further to revisit: the fields this record asked for are built, and any future narrowing of
their shape (a real glob syntax, a deny-list form, `Run` itself gaining a per-call rule
subset) is a new question for a new record, not a reopening of this one.
