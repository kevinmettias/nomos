---
id: ARC-ROADMAP-001
type: architecture
title: Nomos Core's near-term boundary is the headless enforcement loop, not architecture and feature intelligence
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - roadmap
  - architecture
  - sequencing
  - boundaries
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
  - target: OD-GATE-004
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
  - target: OD-RULES-002
    type: relates-to
  - target: D-132
    type: relates-to
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-TRACE-002
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
---

# Nomos Core's near-term boundary is the headless enforcement loop, not architecture and feature intelligence

## Question

The workspace has grown a real vertical slice — capability registry, two independent Rust
providers, a fact store, deterministic rules, applicability, coverage, and an early
correction lifecycle — beside a much larger specified surface that has not been built:
architecture discovery, feature topology, runtime/debug intelligence, and rich clients. What
was missing was not a list of gaps. It was a boundary: which of those belong to "the first
usable Nomos product" and which belong to what that product is later extended by.

Getting this wrong in either direction is a real cost. Treating architecture and feature
intelligence as near-term work pulls effort into consumer-side algorithms before the
substrate they consume (facts, applicability, gates, corrections) is production-grade.
Treating the enforcement loop as already sufficient without naming what still separates it
from a product — first-class gates, baselines, suppressions, a headless workflow engine, a
stable application-service boundary, agent/model infrastructure — leaves the gap
undiscoverable until someone tries to deploy the thing continuously.

## What Was Measured

Two independent external architecture reviews of the `dev` branch were each checked against
the real workspace rather than accepted on read. Concretely verified, not merely repeated
from the reviews: the crate/band structure in `Cargo.toml` and `README.md`; the CI gate's
actual steps in `.github/workflows/gate.yml`; the presence and shape of `Applicability`
(`crates/contracts/nomos-contracts/src/finding/applicability.rs`), `Coverage`/`Claim`
(`crates/model/nomos-model/src/evidence`), and `nomos-corrections`' preview/stage/validate/
commit/rollback chain; and eight specific performance claims against
`crates/substrate/nomos-analysis` (six confirmed as real hazards deliberately left for later
work, one — the recursive `Tarjan::Visit` with no depth guard — fixed by
`P13-TARJAN-ITERATIVE`, verified by reverting it and reproducing a native stack overflow on
the unfixed code before restoring the fix).

Separately, the 363 requirements the v14 corpus carries at `authority:
canonical-normative-record` were surveyed structurally: every filename under
`01_authoring/artifacts/requirements` in the v14 corpus, grouped into the corpus's own 61
requirement-family prefixes by count, with one representative requirement title read per
family. This is **not** the formal per-requirement `Met`/`Diverges`/`NotBinding` assessment
`OD-TRACE-001` and `OD-TRACE-002` govern — no `.assessment` file was written by this survey,
and the assessed set stays at the six entries those records already describe. It is a
coarser question: does the corpus's own family structure support the boundary this record
draws, or contradict it.

It supports it. Grouping the 61 families by title against the two-tier split below,
roughly three-quarters of the 363 requirements (kernel, package, rule, finding, gate,
workflow, agent, and the cross-cutting non-functional families) fall on the near-term side;
the remainder clusters almost exactly onto `ATL` (Atlas), `ARC`/`ARC-AN` (Architecture
Explorer), `FEAT` (feature topology), `PLACE` (placement analysis), `PLAY` (feature-path
playback), `DBG` (debugger integration) and `TEST` (feature-scenario test intelligence) —
the corpus's own grouping, not one imposed on it. Specific requirements upgrade parts of
this record from judgment to binding text: `WF-001` defines gate policy (phases, thresholds,
coverage) in terms matching the Gate object named below; `IF-001` requires every
user-visible query or action to have a canonical application service, which is the
CLI/API/MCP-parity item named below, not a proposal; `MODEL-ROUTE-037` names
`ModelBackendPackage` and `AgentExecutorPackage` as distinct versioned interfaces, matching
the split named below rather than inventing it.

## The Boundary

