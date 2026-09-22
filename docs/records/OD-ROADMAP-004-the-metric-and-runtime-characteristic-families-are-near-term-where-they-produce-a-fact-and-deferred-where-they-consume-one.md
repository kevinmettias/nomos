---
id: OD-ROADMAP-004
type: decision
title: The metric and runtime-characteristic families are near-term where they produce a fact and deferred where they consume one
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - roadmap
  - sequencing
  - analysis
  - capability
  - evidence
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
  - target: OD-ROADMAP-002
    type: relates-to
  - target: OD-ANALYSIS-004
    type: relates-to
  - target: OD-ANALYSIS-006
    type: relates-to
  - target: OD-ANALYSIS-007
    type: relates-to
  - target: OD-TRACE-001
    type: relates-to
  - target: ARC-CONFORMANCE-001
    type: relates-to
---

# The metric and runtime-characteristic families are near-term where they produce a fact and deferred where they consume one

## Question

`ARC-ROADMAP-001` sorted the corpus's 61 requirement families into a near-term tier and a
deferred tier by one criterion — "Treating architecture and feature intelligence as near-term
work pulls effort into consumer-side algorithms before the substrate they consume (facts,
applicability, gates, corrections) is production-grade" — and named the families on the
deferred side by prefix: `ATL`, `ARC`/`ARC-AN`, `FEAT`, `PLACE`, `PLAY`, `DBG` and `TEST`,
with `EGRAPH` held out of both lists by its constraint 3. It names no family that measures
anything. Seven families — `MET`, `MET-COMP`, `PERF-MODE`, `NFR-PERF`, `INSTR`, `CAL` and
`RUNTIME`, 44 requirements between them — appear on neither side, and the record's own
rounding ("roughly three-quarters of the 363 requirements ... fall on the near-term side;
the remainder clusters almost exactly onto ...") is where they went unnoticed.

That is a gap in placement, not in the corpus. `OD-ANALYSIS-004` and `OD-ANALYSIS-006`
settled how a program-semantics fact and a runtime observation are *expressed* — through the
five existing epistemic types, never a domain-local substitute — and `OD-ANALYSIS-007`
settled what would pick the first such capability. None of the three places a corpus family
and none mentions a metric. So a session reaching for complexity, allocation, latency or any
other characteristic of a program finds the vocabulary decided and the sequencing undecided.
The honest answer today is not "deferred"; it is "nobody has said", and that is the state the
item that reserved this record was written to end.

## What Was Measured

**Corpus and revision.** Every count below was taken by listing
`01_authoring/artifacts/requirements/` in the code-standards checkout at
`f0d820729acfa70a2fa60bd98eb6704fba7cf90b`, whose domain volumes each carry the header line
`Domain-owned edition v14.19 • 4 August 2026` (line 20 of every file under
`01_authoring/domain_volumes/`). That directory holds 363 files in 61 prefixes, the
population `ARC-ROADMAP-001` and `OD-TRACE-001` both describe. This repository was at
`8338c6ec6bbb095cb897675643c82e4bb62d379f` when the item was claimed.

**The seven families**, every file read in full:

| Prefix | Count | Identifiers | Corpus section |
|---|---|---|---|
| `MET` | 2 | `MET-006`, `MET-007` — the family's numbering starts at 006; no `MET-001` through `MET-005` exists | `volume-06.7-5-3-evidence-authority-and-metric-aggregation` |
| `MET-COMP` | 8 | `MET-COMP-001` through `MET-COMP-008` | `volume-03.multidimensional-comparison-projections` |
| `PERF-MODE` | 6 | `PERF-MODE-001` through `PERF-MODE-006` | `volume-12.10-3-1-operating-mode-performance-and-scale-contracts` |
| `NFR-PERF` | 4 | `NFR-PERF-001` through `NFR-PERF-004` | `volume-12.10-1-core-non-functional-requirements` |
| `INSTR` | 6 | `INSTR-001` through `INSTR-006` | `volume-07.7-7-3-runtime-integration-tiers-and-semantic-instrumentation` |
| `CAL` | 9 | `CAL-001` through `CAL-009` | `volume-08.7-8-3-architecture-decisions-discovered-contracts-and-verification-models` |
| `RUNTIME` | 9 | `RUNTIME-001` through `RUNTIME-009` | `volume-07.7-7-2-runtime-rule-lifecycle` |

44 in total, every one at `authority: canonical-normative-record`, `status: normative`,
`maturity: accepted`.

**Assessment.** `tests/contract/requirements/` holds 39 `.assessment` files, in the families
`AGT`, `AGT-EXEC`, `CAP`, `CHK`, `COR-EXEC`, `EVID`, `MODEL-ROUTE`, `WF` and `WORK-LEDGER`.
None of the 44 is assessed. Under `OD-TRACE-001` each is `Unassessed`, which that record made
a state rather than a gap, and this record leaves every one of them there.

**Prior placement.** No file under `docs/records/` names any identifier in the seven
families. `ARC-ROADMAP-001` names nine prefixes — the seven deferred ones and `EGRAPH` — and
three individual requirements (`WF-001`, `IF-001`, `MODEL-ROUTE-037`); none of the seven
prefixes is among them.

**What the workspace holds.** `crates/` was grepped for the families' vocabulary, with the
specification crates' own test corpora set aside (they embed the corpus text and match
everything):

