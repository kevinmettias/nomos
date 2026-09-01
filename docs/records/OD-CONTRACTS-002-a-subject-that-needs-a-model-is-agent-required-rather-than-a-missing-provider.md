---
id: OD-CONTRACTS-002
type: decision
title: A subject that needs a model is agent-required, rather than a missing provider or nothing at all
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - contracts
  - coverage
  - completeness
  - requirements
  - agents
relations:
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-CAPABILITY-001
    type: relates-to
---

# A subject that needs a model is agent-required, rather than a missing provider or nothing at all

## Question

`CHK-003` requires that every run report evaluated, excluded, unsupported, unavailable,
failed, not-applicable, **and agent-required** combinations. `Applicability` in
`nomos-contracts` answered six of those seven with ten variants and had no state for the
seventh, so a subject that needs a model to be judged had nowhere honest to go.

## Whether CHK-003 Binds This Build

It binds. Four grounds, and the third is the one that settles it.

`CHK-003` is `status: normative`, `maturity: accepted`, `authority:
canonical-normative-record` in volume 05, and `D-132` puts the corpus above the plan. That
makes it binding by default; the question is only whether something about this build takes
it out of scope.

Nothing does, because the concept is already here one layer away. `EvidenceClass` carries
`AgentJudged`, and `EvidenceClass::Is_Mechanical` exists precisely so a model's judgment
cannot be reported as a machine having checked something. The protocol could already tell
mechanical from agent-produced evidence when describing *a claim*, and could not say it at
all when describing *coverage*.

The corpus asks for the category to be reported, not merely named. `US-CHK-001`'s acceptance
in volume 04 reads: applicable, not-applicable and unsupported counts are shown; **agent-required
and correctable combinations are identified**. Identified separately from the counts, in the
applicability preview, before anything runs.

And agent-required is not a provider question. `WF-006` makes executors — API-hosted,
subscription-agent, human, recorded-replay — a kind distinct from providers, sharing one
task/result protocol. The capability registry answers about providers and only about
providers, so no answer it can give is the right one here.

This assessment was authored against `NOMOS_V14_CORPUS` and is committed here rather than
recomputed at check time, which is the shape `OD-TRACE-001` decided and `OD-GATE-001` forces:
CI has no corpus, and a guard that reads the corpus at check time would be green by being
unable to look.

## The Decision

`Applicability::AgentRequired` — the rule binds the subject, no mechanical provider can
judge it, and reaching a judgment needs a model. It is neither `Is_Evaluated` nor
`Is_Coverage_Debt`, and `Applicability::Is_Agent_Required` is the one place that decides so.

`Coverage::Agent_Required` gives the gaps back, alongside `Coverage::Debt`. The three
buckets are disjoint: no state answers two of the predicates, and a test asserts it over the
whole enum rather than over a list of the states someone remembered.

`Coverage::Is_Complete` now requires the agent-required gaps to be empty as well. That is
the correction that matters most and it is not obvious, so it is stated plainly: being
outside `Debt` is a claim about *what the remedy is*, not about whether the run finished.

## Which Mis-filing Each Case Was

Both available answers were wrong, in opposite directions, and this variant is a correction
of each rather than an addition beside them.

**Filed as `MissingCapability`.** That variant says no installed provider offers a capability
the rule requires, which points the reader at installing one. For an agent-required subject
nothing is missing and there is nothing to install. It was also counted as coverage debt, so
a run reported debt that no installation could ever pay.

**Left out of `Coverage` entirely.** The subject appeared in neither `evaluated` nor `gaps`,
and `Is_Complete` therefore returned true. The second half of `Coverage` is the half that
matters and the half every tool forgets; this was that failure inside the type built to
prevent it.

## Why Not NotApplicable

Because the rule does bind. `NotApplicable` is the only variant that is a positive statement
about the absence of a judgment, and reusing it would make it a positive statement about two
incompatible things — the rule not reaching this subject, and the rule reaching it and
waiting on a model.

## Why DisplayLabel Grows A Variant Too

`DisplayLabel` deliberately loses information, and four applicability states already collapse
into `Unavailable`. Collapsing agent-required into it would reproduce the first mis-filing at
the layer most readers ever see, and a matrix cell reading `Unavailable` sends them to
install something that does not exist. So the presentation enum widens as well. That widening
is the price of the correction, and it is paid once rather than by every client re-deriving a
mapping.

## What This Does Not Do

**No executor.** Nothing in this workspace can run a model, and no `AgentJudged` claim is
produced anywhere. This decides only that a run can *say* a subject needs one. `WF-006`'s
task/result protocol remains unbuilt and is not in scope here.

**Nothing yields `AgentRequired` yet.** No resolution path returns it, and that is correct:
which subjects need a model is a property of a rule's declared requirements, not an outcome
of provider resolution, so the capability registry should never produce it. The first
producer arrives with the first rule that declares work no provider can do.

**No requirement-assessment registry.** `OD-TRACE-001` says whichever of `P10-AGENT-REQUIRED`
and `P10-LEDGER-CORPUS` lands first carries the registry it decided. This item's territory is
two source files, two surface snapshots and this record; a registry needs a home and a guard
that are outside it, and territory cannot be widened mid-claim. So `CHK-003`'s verdict is
**Met** and this record is where it says so, in prose, until an entry replaces it. That entry
is opened as `P10-TRACE-REGISTRY`, and it is the only thing between this record and the
mechanism `OD-TRACE-001` specified.

## Status

Closed by `P10-AGENT-REQUIRED`.