Two tiers. Given as sets, not a numbered sequence — the survey above establishes which side
of the line each item falls on, not a validated ordering *within* the near-term side. Any
internal sequencing among these items is a separate, later judgment call, not something this
record settles.

```text
Nomos Core / near-term
├ stable kernel and identities
├ analysis + incremental fact infrastructure
├ language/provider/capability system
├ deterministic rules
├ applicability + truthful coverage
├ first-class gates (the product object; see constraint 5)
├ baselines / suppressions / adoption
├ corrections
├ headless workflow orchestration
├ application-service boundary
├ CLI / API / MCP projections
├ model backend + agent executor infrastructure
└ production hardening / continuous enforcement
```

```text
Deferred consuming systems
├ architecture discovery/inference
├ feature topology and path tracing
├ placement analysis
├ runtime/debug intelligence
├ test intelligence built on those models
├ Atlas
├ desktop
├ web
└ mobile
```

EGRAPH — the relationship/evidence graph the corpus's `EGRAPH` family describes connecting
`ChangeIntent`, `CorrectionPlan`, `AgentWorkResult` and workflow state — is deliberately
absent from both lists. Constraint 3 states what happens to it instead.

## Four Constraints

### 1. The corpus is normative; this record and the game plan are sequencing aids

`D-132` already rules that the external "nomos full game plan" reasoning trace is advisory,
superseded by the corpus wherever they disagree. This record is the same kind of document —
useful for sequencing, wrong to treat as authority over the corpus. The 363-requirement
corpus stays what `OD-TRACE-001` already made it: the binding authority, assessed by a
committed entry per requirement, not derived at check time. This record's structural survey
is not that assessment and does not claim to be. Six requirements are formally assessed
today (`OD-TRACE-002`); 357 remain `Unassessed`, which `OD-TRACE-001` already established as
a state, not a gap. Formal assessment population continues as its own governed, many-hands
work, unaffected by this record.

### 2. `OD-PACKAGE-008`'s wait is superseded by `OD-ROADMAP-001`; a `RulePackage` manifest is in scope now

At version 1, this constraint held `OD-PACKAGE-008` authoritative and named no
`RulePackage` manifest as scheduled. `OD-ROADMAP-001` retires that wait, along with the
matching waits in `OD-PACKAGE-006`, `OD-PACKAGE-010`, `OD-PACKAGE-011`, `OD-PACKAGE-012` and
`OD-CORRECTIONS-001` — see that record for why a caution requiring the same override once
per record, once per session, is itself the cost being retired rather than protection worth
keeping. "Rule platform" in the near-term list above now includes building
`nomos-rule-package` and a `RulePackage` install/enable/disable lifecycle directly, not only
scaling what already exists against more real rules. `OD-PACKAGE-008`'s own field-by-field
measurement of `ARCH-002`'s contents list — which fields converge across the four real
rules and which do not — stays true and stays the right starting point for what to build;
only the conclusion that building must wait is superseded.

### 3. EGRAPH stays a kernel constraint, not a kernel construction, on its own footing

The corpus weighs `EGRAPH` at ten requirements and ties it to systems this record defers
(architecture, feature topology) as well as one that exists (`nomos-corrections`) and two
that don't yet (workflow, agent execution). `OD-ROADMAP-001` retires the population-of-zero
wait specifically for the AgentExecutor/ModelBackend/RulePackage/corrections cluster it
names; `EGRAPH` is not in that list; and unlike that cluster, nothing has asked for `EGRAPH`
construction ahead of a consumer. So no dedicated relationship-graph crate is scheduled by
this record, on the same reasoning as before, unrelated to the constraint 2 supersession.

What *is* decided: kernel, corrections and workflow work must not foreclose a later
generalization — identity, evidence classification, and typed-relationship vocabulary
should be shaped so a future `EGRAPH` can be added without redesigning what came before it,
the same "must not be impossible to add later" bar the architecture/feature tier is held to
below. That is a weaker, checkable claim ("does not preclude"), not "is first-class and
built." Revisit construction once a second real consumer exists to check the first one's
shape against, or once `EGRAPH` itself is named in a scope like `OD-ROADMAP-001`'s.