- *metric* — `PackageKind::MetricProvider` in
  `crates/contracts/nomos-contracts/src/package_kind.rs`, a kind in that closed set
  documented as "A source of metric observations."; nothing in the workspace declares
  itself one. Four doc comments in `crates/kernel/nomos-model` say a subject is a thing "a
  rule, metric or finding can be about". No metric descriptor, no metric value, no type
  carrying a unit.
- *complexity*, *characteristic* — nothing.
- *tradeoff* — `unresolved_tradeoffs: Vec<String>` on `nomos-corrections`'
  `CorrectionChoice`, a direct transcription of `COR-012`'s five recorded nouns, written by a
  correction's author as free text; and three doc comments in `nomos-rules` using the word
  about their own design. No projection.
- *calibration* — `RuleCalibration { rule, rationale }` and `AdoptionPolicy` under
  `crates/orchestration/nomos-gate-orchestration/src/policy/rule_calibration.rs`, with a
  serializable twin in `nomos-api`. Its own doc names its family: `ADOPT-CONFIG-*`'s
  per-repository "advisory rather than blocking" override, matched by rule alone. That is
  not `CAL-001`'s `RuleCalibrationCase`, which links findings, dispositions, corrections, run
  evidence and package versions. The word is shared; the requirement is not.
- *instrument* — `GateRunProvenance.instrument` and `Instrument_Digest` in the gate crate,
  and `JudgmentDifference::Instrument`: the digest of the `nomos` build that judged a run,
  so two runs can say whether the same tool measured them. Not `INSTR`'s instrumentation of a
  subject program.
- *`RuntimeObserved`* — one producer and one reader outside test code:
  `nomos-connector-coderabbit`'s `nomos.cap.review.finding`, whose contract claims the level
  because "reading one already-posted review comment by its own stable id is a live read of
  GitHub's current record for it", and `nomos-rules`' `checks/review.rs`, which requires it. A
  review comment read live is not a subject program executed; nothing produces or reads a
  characteristic of a program's execution.
- *`SemanticallyResolved` program code* — `nomos.cap.rust.copy_clones` and
  `nomos.cap.rust.nested_locks` in `crates/languages/nomos-lang-rust-compiler`, each stating
  a resolved fact about one site (a `Copy` receiver cloned; a lock guarding a value already
  behind a lock). Neither is a count, and neither carries a unit.

