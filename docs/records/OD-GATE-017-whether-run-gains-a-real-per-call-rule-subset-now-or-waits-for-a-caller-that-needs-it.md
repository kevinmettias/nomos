---
id: OD-GATE-017
type: decision
title: Whether Run gains a real per-call rule subset now, or waits for a caller that needs one
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - gate
  - orchestration
  - rules
  - architecture
relations:
  - target: OD-GATE-014
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
  - target: OD-RULES-008
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
---

# Whether Run gains a real per-call rule subset now, or waits for a caller that needs one

## Question

`OD-GATE-014` built `ScopeSelector` and `RuleSelector` under a standing user override, but its
own closing section named a boundary it deliberately left short of: `RuleSelector` filters
findings by `RuleId` only *after* `nomos_check_orchestration::Run` has already computed every
one of them, "honestly short of a real narrowing of what runs," because changing that "needs a
signature change to `Run` itself across every one of its callers, which this increment does not
attempt." Its last line named the next question directly: "any future narrowing of their shape
... `Run` itself gaining a per-call rule subset ... is a new question for a new record, not a
reopening of this one." This record is that one.

It also answers a trigger from a separate lineage. `OD-RULES-009` measured an external
architecture review's case for building `ARC-ROADMAP-001`'s full "shared demand planner" —
rules declare capability requirements, a planner deduplicates them into one fact DAG and
resolves providers generically, and both `check` and `gate` execute through the planned result
— and declined to elevate it to P0, on the ground that its own recorded trigger, "selection
creating unread work," had not fired: `OD-GATE-014` was "still open" at that record's HEAD, and
"nothing in this workspace today wants to run a subset of rules or a subset of scope."
`OD-RULES-009`'s own "What Would Decide It" section named the concrete case that would fire it:
"`OD-GATE-014`'s `ScopeSelector`/`RuleSelector` getting built is the concrete case this would
arrive through." That has since happened, and a real caller now depends on it on every
invocation that uses it.

## What Was Measured

**A real caller already selects fewer than every rule, and `Run` cannot see it.** Verified
directly against the real code, not assumed: `nomos gate run --rule` (`Named_Values(rest,
"--rule")`, `crates/host/nomos-cli/src/gate/parsing.rs`) constructs a non-empty `RuleSelector`
that `Run_Gate` (`crates/orchestration/nomos-gate-orchestration/src/run_gate.rs`) already
narrows `blocking_findings`, `calibrated_findings`, `suppressed_findings` and
`baselined_findings` by, through `Reduced`. `Run_Gate`'s own private `Judged` helper calls
`nomos_check_orchestration::Run` with no rule argument at all — a user who runs `nomos gate run
--rule naming-convention` today still pays for `cargo metadata`
(`Materialize_Dependencies`), the reachability heuristic (`Materialize_Reachability`) and all
four `Check_*` calls, and `check_outcome` still carries dependency-direction and
unread-reaches-finding findings that no path they took asked for and that `Reduced` discards
immediately after computing them.

**`OD-HOST-004`'s own flip condition has fired.** That record states plainly: "`Run` flips the
first time a rule ... is meant to run for *some* check invocations and not others — selected by
something the request states, rather than by whether the crate happens to know about it."
`--rule` is exactly that: a request-carried property `GateCommand.rules` already holds, not
something `Run`'s own body decides. This is not a hypothetical future rule or a wish; it is the
CLI flag as shipped and callable today.

**`OD-RULES-009`'s named trigger has fired, and only that one.** Its "What Would Decide It"
list named four triggers. The first — "selection creating unread work," concretely `OD-GATE-
014`'s selectors getting built — has fired, exactly as that record predicted it would arrive.
The other three have not, checked directly at this record's own HEAD rather than assumed:
`nomos-rules` gained a real fifth rule while this record was being written
(`P14-RULES-005-ROLE-SURFACE-AGENT-REQUIRED`, `Check_Declared_Role_Matches_Surface`), but its
own commit states it is "additive and unwired into `Run` — the same first-increment shape
`RuleRegistry` itself used," the identical two-stage pattern `Check_Unread_Reaches_A_Finding`
already went through. `Run` (`run.rs`) still calls exactly the same four rules it always has;
nothing about this fifth rule's arrival changed what `Run` computes. Its shape is also evidence
against, not for, "a fifth rule's required-capability shape reconverging with an existing one":
it reads plain committed files rather than a `FactReader`, states no capability contract at all,
and raises every finding as unconditionally `Applicability::AgentRequired` — a structurally new
family, not a second instance of `nomos.cap.syntax.items`, `nomos.cap.dependency.edges` or
`nomos.cap.controlflow.reachability`. Nor has any materialization step been measured to cost
real, wasted work for a population where facts are shared and reused — verified directly,
`run.rs` still writes every fact into one `MemoryFactStore` through one shared `Reader`, and no
rule recomputes a fact another rule already produced. The review's larger claim — that a
generalized planner should deduplicate fact requirements across a growing, diverging rule
population and resolve providers generically ahead of a hand-written call list — rests on those
three still-unfired triggers, not on this one, and this record does not reopen them.

**Every existing caller of `Run` is unaffected by an empty selection.** `nomos check`
(`crates/host/nomos-cli/src/check.rs`) has no rule-selection concept of its own and passes an
empty selection, preserving today's unconditional behavior exactly — the identical "empty
`include` is select-everything" default `RuleSelector` already establishes for `Run_Gate`.
`nomos gate explain` (`Explain_Gate`) deliberately ignores `command.rules` today, by its own
documented design: "independent of `command.scope` and `command.rules`: those narrow a real
run's disposition over many findings, and this answers a question about one named finding as
check would produce it right now." `Explain_Gate` must keep passing an empty selection through
`Judged`, not `command.rules` — threading the command's own selection through here would
silently narrow what `explain` can ever answer a query about, which is not this record's
question to reopen.

