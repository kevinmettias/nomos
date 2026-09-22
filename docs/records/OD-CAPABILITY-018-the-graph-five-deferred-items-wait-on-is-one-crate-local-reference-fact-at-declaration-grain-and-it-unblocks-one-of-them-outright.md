---
id: OD-CAPABILITY-018
type: decision
title: The graph five deferred items wait on is one crate-local reference fact at declaration grain, and it unblocks one of them outright
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - analysis
  - architecture
  - roadmap
  - boundaries
relations:
  - target: OD-RULES-024
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-ANALYSIS-004
    type: relates-to
  - target: OD-ANALYSIS-007
    type: relates-to
  - target: OD-ANALYSIS-010
    type: relates-to
  - target: ARC-CONFORMANCE-001
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-GATE-002
    type: relates-to
  - target: OD-ROADMAP-004
    type: relates-to
  - target: OD-ANALYSIS-006
    type: relates-to
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-RULES-003
    type: relates-to
  - target: OD-SYNTAX-002
    type: relates-to
---

# The graph five deferred items wait on is one crate-local reference fact at declaration grain, and it unblocks one of them outright

## Question

`ARC-ROADMAP-001` version 4 replaced its two-tier boundary with a per-item condition and
reported that no deferred item's condition is met. Five of the nine conditions are the same
kind — architecture discovery and inference, feature topology and path tracing, placement
analysis, runtime and debug intelligence, and test intelligence — and that record states the
condition once: "The condition is that the fact the item consumes is decided and provided."
Atlas is its third kind, "Content-dependent", whose own condition is "that at least one item
in the first kind produces a fact worth showing". So six of the nine deferred items stand
behind a fact nobody has decided.

`OD-RULES-024` measured the same gap from the rule side and declined to close it inside a
rule: architecture drift "presupposes an observed graph finer than package-level dependency
edges — module reaches module, type flows into type, function calls function — and this
workspace materializes none of them," and it is "a materialization question of the same
weight `OD-RULES-010`'s `ToolProvider` instances answered for lint and dependency-policy
facts, not a detail to invent inside `architecture_drift.rs`." Its version 2 amendment
recorded that the *declared* half arrived as `nomos.cap.architecture.declaration` and that
"the observed call-graph or data-flow fact it would be judged against does not."

Nobody has decided it since. The question is what that fact is — its grain, whether it is one
capability or several, what a provider must have resolved to claim each level it offers, and
how a fact whose inputs are a whole crate is invalidated — decided from what this workspace
actually carries rather than from the words the requirement is written in.

## What Was Measured

Every reading below was taken against the tree at `9f13b1e7`, from the files named.

### The three existing facts, and the distance from each to a graph

**`nomos.cap.dependency.edges` is package grain and has no sub-package endpoint.**
`crates/capabilities/nomos-cap-dependency/src/dependency_kind/dependency_payload.rs` declares
`DependencyPayload` with two fields, `package: String` and `edges: Vec<DependencyEdge>`; the
edge in `dependency_edge.rs` carries `target: String`, a `DependencyKind` and `optional: bool`,
and its own doc fixes the target as "The dependency's own package name, as Cargo resolved it".
The contract's summary is "One package's own first-party dependency edges — every other
workspace member it names as a dependency". Nothing in the payload names a module, a type, a
function, or a file. The distance from here to "module reaches module" is not a refinement of
this fact; it is a different endpoint vocabulary.

**`nomos.cap.controlflow.reachability` is one path inside one function in one file.** Its
contract summarises itself as "Whether a control-flow path forward from a fact-read failure —
a match arm binding an `Err` from a capability read — reaches a `Finding` construction before
the enclosing function returns", and `arm_shape/reachability_payload.rs` carries "One file's
full set of flagged reachability sites". `reachability_site.rs` states the scope in its own
field doc: "Tier-1's canonical subject is a control-flow edge inside one function body in one
file; a fully qualified path through enclosing `impl`/`mod` blocks is real information a sound
provider should carry, and is not attempted here." The site names its enclosing function
*unqualified*, so two same-named functions in one crate are indistinguishable in the payload.
This fact cannot be widened into a graph without changing what its own endpoints mean.

