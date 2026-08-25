---
id: OD-GATE-016
type: decision
title: Whether Gate needs a CoveragePolicy that lets unsupported or unanalyzed scope affect disposition
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - coverage
  - applicability
  - architecture
relations:
  - target: OD-GATE-014
    type: relates-to
  - target: OD-GATE-015
    type: relates-to
  - target: OD-COMPLETENESS-004
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
---

# Whether Gate needs a CoveragePolicy that lets unsupported or unanalyzed scope affect disposition

## Question

An external architecture review (read against this workspace rather than accepted on read)
names Gate's missing `CoveragePolicy` — the inability to distinguish a clean, fully-analyzed
run from one where scope went unanalyzed or unsupported — as a real, currently-undecided gap.
`WF-001`'s binding clause names "coverage" and "unsupported-analysis policy" among what "a
gate shall define," alongside phases, thresholds, waivers, approvals and blocking behavior;
`ARC-ROADMAP-001` constraint 5 separately lists `CoveragePolicy` by name among the properties
of the product-level `Gate` object, beside `ScopeSelector`, `RuleSelector`,
`ApplicabilityPolicy`, `BaselinePolicy` and `SuppressionPolicy`. `OD-GATE-015` measured the
same `WF-001` clause and its own corpus section structure, and deliberately scoped itself to
only three of its named concerns — suppression, baseline, adoption — leaving coverage, phases,
thresholds and approvals untouched; its own "Current Position" section never mentions
`CoveragePolicy`. Whether coverage should now be decided the way suppression, baseline and
adoption already were is this record's question.

## What Was Measured

**The machinery `CoveragePolicy` would consult already exists and is real.** Verified
directly: `Applicability::Is_Coverage_Debt()`
(`crates/contracts/nomos-contracts/src/finding/applicability.rs`) and `Claim`
(`Claim::Complete`/`Claim::Incomplete`, computed by `Claim_Of` in
`crates/orchestration/nomos-check-orchestration/src/outcome/claim.rs`) are not missing types —
they are unconsulted ones, tested and shipped since `OD-COMPLETENESS-004`.

**Gate's own `Disposition` already names why it does not read `Claim`, and names the decision
that settled it.** Verified directly:
`crates/orchestration/nomos-gate-orchestration/src/outcome/gate_run_outcome.rs`'s `Disposition`
carries a doc comment stating `Claim` "is deliberately not consulted here, the same choice
`OD-COMPLETENESS-004` already made for `nomos check`'s own exit code: reported, not gated on."

**`OD-COMPLETENESS-004` measured this exact question once already, for a different surface,
and explicitly left the Gate-layer question open.** Read directly: it settled that `nomos
check`'s own fixed exit code should report coverage debt (`claim: incomplete` and a per-variant
breakdown) without failing the build over it, on measured grounds specific to that surface —
this repository's own `tests/corpus/analysis/gamma/broken.rs` fixture makes every real run
`Claim::Incomplete` today, so a stricter exit code there would turn this repository's own gate
permanently red over an admitted, deliberate gap. Its own "What This Does Not Do" section
states: "It does not give the gate a stricter policy... That is a live, separate decision this
record declines to make by accident." This record is that decision.

**Gate has no state for it today.** Verified directly: `GateRunOutcome`
(`outcome/gate_run_outcome.rs`) has exactly three variants — `Passed`, `Failed`,
`Indeterminate` — and `Indeterminate` is assigned only when the underlying `CheckOutcome` never
reached `Judged` at all (an unreadable root, no source found, a self-contradictory registry).
`Run_Gate`'s own `Reduced` function (`run_gate.rs`) never inspects `outcome`'s `claim` field. A
gate run today reports `Passed` whenever no finding blocks, even when a subject in scope
carried `Applicability::MissingCapability`, `Unparseable` or another coverage-debt state — the
same "unknown read as pass" defect `Applicability`'s own module doc names as the product's
first principle, now open one layer up, at the layer whose disposition a real CI step actually
gates (`P13-GATE-RUN-CI-CALLER`).