## Decision

**Build it now.** `nomos_check_orchestration::Run` gains a parameter naming which rules a
caller wants computed, typed as `&[RuleId]` with the same "empty is every rule" semantics
`RuleSelector::include` already has, so no existing construction site changes meaning by
default:

- `Run`'s internal `Materialize_Capabilities` step calls `Materialize_Dependencies` only when
  `DEPENDENCY_DIRECTION` is selected, and `Materialize_Reachability` only when
  `UNREAD_REACHES_FINDING` is selected — the two materializations with a real cost
  (`cargo metadata`'s process launch; a per-file heuristic pass) that no rule but their own
  consumer reads.
- `Run`'s internal `Judged` step calls each of the four `Check_*` functions only when its own
  `RuleId` is selected.
- `nomos-cli`'s `check::Run` passes an empty selection (`nomos check` keeps judging everything,
  unconditionally, exactly as `OD-HOST-004` already requires for a rule with no per-request
  variance).
- `nomos-gate-orchestration::run_gate::Judged` passes `command.rules.include` through to `Run`,
  so `Run_Gate` stops computing what its own caller's `--rule` already said it does not want.
- `nomos-gate-orchestration::explain::Explain_Gate` passes an empty selection through the same
  `Judged`, unchanged from today's independence from `command.rules`.

This is a fixed, hand-written mapping from each of today's four `RuleId`s to the fact(s) it
needs — the same "composition, not choice" shape `OD-HOST-004` and `OD-RULES-009` already
approved for *whether* a rule participates, extended to a second axis (whether its
materialization runs at all) rather than replaced by a different mechanism. It is not a
declared, generic requirement-to-provider resolution a planner reads; a fifth rule still needs
its own hand-written entry in both places, exactly as a fourth rule already did.

## What This Record Does Not Do

It does not build `ARC-ROADMAP-001`'s shared demand planner. No rule declares a
capability requirement a planner reads generically, no fact DAG is deduplicated across an
arbitrary population, and no provider is resolved ahead of a hardcoded call list. `OD-RULES-
009`'s decline of that larger artifact stands on its own remaining, still-unfired triggers; this
record closes only the one trigger that fired, and the amendment naming that is `OD-RULES-009`'s
own, not rewritten here.

It does not change `RuleSelector`'s own shape, `ScopeSelector`, or any command-line surface —
`--rule` already exists and is unchanged; only what happens to a source once `Run` is called
with it changes.

It does not touch `Explain_Gate`'s documented independence from `command.rules`, and does not
give `nomos check` a rule-selection concept it has never had.

It does not decide anything about a sixth or later rule's own required-fact mapping in advance
of that rule existing — each new rule still earns its own line in `Run`'s hand-written mapping
when it ships, the same as today.

## Built: Verified Directly Against The Real Code

`P14-GATE-017-RUN-RULE-SUBSET-FIRST-INCREMENT-2` built exactly the shape this record decided,
checked directly at merge rather than assumed from the plan above. `nomos_check_orchestration::
Run` (`crates/orchestration/nomos-check-orchestration/src/run.rs`) takes a fifth parameter,
`selected: &[RuleId]`, read through a private `Wants` helper with the same "empty is every
rule" semantics `RuleSelector::include` already has. `Materialize_Capabilities` calls
`Materialize_Dependencies` only when `DEPENDENCY_DIRECTION` is wanted and `Materialize_
Reachability` only when `UNREAD_REACHES_FINDING` is wanted; `Judged` calls each of the four
`Check_*` functions only when its own `RuleId` is wanted. `nomos-cli::check::Run` passes `&[]`.
`nomos-gate-orchestration::run_gate::Judged` gained the same parameter and both its callers
were updated: `Run_Gate` passes `command.rules.include`, and `Explain_Gate` passes `&[]`,
preserving its documented independence from `command.rules` now that a non-empty selection
narrows computation and not only disposition.

A counting `ProcessLauncher` test (`Test_A_Deselected_Dependency_Rule_Should_Not_Launch_Cargo_
Metadata`) proves the skip is structural: zero launches when `DEPENDENCY_DIRECTION` is not
selected, exactly one when it is. This is the evidence a findings-only test could not give,
since a healthy repository's `cargo metadata` call raises no finding on success — "deselected"
and "selected but clean" would otherwise render identically.

One existing test pinned the old, now-superseded behavior. `nomos-gate-orchestration`'s
`Test_A_Deselected_Rules_Finding_Should_Not_Block` asserted that a deselected rule's finding
"must still be judged and carried, just not blocking" — exactly the shape this record replaces.
It is now `Test_A_Deselected_Rules_Finding_Should_Not_Exist`, asserting the finding is absent
from `check_outcome` entirely, because `Run` was never asked to compute it.

`rule_selector.rs`'s own module doc, which stated in its own words that this exact change would
be "a separate, larger item, not this one," was corrected by `P14-RULE-SELECTOR-DOC-STALE-2` to
describe `RuleSelector`'s real, current role: naming the same selection `Run` itself now reads,
plus the narrower disposition filter it still applies on top for whichever rules did run.

## Status

Accepted, and built. `P14-GATE-017-RUN-RULE-SUBSET-FIRST-INCREMENT-2` (not `-FIRST-INCREMENT`,
which was declined and re-added once its territory was found to omit a file the change could
not land without) built the signature change and both callers this record names, verified
directly against the real code once merged.