**`nomos.cap.syntax.items` carries the endpoint vocabulary and, on purpose, no edges.**
`payload_item.rs` declares `PayloadItem` with `ordinal`, `kind`, `visibility`,
`qualified_name`, `documentation` and `shape` — a name qualified by syntactic nesting, and a
coarse shape — and nothing naming a type a signature refers to. The ceiling in
`nomos-cap-syntax/src/contract.rs` says why it may not grow one: "[`FactVariant::Syntactic`] is
the ceiling because the capability is about what a file says on its face. A compiler-backed
provider that resolves names is answering a different question and belongs behind a different
contract." `OD-GATE-002` made the same stance for the surface reader — "The derivation is a
source-level reading, not rustdoc's resolution" — and did not reverse it. So the fact that
already holds the right endpoint names is precisely the fact that may not resolve them.

**The finest declared adjacency is no longer a constant in a rule.** `OD-RULES-024` version 1
measured "`SAME_ZONE_EDGES` is the finest declared adjacency this workspace has" over eleven
zones. Neither `SAME_ZONE_EDGES` nor a `ZONES` constant exists in `crates/` at this revision;
the declaration moved to `nomos-architecture.json`, read through
`nomos.cap.architecture.declaration`, and it now declares twelve components, seventy-two
members and twenty-nine package-pair exceptions. The number changed and the grain did not: the
finest *declared* adjacency is still a component-to-component permission over whole packages,
and there is still nothing observed below it.

### Whether the engine for a graph is already linked, which changes what is being decided

It is. `crates/languages/nomos-lang-rust-compiler` depends on `ra_ap_hir`, `ra_ap_ide_db`,
`ra_ap_load-cargo`, `ra_ap_project_model` and `ra_ap_syntax`, and `src/reading.rs` loads a real
Cargo project with a discovered sysroot and hands back a `RootDatabase` with every file of the
one crate under the root. Its own module doc states that the loader was factored out for
exactly the reason that matters here: "[`Load_Crate`] is that shared part, factored out so
`crate::nested_lock_reading::Discover_Nested_Locks` asks the same loaded [`Semantics`] a
different question rather than re-solving sysroot discovery a second time."
`Test_Discover_Crate_Should_Find_Exactly_The_Real_Clone_On_Copy_Call` holds that this load
resolves for real against a fixture crate, and the two questions already asked of it —
`Type::is_copy` on a resolved call expression, and a resolved generic argument read through a
type alias — are name resolution and type resolution, which is what a reference edge needs.

This is the difference between designing a capability and widening an offer, and it decides
the mechanism question by `ARC-CONFORMANCE-001`'s own test rather than by preference. That
record states: "The test for native analysis is that no provider exposes the fact, never that
writing it natively would be convenient," and "A capability whose fact *is* obtainable from
`rustc`, Clippy or `rust-analyzer` fails the test regardless of how easy it would be to write a
native check for it instead." A resolved reference edge between two Rust declarations is
obtainable from `rust-analyzer`, and `rust-analyzer`'s engine is already a dependency of this
workspace. A native re-derivation of it fails the test.

`OD-ANALYSIS-010` declined a compiler integration for `nomos.cap.controlflow.reachability` and
was explicit that the decline is fact-specific, reserving the opposite answer for a fact like
this one: "nothing here forecloses a future rule whose own subject genuinely needs a fact only
a real compiler frontend can produce — generic instantiation, trait resolution, or
borrow-checker output, none of which this candidate's own scope touches." A reference graph
needs the first two, so that reservation is what this record spends rather than a boundary it
crosses.

### The five consumers are not one consumer, measured one at a time

- **Architecture discovery and inference** wants reachability among the repository's own
  structural units: `OD-RULES-024`'s "module reaches module, type flows into type, function
  calls function", judged against the declaration `nomos.cap.architecture.declaration` already
  carries. Every one of those three is a projection of a declaration-to-declaration edge onto
  one or other end's owner.
