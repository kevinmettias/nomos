---
id: ARC-ROADMAP-001
type: architecture
title: Nomos Core's near-term boundary is the headless enforcement loop, not architecture and feature intelligence
status: accepted
version: 4
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
  - target: OD-ROADMAP-003
    type: relates-to
  - target: OD-ROADMAP-004
    type: relates-to
  - target: OD-ROADMAP-005
    type: relates-to
  - target: OD-RULES-024
    type: relates-to
  - target: OD-PROJECT-007
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

## Amendment: The Tier Is Replaced By A Per-Item Condition, Measured Item By Item At `53de19a4`

Added at version 4 by `P125-WHETHER-THE-DEFERRED-TIER-STILL-HOLDS`. Nothing above is
rewritten, and the boundary above is quoted rather than edited so the change can be checked
against it.

This record's boundary rested on one stated reason: "Treating architecture and feature
intelligence as near-term work pulls effort into consumer-side algorithms before the
substrate they consume (facts, applicability, gates, corrections) is production-grade." That
was measured when the near-term tier was substantially unbuilt. A person has since asked for
architecture and feature intelligence, Atlas, feature topology, placement analysis, runtime
and debug intelligence and test intelligence to be built, every one of which sits on the
deferred tier. Building them against a standing boundary would be a session overriding an
architecture decision by doing the work, which is the move this repository refuses everywhere
else. So the condition was re-measured rather than assumed, and the answer below is not the
answer being asked for.

### How this was measured

Every row was taken against the tree at `53de19a4`, read through `git show` rather than off
the working tree, which carried three live sessions' uncommitted work at the time. Two numbers
are live runs of the committed-era binary rather than reads: `gate plan --root .`, and a
`check` scoped to `crates/kernel/nomos-store`. The three corpus variables were unset, as CI
has them.

The words are this record's own item's — built, partial, absent. One further state
`OD-PROJECT-007` already coined is used where it fits: **declared and unobserved**, a member
exporting something no composition takes. That record's caution is the one this measurement is
most exposed to, and it was applied on purpose. A crate existing is not evidence a thing is
built; the question asked of every row is what reaches it.

One commit landed from a peer session between the measurement and this amendment, `dbb2c7c7`,
and it touches the item most exposed to it. It gives gate policy a declared layering and an
effective policy naming what decided each field, which is a real advance on item 6 and is not
folded into that row, because the row was taken at `53de19a4` and a measurement that quietly
absorbs later work is not a measurement. Both gaps that row names were re-checked against
`dbb2c7c7` directly and both survive it: no type named `ApplicabilityPolicy` exists there
either, and `EvidenceClass` still appears in the gate crate's policy modules only in test
fixtures. No other row is reached by that commit.

### The near-term tier, item by item

**1. Stable kernel and identities — built.** `nomos-model` carries identity, subject,
evidence, transition and digest as separate modules; `nomos-store` carries the
content-addressed document store with an explicit write authority. Sixty-nine blessed surface
snapshots under `tests/contract/surface/` hold those surfaces still, and a widened surface
reddens a gate step rather than passing. The one qualification worth writing down: the root
manifest sets `publish = false`, so every surface is stable against this repository's own
snapshots and against no external consumer.

**2. Analysis and incremental fact infrastructure — partial.** `FactKey`, `InputDigest`,
`GenerationCause`, `MaterializedFact`, `Supersession`, dependency propagation, guarantee
broadening and an `InvalidationReport` are all real and all exercised. The gap is one line:
`MemoryFactStore` is the only implementation of `FactStore` in the workspace. Nothing writes a
fact anywhere a second process could read it, so incrementality is real within one run and
does not exist between two. Every reuse figure this workspace quotes is a second-pass figure
inside a single process.

