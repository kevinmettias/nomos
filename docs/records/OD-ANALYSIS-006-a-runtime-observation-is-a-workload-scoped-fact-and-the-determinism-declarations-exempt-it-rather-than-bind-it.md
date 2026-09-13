---
id: OD-ANALYSIS-006
type: decision
title: A runtime observation is a workload-scoped fact, and the determinism declarations exempt it rather than bind it
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - analysis
  - capability
  - contracts
  - determinism
  - boundaries
relations:
  - target: OD-ANALYSIS-004
    type: relates-to
  - target: ARC-CONFORMANCE-001
    type: relates-to
  - target: OD-SYNTAX-002
    type: relates-to
  - target: OD-DETERMINISM-002
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
---

# A runtime observation is a workload-scoped fact, and the determinism declarations exempt it rather than bind it

## Question

`FactVariant` (`crates/contracts/nomos-contracts/src/guarantee/fact_variant.rs`) orders
`RuntimeObserved` as its strongest resolution level — "established by observing execution" —
and `EvidenceClass` (`crates/contracts/nomos-contracts/src/finding/evidence.rs`) carries
`Observed` ("directly observed at runtime") above every class but `Verified`. Neither has a
producer anywhere in this workspace. `OD-ANALYSIS-004` settled the level beneath it,
`SemanticallyResolved`, by fixing what a producer must have resolved before claiming it, and
closed by naming exactly what it left open: static facts a resolved compiler model states
without executing anything are `SemanticallyResolved`'s territory; "facts that can only be
established by running the program and watching what it actually does... [are] the shape
`RuntimeObserved` exists for." That is the boundary this record sits on the far side of.

A runtime fact is not more of the same kind of thing the other four levels are. Every
producer this workspace has — and every one `OD-ANALYSIS-004` described — is a function of
its subject's own content: the same source, read by the same provider at the same guarantee,
yields the same fact, which is exactly what lets `crates/substrate/nomos-analysis/src/fact/key.rs`
key a fact by subject, provider, guarantee, build variant and configuration and treat two
matches as the same fact. A runtime observation is a function of an *execution* — it carries a
workload, a machine, a build variant and whatever the machine was doing at the time — and
running the identical subject through the identical provider does not reproduce it, because
nothing about "identical subject" pins down which path a workload takes or how long a lock is
held while it runs. `tests/contract/tests/determinism_declarations.rs` and the `Strategy`
trait it enforces (`crates/contracts/nomos-contracts/src/determinism/strategy.rs`) already
hold every existing producer to a determinism triple written against exactly the case that
does not hold here.

Three questions have to be settled before the first runtime producer is written, for the same
reason `OD-ANALYSIS-004` settled its two before the first program-semantics producer was:
both are cheap to decide now, with no capability crate yet shipping a private answer to
either, and expensive to unwind afterward.

1. What identifies a runtime observation as a fact, and what it carries beyond the value.
2. How a runtime fact composes with a static fact into one claim.
3. What a runtime-informed claim means when no runtime observation exists for its subject.

## A Runtime Observation Is Identified By Its Workload And Its Environment, Not By Its Subject Alone

`FactKey` already carries nine parts: `contract`, `contract_version`, `subject`,
`semantic_inputs`, `provider`, `provider_version`, `guarantee`, `variant` (`BuildVariantId` —
"one configured program variant: target, features, profile, toolchain") and `configuration`
(`ConfigurationId` — the digest of a fully resolved effective policy). That set is complete
for every producer this workspace has today because each one's fact is fully determined by
those nine parts: fix the subject's content, the provider, the guarantee and the build
variant, and a `Syntactic` or `SemanticallyResolved` fact about it cannot come out differently
between two runs.

A runtime observation is not determined by those nine parts, and reusing the key as-is would
let two different observations collide as though they were one fact, or let one observation
be read back for a workload it was never taken under. Two dimensions are missing, and a
runtime fact's identity carries both beyond the value, alongside what it already inherits from
`FactKey`:

- **The workload.** What was executed to produce the observation — the entry point exercised,
  the inputs or scenario that selected which of the subject's paths actually ran, and any seed
  or parameter that would change which paths those are. A runtime fact only says something
  about the paths its workload actually took; an observation of an allocation-free claim taken
  under a workload that never calls the allocating branch is not evidence the branch is
  allocation-free, and an identity that dropped the workload could not tell the two apart. This
  is a new dimension — nothing in `nomos-contracts::identity` names it today.