- **Placement analysis** wants the same edges *and* a number over them — a cohesion or coupling
  measure. `ARC-ROADMAP-001` version 4 already places that half elsewhere: "`OD-ROADMAP-004`
  places the metric-consuming half deferred in its own right, on the ground that its substrate
  is untriggered rather than merely unbuilt, so this item fails twice over."
- **Test intelligence** wants "a fact linking a test to what it covers", and
  `nomos.cap.test.material.policy` is measurably not it: its contract summarises itself as "A
  repository's own fixture locations -- repository-relative directory prefixes under which its
  test material lives", which classifies where test material sits and says nothing about what
  it exercises. A static edge from a `#[test]` function to a function it names is a different
  claim from coverage, and it is an over-approximation of one: it includes what the test can
  reach and cannot say what a run touched.
- **Feature topology and path tracing** wants a correspondence between a named feature and the
  code that realizes it. The endpoint on the feature side has no referent anywhere in this
  workspace: no capability id, no `RequiredFact` variant and no payload type carries it. The
  fourteen `RequiredFact` variants in
  `crates/rules/nomos-rules/src/rule_descriptor/required_fact.rs` are `SyntaxItems`,
  `DependencyEdges`, `LintDiagnostics`, `DependencyPolicy`, `Reachability`, `NamingPolicy`,
  `LimitsPolicy`, `ScriptingPolicy`, `GoalsPolicy`, `WordsPolicy`, `TestMaterialPolicy`,
  `ReviewFindings`, `RequirementTrace` and `ArchitectureDeclaration`, and seventeen capability
  ids are declared in library code under `crates/capabilities`, `crates/languages` and
  `crates/connectors`. None of either set names a feature.
- **Runtime and debug intelligence** wants an execution observed rather than a reference
  resolved. `OD-ANALYSIS-006` settled that vocabulary and `ARC-ROADMAP-001` version 4 records
  that the one `RuntimeObserved` offer in this workspace observed a posted review comment
  rather than an execution.

Treating these as one consumer is what a record claiming a single capability serves all five
would do, and it is the false-coverage failure `OD-COMPLETENESS-001` and `OD-RULES-023`
declined to ship.

### How a whole-crate fact is invalidated today, which is not by its declared granularity

`IncrementalGranularity` is declared per capability and reported by the store; it does not
decide what an invalidation reaches. In
`crates/substrate/nomos-analysis/src/fact/memory_store/invalidation.rs`, the walk selects the
facts a cause hits through `cause.Is_Naming(key)` and nothing else, and
`crates/substrate/nomos-analysis/src/generation_cause.rs` defines that predicate by identity:
`SubjectChanged` matches `key.subject == *subject`, `SnapshotReplaced` matches
`differing.contains(&key.subject)`. `Test_Is_Naming_Should_Match_Only_The_Subject_A_Change_Names`
holds exactly that. Granularity enters the walk once, in `Note_Broadening`, which records a
`Broadening` on the report — its own comment says the caller "is entitled to know which
provider did that and to how many facts" — and `Broadened_To`'s only caller outside
`Test_Broadened_To_Should_Take_The_Coarser_Granularity` is that line.

Two routes to a whole-crate answer exist in this workspace and they differ in whether an edit
inside the crate reaches the fact.

**The derived route reaches it.** `nomos.cap.module.index`, in
`crates/languages/nomos-lang-rust/src/rollup/`, is this workspace's one non-leaf fact. It
declares `IncrementalGranularity::Project` and its module doc states the consequence rather
than hiding it: "[`IncrementalGranularity::Project`] makes the engine broaden a file-granular
cause and record on [`nomos_analysis::InvalidationReport::broadened`] that it did — the cost of
the rollup, stated rather than absorbed." Its edges are observed, not asserted: "A hand-written
edge list is a *claim* about what was read; this is a record of it, and the two diverge the
first time a read is added and the list is not. That includes the reads that found nothing: a
member with no fact is still an edge, because the day the parser does have something for it,
this rollup is stale and only the edge knows." `Materialize_Index` passes the reader's observed
dependencies to `store.Materialize`, and `Index_Key` digests the members into
`semantic_inputs`.