**3. The language, provider and capability system — built, with four offers composed into
nothing.** `Registered()` in `nomos-check-orchestration`'s composition root declares fourteen
capabilities and takes seventeen offers against them, over three languages. Against that,
twenty-one `Provider_Offer` functions are exported across `crates/languages`,
`crates/repository`, `crates/connectors` and `crates/capabilities`, so four are declared and
unobserved: `nomos-lang-csharp`'s syntax offer, both of `nomos-lang-rust-compiler`'s, and
`nomos-lang-rust`'s `nomos.cap.module.index` rollup, which is offered only inside its own
crate's tests. The compiler-backed provider is the sharpest case, because it is the only
provider in this workspace backed by a real semantic engine and no crate under
`crates/orchestration` or `crates/host` names it. The capability system is built; two of its
most capable providers are not wired to anything.

**4. Deterministic rules — built.** `DESCRIPTORS` in `nomos-rules` holds seventy-one entries,
each naming its subject kind, the fact families it requires and the record it cites. Run
directly, `gate plan --root .` prints `rules: 71` and exits 0. Determinism is a gate step in
its own right over eight crates.

**5. Applicability and truthful coverage — built.** `Applicability` carries eleven variants,
deliberately has no `Default` and deliberately has no `is_pass`; `Claim` rolls a run up to
complete or incomplete off those variants rather than off a finding count. Run directly, a
`check` over `crates/kernel/nomos-store` answered "14 file(s) examined, 14 with a syntax fact,
2 finding(s), 0 of which can fail a build", then "claim: incomplete" with
"ProviderUnavailable: 2". A run that could not materialize two capabilities said so instead of
reporting clean. This is the item with the least distance between what the record promised and
what the tree does.

**6. First-class gates, the product object — partial.** Of the twelve elements this record's
own constraint 5 enumerates, ten have a real type with a real reader: `ScopeSelector`,
`RuleSelector`, `SuppressionPolicy`, `BaselinePolicy`, `AdoptionPolicy`, `CoveragePolicy`,
phases, thresholds, approvals and failure disposition, with all four verbs — plan, run,
explain, compare — reachable. Two have none. `ApplicabilityPolicy` appears nowhere in the
workspace except inside the quotation of this record in `nomos-gate-orchestration`'s own
module doc; no type of that name exists. Evidence requirements are the second:
`nomos_contracts::Finding` carries an `EvidenceClass`, and within the whole gate crate that
type appears only in test fixtures, so no policy reads it. The gate crate's own doc says both
of these about itself, which is why this row is a confirmation rather than a discovery.

**7. Baselines, suppressions and adoption — built.** `GatePolicyFile` holds six fields —
suppressions, baseline, adoption, coverage, phases, approvals — `Resolve_Gate_Policy` reads
them off a `nomos-gate.json` under the run's root, and `Run_Gate` calls it. An end-user
repository can author all six. The CLI is a narrower surface than the file:
`crates/host/nomos-cli/src/gate/parsing.rs` still constructs every policy at its default and
passes `phases: Vec::new()`, so a phase is authorable through the file and not through argv.
That is a host gap rather than a missing product object.

**8. Corrections — partial.** The preview, stage, validate, commit and rollback lifecycle is
real, and `nomos-correction-orchestration` is a seam both hosts call rather than a CLI module.
What is partial is reach: `CorrectionFamily::ALL` holds two entries against seventy-one rules,
so 2 of 71 rules can have a fix proposed for them at all, and nothing ranks candidates.

**9. Headless workflow orchestration — partial.** `Run` takes an ordered slice of step plans,
`Is_Coherent` gets its first real consumer, retry, timeout and compensation are honored rather
than merely declared, and four `Body` variants cover five dispatch targets. Its only caller
outside its own tests composes exactly one step per invocation, by that module's own statement,
and nothing in the workspace parses a workflow definition. So the tier can execute a sequence
and no person can author one. The crate's own doc lists what stays out — published artifacts,
branch and merge, versioned replayable definitions, cache and cancellation runtime — and that
list is accurate.