**`Claim` as currently computed does not match what a Gate-layer policy would need to consult.**
Verified directly: `CheckOutcome::Judged`'s `claim` is computed once, project-wide, over every
finding the underlying `nomos check` produced — before `Run_Gate`'s own `command.rules` and
`command.scope` narrow what counts toward disposition. A `CoveragePolicy` consulted correctly
would need `Claim` recomputed over the rule-and-scope-selected findings, the same "selection
narrows what counts" shape `Reduced` already applies before computing `blocking_findings`,
`calibrated_findings`, `suppressed_findings` and `baselined_findings` — reusing the whole-run
`claim` `check_outcome` already carries would silently answer a different, broader question
than the one Gate's own `RuleSelector`/`ScopeSelector` say it is judging.

**The standing override that built `OD-GATE-014` and `OD-GATE-015`'s three concerns already
names `CoveragePolicy` by name.** The user's 2026-08-19/20 "proceed with all remaining work"
directive lists Gate policy as `ScopeSelector`/`RuleSelector`/`CoveragePolicy`/`BaselinePolicy`/
`SuppressionPolicy` together, the same list `OD-GATE-014` and `OD-GATE-015` were each built
under, with the explicit instruction to treat every item named as unblocked and not re-raise
the deferral question for any of them.

## Decision

**Build a first real increment now, under the override — narrower than `WF-001`'s full shape,
the same discipline each of `OD-GATE-015`'s three concerns followed in turn.** A
`CoveragePolicy` consulted by `Run_Gate` alongside `command.rules`, `command.adoption`,
`command.suppressions` and `command.baseline`: when a repository sets it to require
completeness, `Run_Gate` recomputes `Claim` over the rule-and-scope-selected findings — not the
whole-run one `check_outcome` already carries — and reports `GateRunOutcome::Indeterminate`,
never silently `Passed`, whenever that recomputed claim is `Incomplete`. Unset, behavior is
unchanged: `OD-COMPLETENESS-004`'s "reported, not gated on" default stays the default for a
repository that never opts in, the same way an empty `SuppressionPolicy`, `BaselineDebt` or
`AdoptionPolicy` changes nothing today.

This first increment does not attempt `WF-001`'s full shape. Declared phases, thresholds and
approvals stay exactly as unbuilt as `OD-GATE-015` already found them — this record is about
coverage alone, the one `WF-001` concern `OD-GATE-015` left unmeasured, not the rest of what
that clause names. No CLI flag or configuration file constructs a `CoveragePolicy` yet — the
same absence `SuppressionPolicy`'s, `BaselineDebt`'s and `AdoptionPolicy`'s own first increments
each declined to fill, for the reason each of them gave: nothing in this workspace has any
config-file authoring convention at all, so inventing one now, before a real caller needs it,
would repeat the mistake `OD-GATE-015` already declined once. `Explain_Gate` reporting a
coverage-caused `Indeterminate` by name, the same way it already reports `baselined_by` and
`calibrated_by`, is left to the item that builds this increment, not decided here.

## What This Does Not Do

- It does not change `nomos check`'s own exit code or `report.rs`. `OD-COMPLETENESS-004`
  settled that question for that surface; this record settles the separate, Gate-layer question
  its own text named as live and left open.
- It does not decide what `CoveragePolicy` looks like beyond the binary require-completeness
  case measured here. A minimum-`Applicability` threshold, a per-rule or per-scope coverage
  requirement, or anything `WF-001`'s richer text might eventually support is left to real
  evidence once a repository has this first shape to react to.
- It does not touch `Finding::Can_Fail_A_Build`, `Applicability`, `Claim`, or any rule. Every
  type this record's increment reads already exists and is unchanged by it.

## Status

Accepted. Names the first increment to build; a follow-on capability item builds it and amends
this record the way `P13-GATE-015-SUPPRESSION-RECORD`, `-BASELINE-RECORD` and `-ADOPTION-RECORD`
each amended `OD-GATE-015` once their own concern landed.