**The leaf route does not reach it, silently.** `nomos-lang-rust-compiler`'s
`src/fact_context.rs` keys its crate-wide fact at `nomos_model::Subject_Of_Path(&root…)` with
`semantic_inputs: InputDigest::Of(&[])`, and says both things plainly: "`semantic_inputs` is
empty, deliberately … this provider's real input is a real compiler frontend's own analysis of
every file reachable from `root`, which no caller has independently", and `Materialize_Crate`
produces "a leaf: nothing here reads another fact this or any other provider produced."
`nomos-lang-rust-deny` does the same at `Subject_Of_Path("")`. A `SubjectId` is a content digest
of a normalized path (`crates/kernel/nomos-model/src/path.rs`) with no containment relation, so
a cause naming an edited file's subject does not name a fact keyed at the crate root, and a
`SnapshotReplaced` carrying that file in `differing` does not either. Editing a file inside the
crate leaves the crate-wide fact reading as current.

One further measurement bounds how alarming that is today and must not be mistaken for a
defence of it: every construction of `GenerationCause` in this workspace is under `#[cfg(test)]`
or in a `tests/` directory. Nothing in the product drives invalidation at all yet, so the leaf
route's gap is latent rather than observed.

### What a rule in the Rules zone may read

`nomos-architecture.json` places `nomos-rules` in `Rules`, and `Rules` permits exactly
`Protocol`, `Substrate` and `Capability Contract` — never `Provider`, which is where every
`nomos-lang-*` crate sits, `nomos-lang-rust-compiler` included. `nomos-rules`'s own manifest
holds that line by construction and says so per dependency: "nomos-cap-dependency and not
nomos-lang-rust-cargo … this crate names a capability contract and lets the registry choose who
answers it, never the provider that does the answering." Its reading module reaches
`nomos_analysis::{FactReader, InputDigest, MaterializedFact}`, `nomos_capability::Requirement`
and the `nomos_cap_*` payload types, and nothing else.

`OD-ANALYSIS-007`'s amendment already derived the consequence for a contract housed beside its
provider: "the moment a descriptor in `nomos-rules` names either capability, a second party is
naming the contract from a zone that cannot see the file it lives in."

## The Decision

**One capability is decided. It is `nomos.cap.reference.edges`: the resolved references among
one crate's own declarations.** The name deliberately parallels `nomos.cap.dependency.edges`,
because the two answer the same shape of question at two grains — that one between packages, as
Cargo resolves them; this one between declarations, as a resolved semantic model binds them —
and a consumer that needs both is joining edge sets rather than reconciling two vocabularies.

### 1. The grain is declaration to declaration, and the endpoints are the pair the syntax fact already produces

An edge names a source declaration, a target declaration and the kind of reference. Each
endpoint is the pair `nomos.cap.syntax.items` already writes: the `SubjectId` of the file the
declaration is written in, and its name qualified by nesting in the shape
`PayloadItem::qualified_name` carries.

Declaration grain rather than module, type or function grain, for a reason that is a property
of the lattice and not a preference. Module-to-module, type-to-type and function-to-function
reachability are each a projection of declaration-grain edges onto one or other endpoint's
owner, and every projection is computable from the edges; no edge set at a coarser grain
recovers a finer one. Choosing module grain would decide, inside the fact, which of
`OD-RULES-024`'s three readings architecture drift is allowed to make.

The edge kind is domain-local vocabulary and belongs in the payload, which `OD-ANALYSIS-004`
permits explicitly — "its own per-item record shape … is free to be domain-local, exactly as
`SyntaxPayload` is". This record does not enumerate the kinds; a provider states the kinds it
distinguishes, and a kind it cannot distinguish is reported as an unresolved reference rather
than collapsed into a neighbouring one.

