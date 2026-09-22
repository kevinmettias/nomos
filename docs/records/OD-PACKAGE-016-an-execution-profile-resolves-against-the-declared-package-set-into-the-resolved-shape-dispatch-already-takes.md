---
id: OD-PACKAGE-016
type: decision
title: An execution profile resolves against the declared package set into the resolved shape dispatch already takes
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - package
  - model
  - routing
  - resolution
relations:
  - target: OD-PACKAGE-012
    type: relates-to
  - target: OD-PACKAGE-011
    type: relates-to
  - target: OD-PACKAGE-010
    type: relates-to
  - target: OD-PACKAGE-013
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-EXECUTOR-008
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
  - target: OD-ROADMAP-005
    type: relates-to
---

# An execution profile resolves against the declared package set into the resolved shape dispatch already takes

## Question

`OD-PACKAGE-011` licensed twenty-five `MODEL-ROUTE` identifiers and `nomos-model-package`
carries ten maturities of vocabulary for them. `README.md` states the outcome plainly: none of
it is wired into a real consumer yet. An item asking for that consumer — a dispatch that
selects its backend by resolving a declared execution profile rather than by a hard-coded
choice — was **declined**, and the decline was a measurement rather than a preference. Its
territory was one file, and reaching its own `done_when` needs a resolver, a set to resolve
against, a call site that stops naming a backend, and four reassessments. None of those is
reachable from the one file it reserved.

That item named three questions it surfaced and did not answer. No successor with reaching
territory can be authored until they are decided, because a worker handed the wrong answer
would either build `ResolvedModelExecution` past the condition its own deferral rests on, or
invent a second resolution mechanism beside the capability registry. They are:

1. What does a resolver read, given `ResolvedModelExecution` stays deliberately unbuilt?
2. Is the resolver one item and the dispatch wiring another, or is it one?
3. `OD-PACKAGE-012`'s binding question, owed for this case and not yet paid: does a model
   executor's selection genuinely fit the existing capability/provider/evidence shape, or does
   it need its own?

## What Was Measured

Taken from the crates named, at this revision.

### The two ends already exist, and they are already the same shape

`ModelExecutionProfile` carries `selector: ModelSelector` and `effort: EffortLevel`.
`DispatchConfig` carries `effort: EffortLevel` and `backend: Backend`. `Backend`'s own doc
comment already says what it is:

> this type is only the resolved choice both hosts now share, not a second, host-local copy of
> it.

Nothing maps the declared shape onto the resolved one. That missing map is the whole of what
this record decides.

### Where a profile is declared today, and where a backend is chosen today

Of `MODEL-ROUTE-001`'s five referencing surfaces, exactly one is a real typed field:
`GateCommand.model: Option<ModelExecutionProfile>`. A search of that crate for the name `model`
returns its declaration and no other use — the field is **never read**. And `nomos-cli` sets it
to `None` with a comment stating that no flag authors one.

The choice that *is* made is a string match. `Backend_Body_From_String_Arguments` in
`nomos-cli`'s workflow parser turns `--executor claude-code` and `--model-backend ollama`
directly into `Body::ClaudeCode` and `Body::Ollama`. That is a caller naming a backend
directly, literally, in a parser — which is the hard-coded choice the consumer item was written
to remove.

### Nothing declares a catalog

`ModelSelector`'s five variants all carry raw strings, and `model_selector.rs` states why: there
is no live provider, catalog, or entitlement system anywhere in this workspace yet to resolve
an identity, a family name or a predicate against. That is still true, and it is stronger than
the doc comment says. `Read_Manifest` and `Parse_Manifest` can read a model backend package
manifest, and **no manifest file exists anywhere in this tree to read.** `ModelSelection` is
constructed nowhere outside `nomos-model-package`'s own unit tests.

So a resolver cannot read a catalog from disk today, because nothing has written one. That is a
fact a successor item has to be authored around rather than discover halfway through.

### What each selector variant needs, against what this build has

