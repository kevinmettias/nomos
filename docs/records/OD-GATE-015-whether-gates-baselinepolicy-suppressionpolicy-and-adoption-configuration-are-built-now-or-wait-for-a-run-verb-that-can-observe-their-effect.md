---
id: OD-GATE-015
type: decision
title: Whether Gate's BaselinePolicy, SuppressionPolicy and adoption configuration are built now, or wait for a run verb that can observe their effect
status: accepted
version: 6
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
self.applicability.Is_Evaluated()` — two conditions, with no suppression hook between them
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

## `run` Gets A Real Caller: What Changed And What Did Not

`P13-GATE-RUN-CI-CALLER` gave this record's own named trigger exactly what it asked for: a
real caller whose build `run`'s disposition gates. Verified directly against the real
workflow file, not assumed: `.github/workflows/gate.yml`'s `Rules` step now invokes `cargo
run --quiet -p nomos-cli --bin nomos -- gate run --root .`, and Actions fails that step —
and so the whole gate job — on any non-zero exit, the identical zero-is-the-only-success
policy `OD-GATE-004` already established for `check`. This repository's own pull requests
now succeed or fail on `Disposition`'s reduction rather than on `check`'s exit code taken
directly. The first named trigger has fired, in full — not the narrower "`run` exists" sense
`P13-GATE-014-015-RUN-TRIGGER-FIRED` recorded, which left the caller itself still absent.

The second, independent trigger has not. The switch landed the same day it was proposed,
against a `Rules` step that had never yet failed on a real finding through this path — no
debt has had the chance to accumulate, and no false-positive or rationale friction has been
observed, because the step is new. Nothing yet argues for which of `SUP-*`, `BASELINE-*` or
`ADOPT-CONFIG-*` to build first, or that any should be built now rather than waited on
further.

## What Would Decide It

The first named trigger — `Gate` gaining a `run` verb — has fired: `run` is real, and
`Disposition`'s reduction is exactly the seam a suppression hook or a baseline's
tolerated-debt scope would sit inside, closing the risk of shaping either policy type
around a `Plan`-only `Gate` that never judged anything. What has now also fired is the
deeper condition that trigger was standing in for: a real caller whose build `run`'s
disposition actually gates — this repository's own CI, since `P13-GATE-RUN-CI-CALLER`.
Baseline and suppression policy would no longer be shaped against a `run` nobody's build
depends on.

A second, independent trigger particular to this question, unchanged by either landing: a
real caller — this repository's own CI, or a consuming repository — accumulating enough
existing debt, or enough false-positive/rationale friction, that an unconditional
`Blocking` gate becomes impractical to adopt. That caller's own shape of debt would still
argue for which of the three concerns to build first, rather than building all three
speculatively from the corpus's naming alone. Nothing has accumulated yet.

## One Of Three Is Built, Under Override — The Other Two Are Not

`P13-GATE-015-SUPPRESSION-FIRST-INCREMENT-2` built `SuppressionPolicy`, the first of this
record's three named concerns, under the same standing user override
`OD-GATE-014`'s own amendment already records — not by the shared trigger firing, which
still has not. This record's own "One Question, Three Concerns" section already argued
against resolving `SUP-*`, `BASELINE-*` and `ADOPT-CONFIG-*` together, since they share no
real field shape; this amendment is that argument bearing out. Only the suppression concern
is built. `BaselinePolicy` and adoption configuration remain exactly as unbuilt as before,
and now wait on their own second, independent trigger rather than a trigger a built concern
no longer needs.

Verified directly against the real code, not assumed: `Suppression`
(`crates/orchestration/nomos-gate-orchestration/src/suppression.rs`) carries the six `SUP-*`
dispositions by name (`InlineSuppression`, `RepositoryPolicyException`, `TemporaryWaiver`,
`AcceptedBaselineDebt`, `FalsePositiveDisposition`, `FormalRiskAcceptance`) and matches a
finding by `rule`/`subject` — the identity `Finding` already carries, no new addressing
scheme invented. `Run_Gate` partitions a matched, otherwise-blocking finding into
`GateRunResult::suppressed_findings` rather than `blocking_findings`; it stays visible in
both that field and `check_outcome`, never silently dropped. No CLI flag or configuration
file constructs a `Suppression` yet — nothing in this workspace has any config-file
authoring convention at all, so inventing one now, before a real caller needs it, would
repeat the exact mistake this record's own reasoning already declined. `owner`/`approver`/
dates/revalidation triggers, `SUP-*`'s other required fields, are not enforced: nothing
constructs a `Suppression` today, so validating fields nothing populates would validate
against nothing.

## Two Of Three Are Built, Under Override — Adoption Is Not

The user's broader 2026-08-19/20 "proceed with all remaining work" directive named Gate
policy's `BaselinePolicy` by name among the unblocked near-term tier — the same kind of
standing override this record's own "Status" section already said would resolve it, given
by name rather than inferred from `ARC-ROADMAP-001`'s naming alone.
`P13-GATE-015-BASELINE-FIRST-INCREMENT` built it, deliberately narrower than `BASELINE-*`'s
full corpus shape, for the same reason `SuppressionPolicy`'s own first increment was narrow:
no new-code, diff- or identity-based detection distinguishes tolerated debt from a
reintroduced or genuinely new finding, and no scope beyond the named `rule`/`subject` pairs
an entry lists. Adoption configuration remains exactly as unbuilt as before.

Verified directly against the real code, not assumed: `BaselineDebt`
(`crates/orchestration/nomos-gate-orchestration/src/baseline.rs`) matches a finding by
`rule`/`subject`, the identical identity `Suppression` already matches by. `Run_Gate` checks
`suppressions` first and `baseline` second, so a finding matched by both reports as
suppressed and the two result lists never double-count it; a matched finding moves from
`blocking_findings` to `GateRunResult::baselined_findings` rather than disappearing, and
`Explain_Gate` reports the same disposition through `Explanation::Found::baselined_by`. No
CLI flag or configuration file constructs a `BaselineDebt` yet, the same absence
`SuppressionPolicy`'s fifth increment already declined to fill for the same reason.

## All Three Are Built, Under Override

The user's 2026-08-19/20 "proceed with all remaining work" directive named adoption
configuration by name, the last of this record's three concerns still open, now that
suppression and baseline are both built. `P13-GATE-015-ADOPTION-FIRST-INCREMENT-2` built it
(the original `P13-GATE-015-ADOPTION-FIRST-INCREMENT` was declined for an authoring mistake
in its own predicate, unrelated to what it built), taking the narrowest real instance of
`ADOPT-CONFIG-003`'s "declared gates, phases and calibration policy": a rule-level
calibration, not a third per-finding matcher of `Suppression`'s or `BaselineDebt`'s shape.

Verified directly against the real code, not assumed: `RuleCalibration`
(`crates/orchestration/nomos-gate-orchestration/src/adoption.rs`) matches a `Finding` by
`rule` alone, deliberately not `rule`/`subject` -- a
different addressing scheme from `Suppression` and `BaselineDebt`, since adoption is framed
as a layer above both rather than a third way to name one finding. `Run_Gate` checks
`AdoptionPolicy` before `suppressions` and `baseline`, so a finding matched by more than one
reports as calibrated; a matched finding moves from `blocking_findings` to
`GateRunResult::calibrated_findings` rather than disappearing, and `Explain_Gate` reports
the same disposition through `Explanation::Found::calibrated_by`. No CLI flag or
configuration file constructs an `AdoptionPolicy` yet, the same absence
`SuppressionPolicy`'s and `BaselinePolicy`'s own first increments already declined to fill.
No declared phases, thresholds or approvals either -- `ADOPT-CONFIG-*`'s full corpus shape
-- and no separate consumer-owned configuration file: `RuleCalibration`'s own doc says why.

## Status

Accepted. `BaselinePolicy`, `SuppressionPolicy` and now `AdoptionPolicy` all exist and are
consulted by a real `run` and a real `explain`, closing what this record asked -- not by the
second, independent trigger this record names (no real caller has yet accumulated debt or
exemption friction of its own), but by the user's own standing override, given by name for
each of the three concerns in turn and recorded above rather than left to be inferred.
Nothing further to revisit under this record: any future widening of one policy's own shape
(`SUP-*`'s owner/approver/date fields and revalidation triggers, `BASELINE-*`'s diff- and
identity-based new-code detection, `ADOPT-CONFIG-*`'s declared phases and a real
authoring/config surface for any of the three) is a new question for a new record, the same
way `OD-GATE-014`'s own acceptance already treats a future narrowing of `ScopeSelector`/
`RuleSelector`.