**10. The application-service boundary — partial, and being reopened as this is written.**
`nomos-api` exports thirty `Handle_*` functions, which is a real convergence point. There is no
operation-surface crate beneath the hosts, so each host still assembles the product itself:
`nomos-cli` names twenty-six `nomos-*` crates, eight of them orchestration crates, and
`nomos-api` names twenty-two. `IF-001`'s canonical application service per user-visible query
or action is satisfied for the verbs `nomos-api` covers and not by a boundary the hosts depend
on. `OD-ROADMAP-005` authorizes building exactly that surface as its third piece, which is the
strongest available evidence that this item is not finished.

**11. CLI, API and MCP projections — partial.** Four binaries ship: `nomos`, `nomos-lsp`,
`nomos-mcp`, `nomos-surface-provenance`. `nomos-api-transport` serves a closed `ServedMethod`
registry of six operations and `nomos-mcp` publishes the same six as tools over XVPE's
catalogue contract, never naming a handler. Six of thirty handlers is deliberate and
`OD-HOST-007` and `OD-HOST-014` decide it, so it is not a shortfall — but three of the unserved
handlers are product-side rather than repo tooling: agent execute, agent judge role, and
workflow run reach no remote client. An LSP projection exists that this record's list never
named.

**12. Model backend and agent executor infrastructure — partial, and being reopened as this
is written.** The task envelope, `WorkResult`, substantiation, `PrepareChangeContext`'s
resolved form, profile resolution answering an unresolvable selector with a measured
`ProfileAbsence`, one real executor and one real model backend, and three real routing
consumers are all there. What is not there is any abstraction over them: the workspace contains
no trait named `AgentExecutor` or `ModelBackend`, and in fact no `pub trait` at all in
`crates/agent`, `nomos-agent-orchestration` or `nomos-model-package`. `Backend` is a closed
two-variant enum. `OD-ROADMAP-005` authorizes the port as its second piece.

**13. Production hardening and continuous enforcement — absent, in the product sense.** This is
the one row with no product-side evidence at all, and it is the row the deferral's own wording
turns on. What exists is self-application: a fifteen-step CI gate whose `Rules` step runs
`gate run --root .`, which is this repository enforcing itself and which this record's
constraint 5 already distinguishes from the product. Against the product reading: the root
manifest sets `publish = false`; no type named `OperatingModeProfile` exists, and that phrase
occurs in this repository only inside a specification test fixture; there is no benchmark, no
latency, memory, freshness or invalidation budget, and no degraded-mode behaviour anywhere;
three corpora that carry this workspace's strongest scale claims live outside it and CI has
none of them, which `tests/contract/tests/corpus_gates.rs` declares rather than hides; and no
repository other than this one is under continuous enforcement by this product.

Four built, eight partial, one absent.

### Whether the condition the deferral rested on still holds

**It no longer decides, and that is the finding rather than a hedge.** The condition has two
halves and at this revision they answer in opposite directions.

Read as "before the substrate is built", it is false. Facts, applicability, gates and
corrections each have a real type, a real reader and a real run, and one of the four —
applicability and truthful coverage — is fully built by any reading.

Read as "before the substrate is production-grade", it is true, and item 13 is why. The one
near-term item that would turn the other twelve into a product is the one item with no
product-side evidence at all. A fact store that does not survive a process, two of seventy-one
rules with a correction, a gate missing two of the twelve elements this record itself
enumerated, and a workflow tier nobody can author a plan for are each a distance from
production-grade that nothing here has closed.

So a session checking this record's stated reason against the tree finds it half true, and the
record offers no second test. That is not a boundary a reader can act on. It is a prohibition
whose justification each session must re-derive from scratch, which is precisely the cost
`OD-ROADMAP-001` retired for a different caution, and the reason this amendment was owed.

Replacing the test is not the same as lifting the prohibition, and the next section is why the
two come apart here.