### 4. Ecosystem ownership is `ARC-ECOSYSTEM-001`'s, unrepeated here

`ARC-ECOSYSTEM-001` (version 4) already governs what Nomos, KWB, XVPE, and repository
tooling each own, including the specific question an earlier draft of this reasoning got
wrong: package installation is XVPE's, by explicit adoption of `D-091`, not unfinished
Nomos work. That adoption's own text draws a narrower line than "installation and
activation" restates: `D-091` names package *mechanism* — envelope, resolution,
registries, ingestion, transactional installation, rollback and recovery, signatures — and
`ARC-ECOSYSTEM-001` keeps package *meaning* on this side of the seam, package activation
semantics named among it explicitly, the same way `D-122`'s adoption kept rule and finding
semantics on this side of the analysis-mechanism crossing. This record governs sequencing
inside Nomos's own boundary. It adds no ownership claim and restates none; where the two
are read together, `ARC-ECOSYSTEM-001` is the authority on who owns what, and this record
is the authority on what order Nomos builds its own share in.

### 5. "Gate" here means the product object, not `OD-GATE-004`'s CI step

`OD-GATE-004` wired this repository's own rule layer into its own CI as a step named
`Rules`, so that `nomos check` judges this workspace on every pull request. That is
self-application — this repository enforcing itself — and it is done. It is not the
product-level `Gate` this record lists as near-term: a first-class object an end-user
repository would configure (`ScopeSelector`, `RuleSelector`, `ApplicabilityPolicy`,
`CoveragePolicy`, `BaselinePolicy`, `SuppressionPolicy`, required phases, evidence
requirements, failure disposition), exposed as `nomos gate plan` / `run` / `explain` /
`compare` over the same application-service boundary the CLI/API/MCP item shares. Nothing in
`OD-GATE-004`, `OD-RULES-001` or `OD-RULES-002` builds that object; they establish the rule
layer's own soundness and this repository's own enforcement, which the product Gate will
depend on rather than duplicate.

## What This Record Does Not Do

- It does not schedule work. It draws a boundary; ledger items that build inside it are
  authored separately, against this record where relevant.
- It does not claim the 363-requirement corpus has been reconciled. It is a structural
  survey by family and representative title, not a per-requirement audit.
- It does not reopen `OD-PACKAGE-008`'s field-by-field measurement, restate
  `ARC-ECOSYSTEM-001`, or redefine what `OD-GATE-004` already did — only constraint 2's
  conclusion is superseded, by `OD-ROADMAP-001`, and named there rather than re-argued here.
- It does not order the near-term tier internally. Which of those items is built next is a
  separate judgment, informed by this record but not fixed by it.
- It does not commit to building `EGRAPH`, or any dedicated relationship-graph crate, now.

## Status

Accepted, drawn by `P13-CORE-ROADMAP-RECONCILIATION-3`. It schedules no item and orders
nothing within either tier, so nothing discharges it as a whole; what would revisit each
part is named in place instead — constraint 3 once workflow orchestration or agent
execution reach the point of needing typed cross-entity relationships, constraint 4 by an
amendment to `ARC-ECOSYSTEM-001` rather than to this record. Amended to version 2 by
`P13-ROADMAP-001-POPULATION-CAUTION-RETIRED`, which retired constraint 2's wait via the new
anchor record `OD-ROADMAP-001` — see that record for the reasoning and the full list of
records it supersedes. Amended to version 3 by `P14-ROADMAP-001-ACTIVATION-SEMANTICS-CORRECTION`,
which corrected constraint 4's own restatement of `ARC-ECOSYSTEM-001`'s adopted `D-091`
clause: that adoption keeps package *activation semantics* on Nomos's side of the seam, and
this record's prior text erased that carve-out by compressing "installation" and
"activation" into one XVPE-owned phrase. Package installation mechanism is unchanged as
XVPE's; nothing else about either record's ownership table moves.
