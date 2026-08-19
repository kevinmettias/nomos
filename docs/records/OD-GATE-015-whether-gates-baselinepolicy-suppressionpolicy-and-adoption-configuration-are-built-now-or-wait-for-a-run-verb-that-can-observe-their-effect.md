---
id: OD-GATE-015
type: decision
title: Whether Gate's BaselinePolicy, SuppressionPolicy and adoption configuration are built now, or wait for a run verb that can observe their effect
status: open
version: 2
authority: canonical-normative-record
tags:
  - gate
  - orchestration
  - baseline
  - suppression
  - adoption
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-GATE-014
    type: relates-to
  - target: D-135
    type: relates-to
---

# Whether Gate's BaselinePolicy, SuppressionPolicy and adoption configuration are built now, or wait for a run verb that can observe their effect

## Question

`ARC-ROADMAP-001` names "baselines / suppressions / adoption" as part of Nomos Core's
near-term boundary, one line among many, without ordering or scoping it. `nomos-gate-
orchestration/src/lib.rs` already lists `BaselinePolicy` and `SuppressionPolicy` among the
`WF-001` clauses its first increment deliberately does not touch — the same "what this
increment is not" list `ScopeSelector`/`RuleSelector` appear in, which `OD-GATE-014` already
turned into an open decision with a stated trigger. Nothing has done the same for baseline,
suppression and adoption. This record does.

## What The Corpus Specifies

`ARC-ROADMAP-001`'s one-line bullet understates what the v14 corpus actually asks for.
Reading `01_authoring/artifacts/requirements` directly (not accepted from the bullet):
fourteen requirements, `authority: canonical-normative-record`, across four named sections,
none of which overlap `OD-TRACE-002`'s six formally assessed entries (checked directly
against both):

- **`SUP-*`** (4 requirements, `05.7-2-2 suppression and waiver governance`): six distinct
  disposition types — inline suppression, repository policy exception, temporary waiver,
  accepted baseline debt, false-positive disposition, formal risk acceptance — not one
  generic suppression. Required fields include rule/version range, scope, rationale, owner,
  approver, creation/expiry/review dates, and linked evidence. Human authorization is
  required for an agent-created suppression. Revalidation triggers include a rule upgrade,
  an identity transition, moved code, an expired date, changed evidence, or a removed
  finding.
- **`SUP-EVID-*`** (2 requirements, `05.7-14-3`): the same six-way distinction as an
  evidence/reporting concern, and a requirement that revalidation detect stale, unused,
  moved, expired, evidence-changed and rule-version-changed suppressions rather than
  treating them as clean findings.
- **`BASELINE-*`** (3 requirements, `05.7-14-2 baseline and new-code gating`): baseline mode
  may tolerate existing debt only within an explicit scope, and must still block new,
  reintroduced, safety-critical or out-of-scope findings. New-code scope is based on
  declared source geometry, identity and diff evidence rather than an opaque baseline
  snapshot file — a direct rejection of the common industry shape. Scoped gating must
  preserve suppressed/waived/unsupported/unavailable/unparseable/excluded/not-applicable/
  evaluated states distinctly, rather than collapsing into `Applicability`'s existing
  honesty states.
- **`ADOPT-CONFIG-*`** (4 requirements, `05.7-14-1 repository adoption and calibration
  contract`): adoption is expressed through declared gates, phases and calibration policy,
  explicitly not by forking rule implementations. Effective policy separates consumer-owned
  files (workflow, calibration, suppressions, run overrides) from Nomos-owned rule logic.
  Consumer disagreement with a rule is represented as typed configuration — suppression,
  waiver, baseline debt, false-positive, risk acceptance — never silence.

`WF-001` (`05.7-4 gates, phases and workflows`) is the binding clause tying all four
together: "a gate shall define policy: required phases, thresholds, coverage,
unsupported-analysis policy, waivers, approvals, and blocking behavior" — the exact sentence
`nomos-gate-orchestration/src/lib.rs` already quotes as the source of `BaselinePolicy` and
`SuppressionPolicy`.

## Current Position