- **The observation environment.** The machine and conditions the execution ran under — enough
  to say whether a repeated observation is the same experiment or a different one: hardware
  class, operating system, and anything about contention or load at the time that would give an
  identical workload on an identical build a materially different measured result. This, too,
  is a new dimension.

**The build variant is not a new dimension.** `BuildVariantId` already exists and already
means what a runtime fact needs it to mean — which optimization level, target and feature set
the executed binary carried — so a runtime producer reuses it exactly as every existing
producer does, rather than growing a second identity for the same fact. The pattern `OD-ANALYSIS-004`
already states for domain vocabulary applies here on the identity side too: a runtime capability
is free to define its own payload shape, but is not free to reinvent an identity dimension this
workspace already has a name for.

This record does not define the workload or environment identities as Rust types, for the same
reason `OD-ANALYSIS-004` did not define a program-semantics payload: that is a capability's own
territory, built and judged against this record rather than by it. What this record fixes is
that a runtime fact's identity is incomplete without both, and a producer that keys its facts
only by `FactKey`'s existing nine parts has under-identified them — two observations taken under
different workloads, or on different machines, are not the same fact merely because their
subject, provider and build variant agree.

## A Runtime Fact Composes With A Static Fact At The Weaker Of The Two

`ARC-CONFORMANCE-001` already states the general shape a conformance claim takes: it is
"composed from architecture..., requirements..., history..., runtime evidence..., and
policy... checked against each other." A claim in this family is the concrete case that
shape was written to cover, and the WHY that reserved this item names it exactly: whether an
allocation site on a path declared allocation-free actually allocates, whether a lock that is
statically permitted is contended past a declared latency budget, whether a code path that
type-checks is ever taken. Each combines three things and is nothing without any one of them —
a static fact (the site exists, the lock is statically permitted, the path type-checks), a
runtime fact (the site was reached and allocated, the lock was held this long, the path never
ran), and a declared constraint this repository's own architecture or policy states (this path
must not allocate, this lock's contention has a budget, this branch must be reachable). Static
alone says a thing is legal; runtime alone says it is slow, or that it happened, or that it did
not; the constraint is what turns either into a finding, exactly as the WHY states.

Composing a runtime fact with a static fact into one claim does not need a new mechanism. It
needs the one this workspace already has for combining evidence of unlike strength, applied to
a case that draws on two facts instead of one:

- **`EvidenceClass::Weaker_Of`** already states the rule: "the class of a conclusion drawn from
  evidence of both classes. Always the weaker." (`crates/contracts/nomos-contracts/src/finding/evidence.rs`).
  A composed claim's evidence class is the weaker of its static fact's class and its runtime
  fact's class — an `AgentJudged` reading of a lock site paired with a `Verified` runtime trace
  of its contention does not average into something stronger than `AgentJudged`, for the same
  reason `Test_Combining_Should_Never_Exceed_The_Weaker_Input` already holds every other
  combination to.
- **`Guarantee::Satisfies`** already refuses to average across axes: "every axis, not a score. A
  provider that is sound but syntactic does not satisfy a requirement for semantic resolution
  however sound it is." A composed claim's guarantee is bound by whichever of its constituent
  facts is weaker on a given axis, the same way a single provider's guarantee is already checked
  axis by axis rather than blended.

Nothing about `FactVariant`'s ordering changes to make this true — `SemanticallyResolved` and
`RuntimeObserved` are not stacked levels of the same fact being progressively strengthened,
they are two different facts about two different things (what the code permits, and what an
execution did), cited beside each other in one finding rather than reduced to one scalar. What
composes is the *evidence* backing the claim, on the two axes that already have a rule for
combining unlike strengths — not the resolution levels themselves, which stay attached to the
fact each was established at and are reported as what they are.

## The Determinism Declarations Are Not Satisfiable As Written, And What Replaces Them Is Already Named

`Strategy` (`crates/contracts/nomos-contracts/src/determinism/strategy.rs`) obliges every
execution domain in this workspace to declare `DeterminismStrength`, `ReproducibilityScope`
and `TraceEquivalence`, and `tests/contract/tests/determinism_declarations.rs` holds every
domain that claims reproducibility to a harness that checks it. Every domain the six-row table
names today either claims `State` or `StateTemporal` — "the same inputs yield the same
outputs" — or explicitly claims neither, and the one row that does not,
`DeterminismStrength::None`, already carries the doc comment that decides this question before
this record states it: "the honest declaration for anything reading a clock, sampling, or
consuming a model backend."