### What each deferred item actually waits on

`OD-RULES-024` is both the precedent and the caution. Architecture drift was asked for, was
wanted, and could not be built, because it needed an observed call-graph or data-flow fact this
workspace does not produce; that record's version 2 amendment records that the *declared* half
now exists as `nomos.cap.architecture.declaration` and that the observed half still does not.
Applied to each deferred item at `53de19a4`, naming the substrate fact it would consume and
whether anything produces it:

- **Architecture discovery and inference** consumes an observed graph finer than package
  granularity — module reaches module, function calls function. Nothing produces it.
  `DependencyPayload` carries `package` and a list of edges, which is Cargo-level and nothing
  finer; `ReachabilityPayload` carries flagged sites within one file. The declared half exists
  and the observed half does not, exactly as `OD-RULES-024` v2 states.
- **Feature topology and path tracing** consumes a correspondence between a named feature and
  the code that realizes it. Nothing produces it, and nothing has decided what such a fact
  would be. No capability contract, descriptor, payload, `RequiredFact` variant or provider in
  this workspace carries the word.
- **Placement analysis** consumes the same observed graph as architecture discovery, plus a
  cohesion or coupling measure, which is a metric fact. Neither is produced. `OD-ROADMAP-004`
  places the metric-consuming half deferred in its own right, on the ground that its substrate
  is untriggered rather than merely unbuilt, so this item fails twice over.
- **Runtime and debug intelligence** consumes a `FactVariant::RuntimeObserved` fact about a
  program's execution. The vocabulary exists and `OD-ANALYSIS-006` settled it; the producer
  does not. Exactly one offer in this workspace declares `RuntimeObserved`,
  `nomos-connector-coderabbit`, and what it observed is a posted review comment rather than an
  execution. Part of this item is further out of reach than a missing fact:
  `OD-ROADMAP-004` records `RUNTIME-005`'s `ActiveRuntimeGuard` as **withheld by the corpus**,
  needing a product, security and authorization decision outside this repository's sequencing,
  and nothing this record decides can release it.
- **Test intelligence** is, in this record's own words, "built on those models", so it inherits
  the two absences above. It also needs one of its own: a fact linking a test to what it
  covers. Nothing produces it. `nomos.cap.test_material.policy` classifies whether material is
  test material; it says nothing about what that material exercises.
- **Atlas** consumes a queryable surface and facts that survive the process that made them. The
  surface half is produced: six served operations behind one dispatch, an MCP catalogue and a
  JSON transport. The persistence half is not, per item 2. Its content is the five items above.
- **Desktop, web and mobile** consume the same surface, the same missing persistence, and a
  delivery story that item 13 does not have.

**Five of the nine deferred items fail `OD-RULES-024`'s test outright**, and they fail it for a
reason no amount of production hardening will touch. The other four are clients whose content
is the five that do. Not one of the nine is waiting on the tier.

### The decision

**The two-tier boundary is replaced by a per-item condition.** The two lists above stay as
drawn and stay accurate about where each item sits today; what is withdrawn is the single
shared reason, which has stopped being readable off the tree. In its place each deferred item
carries its own condition, and the conditions fall into four kinds:

1. **A missing capability decision** — architecture discovery and inference, feature topology
   and path tracing, placement analysis, runtime and debug intelligence, and test intelligence.
   The condition is that the fact the item consumes is decided and provided. This is a
   capability question of the weight `OD-RULES-010` and `OD-CAPABILITY-010` were, and
   `OD-RULES-024` is explicit that it is not a detail a rule's own implementation may invent on
   the way past.
2. **Withheld by the corpus** — the `ActiveRuntimeGuard` half of runtime intelligence. Its
   condition is the named external decision `OD-ROADMAP-004` describes, and substrate readiness
   does not stand in for it.
3. **Content-dependent** — Atlas. Its condition is that at least one item in the first kind
   produces a fact worth showing, and that a fact survives the process that made it.