**The scope is one crate, and the cross-crate half is already answered by another fact.**
`Load_Crate` already restricts to files under the analyzed root, for the reason its own doc
gives — the filter "is what makes this reader answer for the one crate it was asked about
rather than for the standard library it had to load to answer honestly". A reference leaving
the crate is reported as leaving it, naming the package it reaches; which packages may reach
which is `nomos.cap.dependency.edges`' answer and this capability does not restate it.

### 2. One capability, not several

Calls, type references and module reaches are one traversal of one resolved model, and a
consumer computing reachability needs all of them or its closure is wrong — a module reaches
another through a type reference exactly as much as through a call. Splitting them would mint
three capabilities over one pass, three schemas, and three chances for a consumer to take two
of the three and report a closure as complete.

`OD-CAPABILITY-002`'s criterion is unaffected and is not being reinterpreted: it is per
contended capability, and it governs *where the contract lives*, which clause 6 below decides on
a different ground.

### 3. What a provider must have resolved at each level it offers

The ceiling is `FactVariant::SemanticallyResolved`, `EvidenceClass` is the producer's to state,
and no new epistemic type is introduced — `OD-ANALYSIS-004`'s rule applies unchanged, and this
family answers its five questions with `FactVariant`, `Guarantee`, `EvidenceClass`,
`Applicability` and `Observation`.

- **`Syntactic`.** An edge read from the text: a name written at a reference site matched
  against declarations the same file declares. Honest only for an unambiguous same-file
  reference. Every other reference site is reported as an unresolved reference, never omitted
  and never pointed at a best guess, and the subject is reported
  `Applicability::PartiallySupported`.
- **`SemanticallyResolved`.** `OD-ANALYSIS-004`'s obligation applies verbatim: the producer must
  have "resolved every name occurrence in its subject to the declaration it actually binds —
  including across module and crate boundaries — and assigned every typed expression its checked
  or inferred type, not the syntactic annotation where the two differ". For an edge that means
  the target endpoint is the declaration the reference actually binds, through imports, aliases,
  generic substitution and trait implementation selection wherever the language decides it
  statically. A site the language does not decide statically — dynamic dispatch through a trait
  object, a call through a stored closure, a macro-generated reference the producer could not
  expand — is reported as an unresolved reference with its source endpoint named, and the
  subject as `Applicability::PartiallySupported`. Resolving a dynamic site to the trait
  declaration and filing that as an edge is the overclaim this clause exists to forbid, because
  a consumer computing reachability cannot tell a real edge from that one once both are in the
  payload.
- **`Approximate` and `Predicted`.** No obligation is stated, because no shape of approximate
  graph evidence has been named by any consumer. A producer offering one owes this record an
  amendment before it does, and a heuristic name match offered as an edge is the false coverage
  `OD-RULES-024` declined.
- **`RuntimeObserved` is not a stronger offer against this capability; it is a different one.**
  Which target a dispatch actually took at run time and which declaration a reference statically
  binds are different questions, and `nomos-cap-syntax`'s own ceiling already settled that shape
  of case: "A compiler-backed provider that resolves names is answering a different question and
  belongs behind a different contract … letting it offer this one at
  [`FactVariant::SemanticallyResolved`] would mean two providers of one capability disagreeing
  about what the capability means." The ceiling therefore sits at `SemanticallyResolved`, and an
  observed-execution edge set is a capability this record does not decide.

The per-site unresolved case uses `Observation`'s three-state and not an `Option`, for the
reason `OD-SYNTAX-002` gives and `OD-ANALYSIS-004` repeats: a producer that could not look and a
producer that looked and found nothing are not the same answer. `Observation` still lives in
`nomos-cap-syntax`. A graph capability needing the identical shape is the second capability
`OD-ANALYSIS-004` named as the trigger for considering its promotion; whether the promotion is
admissible is `OD-CONTRACTS-001`'s band-0 criterion to judge, and this record routes it there
rather than deciding it.

### 4. Incremental invalidation: `Project` granularity, and a declared edge per file read