| `ModelSelector` variant | What resolving it needs | What this build has |
|---|---|---|
| `BackendFamily(String)` | a name for the family a candidate backend belongs to | two real backends, and no name for either |
| `ExactIdentity { identity, pinned }` | a candidate's `ModelSelection::Catalog` | no package declares one |
| `AllowedSet` | the same | the same |
| `PolicyRankedCandidates` | the same, plus a policy object | the same, and no policy object |
| `CapabilityPredicate` | a typed predicate vocabulary | none; the variant holds the raw expression its author wrote |

`BackendFamily`'s own doc says "Any model belonging to a named backend family, such as
`acme-family`" — so the variant names a family, and nothing in the workspace names a backend's
family. `CapabilityPredicate`'s says the vocabulary has not been "typed further than
the raw expression an author wrote".

### The capability shape, read rather than recalled

`Requirement` carries `capability: CapabilityId`, `version: ContractVersion`,
`minimum: Guarantee` and `preferred: Option<ProviderId>`. `ProviderOffer` carries
`provider: ProviderId`, `capability: CapabilityId`, `version: ContractVersion` and
`guarantee: Guarantee`. `Resolution` is `Satisfied { selection, applicability }` or
`Unsatisfied { capability, reason }`, and its own doc says why it is not an `Option`:

> Two results, never a `bool` and never an `Option`. An `Option::None` here would be a caller's
> invitation to write `unwrap_or_default`, and there is no defensible default for "can this
> analysis be performed".

`Unmet` has exactly four variants — `Undeclared`, `NoProvider`, `VersionMismatch { offered }`,
`BelowRequirement { closest }` — and `Remedy` pairs each with the act that closes it:
`DeclareTheContract`, `RegisterAProvider`, `AgreeOnAVersion`, `StrengthenTheClosestOffer`.
`Remedy`'s doc calls itself "the only half of the pair a caller can act on".