A runtime observation reads a clock, or its moral equivalent — it observes what a workload's
execution actually did, and repeating the identical workload on the identical build variant on
the identical machine is not guaranteed to reproduce the identical observed value, because the
value depends on scheduling, contention and timing the process does not fully control. **A
runtime producer cannot honestly claim `DeterminismStrength::State` or `StateTemporal` for the
observation itself**, and `Declaration_Is_Coherent` already forbids the failure mode the WHY
names — a producer that claimed `State` while its trace could not actually be reproduced would
be exactly the corruption the WHY describes, a producer quietly declaring itself deterministic
when it is not, which is worse than an honest `None` because a declaration is what a peer reads
and plans around rather than verifies for itself.

**They are not satisfiable as written, and what replaces them is not a new level.** It is the
existing floor this workspace already has a name for and a worked example of:
`DeterminismStrength::None`, `ReproducibilityScope::SingleRun`, `TraceEquivalence::NotApplicable`
— the same triple `AgentHost` already declares in
`crates/contracts/nomos-contracts/src/determinism/strategy.rs`'s own test module, for the
parallel reason that a model backend's output is a function of an execution rather than of
source alone. A runtime-observation domain occupies this triple honestly rather than omitting a
declaration or reaching for a stronger one that reads better.

This exemption is narrower than it can be misread to be, and stating the boundary is why this
section exists rather than a one-line pointer to the `None` row:

- **It is the observed value that is `None`, not everything a runtime producer does.** Once an
  observation has been captured and written into a `Fact`, storing it, comparing it, serving it
  from a cache, and composing it into a finding are exactly as reproducible as the equivalent
  step for any other fact — the "Fact cache and incremental reuse" and "Analysis kernel" rows of
  the domain table still govern that handling. What `None` disclaims is the claim that
  re-running the workload reproduces the same observed value, not the claim that re-reading an
  already-captured one does.
- **`None` does not exempt a runtime producer from `Declaration_Is_Coherent`.** It must still
  pair `None` with `TraceEquivalence::NotApplicable`, exactly as `AgentHost` does, and the
  workspace's existing test already refuses the inverse — `None` paired with a trace claim — the
  same way it refuses `State` paired with no trace claim at all.
  `Test_Declaration_Is_Coherent_Should_Refuse_A_Trace_Claim_With_No_Strength` covers this today; a runtime declaration adding
  a fourth occupant to that same coherent pairing changes nothing about it.
  `tests/contract/tests/determinism_declarations.rs`'s enforcement is likewise unchanged by a
  `None`-declaring domain arriving: `Rows_Nothing_Declares` filters to rows that
  `Claims_Reproducibility()`, so a domain declaring `None` owes a declaration and a coherent
  pairing, but not a harness proof of reproducibility it never claimed — there is nothing to
  verify about a promise that was never made.
- **A new row of the domain table, not a reuse of the existing one's label.** The `None` row the
  table already carries — "Progress UI, logs, telemetry, agent execution" — is diagnostic output
  and agent judgment, neither of which is a fact this workspace's own rules hold a subject to. A
  runtime observation is a fact, filed the way `nomos-cap-syntax`'s syntactic facts already are,
  and a fact producer sharing a row with progress output would blur exactly the distinction
  `EvidenceClass::Is_Mechanical` exists to keep separate — a runtime capability adds its own row
  to the table, at the same triple, the way `OD-ANALYSIS-004` left its own capability's crate and
  payload for later work rather than deciding them here.

## What A Runtime-Informed Claim Means When No Observation Exists

`Applicability` and `Observation` already carry the reporting this needs, at two different
grains, and a runtime-informed check must use both rather than collapsing them into one
absence.

**No runtime producer for the subject at all** is a capability question, answered the way
`OD-ANALYSIS-004` already states for a missing `SemanticallyResolved` producer: "a rule that
cannot get [the] fact for a subject reports `MissingCapability` or `ProviderUnavailable` exactly
as a syntax rule does." A rule needing `RuntimeObserved` evidence with no runtime provider
installed reports `Applicability::MissingCapability`; one where a runtime provider is installed
but could not run for this subject reports `Applicability::ProviderUnavailable`. Neither reads
as a pass, by the type's own design — `Applicability::Is_Evaluated` is false for both.

**A runtime producer ran, but the observed workload never took a given subject's path** is the
finer-grained case, and it is the one the WHY's own examples turn on: an allocation site the
workload never reached says nothing about whether the site allocates, and a check that reported
it as compliant would be reporting silence as a finding. This is `Observation`'s three-state
shape (`crates/capabilities/nomos-cap-syntax/src/payload/observation.rs`), generalized exactly
the way `OD-ANALYSIS-004` already anticipated it would be — "the day a second capability needs
the identical shape," judged against `OD-CONTRACTS-001`'s band-0 admission criterion — and this
is that second capability. Per item a runtime payload records:

