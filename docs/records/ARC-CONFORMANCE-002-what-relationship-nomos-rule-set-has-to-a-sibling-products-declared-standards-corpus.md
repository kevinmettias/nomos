---
id: ARC-CONFORMANCE-002
type: decision
title: A sibling's declared standards corpus is a fact Nomos reads, and Nomos does not write itself into what enforces it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - conformance
  - ecosystem
  - standards
  - boundaries
relations:
  - target: ARC-CONFORMANCE-001
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
  - target: OD-PLATFORM-003
    type: relates-to
---

# A sibling's declared standards corpus is a fact Nomos reads, and Nomos does not write itself into what enforces it

## Question

The `xvpe` checkout beside this workspace — the one `OD-PLATFORM-003` made this build depend
on, so it is always on disk — ships a committed, machine-readable standards corpus under
`docs/arch/standards`, behind a `rule.schema.json` requiring every rule document to declare
`id`, `severity`, `enforced_by` and `gate` in YAML front matter. `ARC-ECOSYSTEM-001` makes
Nomos the ecosystem's conformance layer. Nomos appears in `enforced_by` zero times, and
Nomos's own rule set was ported from `code-standards` instead. What relationship the two have
is the question this record answers.

## The census

Parsed directly from the front matter of all 375 documents rather than taken from the item
that raised this; every figure below reproduced.

**375 markdown documents: 261 `rule`, 109 `index`, 5 `reference`.** Every one carries front
matter; none is missing it.

**Severity:** 161 `MUST`, 5 `MUST NOT`, 83 `SHOULD`, 1 `SHOULD NOT`, 11 `MAY`.

**Gate:** 224 `review`, 31 `blocking`, 6 `unreachable`. So **86 percent of the corpus declares
that no tool reaches it.**

**`enforced_by` names Nomos zero times.** What it does name, in descending order:
`review` (224 — the value *is* the word, a human), `check-impact-map` (11), `lint` (10),
`check-arch-doc-structure` (3), `check-strategy-docs` (3), `check-platform-boundaries` (3),
`discipline` (2), `strategy-determinism-gate` (2), `layering-direction` (2), then singletons.

**The 224 are not a uniform mass, and where they sit is what decides this record.** Narrowing
to the ones that carry real force — `review`-gated and `MUST` or `MUST NOT` — leaves **132**,
and they cluster:

| count | area |
|---|---|
| 52 | `strategy-surfaces` |
| 19 | `determinism-and-numerics` |
| 16 | `measurement` |
| 12 | `documentation` |
| 12 | `ui` |
| 9 | `workspace-layout` |
| 8 | `security` |
| 4 | `verification` |

**71 of those 132 — 54 percent — are in the two domains this workspace already models
natively.** Nomos has declared determinism domains and a `DeterminismStrength` on every
production; `OD-DETERMINISM-001`/`-002` govern them; one file per `Strategy` declaration is
already this repository's own convention. Titles from that half read *Determinism Boundary
Isolation*, *Forbidden Nondeterminism Sources*, *Strategies Declare Honestly*, *Temporal
Strength Requires Ordering Discipline*. These are not style rules a linter was too lazy to
implement. They are `ARC-CONFORMANCE-001`'s category in its own words — *does this system,
built the way its own records say it should be built, still hold the shape its architecture
commits it to* — asked 132 times by a product that wrote them down and then had to admit
nothing could check them.

## Decision

**The corpus is an external declaration Nomos reads as a fact, and Nomos's absence from
`enforced_by` is correct and stays.** Those are not in tension; they are the two directions of
one relationship, and collapsing them is how this question gets answered wrongly.

**Nomos does not write itself into `enforced_by`.** Those files are `xvpe`'s, this workspace
does not own them, and a conformance layer that edits its subject's declarations to name
itself has stopped measuring and started asserting. `enforced_by: review` is also *true* today
— a human does enforce it — so changing it would be replacing a fact with an intention. The
relationship is one-directional: `xvpe` declares, Nomos reads.

**The corpus's location and shape are declared, never compiled in.** Nomos must not carry
`rule.schema.json`'s field names in its source. A repository under check declares where its
external standards corpus is and what shape it has, in the `OD-RULES-011` form this workspace
already uses for every other rule parameter — the same discipline `OD-RULES-031` applied to a
benchmark's oracle convention a day earlier. A Nomos that hardcodes one sibling's schema is a
Nomos that means nothing for the second sibling, which is the defect `OD-RULES-011` exists to
end.

**What a rule does with the three declared fields**, so that a later increment does not decide
it by accident:

- **`severity` maps to `GateCategory`.** `MUST`/`MUST NOT` are `Blocking`; `SHOULD`/`SHOULD
  NOT` are `Advisory`; **`MAY` produces no finding at all.** A permission is not a claim, and a
  rule reporting on one would be reporting that something was allowed.
- **`gate: blocking` produces no Nomos finding.** `xvpe` declares a tool already reaches these
  31. A second enforcement of one rule is two authorities for one fact, which this repository
  refuses everywhere else; reading them is for knowing they are covered, not for covering them
  again.
- **`gate: review` is the population**, and `Applicability::AgentRequired` is what it becomes.
  `OD-CONTRACTS-002` is exact about what that state does: a run decides only that it can *say*
  a subject needs a model. That is a deterministic answer about an undecidable question, which
  is precisely the seam `ARC-ECOSYSTEM-001` draws — Nomos does not judge *Strategies Declare
  Honestly*, it enumerates every subject that rule reaches and says which need judging.
  `declared-role-matches-surface` is already this exact shape, built and composed.
- **`gate: unreachable` is a finding Nomos can make on its own authority**, and it is the one
  place reading this corpus produces a deterministic verdict rather than a routing decision. A
  declared `MUST` that its own author marks as reachable by nothing is a governing requirement
  with no path to evidence — `ARC-CONFORMANCE-001`'s subject exactly, stated by the subject.
  Six of them today.

## What this record does not do

It writes no contract crate, no provider, no rule, and changes nothing in `nomos-rules`. It
edits nothing anywhere in the `xvpe` tree, which this workspace does not own and must not
modify to make its own reading easier.

It does not decide when the reading increment is built, how the corpus is addressed as a
subject, or how a run is asked for a subset of 224 routing findings rather than all of them.
That last is a real cost and is named here rather than discovered later: a run that emitted
224 `AgentRequired` findings unbidden would be unusable, and `gate run --rule` narrowing
existing today is not the same as selecting by severity, gate or area.

It does not re-open `OD-PLATFORM-003`. The sibling checkout's presence is the precondition
that makes this corpus readable at all, and it is also why this reading can never run in CI —
the same hole the three corpora already sit in, which any increment here inherits and must
declare rather than discover.

It does not claim the 132 are violations, or that any of them fails. They are a population of
declared requirements whose authors recorded that nothing checks them. What is measured here
is the size and shape of that population, not its verdict.

## Status

Accepted. `xvpe`'s standards corpus is a fact Nomos reads through a declared, per-repository
policy; Nomos stays out of `enforced_by`; severity maps to gate category with `MAY` producing
nothing; `blocking` is already covered; `review` becomes `AgentRequired`; and `unreachable` is
the one Nomos verdict this reading yields. No code moves here.