The nearest thing to a metric is three threshold rules in `nomos-rules` — nesting depth
(`checks/nesting_depth.rs`), function arity (`checks/function_shape.rs`) and file size
(`checks/structure.rs`) — each resolving its ceiling from `nomos.cap.limits.policy`
(`crates/capabilities/nomos-cap-limits-policy`, `OD-RULES-011`). The *limit* is a capability
fact a repository declares. The *count* is not: each rule computes it privately, compares,
emits a `Finding`, and discards it. No count is filed under any `FactVariant`, none declares
a unit, a direction or whether two snapshots of it are comparable, and none can be read by a
second rule or compared across two runs — `gate compare`
(`crates/orchestration/nomos-gate-orchestration/src/gate_compare.rs`) diffs finding
dispositions, not values.

`crates/capabilities/` holds thirteen crates now, not the one `OD-ANALYSIS-007`'s version 1
counted; none produces a metric. `OD-RULES-008` records the first real need for a
`SemanticallyResolved` program-semantics fact arriving by `OD-ANALYSIS-007`'s first arm — the
fact a reachability rule's own subject needs — and `OD-ANALYSIS-010` decides its mechanism;
that is evidence the trigger shape works, and it is the shape this record reuses.

## The Criterion, And A Third Disposition

`ARC-ROADMAP-001`'s criterion is one sentence: consumer-side algorithms wait until the
substrate they consume is production-grade. Applied to a family that measures, it asks of
each requirement which side of the fact seam it sits on.

- **The fact-producing half** is anything that states what a measurement *is*: its
  descriptor, the identity that keeps two observations apart, what a producer must have done
  to claim a resolution level, and what a reader may not do with it. Under `OD-ANALYSIS-004` a
  static characteristic of a program — a count, an allocation decided at compile time, a
  resolved escape — is a program-semantics fact at `Syntactic` or `SemanticallyResolved`;
  under `OD-ANALYSIS-006` a characteristic only execution shows — an allocation that happened,
  a latency, a lock held this long — is a `RuntimeObserved` fact keyed by workload and
  environment; and a modelled alternative is `FactVariant::Predicted`, "Modelled rather than
  measured." This half is substrate. It sits on the near-term tier under "analysis +
  incremental fact infrastructure" and "language/provider/capability system", and because it
  is substrate nothing yet asks for, it is built only when the trigger below fires — exactly
  as `OD-ANALYSIS-007` holds the first program-semantics producer.
- **The consuming half** is anything that ranks, compares, visualizes, aggregates or feeds
  back a measurement: a tradeoff projection, a fitness view, a correction ranker, a
  calibration loop. A consumer sits on the tier its substrate sits on, and after it. Where the
  substrate is a metric fact, the consumer is deferred, because that substrate is not merely
  unbuilt but untriggered. Where the substrate is the near-term rule platform itself —
  findings, suppressions, baselines, corrections, package versions — the consumer is near-term
  and sequenced behind the pieces of that platform still landing.

Two of the 44 fit neither tier, and the corpus says so itself. `RUNTIME-005` marks
`ActiveRuntimeGuard` "a distinct deferred capability" that "shall not be inferred, enabled,
or implemented without an explicit product, security, and authorization decision", and names
the decisions that would lift that. A deferred-tier item is buildable the day its substrate
is ready; this one is not. So the third disposition is **withheld by the corpus**, with this
criterion: *the requirement's own text names a decision outside this repository's sequencing
that must exist before the thing may be built, and substrate readiness does not stand in for
it.* Nothing this record or `ARC-ROADMAP-001` decides can release a requirement in that
state. Only the named decision, adopted here as a record, can.

## The Placement

### `MET` — near-term, fact-producing, both requirements

`MET-006`: "Every metric descriptor shall define unit, subject kinds, aggregation operator,
weighting, normalization, missing-data behavior, directionality, baseline requirements,
uncertainty/statistical treatment, and snapshot comparability." That is a capability
contract's payload schema in the corpus's words — the per-item shape `OD-ANALYSIS-004` leaves
domain-local, beside the five questions it does not (`FactVariant`, `Guarantee`,
`EvidenceClass`, `Applicability`, `Observation`). The corpus itself files a metric on the
fact side: `NFR-PERF-004` lists "metric observations" beside "parsed syntax, semantic
results, graphs" as things a rule projection reuses across rules, which is what a fact in the
fact store is.