The capability declares `IncrementalGranularity::Project`, for the reason `OD-ANALYSIS-010`
already stated about resolution that crosses files — "a change to a helper function in a
*different* file can change whether a site in *this* file is sound" — and the reason both
compiler-backed contracts already give for themselves.

That declaration is necessary and it is not what makes invalidation work, because granularity is
reported and never applied. **A reference-graph fact is materialized on the derived route, not
the leaf route.** Concretely: it is keyed at the crate's own subject, and it declares one
dependency edge per file its provider read, on the per-file fact that file already has —
`nomos.cap.syntax.items`, which a real run materializes per file anyway. A file edit then
invalidates that file's syntax fact and reaches the graph fact through the edge, which is the
mechanism `nomos.cap.module.index` already runs on and the mechanism that leaves
`nomos.cap.rust.copy_clones` unreached today.

Two consequences follow and are stated because an implementer would otherwise have to guess
them.

**An edge does not assert that the payload's bytes were consumed.** It asserts that the answer
is a function of that subject. The rollup already reads edges that way — "a member with no fact
is still an edge" — and a graph fact's answer is a function of every file's text whether the
resolved model was reached through `ra_ap_hir` or through those payloads. Declaring the edge
is therefore honest, and omitting it because the provider did not read the payload is the
divergence the rollup's doc warns about.

**The recomputation cost is a whole crate and is reported, not amortized.** There is no partial
refresh of a resolved model here, and the `Broadening` the store records on a file-granular
cause is the statement of that cost, in the rollup's own phrase "stated rather than absorbed". A
consumer that cannot afford a whole-crate recomputation is entitled to see that on the report
and decide; it is not entitled to a declared `File` granularity that would be a precision the
resolution does not have.

### 5. Which deferred item each candidate fact unblocks, and which it does not

Four candidate facts fall out of the five consumers, and this record decides one of them. The
mapping is stated per item because the items differ.

| Deferred item | The fact it consumes | What `nomos.cap.reference.edges` does for it |
|---|---|---|
| Architecture discovery and inference | a resolved reference graph | **Unblocks it.** Module-to-module and type-to-type reachability are projections of these edges, judged against `nomos.cap.architecture.declaration`, which is the pairing `OD-RULES-024` named and found half-missing. |
| Placement analysis | the same graph, plus a cohesion or coupling measure | **Necessary and not sufficient.** The metric half is a separate fact this record does not decide, and `OD-ROADMAP-004` holds it deferred on its own ground. |
| Test intelligence | a test-to-code coverage correspondence | **Does not unblock it.** These edges give a static over-approximation — what a test *can* reach — which is not what a test covered. The coverage fact is undecided. |
| Feature topology and path tracing | a feature-to-code correspondence | **Does not unblock it.** The feature endpoint has no referent in any artifact this workspace holds, so no observed fact can carry it until something declares what a feature is. |
| Runtime and debug intelligence | an execution observation | **Does not unblock it**, and it is not in this family: an observed edge is a different capability by clause 3, and `OD-ROADMAP-004` holds the `ActiveRuntimeGuard` half withheld by the corpus regardless. |
| Atlas | the content of the five above | **Gains one of five.** `ARC-ROADMAP-001` version 4 sets its condition as at least one first-kind item producing a fact worth showing *and* a fact surviving the process that made it; only the first half is touched here. |

So one deferred item is unblocked as far as a decision can unblock it, one is half-unblocked,
and three are not. A record claiming otherwise would be the shape `OD-COMPLETENESS-001` and
`OD-RULES-023` both declined.

### 6. What a rule in the Rules zone may read, which `OD-RULES-024` surfaced and left open

A rule may read this fact, and the two conditions that make that true are decided here rather
than left to whoever builds it.

**The contract and payload live in a crate in the `Capability Contract` zone**, named for the
capability the way the policy contracts already are, and not beside its provider — even though
`OD-CAPABILITY-002`'s contention trigger has not fired. The ground is the zone rule, not
contention: `Permits` gives `Rules` only `Protocol`, `Substrate` and `Capability Contract`, and
a descriptor in `nomos-rules` naming a contract housed in a `Provider` crate is a zone crossing
regardless of how many parties have named it. This is `OD-ANALYSIS-007`'s amendment applied to a
capability that does not exist yet instead of to two that do.