`Guarantee` is built on `FactVariant`, whose five variants are the resolution level a *fact* was
established at: `Predicted` ("Modelled rather than measured"), `Approximate` ("Established by a
method that trades accuracy for cost"), `Syntactic` ("Read from the text or its parse tree, with
no name resolution"), `SemanticallyResolved`, `RuntimeObserved`. `Assurance` is
`Sound`/`Unsound`/`Unknown`, and `Satisfies_Requirement` admits nothing but `Sound`.

`PackageId` and `ProviderId` are both `Named_Identity!` newtypes over `String` — names, not
digests. `ModelRoutePackage` carries `package_id: PackageId`, `package_kind`,
`package_version`, `protocol_range` and `model_selection`. It has no family field.

## Decision

### 1. The resolver lives in `nomos-agent-orchestration`

It must name `Backend` and produce `DispatchConfig`, both of which live there, and it must reach
both backend crates. `nomos-model-package` is in the Provider zone and a provider may not name
an agent, so the crate holding the declared vocabulary cannot be the crate holding the resolved
shape.

`nomos-agent-orchestration` is the only crate that already depends on `nomos-model-package` and
on both `nomos-agent-executor-claude-code` and `nomos-model-backend-ollama`, and it already owns
the type whose own doc calls itself the resolved choice. `nomos-gate-orchestration` reaches the
vocabulary but reaches neither backend. `nomos-workflow-orchestration` is a consumer of the
dispatch seam rather than a place the seam's own choice is made.

### 2. It reads the declared package set, supplied as a value

The resolver resolves against a caller-supplied sequence pairing a dispatch target with the
manifest that declares its model selection. It does not discover packages, read disk, or invent
availability.

The division is not new. `nomos_capability::Registry` is populated by a composition root —
`registry.Offer(nomos_lang_rust::Provider_Offer())` — rather than discovering its own providers,
and `Selection::Over(usable, preferred)` takes the usable set as a value rather than assembling
it. Reading files is a composition root's concern (`OD-HOST-002`), and a resolver that walked
the tree would be a second place that decides which packages exist.

**Why supplied rather than read:** because no manifest exists to read, per the measurement
above. The reader stays where it is. Writing the file and pointing a host at it is a later item
with its own territory, and deciding it here would be deciding for a file that does not exist.

### 3. `Backend` gains a family name, and that is the only addition to existing vocabulary

`BackendFamily` names a family, and for a family name to select anything, a backend has to have
one. `Backend` gains a `Label()` in the form every enum in this workspace already has —
`EffortLevel::Label`, `FactVariant::Label` — returning the two strings the CLI already accepts
as `--executor` and `--model-backend` values.

This is an addition, and this record names it as one rather than letting it arrive as incidental
detail. It is the smallest addition that makes a selector resolvable at all, it names one of two
existing variants rather than introducing a concept, and it touches neither `ModelSelector` nor
`ModelSelection` nor the manifest reader.

`BackendFamily` is the one selector that resolves today, and that is what makes a successor
item's "at least one routing constraint changes which backend is chosen" satisfiable honestly
rather than by construction.

### 4. The other four selectors are unresolved, by named reason

The resolver does not evaluate `CapabilityPredicate`, does not rank `PolicyRankedCandidates`,
and does not manufacture a catalog to make `ExactIdentity` or `AllowedSet` resolve. Each comes
back unresolved, and the reason is the absence the measurement names — no available package
declares a model catalog, no policy object exists, no typed predicate vocabulary exists — rather
than free text.

This is not a stub. A resolver that answered `ExactIdentity { identity: "llama3" }` by
string-matching a name it invented would be reporting a routing decision this build cannot make.
That is the same defect `OD-EXECUTOR-008` refused in the other direction: an executor must not
synthesize the types whose concrete evidence it lacks.

### 5. Two-valued, with a closed diagnosis and a remedy, and no invented ranking

`nomos_capability::Resolution` is the shape and the reason: two results, never a `bool` and never
an `Option`, and on the unsatisfied side a closed diagnosis paired with the act that closes it,
kept separate because a message that merges them is neither useful nor accurate.

So a resolution is either a `DispatchConfig` or an unresolved result naming the selector and why
nothing answered it, where the why is one of the measured absences above.

Where more than one available backend satisfies a selector, the result reports the chosen one
and carries the others beside it, for the reason `Selection` carries `alternatives`: "the answer
to 'who answers' is not complete without 'instead of whom'". The order among them is the
caller's declared order, and the resolver does not rank. `Selection` draws exactly this line
about its own tiebreak — "the order the registry holds its offers in, which is by provider name,
and that is a deterministic tiebreak rather than a judgement" — and its module doc states the
refusal in full:

> Where two offers are equivalent or incomparable this module does not invent a ranking; it
> reports that it did not decide.

### 6. Effort is carried, never mapped

`profile.effort` becomes `config.effort` unchanged. Mapping a canonical level onto a specific
backend's native control is the backend's own job — `MODEL-ROUTE-004` states it as one, and
`EffortLevel`'s own doc says the mapping is "a mapping and a non-equivalence rule for a real
backend to carry out, not a fact this enum could state about itself" — and it is precisely the
category `nomos_capability` already refused to hold:

> Adding a cost axis to `Guarantee` would put a scheduling concern in the crate every non-Rust
> peer reimplements, to answer a question the caller was already answering.

A resolver that mapped effort would be deciding, on a backend's behalf, the one thing only that
backend can state.

### 7. `ResolvedModelExecution` stays unbuilt, and the resolver's output is `DispatchConfig`

`MODEL-ROUTE-029`'s `ModelInputAssemblyIdentity` established that `ResolvedModelExecution` stays
unbuilt until every requirement naming one of its fields is accounted for, and `MODEL-ROUTE-005`
records that this is deliberate rather than an oversight. That condition is not met. Building it
now so that the resolver has a richer output would reverse a deferral without the accounting it
was conditioned on — exactly the failure the decliner warned a successor would walk into.

`DispatchConfig` is the resolved execution shape this workspace already has and already
dispatches through. The resolver consumes the declared vocabulary and produces the resolved one.
It does not mint a third.

### 8. `OD-PACKAGE-012`'s question, answered: it does not fit, on four measured counts

- **The guarantee axis.** `ProviderOffer` requires a `Guarantee`, whose first component is a
  `FactVariant` — the level a *fact* was established at, over five levels that are all about
  facts. A model backend answers a prompt; it does not establish a fact, and there is no honest
  `FactVariant` for prose. `Assurance::Sound` is a completeness claim a provider earns by
  bounding its own gap: `nomos-lang-go` holds `Sound` on both axes because Go has no macro
  system, so there is no region where its parse tree ends and an unexpanded token stream begins,
  which is the same gap that keeps `nomos-lang-rust`'s completeness at `Unknown`. A prose
  backend has no way to bound that gap, so a `Sound` it declared would be a guarantee nobody can
  back — the conforming lie `OD-EXECUTOR-008` exists to refuse.
- **The selector slot.** `Requirement` carries exactly one `preferred: Option<ProviderId>`.
  `ModelSelector` has five ways to name what it wants. Four name something that is not a
  provider implementation at all — a model identity, a set of identities, a family, a ranked
  candidate set — and the fifth is the untyped predicate `Requirement` has no field to hold.
- **The remedy vocabulary.** `Unmet`'s four variants and `Remedy`'s four acts are about
  capability contracts and provider offers: author a contract, register an offer, agree on a
  version, strengthen the closest offer. The absences measured here are different absences whose
  remedies are different acts, so reusing the pair would force them into a diagnosis naming the
  wrong thing.
- **The effort axis**, per decision 6.

`OD-CAPABILITY-002` gives the same answer from the other side: shared machinery earns its place
when a second real party contends for it, and a model backend contends to answer a question, not
to serve an analysis capability with evidence about a subject.

**This is the check `OD-PACKAGE-012` owed, made and answered.** That record's own words: does a
model or agent executor's selection genuinely fit the existing capability/provider/evidence
shape, or does it need its own? It needs its own, on the measurement above rather than by
resemblance. That does not weaken the record. `OD-PACKAGE-012` governs the routing and
conformance system `MODEL-ROUTE-038` through `049` elaborate; this record decides the first
resolution step of it and authorizes no mechanism for the parts that record still governs.

### 9. The wiring is a second item, not this one

Decision 3's table is the whole of what a resolver can honestly do today. Making a *dispatch*
select its backend this way needs a workflow step to carry a profile and the dispatch seam to
stop taking a backend as a given — `nomos-workflow-orchestration` and `nomos-cli` territory,
neither of which this record's own item reserves. The decliner was right that a resolver alone
does not satisfy "rather than by a hard-coded choice". The answer is a second item that depends
on the first, not one item whose territory quietly widens to reach both.

## What This Does And Does Not Invalidate

It does not change `ModelSelection`'s three variants, the manifest reader, or any part of the
manifest maturity `OD-PACKAGE-010` drew. It does not claim any package declares a model catalog;
today none does, and every selector needing one is unresolved by name.

It does not make `GateCommand.model` read. That field is a declared surface with no reader
today, and this record leaves it as it found it rather than implying the resolver closed it.

It does not decide anything about `MODEL-ROUTE-038` through `049` beyond the first resolution
step, and it does not reopen `OD-PACKAGE-011`'s licensing or `OD-ROADMAP-001`.

## Alternatives Considered

**Reading packages from manifest files.** Rejected for this step. No manifest file exists to
read, and a resolver that reads disk is a composition root's concern (`OD-HOST-002`). The reader
stays; the file and the host that points at it are a later item.

**Building `ResolvedModelExecution` first, so the resolver has a richer output.** Rejected per
decision 7: it reverses a deferral without the accounting that deferral was conditioned on.

**Routing through `nomos_capability::Registry`.** Rejected on the four measured counts in
decision 8. This is the alternative `OD-PACKAGE-012` required be considered, and it was
considered against the real types rather than recalled from their names.

**Putting the resolver in `nomos-model-package`.** Rejected in decision 1: Provider may not name
Agent, and the resolved shape is Agent's.

**Resolving `CapabilityPredicate` by evaluating the expression.** Rejected: the workspace has no
typed predicate vocabulary, and evaluating an untyped expression is the guessed predicate
`GateUnknown` exists to refuse.

**Adding a `ModelSelector` variant for "the backend the caller named".** Rejected: it would make
the resolver a pass-through and re-introduce the hard-coded choice one layer in, which is the
defect the consumer item was written to remove.

**Letting the resolver pick among equals by some internal order.** Rejected in decision 5: an
invented ranking is a decision the resolver is not entitled to make, and
`nomos_capability::Selection` already refused the same move for the same reason.

## Amendment: The Resolver Stays, And Its Reason Does Not

Added at version 2, authorized by `OD-ROADMAP-005` decision 2 and built by
`P126-A-PORT-STANDS-BETWEEN-THE-GENERIC-AGENT-PATH-AND-ITS-TWO-BACKENDS`.

**What is superseded.** Decision 1 placed the resolver in `nomos-agent-orchestration` on a
reason this amendment removes: that it "is the only crate that already depends on
`nomos-model-package` and on both `nomos-agent-executor-claude-code` and
`nomos-model-backend-ollama`". That is no longer true of it, and is no longer a reason for
anything. Decision 3, "`Backend` gains a family name, and that is the only addition to existing
vocabulary", is superseded with it: the `Backend` enum and its `Label` are gone, because the
enum was the generic path holding a list of vendors. Decision 5's "carries the others beside
it" is superseded only in its element type, from that enum to the family labels the
declarations state.

**The resolver did not move, and decision 1's conclusion stands on its other leg.** It must
still name the resolved shape a dispatch takes and produce it, and `nomos-model-package` is in
the Provider zone where a provider may not name an agent. So it stays exactly where it was,
and what changed is what it resolves *to*: a declared target now carries its own family label,
its own package declaration, and the port that answers it, and a resolution hands back what
was already there rather than selecting a variant this crate had enumerated.

**What was measured, at `9f13b1e7`.** `Declared_Targets` in `nomos-agent-orchestration`
derived the declared set from `Backend::ALL`, so the crate that resolved also decided which
backends exist. `src/run.rs`'s `Dispatched_Task` matched that enum onto each adapter's own
`Execute_Task`. Both manifests in `crates/orchestration` declared both adapter crates, and the
workflow crate's two declarations were already dead: no line of Rust there named either.

**What was built.** `nomos_agent_contracts::DeclaredTarget` holds `family`, `package` and a
`DispatchPort`, which is `Executor` or `Model` -- the two `PackageKind`s that declare a model
selection, not two vendors. Each adapter states its own declaration and its own `FAMILY`
constant, the shape `nomos_capability::Registry` already uses when a composition root calls
`registry.Offer(nomos_lang_rust::Provider_Offer())`, so two hosts offering the same pair are
two calls rather than two copies of a package declaration that could drift. `Selected_Dispatch`
and `Resolve_Profile` are unchanged in what they decide.

**What this amendment leaves exactly as this record decided it.** Decision 2: the resolver
resolves against a caller-supplied sequence and discovers nothing, reads no file, and invents
no availability. Decision 4: the other four selectors come back unresolved by named reason, and
the match in `Resolve_Profile` still carries no wildcard. Decision 6: effort is carried, never
mapped. Decision 7: `ResolvedModelExecution` stays unbuilt and the output is the resolved
dispatch shape. Decision 8: `OD-PACKAGE-012`'s binding question is answered as this record
answered it, on the four measured counts. Decision 9's wiring landed before this item and is
untouched by it.

**One consequence worth naming, because it reaches a published shape.** A dispatch outcome now
carries the family that answered. Before the port the variant *was* the vendor, so a caller
always knew which backend answered; once a profile can resolve a target nobody typed, that is
knowable only from the declaration that resolved. `nomos-api`'s own response shape carries it
as a field rather than as a variant tag for the same reason.

## Status

Accepted. Decides the three questions the declined consumer item surfaced and did not answer,
and pays `OD-PACKAGE-012`'s binding question with the measurement taken against the real types.
Authorizes one resolver and one addition to existing vocabulary; files the dispatch wiring as a
dependent item rather than widening its own territory.

Amended to version 2 by `P126-A-PORT-STANDS-BETWEEN-THE-GENERIC-AGENT-PATH-AND-ITS-TWO-BACKENDS`
under `OD-ROADMAP-005` decision 2: decision 1's reason and decision 3's addition are
superseded, the resolver stays where it is on decision 1's other leg, and every other decision
here stands as written.