`MET-007`: "A client may not aggregate or blend a metric beyond the descriptor’s declared
semantics. Non-aggregable metrics must remain at their native granularity or use an
explicitly named derived metric." Binding on a client, but it is the descriptor's reading
rule — the reason `MET-006` makes a descriptor declare an aggregation operator at all — the
same way `Guarantee::Satisfies` is the reading rule that makes a declared `FactVariant`
worth anything. It has no build of its own and belongs to the contract, not to any consumer.
Both near-term; neither built until the trigger fires.

### `MET-COMP` — deferred, consuming, all eight

`MET-COMP-001`: "Nomos shall expose MetricTradeoffProjection as the canonical
renderer-neutral comparison contract for current, historical, observed, derived, or
predicted alternatives across an explicitly declared ordered set of MetricDescriptors." A
projection over descriptors none of which exists. `MET-COMP-007` names its consumers — "No
MetricTradeoffProjection, client visualization, correction ranker, or architecture fitness
view shall declare a universally best alternative" — and `MET-COMP-005` its views:
"Pareto-front projections, artifact comparison profiles, tradeoff tables, and bounded
architecture fitness-neighborhood views". Architecture fitness is the deferred tier's own
vocabulary (`ARC`/`ARC-AN`); client visualization is the deferred clients'. The whole family
is deferred: its substrate is untriggered and its named consumers are on the other list.