**No provider-specific type crosses the payload.** An endpoint is a `SubjectId` and a qualified
name string — vocabulary `Protocol` and `Capability Contract` already hold — never an
`ra_ap_hir` handle, a `DefId`, or any opaque identifier only the producing provider can
interpret. An opaque endpoint would be readable only by something that could name the provider,
which is the boundary `Permits` exists to forbid, and it would also make two providers of this
capability unable to answer the same question comparably.

**This settles the constraint for a graph fact and not for representation leakage.**
`OD-RULES-024`'s open question was what fact carries "provider-specific type" without the
checking rule having to see the Provider zone. That is a different fact about a different
subject, and its clean baseline of zero stands unre-measured here.

## What Is Refused, And On What Trigger

Three facts are named and not decided, each with the condition that would bring it back. None
is declined as unwanted.

- **A feature-to-code correspondence.** Refused because the feature endpoint has no referent.
  The trigger is a declaration: something in a repository has to say what its features are, in
  the shape `nomos.cap.architecture.declaration` already has for components, before an observed
  fact can correspond to one. A capability decided before that would be inventing the
  repository's feature vocabulary on its behalf, which is the half `OD-RULES-029` moved out of
  the rules and `OD-RULES-003` turns on.
- **A cohesion or coupling measure.** Refused because `OD-ROADMAP-004` holds the metric family
  on its own condition and this record does not reach it. The trigger is that record's own.
- **A test-to-code coverage correspondence.** Refused because it is a different fact from the
  one decided here and its honest form is an observation of a run rather than a resolution of a
  reference. The trigger is a rule whose subject needs coverage and cannot be answered by the
  static over-approximation clause 5 names, which would also say which of the two it needs.

## What This Record Does Not Do

**It schedules no crate and builds nothing.** No capability crate, no contract, no provider, no
payload type, no `RequiredFact` variant and no composition entry is created or scheduled here.
Naming the zone a future contract belongs in is a constraint on whoever builds it, not an item.

**It does not lift any deferral.** `ARC-ROADMAP-001` version 4 sets the first-kind condition as
the fact being "decided and provided". This record decides one fact and provides nothing, so
architecture discovery and inference is half-conditioned rather than available. Amending that
record is not this record's territory and is a following item's.

**It does not amend `OD-RULES-024`.** That record's measurement stands, including its
representation-leakage baseline, and the number it gave for declared adjacency is updated here
as a measurement at a later revision rather than corrected as an error.

**It does not decide the mechanism's second provider, or any language but Rust.** The capability
is language-neutral by construction — its endpoints are a file subject and a qualified name —
and which engine answers it for which language is a provider's offer, judged by
`ARC-CONFORMANCE-001`'s test each time, not settled here.

**It does not fix the leaf route's invalidation gap.** That a crate-wide leaf fact is unreached
by an edit inside its crate is measured above and left where it was found; deciding whether the
existing leaf facts move to the derived route is an item with its own territory, and nothing in
the product drives invalidation today.

**It does not add a `FactVariant` level, reorder the five, or introduce an epistemic type.**
`OD-ANALYSIS-004`'s vocabulary rule is applied, not extended.

## Status

Accepted, drawn by `P125-THE-FACT-FIVE-DEFERRED-ITEMS-WAIT-ON`. The fact five deferred items
were reported to share is decided as one capability at declaration grain, with its levels, its
invalidation route and its zone constraint fixed; three sibling facts are refused with named
triggers, and the per-item mapping records that the one decision moves one deferred item
outright, half of a second, and none of the remaining three. What would revisit it: a provider
offering `Approximate` or `Predicted` graph evidence, which clause 3 requires an amendment for;
a second capability needing `Observation`, which `OD-CONTRACTS-001` judges; and any of the three
named triggers firing for the refused facts.