- **`Observation::NotObserved`** — the observed workload never exercised this site. Nothing is
  claimed either way; this is the "did not look" answer `OD-SYNTAX-002` already named
  ("not observed is not absent") and the one the WHY requires a runtime-informed check be able
  to give, rather than reporting the site clean because nothing was seen.
  `Observation::Was_Observed` is false, exactly as for a syntax provider that could not look.
- **`Observation::Absent`** — the workload reached the site and the fact in question did not
  occur (the allocation did not happen, the lock was not contended past the budget). This is a
  genuine finding of absence, not a stand-in for silence, and it is the one case a runtime
  observation actually earns.
- **`Observation::Present(value)`** — the workload reached the site and the fact occurred; the
  value is what was measured.

A rule whose subject is only partially covered by what the observed workload reached reports
`Applicability::PartiallySupported` for the subject and states, in its own payload, which sites
were `NotObserved` — the same discipline `OD-ANALYSIS-004` already states for partial semantic
resolution: "what was not covered is recorded separately and is not implied to be clean." It
does not fall back to a weaker `FactVariant` for the whole subject, since the sites the workload
did reach were genuinely `RuntimeObserved`, and it does not report `Supported` for a subject
whose coverage depends on which paths a workload happened to take, because `Supported` promises
every capability was evaluated "at the guarantee it asked for," which a partially-exercised
workload has not delivered.

## What This Record Does Not Do

It does not build a capability, a payload schema, or a crate under `crates/capabilities`. The
first runtime-observation producer is separate work, reserving its own territory, and this
record is what that work is measured against — the same relationship `OD-ANALYSIS-004` holds to
the first program-semantics producer, and `ARC-CONFORMANCE-001` holds to every capability
argument.

It does not add a workload identity or an environment identity as a Rust type, a field on
`FactKey`, or a change to `crates/substrate/nomos-analysis`. It states that a runtime fact's
identity is incomplete without both; defining their shape is the first runtime producer's own
work, judged against this record.

It does not add a row to the domain table in `crates/contracts/nomos-contracts/src/determinism.rs`,
or an `impl Strategy` anywhere. It states what triple a runtime-observation row must declare —
`None` / `SingleRun` / `NotApplicable` — and that the row is distinct from the existing
"Progress UI, logs, telemetry, agent execution" row; adding the row is the first runtime
producer's obligation, discharged the way `tests/contract/tests/determinism_declarations.rs`
already requires of every domain that arrives.

It does not promote `Observation` out of `nomos-cap-syntax`, change its three states, or change
`Applicability`, `EvidenceClass`, `FactVariant` or `Guarantee` in any way. Every type this
record names is unchanged by it, exactly as `OD-ANALYSIS-004` left them.

It does not enumerate every claim a runtime-informed capability may eventually make. The
allocation, contention and reachability shapes named above are illustration of the composition
rule, not a closed list, the same status `OD-ANALYSIS-004`'s own five worked shapes have.

It does not decide which crate a runtime-observation capability lives in, whether it is one
crate or several, or how a workload is actually driven or an environment actually sampled.
`OD-CAPABILITY-002`'s criterion governs the first question unchanged; the second and third are
implementation questions this record does not reach.

## Amendment: The Coherence Test Named Above Was Renamed

Version 1 said `Test_No_Strength_Should_Refuse_A_Trace_Claim` "covers this today". That name
does not exist and had not for some time when `OD-SPEC-017`'s census over every record found
it on 2026-09-13.

**The assertion still holds and the test still exists**, as
`Test_Declaration_Is_Coherent_Should_Refuse_A_Trace_Claim_With_No_Strength` in
`crates/contracts/nomos-contracts/src/determinism/strategy.rs`, beside its three siblings
covering the rest of `Declaration_Is_Coherent`'s pairings. The rename added the function it
tests as a prefix; nothing about what it refuses changed, and nothing about this record's
reasoning depends on the name.

It is corrected rather than left because the claim was a present-tense claim about coverage.
`D-134` ranks a false claim of coverage above an admitted gap, and `spec/domain-specification.md`
republished this one for as long as it stood — which `OD-SPEC-017` decided is the case no
reading of that question leaves alone.

## Status

Closed by `P12-RUNTIME-EVIDENCE`. Version 2, amended once to name the coherence test the
renamed one became.