Two notes, so nothing is re-derived when it is built. First, `MET-COMP-002` ("evidence
classification, provider and configuration provenance, uncertainty") and `MET-COMP-006`
("Predicted alternatives shall remain distinguishable from observed or verified
alternatives ... and shall not be represented as a measured outcome") are already answered by
`EvidenceClass`, `FactKey`'s provider and configuration parts, and the `Predicted` and
`RuntimeObserved` ends of `FactVariant`; the projection inherits `OD-ANALYSIS-004`'s
vocabulary rule and mints none of its own. Second, the one entry in `MET-COMP-005`'s list
that is near-term, "candidate-correction comparisons", does not pull the projection forward.
`OD-ROADMAP-001` licenses `CorrectionCandidate` ranking now, and `MET-COMP-007`'s
obligations on a ranker — objective weights, rejected alternatives, unresolved tradeoffs —
are the nouns `CorrectionChoice` already transcribes from `COR-012`. A ranker over *metric*
alternatives needs metric facts, and none exist.

### `PERF-MODE` — near-term, a contract on the product, split by mode

This family is not about a subject repository's characteristics; it is about this product's.
`PERF-MODE-001`: "Every shipped operating mode shall have an OperatingModeProfile defining
representative dataset scale, cold and warm latency targets, freshness guarantee,
cancellation responsiveness, peak and steady-state memory budget, incremental invalidation
budget, streaming/pagination behavior, approximation or truncation policy, and degraded-mode
behavior." It is neither producer nor consumer of the metric seam, so the criterion does not
reach it; `ARC-ROADMAP-001`'s own list does, under "production hardening / continuous
enforcement". That record verified eight performance claims against
`crates/substrate/nomos-analysis` and left six "as real hazards deliberately left for later
work"; a profile per shipped mode is what turns a hazard into a budget with a pass or a fail.

`PERF-MODE-002` lists the initial profiles and splits along the tiers by mode. "active-editor
incremental check, changed-file check, local gate, repository snapshot analysis, ... workflow
execution, ... historical comparison" name modes that exist or are near-term — the
`nomos-lsp` host, `nomos check`, the gate verbs in `nomos-gate-orchestration` (`plan`,
`run`, `explain`, `compare`), `nomos-workflow-orchestration` — and their profiles are
near-term with them. "Atlas navigation, large graph query, capture import, ... hosted
organization dashboard" name deferred modes, and a profile arrives with its mode.
`PERF-MODE-003` through `PERF-MODE-006` are the family's measurement discipline — targets
name "hardware/environment class, repository or graph scale, build variants, provider set,
cache state, concurrency, network assumptions, measurement method, percentile, sample count,
and acceptance threshold"; freshness is "specified independently from response latency";
degraded modes "state which guarantees are retained"; release acceptance benchmarks
"representative profiles" — and go with `PERF-MODE-001`. A benchmark of this product is a
runtime observation of it, and `PERF-MODE-003`'s identity list is `OD-ANALYSIS-006`'s
workload-and-environment identity in the corpus's words, so when profiles are measured they
are keyed as that record says and not by a second scheme.

### `NFR-PERF` — split, each requirement with the subsystem it binds

- `NFR-PERF-001`: "Interactive source/architecture lookup at a debugger stop should be served
  from indexes and ordinarily complete within 50 ms locally, excluding provider calls." Binds
  the debugger integration — **deferred**, with `DBG` and "runtime/debug intelligence".
- `NFR-PERF-002`: "Editing feedback should use changed-region analysis and target sub-second
  local diagnostics for cached deterministic rules; slower tools must stream status." Binds
  incremental analysis and the editor host — **near-term**, under "analysis + incremental
  fact infrastructure".
- `NFR-PERF-003`: "Atlas/architecture clients shall request semantic-zoom summaries rather
  than entire huge graphs and support progressive refinement." Binds Atlas — **deferred**.
- `NFR-PERF-004`: "Rule projections shall reuse parsed syntax, semantic results, graphs, and
  metric observations across rules and clients." Binds the fact store — **near-term**, and it
  is the sentence that placed `MET-006` on the fact side above.

### `INSTR` — split at the tier boundary the family itself draws

`INSTR-001`: "Runtime evidence integrations shall identify one descriptive tier: Tier 1
GenericExternalAdapter, Tier 2 NomosInstrumentationSdk, or Tier 3 NativeRuntimeIntegration.
Tier shall not substitute for capability guarantee, coverage, provider trust, or evidence
classification." The second sentence is `OD-ANALYSIS-004`'s rule stated from the corpus's
side: a tier is a fact about the provider, and three of the four things it may not stand in
for are already typed here as `Guarantee`, `Coverage` and `EvidenceClass`, which keep their
authority.

- **Near-term, fact-producing**: `INSTR-001` (the vocabulary), `INSTR-002` ("Tier 1 adapters
  may import or stream established profiler, trace, debugger, allocation, build, and compiler
  outputs, including ETW/EventPipe, perf, pprof, Tracy, DAP, compiler timing reports, and
  provider-native exports where available") and `INSTR-006` ("Clients and rules shall not
  treat Tier 1, Tier 2, and Tier 3 evidence as semantically interchangeable merely because
  they expose similarly named metrics or events"). A Tier 1 adapter is
  `ARC-CONFORMANCE-001`'s case exactly — a provider already exposes the fact and Nomos
  composes it — and it is the cheapest producer of a `RuntimeObserved` fact under
  `OD-ANALYSIS-006`, because a capture is an input a headless run reads, like a tree.
  `INSTR-006` is a reader's obligation and part of the same contract.
- **Deferred**: `INSTR-003` and `INSTR-004` (the Tier 2 SDK) and `INSTR-005` (Tier 3
  native). `INSTR-003`'s content is the deferred tier's own — "semantic spans, feature-stage
  events, subsystem boundary crossings, test/scenario identity" is `FEAT`, `PLACE` and `TEST`
  vocabulary — and `INSTR-004` makes the SDK a shipped product surface in its own right
  ("versioned, low-overhead, opt-in, privacy-aware, offline-capable, and independent of Nomos
  internal storage, transport, sampling, visualization, and deployment implementations").
  `INSTR-005`'s "task, replay, scheduling, allocation, subsystem, and causal identities" is
  runtime/debug intelligence, which `ARC-ROADMAP-001` defers by name.

### `CAL` — near-term, consuming, behind the rule platform it consumes

`CAL-001`: "Nomos shall support a RuleCalibrationCase that preserves links among recurring
findings, suppressions, false-positive and false-negative dispositions, corrections, run
evidence, provider coverage, rule interactions, KnowledgeWorkbench claims and decisions,
motivating defects, examples, counterexamples, and affected RulePackage versions." Every
noun in that list but two is on the near-term tier and most are built: findings, suppressions
and baselines (`nomos-gate-orchestration`'s policy modules), corrections
(`nomos-corrections`), run evidence (`GateRunProvenance`, `compare`), provider coverage
(`Applicability`, `Coverage`). The case record is the consumer that joins them. So the family
is near-term — a consumer whose substrate is the near-term tier itself — and sequenced behind
two things, each until its own condition clears:

- **`RulePackage`'s lifecycle**, which `OD-ROADMAP-001` licensed and `ARC-ROADMAP-001`
  constraint 2 brought into scope. `CAL-001`'s "affected RulePackage versions", `CAL-004`'s
  "resulting RulePackage version" and `CAL-007` ("Historical results shall continue to
  resolve against the normative version that governed them") have nothing to point at until
  a package has a version.
- **`OD-ROADMAP-002`'s gate-policy pause, for as long as it stands.** `CAL-006` ("Canary
  results shall not silently affect blocking policy") and `CAL-009` ("shall not modify
  applicability, severity, thresholds, suppressions, gate disposition, or active enforcement
  without the existing governed review, versioning, publication, and rollout process")
  describe a loop whose write end is gate policy vocabulary. While that pause stands, the
  mechanism by which a calibration reaches a threshold or a disposition waits, as that record
  says, "for the gate to stop selecting twice and to compile into one resolved run plan"; the
  pause lapses on that record's own terms — its successor landed, or abandoned — and this
  record adds no wait of its own beyond it. The read end — `CAL-005`'s canary,
  measuring "false positives, false negatives or known misses where measurable, provider and
  target coverage, execution cost, latency" against "pinned corpora" — is not paused, and this
  repository already does it by hand: it runs its own rules over its own tree in CI
  (`OD-GATE-004`) and over corpora CI cannot see (`OD-GATE-001`), recording none of the
  measurements `CAL-005` names.

`CAL-002` and `CAL-003` are the two nouns not on the near-term tier: KnowledgeWorkbench.
`CAL-002` keeps the export "without making KnowledgeWorkbench part of deterministic gate
execution", and `CAL-003` keeps KWB's outputs "claims or proposals" that "shall not modify
applicability, severity, thresholds, suppressions, or enforcement automatically". That
crossing is `ARC-ECOSYSTEM-001`'s — its ownership table answers "May a run's own recorded
observations become KWB knowledge?" with "KWB, only carrying the runs as provenance" — and
`ARC-ROADMAP-001` constraint 4 keeps ownership there. On this side of the seam an export has
nothing to carry until a `RuleCalibrationCase` exists, so the two sit where `CAL-001` sits.
`CAL-008`'s "ObservedFact → GeneralizedClaim → AdoptedNorm → ExecutableEnforcement", with no
later stage "inferred solely from the existence of an earlier stage", is the principle
`ARC-CONFORMANCE-001` already states for KWB evidence — KWB provides evidence, Nomos decides
the claim — and nothing new is built for it.

### `RUNTIME` — three dispositions

`RUNTIME-001`: "Runtime-rule execution modes shall be OfflineCaptureEvaluation,
StreamingObservation, or ActiveRuntimeGuard. These modes are distinct capability, authority,
performance, and deployment contracts and shall never be inferred from one another."

- **Near-term, fact-producing**: `RUNTIME-001`, `RUNTIME-002`, `RUNTIME-006`, `RUNTIME-008`,
  `RUNTIME-009`. `RUNTIME-002`'s mode — "evaluate retained traces, profiles, replays,
  debugger observations, benchmark artifacts, or allocation captures after execution without
  altering the observed process" — is the one a Tier 1 adapter feeds and a headless gate run
  can consume. `RUNTIME-006` ("workload or scenario, observation window, build and source
  snapshot, build variant, provider and instrumentation mode, warm-up, sampling or tracing
  policy, hardware and environment, aggregation and statistical method, baseline
  compatibility, permitted evidence loss, symbolization quality, and enforcement role") is
  `OD-ANALYSIS-006`'s identity — workload and environment beyond `FactKey`'s nine parts,
  `BuildVariantId` reused — in the corpus's words. `RUNTIME-008` ("Missing events, dropped
  samples, incomplete symbolization, incompatible hardware, divergent workloads, insufficient
  warm-up, unstable variance, or baseline mismatch shall produce explicit evidence
  limitations or non-pass applicability states rather than a clean result") is that record's
  `Observation::NotObserved` and `Applicability::PartiallySupported` treatment. `RUNTIME-009`'s
  examples — "allocation prohibition in a configured hot path, ... latency or queue-depth
  budget" — are the ones `OD-ANALYSIS-006` already uses. All of it is settled in shape and
  unbuilt, and it is built when the trigger fires.
- **Near-term, under `OD-ROADMAP-002`'s gate-policy pause while it stands**: `RUNTIME-007`.
  "Enforcement roles shall be Observational, Advisory, ReviewRequired, or Blocking. Blocking
  runtime policy shall require compatible baselines, minimum evidence sufficiency, configured
  loss limits, and explicit repository or organization authorization." That is a gate policy
  increment — roles, thresholds, an authorization — and it is bound by that pause exactly as
  `CAL-006` and `CAL-009` are, and for exactly as long.
- **Deferred**: `RUNTIME-003`. "StreamingObservation shall evaluate events during execution
  and may publish findings, alerts, or evidence, but shall not stop, reject, redirect, or
  mutate application execution unless separately authorized as an ActiveRuntimeGuard." Live
  evaluation during a subject's execution is a runtime-intelligence shape; nothing on the
  near-term tier evaluates anything while a subject runs, and the headless loop consumes a
  capture, not a stream. `RUNTIME-005` places the mode "within the approved product
  contract", so it is deferred, not withheld.
- **Withheld by the corpus**: `RUNTIME-004` and `RUNTIME-005`. `RUNTIME-004`'s guard "may
  stop, reject, throttle, redirect, or otherwise alter execution and therefore requires an
  explicit guard-capability contract, owner, deployment policy, fail-open/fail-closed
  behavior, latency and resource budgets, safety analysis, rollback/recovery behavior, and
  independent authorization", and `RUNTIME-005` withholds it in full: "ActiveRuntimeGuard is
  a distinct deferred capability and shall not be inferred, enabled, or implemented without
  an explicit product, security, and authorization decision. [Maturity: Release-scope
  constraint; Target: Deferred beyond Release 5; Decisions: D-045 and D-050]". Those two
  decisions are the corpus's, and this repository has adopted neither as a record. Until one
  is, no substrate readiness and no tier places the guard.

## What Builds The First Metric-Family Capability

`OD-ANALYSIS-007`'s trigger has two arms — a rule whose own subject needs the fact "to reach
a verdict it cannot reach at `Syntactic` or `Approximate` today", or a record of this
repository "naming a specific claim in this family as something a conformance check must
make" — and `OD-RULES-008` records the first arm firing for reachability, with
`OD-ANALYSIS-010` deciding the mechanism. The same two arms pick the first metric-family
capability, with one adjustment the measurements above force. A metric's *count* is not missing: three rules compute one today and reach their
verdicts from it. What is missing is the descriptor, and a descriptor is owed at the moment a
count has two readers who must agree what the number means — the contention criterion
`OD-CAPABILITY-002` already uses to decide when a contract earns a crate. So, in this family:

1. **A second reader of a count.** A rule, a gate verb or a projection that needs a value
   another rule already computes and cannot get it without recomputing it: `gate compare`
   asked whether a depth or an arity *rose* between two runs, which is a baseline for a value
   where today's baseline is for a finding; or a second rule reading a first rule's count.
   That reader's need names the descriptor `MET-006` requires — which unit, which direction
   is worse, whether two snapshots are comparable. Until it exists a count read by one rule is
   that rule's private intermediate, and filing it as a fact would be inferring a need from a
   wish, which `OD-ANALYSIS-007` already names as the mistake via `D-135`.
2. **A rule needing a `RuntimeObserved` fact about a subject's execution**, which is
   `OD-ANALYSIS-006`'s producer arriving through `OD-ANALYSIS-007`'s first arm. `RUNTIME-009`'s "allocation prohibition in
   a configured hot path" is the candidate the corpus itself offers, and it brings
   `INSTR-002`'s Tier 1 adapter and `RUNTIME-002`'s offline mode with it, because those are
   the cheapest way the fact enters.
3. **A record of this repository naming a characteristic a conformance check must hold this
   workspace to** — a budget, a ceiling, an allocation-free path — declared as a claim rather
   than as a rule's threshold. `nomos.cap.limits.policy` is the closest existing thing and is
   deliberately not this: it carries the limit a repository declares, not a measurement of it.

When any arm fires, the capability takes the shape `OD-ANALYSIS-004` fixed: a domain-local
payload carrying `MET-006`'s descriptor fields; `FactVariant` at the level the producer
actually reached (`Syntactic` for a count read from text, `SemanticallyResolved` for one read
from a resolved model, `RuntimeObserved` for a capture, `Predicted` for a modelled
alternative); `EvidenceClass`, `Applicability` and `Observation` unchanged; and, for a runtime
producer, the `None` / `SingleRun` / `NotApplicable` row and the workload-and-environment
identity `OD-ANALYSIS-006` requires. Which crate it lives in is `OD-CAPABILITY-002`'s
question, not this record's.

## What This Record Does Not Do

- It schedules no crate, names no capability identifier, defines no payload schema and
  authors no item. It changes no code. Every type it names is unchanged by it.
- It edits none of `ARC-ROADMAP-001`, `OD-ANALYSIS-004`, `OD-ANALYSIS-006` or
  `OD-ANALYSIS-007`; it relates to them and reads their criteria onto seven families those
  records did not reach. `ARC-ROADMAP-001`'s two lists are unchanged. This record says which
  list each of 44 requirements would appear on, and names the two that appear on neither.
- It assesses no requirement. `OD-TRACE-001`'s committed set stays at 39 entries and none of
  the 44 gains one; placing a family on a tier is not a verdict about whether a site meets it.
- It adopts neither of the corpus decisions `RUNTIME-005` names. Adopting one is a record of
  its own, and is the only thing that releases `ActiveRuntimeGuard`.
- It orders the near-term tier only where `OD-ROADMAP-001` and `OD-ROADMAP-002` already do:
  `CAL` behind `RulePackage`'s lifecycle, and `CAL-006`, `CAL-009` and `RUNTIME-007` behind
  `OD-ROADMAP-002`'s gate-policy pause for as long as that record holds it. It adds no
  ordering of its own, and none that outlives its source.
- It does not restate ownership. Where `CAL-002` and `CAL-003` cross to KnowledgeWorkbench,
  `ARC-ECOSYSTEM-001` governs, per `ARC-ROADMAP-001` constraint 4.

## Status

Accepted, drawn by `P121-METRIC-FAMILIES-HAVE-NO-ROADMAP-TIER`. It schedules nothing, so
nothing discharges it as a whole; what would revisit each part is named in place. The
placement of `MET`, `INSTR-002` and the near-term `RUNTIME` requirements is exercised, not
reopened, when one of the three trigger arms fires. `CAL`'s sequencing lapses as
`RulePackage`'s lifecycle lands and as `OD-ROADMAP-002`'s gate-policy pause lapses on its own
terms, whichever way it does. `RUNTIME-004` and
`RUNTIME-005` move only by a record adopting a decision `RUNTIME-005` names. And if the tiers
themselves move, that is an amendment to `ARC-ROADMAP-001`, and this record follows it rather
than the other way round.
