---
id: OD-ANALYSIS-012
type: decision
title: A rule that judged an empty population is reported apart from one that judged clean
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - analysis
  - architecture
  - completeness
relations:
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-COMPLETENESS-004
    type: relates-to
  - target: P43-SCRIPT-RULES-CANNOT-FIRE
    type: relates-to
---

# A rule that judged an empty population is reported apart from one that judged clean

## Question

`CheckOutcome::NoFacts` catches one instance of a general shape: a run whose syntax facts
never materialized reports that fact rather than rendering as a clean tree. Nothing
generalizes the same protection to a single composed rule whose own subject population was
empty while the rest of the run judged real subjects. `P43-SCRIPT-RULES-CANNOT-FIRE`
measured a real, live instance: four composed rules — `scripts-use-a-portable-shebang`,
`a-script-declares-its-purpose`, `executed-scripts-set-nounset`,
`declared-tooling-language-for-scripts` — now walk a real, reachable population that is
simply empty in this tree today, and their zero findings render exactly as if they had
examined real scripts and found them clean. A person required a decision on how a run owes a
reader that distinction, independent of whichever fix the walker itself got.

## What Was Measured

**Every existing outcome vocabulary was read for a shape that already fits, and none does.**
`nomos_contracts::Applicability`'s ten variants are all per-*finding* dispositions — each one
presupposes a `Finding` with a `subject` to attach itself to. An empty population has no
subject at all: there is nothing to build a `Finding` around, so no `Applicability` value,
including `ProviderUnavailable` and `NotApplicable`, can carry this fact. `Claim_Of` (`crates/
orchestration/nomos-check-orchestration/src/examined/claim.rs`) proves the gap directly:
`Test_Claim_Of_Should_Report_Complete_For_An_Empty_Findings_List` shows an empty findings
list already reports `Claim::Complete` — the identical verdict a rule that judged real
subjects and found them clean would produce. `Examined { files, facts }` is the nearest
structural precedent — "two denominators, not one," because "0 findings over 400 files" and
"0 findings over 400 files none of which produced a fact" are different claims — but it is
computed once for the whole run's syntax layer, not per composed rule, and does not reach
`Check_Goals_And_Parts_Line_Up` (a `SubjectKind::Workspace` rule with no per-file population
at all) or the four script rules (`SubjectKind::SourceText`, reading whatever the walk
collected rather than a syntax fact).

**The four rules `P43-SCRIPT-RULES-CANNOT-FIRE` names are the real, present instance, not a
hypothetical.** That correction fixed the walkers that fed them and measured, directly, that
this repository has zero real `.sh`/`.ps1`/`.psm1`/`.bat`/`.cmd` files anywhere outside
`target`/`.git` today. All four rules are now composed against a real, reachable, currently
empty population. Their own zero findings are honest about the tree — nothing wrong exists —
but a reader cannot tell that from a rule that examined a hundred real scripts and found
every one compliant. `P43-SCRIPT-RULES-CANNOT-FIRE`'s own commit message states this outright:
"the four rules will fire the moment a real script with a defect exists to walk into" — which
is a promise about the future, not a fact this run's own report states about the present.

**Where the verdict would be computed is already known, because the population itself is
already known there.** `run_context.rs`'s `Findings_For_Selected_Rules`/`With_Composed_Rules`
already holds, for every `ComposedRule`, the exact slice its check closure is about to read —
`sources` for the text and syntax-fact rules, `capabilities.dependency_sources`/
`lint_sources`/`policy_sources` for the capability-backed ones. The population size is not a
new fact to materialize; it is a count already sitting in scope at the one place every
composed rule's own subjects are threaded through, before its closure is called.

## The Decision

**An empty population is a new fact reported alongside a run's findings, not a new
`Applicability` variant and not a new `CheckOutcome` variant.** It is not `Applicability`
because there is no subject for a per-finding disposition to describe. It is not a new
`CheckOutcome` arm because `CheckOutcome::NoFacts` already owns the coarser claim — the whole
syntax layer failed for every source — and a per-rule empty population is a narrower, still-
real fact that can be true while the rest of the run judges plenty. The right extension is
`CheckOutcome::Judged` itself, alongside `findings`, `examined` and `claim`: a third
denominator, in `Examined`'s own words, naming which composed rules examined a real,
nonempty population and which examined none — computed at the same point in `run_context.rs`
that already holds each rule's own source slice, at no new materialization cost.

**Its disposition is Advisory, the same as `ProviderUnavailable`'s existing `GateCategory`,
and it does flip `Claim` to `Incomplete`.** `Claim::Incomplete`'s own definition is "the run
did not reach a judgment about it" — a rule with zero subjects reached no judgment about
anything, the identical shape a coverage-debt finding already represents, not merely a milder
version of it. Reporting it as `Complete`, the way an empty findings list does today, is the
exact lie this record exists to name: `Claim` currently cannot distinguish "every rule judged
real subjects and found them clean" from "some rules judged nothing at all," and a person
reading a clean, `Complete` run has no way to learn that four of the rules that would have
told them about a broken script never got the chance to look for one.

**Applied to the four rules `P43-SCRIPT-RULES-CANNOT-FIRE` names: today, in this tree, all
four report an empty population under this decision**, since the same repository-wide search
that correction already ran found zero real scripts of any of the five recognized
extensions. This is not a defect in either correction — the walker fix was necessary and
correct on its own terms, and today's population really is empty — it is exactly the fact
this record's own mechanism exists to surface rather than hide.

## What This Record Does Not Do

**No code moves here.** `CheckOutcome::Judged`'s new field, the population count computed in
`run_context.rs`, and `Claim_Of`'s own extension to read it are named precisely enough for a
follow-up item's territory to be declared completely, rather than a decision to build
against.

It does not touch `Applicability` or add a variant to it. The measurement above found every
existing variant presupposes a subject this fact does not have, and inventing one anyway
would misuse a per-finding vocabulary for a per-rule fact.

It does not decide `P41-APPLICABILITY-IN-THE-PLAN`. That item asks which subjects a rule
*declines* once a plan resolves applicability in advance — a question about a plan that does
not exist yet, per `P41-RUN-PLANNER`'s own stranded state. This record's own question — what a
run owes a reader about a rule whose population was already empty when it ran — holds with or
without a plan, is true today, and is answered here rather than left waiting on a planner
that has not been built.

It does not change what `P43-SCRIPT-RULES-CANNOT-FIRE` already did. That correction remains
the right fix to the walkers; this record adds the reporting layer that would have made its
own prior silence visible without needing that specific investigation to find it by hand.

## Status

Accepted. An empty population is a per-rule fact reported in `CheckOutcome::Judged` itself,
Advisory and `Claim`-flipping the same way `ProviderUnavailable` already is, computed from
population sizes `run_context.rs` already holds. The four rules `P43-SCRIPT-RULES-CANNOT-FIRE`
measured are the real, present instance: all four report an empty population in this tree
today, honestly, rather than an indistinguishable clean.