4. **Delivery** — desktop, web and mobile. Their condition is near-term item 13, production
   hardening and continuous enforcement, which is the one near-term item this amendment reports
   as absent.

**No deferred item's condition is met at `53de19a4`.** The practical effect is that the
deferred tier does not move today — but it now waits on nine things a session can check rather
than on one thing it has to re-argue, each lapsing on its own evidence the way
`OD-ROADMAP-002`'s three pauses did under `OD-ROADMAP-003`.

A condition being met makes an item available for an item of its own, with its own territory
and its own measurement. It is not a licence to build, and it is not a schedule.

### What this amendment does not do

**It schedules no implementation and authors no building item.** It moves no item to the
near-term tier, because no item's condition is met.

**It does not decide any of the missing capabilities.** Naming that an observed call-graph
fact, a feature-correspondence fact and a test-coverage fact are what five deferred items
consume is not designing any of them, and `OD-RULES-024`'s refusal to design them inside a
consumer stands.

**It does not withdraw the properties the deferred items are for.** Every one of them remains
real and wanted. What this amendment says is that five of them are undecided as unready rather
than deferred as unwanted, which is `OD-RULES-024`'s own distinction.

**It does not retire near-term item 13 or turn it into a deferred item.** Production hardening
stays near-term; the amendment reports it as absent, which is a measurement and not a
reclassification.

**It does not reopen constraints 1, 3, 4 or 5, or `OD-ROADMAP-001`'s supersession of
constraint 2.** `EGRAPH` stays where constraint 3 put it. Ecosystem ownership stays
`ARC-ECOSYSTEM-001`'s.

**It does not stand beside this record as a second sequencing authority.** It is an amendment
to the record that drew the boundary, for the reason `OD-AGENT-001` gives about restating an
authority rather than routing to it, which applies to a roadmap as much as to an agent file.

### Consistency with `OD-ROADMAP-005`

`OD-ROADMAP-005` landed one commit before this amendment was authored and bears on it in both
directions, so the relationship is stated rather than left to be inferred.

It is evidence for the measurement above. Three of the eight pieces it authorizes are
near-term-tier items of this record — the provider composition root is item 3, the operation
surface is item 10, the agent port is item 12 — which is an independent reading that those
three are not finished, arrived at by a different route than this one.

It is also the shape this amendment follows. That record answers an owner instruction by
bounding it: naming the clause of each record it supersedes, at a stated version, leaving every
measurement those records made standing, and licensing no scope beyond its enumeration. This
amendment does the same to one clause of one record — the single shared reason this record's
tier rested on — and leaves the rest of it standing.

The two do not overlap. `OD-ROADMAP-005` supersedes eight named deferrals in other records and
its own text is careful that it does not touch this record's two-tier boundary; this amendment
touches only that boundary and supersedes none of its eight. And nothing here is that kind of
override: an owner asked for the deferred tier, and the answer is that five of the six things
asked for are held back by a fact nobody produces rather than by a boundary anybody can lift.
The remedy for those is a capability decision, not a roadmap decision.

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

Amended to version 4 by `P125-WHETHER-THE-DEFERRED-TIER-STILL-HOLDS`, which re-measured the
near-term tier item by item at `53de19a4` — four built, eight partial, one absent — and found
the single shared reason the two-tier boundary rested on no longer readable off the tree: the
substrate is built as mechanism and is not production-grade, and the condition's two halves now
answer in opposite directions. The tier is replaced by a per-item condition, one per deferred
item, in four kinds; no deferred item's condition is met at that revision, so nothing moves and
nothing is scheduled. Five of the nine deferred items are held back by a fact this workspace
does not produce rather than by any sequencing decision, which is `OD-RULES-024`'s own finding
read one tier up. Read the amendment above before reading either list as a standing
prohibition, and `OD-ROADMAP-004` for the seven metric and runtime families neither list
sorted.
