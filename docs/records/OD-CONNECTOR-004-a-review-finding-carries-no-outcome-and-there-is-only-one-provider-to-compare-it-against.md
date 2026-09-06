---
id: OD-CONNECTOR-004
type: decision
title: A review finding carries no outcome and there is only one provider to compare it against
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - connector
  - review
  - contracts
  - corrections
relations:
  - target: ARC-CONNECTOR-001
    type: relates-to
  - target: OD-CONNECTOR-002
    type: relates-to
  - target: OD-EXECUTOR-006
    type: relates-to
---

# A review finding carries no outcome and there is only one provider to compare it against

## Question

An external architectural review of this workspace, checked directly against the real tree
rather than taken on faith, named a stage past ingestion that nothing here builds: once a
review finding enters nomos, what happens to it — accepted, rejected, judged a duplicate,
superseded by a later finding, or the correction built from it succeeded or failed — and
does anything carry that outcome forward as evidence for comparing one review provider
against another. This record measures the gap directly rather than assuming the review's
framing is already correct, names why a disposition cannot be bolted onto what exists today,
and states what would actually need to be true before the comparison the review has in mind
is buildable.

## What carries a review finding today, checked directly

`nomos-connector-coderabbit`'s `FindingPayload`
(`crates/connectors/nomos-connector-coderabbit/src/payload/finding_payload.rs`) is eight
fields: `external_system`, `external_id: ReviewFindingId`, `locator`, `category`,
`severity`, `path`, `line`, `message`. Its own module doc states the boundary this record
confirms rather than restates: the payload is vendor shape carried forward, kept apart from
any judgment about validity or bearing. Nothing downstream of `translation.rs` adds a field;
`Check_Review_Findings` (`crates/rules/nomos-rules/src/checks/review.rs`) relays the payload
straight into `nomos_contracts::Finding`.

`Finding` (`crates/contracts/nomos-contracts/src/reporting/finding.rs`) is `rule`, `subject`,
`subject_name`, `applicability`, `evidence`, `gate`, `summary`, `locations` — eight fields,
every one documented as load-bearing, and the doc comment for the type states the design
principle directly: "every field is load-bearing, and the ones that look redundant are the
ones that are not." None of the eight opens a place for what happened to the finding after
it arrived. Adding a ninth is not this record's business to decide by omission — see
"Where it would live," below — but the confirmation itself is real: **today, nothing.**

`EvidenceClass::AgentJudged` (`crates/contracts/nomos-contracts/src/reporting/finding/evidence_class.rs`)
already forbids the nearby shortcut a disposition could be mistaken for. Its own doc:
"it cannot be promoted afterwards — promotion would have to happen somewhere that no longer
holds the evidence, which is how 'the agent said the tests pass' becomes 'the tests pass'."
A disposition is not a promotion of evidence class and must not be built as one: `evidence`
answers how a claim was *come by*, at the moment the rule produced it, and a later
acceptance or rejection does not change how it was come by. Conflating the two would let a
finding's provenance drift after the fact, exactly what `AgentJudged`'s floor exists to
prevent.

`nomos-corrections`' nearest existing concept, `CorrectionDecision`
(`crates/corrections/nomos-corrections/src/correction_decision.rs`), answers a different
question at a different time: whether a *candidate correction* was selected automatically
or left for review, decided at selection time, before anything is committed or rolled back.
It says nothing about a finding's own outcome, and nothing in `nomos-corrections` records
whether a correction that *was* selected and applied later succeeded or was rolled back.

## Why a disposition is a new fact, not a rewrite of an existing one

Two identities already exist that a disposition could key on, and neither is the file
location the review's own framing might suggest. `Finding::subject: SubjectId` is derived
from `subject_name`, documented explicitly as stable across a file moving — "a universe
that moves to a different file is the same universe, and a finding keyed on its path would
read as a finding closed and a new finding opened." Independently,
`nomos-connector-coderabbit::ReviewFindingId` (`crates/connectors/nomos-connector-coderabbit/src/review_finding_id.rs`)
already identifies one external review comment durably, at the connector's own layer,
before translation. A disposition fact keyed on `subject` would compare across providers at
the level `Finding` already unifies; one keyed on `external_id` would stay provider-scoped.
Either is a coherent choice, and choosing between them is design work this record does not
do — the point here is narrower: an identity to key on already exists in both places a
disposition could live, so "nothing to key on" is not the open question.

## Where it would live

Not `Finding` itself, by that type's own stated discipline: every field is load-bearing and
declared once, and a disposition computed *after* a finding is produced is not something the
rule that produced the finding can populate — the same reason `evidence` cannot be
retroactively strengthened applies to adding a ninth field nobody upstream can fill in. The
natural home is a new fact, addressed by whichever identity is chosen above, written by
whatever later stage of nomos observes a person's or a correction's action on the finding —
most plausibly `nomos-corrections`, since it is the crate that already models a correction's
own staged lifecycle (`CorrectionCandidate`, `CorrectionPlan`, `CorrectionDecision`) and
would be the natural writer of "the correction built from this finding succeeded" or
"failed." No such fact, type, or writer exists anywhere in `nomos-contracts`,
`nomos-connector-coderabbit`, or `nomos-corrections` today; this record does not create one.

## Why comparing providers is not buildable yet regardless

There is exactly one review connector in this workspace — `nomos-connector-coderabbit` is
the only implementor `Check_Review_Findings` composes over. A disposition fact would let one
provider's findings be scored against outcomes, but "comparing providers" needs a second
provider's findings scored the same way before there is anything to compare against. Building
a disposition mechanism now would be building a selection mechanism with one candidate to
select from — real, buildable work stays possible (a disposition fact keyed on `subject`,
written by a new `nomos-corrections` type), but a *comparison* mechanism is not, and this
record declines to invent the second provider in order to make one buildable today.

## Decision

No code changes accompany this record. It states, for whoever next reaches for a review
finding's outcome: the gap is real and exactly where the review named it; a disposition
belongs beside `nomos-corrections`' existing staged-lifecycle types, keyed on an identity
that already exists (`Finding::subject` or `ReviewFindingId`, a later choice); it must not be
folded into `EvidenceClass` or `Finding` itself; and a provider-comparison mechanism built on
top of it is blocked on a second real review provider existing, not on this decision.