Verified directly against the real code, not assumed: `GatePlan`
(`crates/orchestration/nomos-gate-orchestration/src/outcome.rs`) is "what a gate would
evaluate, without evaluating it" — report-only, the one real variant `nomos-gate-
orchestration` has. `Finding::Can_Fail_A_Build`
(`crates/contracts/nomos-contracts/src/finding.rs`) is `self.gate.Can_Fail_A_Build() &&
self.applicability.Was_Evaluated()` — two conditions, with no suppression hook between them
and no third condition a baseline or waiver could occupy. `nomos-corrections` previews,
stages, validates, commits and rolls back a `Workspace` edit; it has no concept of tracking
or exempting a finding over time, so there is nothing there for a baseline to hang off
either.

There is, in short, no caller anywhere in this workspace that either policy type could have
an observable effect on. A `BaselinePolicy` or `SuppressionPolicy` field added to `Gate`
today would sit beside `GatePlan.rules`, read by nothing, changing no outcome — the same
shape `ScopeSelector`/`RuleSelector` would have had, which is exactly why `OD-GATE-014`
declined to build those on `ARC-ROADMAP-001`'s naming alone. `ARC-ROADMAP-001` itself
already disclaims settling this: it "does not order the near-term tier internally," and
naming baselines/suppressions/adoption as near-term is not the same as naming them the next
increment to build.

Unlike `OD-GATE-014`'s selectors, this question was never a "wait for a second instance to
check the first one's field boundaries against" — nothing baseline- or suppression-shaped
has been built even once, so there is no first instance to generalize from. The closer
analogy is `OD-GATE-014`'s own second, independent trigger.

## One Question, Three Concerns

The corpus's own structure — four separately-sectioned families, not one — argues against
inventing one unified shape. Suppression (`SUP-*`, `SUP-EVID-*`) is per-finding, needs the
richest data model of the three (six disposition types, owner/approver/dates, revalidation
triggers), and is independently useful on its own. Baseline (`BASELINE-*`) is about
existing-debt scope and new-code detection — diff- and identity-based, genuinely different
mechanics from a per-finding record, with no owner/approver/date fields of its own.
Adoption (`ADOPT-CONFIG-*`) sits a layer above both: `ADOPT-CONFIG-003` explicitly lists
suppression, waiver and baseline debt as the vocabulary a consumer's disagreement is
expressed *through*, making it a consumer of the other two's types rather than a peer
concern.

So this record poses one question — because all three share the identical trigger and the
identical current state of zero built instances — but its answer, when the trigger fires,
should not be a single build item. `SUP-002`'s field-rich, per-finding data model and
`BASELINE-002`'s diff-based, scope-oriented approach have no shared shape worth unifying
prematurely; forcing one now would repeat the exact mistake `D-135` names, inferring
genericity from a wish rather than a demonstrated need — except here the "wish" would be
economy of one record rather than economy of one type.

## `run` Arrived: What Changed And What Did Not

`P13-GATE-RUN-FIRST-INCREMENT-3` gave `Gate` a real `run` verb and a real `GateRunOutcome
{ Passed, Failed, Indeterminate }`, so `Finding::Can_Fail_A_Build`'s two conditions are no
longer the only thing standing between a finding and a build result. Verified directly
against the real code, not assumed: `nomos_gate_orchestration::Disposition` now reduces a
run's findings into that verdict, and there is a real place — between `Disposition`'s
reduction and the findings it reduces — a suppression hook or a baseline's tolerated-debt
scope could sit, where before there was only `Plan`'s unevaluated report.

Nothing calls `Disposition` except `run`, and nothing calls `run` except this record's own
tests. Verified directly against `.github/workflows/gate.yml`, not assumed: the `Rules`
step — `OD-GATE-004`'s own CI step — still invokes `nomos check` directly, not `nomos gate
run`. `run`'s own disposition, whichever of the three it comes to, has no consumer whose
build it actually gates. A `BaselinePolicy` or `SuppressionPolicy` added today would still
sit beside a disposition nothing outside this record's own tests observes.

## What Would Decide It

The first named trigger — `Gate` gaining a `run` verb — has fired: `run` is real, and
`Disposition`'s reduction is exactly the seam a suppression hook or a baseline's
tolerated-debt scope would sit inside, closing the risk of shaping either policy type
around a `Plan`-only `Gate` that never judged anything. What has not fired is the deeper
condition that trigger was standing in for: a real caller whose build `run`'s disposition
actually gates. `run` exists, but nothing depends on what it says — `OD-GATE-004`'s own CI
step still enforces this repository through `nomos check`, not through `Gate`. Baseline and
suppression policy shaped against a `run` nobody's build depends on would repeat the exact
mistake this record already declined once, one layer further in.

A second, independent trigger particular to this question, unchanged by `run`'s arrival: a
real caller — this repository's own CI, or a consuming repository — accumulating enough
existing debt, or enough false-positive/rationale friction, that an unconditional
`Blocking` gate becomes impractical to adopt. That caller's own shape of debt would still
argue for which of the three concerns to build first, rather than building all three
speculatively from the corpus's naming alone.

## Status

Open. `Gate` has a real `run` verb and a real disposition now
(`P13-GATE-RUN-FIRST-INCREMENT-3`, `P13-GATE-014-015-RUN-TRIGGER-FIRED`), which closes the
risk of shaping baseline or suppression policy around a `Plan`-only `Gate`. No caller in
this workspace observes that disposition yet: `OD-GATE-004`'s CI step enforces through
`nomos check` directly, not through `Gate`. Revisit when `gate run` gains a real caller
whose build its disposition gates, or when a real caller's accumulated debt or exemption
need names a concrete shape for one of the three concerns above — and treat that as a
trigger for the concern it names, not for all three at once.
