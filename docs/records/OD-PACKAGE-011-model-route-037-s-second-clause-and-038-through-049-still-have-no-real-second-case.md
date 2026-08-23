---
id: OD-PACKAGE-011
type: decision
title: MODEL-ROUTE-037's second clause and MODEL-ROUTE-038 through 049 still have no real second case
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - package
  - model
  - agent
  - roadmap
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-PACKAGE-010
    type: relates-to
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-CORRECTIONS-001
    type: relates-to
---

# MODEL-ROUTE-037's second clause and MODEL-ROUTE-038 through 049 still have no real second case

## Question

`OD-PACKAGE-010` built `nomos-model-package`'s first manifest maturity and named exactly
what it deferred: `MODEL-ROUTE-037`'s second clause (catalog entries' real nine-state
shape) and `MODEL-ROUTE-038` through `049` (resolution invalidation, a normative
`RoutingPolicyConformanceSuite`, conformance-result preservation, publication/canary
gating), each waiting for "a real case to check its shape against" rather than an invented
one. The user's standing override to build `ModelBackend`/`AgentExecutor` infrastructure
now, not wait for population, does not by itself license inventing that shape -- the same
boundary `OD-PACKAGE-010` itself drew between authorizing *something* now and authorizing
all thirteen requirements at once. This record checks whether a real second case has
appeared in the live workspace since `OD-PACKAGE-010` landed, rather than assuming the
answer is still no.

## What Was Measured

Grepped directly across the whole workspace for every real name this deferred maturity
would need a caller of: `ModelBackendPackage`, `AgentExecutorPackage`, `ModelSelection`,
`nomos_model_package`. Outside `crates/packages/nomos-model-package` itself, the only
matches are declarations, not consumers: the `PackageKind` enum in
`crates/contracts/nomos-contracts/src/package.rs`, the crate's band-26 registration in
`Cargo.toml`, `README.md`, `tests/contract/tests/boundaries/bands.rs` and
`crates/rules/nomos-rules/src/dependency/bands.rs`, and this record's own siblings under
`docs/records/`. Nothing reads a `ModelRoutePackage`, resolves a `Catalog` entry, or
routes a request to a model or executor anywhere in this build.

Grepped separately for the domain vocabulary `MODEL-ROUTE-037`'s nine states and
`038`-`049`'s routing/conformance system would need a real instance of to check a shape
against: `entitlement`, `deprecat*`, `mutable alias`, `discovered identity`, `configured
identity`. The only hits are two unrelated uses of the English word "entitlement" in
`nomos-ledger` (an ownership check on a ledger claim, nothing to do with model access).
Grepped for any real model-provider integration -- `openai`, `anthropic api`, `model
catalog`, `list_models`, `provider registry` -- and found only the declaration comments in
`nomos-model-package` itself. No live catalog fetch, no provider client, no entitlement
system, no telemetry pipeline, no canary machinery exists anywhere in this workspace.

Read `crates/packages/nomos-lang-package/src/language_version.rs` -- `RustEdition`, the
one real precedent `OD-PACKAGE-010` cited for when resolving a raw `Vec<String>` domain
into a typed shape is licensed rather than invented. `RustEdition` is licensed by a real,
closed, externally verifiable fact: Rust has shipped exactly four editions, spelled
exactly the way `Cargo.toml`'s own `edition` field spells them, and nothing in this
workspace's own code needed to invent that enumeration -- it transcribed it. `MODEL-
ROUTE-037`'s nine catalog-entry states (configured identity, discovered identity, exact
revision, mutable alias, deprecation, temporary unavailability, removal, entitlement
availability, executor-selected identity) name a design taxonomy for a routing and
entitlement system, not a closed fact any real provider, catalog, or entitlement service
in this workspace exposes today. Resolving `Catalog(Vec<String>)` into that nine-state
shape now would be inventing the taxonomy `RustEdition` never had to invent -- the same
gap this workspace's honesty vocabularies exist to name rather than paper over.

`MODEL-ROUTE-038` through `049`, read in full from `NOMOS_V14_CORPUS`, each presuppose
infrastructure this workspace has none of: `038` needs a real catalog and package
contract to resolve a configured reference against; `039` and `044` need a running engine
and a cache to invalidate; `040`-`042` need a provider or executor capable of supplying
continuity evidence; `043` needs `ResolvedModelExecution`/`ModelInputAssemblyIdentity`
records, which exist nowhere in this workspace's types; `045`-`047` need policy
implementations and routing telemetry adapters for a `RoutingPolicyConformanceSuite` to be
normative *for*; `048` needs a real resolution to diff; `049` needs package publication,
engine release qualification, and canary validation as real events, none of which this
workspace runs. None of it changed since `OD-PACKAGE-010` measured the same absence.

## The Finding

**No real second case exists yet for either deferred piece. Naming this precisely, rather
than leaving a future session to re-run the same grep and wonder whether something was
missed, is what this record does.**

This is not a decision to build nothing forever -- `OD-PACKAGE-010` already named both
pieces as later maturities, not permanently out of scope. It is confirmation that the
condition which would license the next increment (a real case to check a shape against,
the same population-of-zero caution `OD-PACKAGE-006`, `OD-PACKAGE-008` and `OD-
CORRECTIONS-001` all apply elsewhere in this workspace) has not yet been met, checked
against the workspace as it stands today rather than assumed unchanged from `OD-PACKAGE-
010`'s own measurement two commits ago.

### The `OD-CORRECTIONS-001` trigger does not fire here either

`OD-CORRECTIONS-001` names two conditions that would decide corrections' own next
increment; the second is "a real driver for agent- or interactive-class corrections
exists," naming `ModelBackend`/`AgentExecutor` infrastructure becoming real as the
condition that would supply one. `nomos-model-package` does not supply it. `OD-PACKAGE-
010` was explicit that it "does not give either package kind a real backend or executor
implementation; nothing in this workspace registers one" -- this crate is a manifest
*format* for declaring a `ModelBackendPackage` or `AgentExecutorPackage`, the same way
`nomos-lang-package` is a manifest format and not a compiler. Nothing in this workspace
can yet act as a model backend or an agent executor, so there is still no real candidate
to drive `COR-001`'s agent- or interactive-class correction categories. `OD-CORRECTIONS-
001`'s second trigger remains unfired; this is recorded here so the next session checking
either record does not have to rediscover it from scratch.

## What This Does Not Do

It does not touch `crates/packages/nomos-model-package` -- no code changes. It does not
resolve `MODEL-ROUTE-037`'s nine-state catalog-entry shape, invent a
`RoutingPolicyConformanceSuite`, or build any piece of `MODEL-ROUTE-038` through `049`. It
does not reopen `OD-PACKAGE-010`, which stands exactly as it left the boundary. It does
not build anything in `crates/corrections/nomos-corrections`, which `OD-CORRECTIONS-001`
already scoped precisely and which stays untouched here.

## What Would Decide The Next Increment

Unchanged from `OD-PACKAGE-010`, restated because it still has not arrived:

- **A real `ModelBackendPackage` or `AgentExecutorPackage` consumer appears** -- something
  in this workspace that actually resolves a `Catalog` entry against a real provider,
  catalog service, or entitlement system -- giving the nine-state shape a real case to
  check itself against, the same way `RustEdition` had `Cargo.toml`'s real edition grammar
  rather than an invented enumeration.
- **Real routing, entitlement, or conformance infrastructure exists** -- a live catalog, a
  cache to invalidate, a policy implementation, or a release/canary process this workspace
  actually runs -- giving `MODEL-ROUTE-038` through `049` something to build a first
  increment against instead of a corpus description alone.

Until either arrives, `nomos-model-package` stands exactly where `OD-PACKAGE-010` left
it -- a real, tested first manifest maturity with no second maturity to build yet -- rather
than an unmeasured "what's next" a future session has to re-derive.

## Status

Accepted. Confirms, by direct measurement against the live workspace rather than
assumption, that neither deferred piece `OD-PACKAGE-010` named has a real second case yet,
and that `OD-CORRECTIONS-001`'s agent/interactive-correction trigger has not fired.
Schedules no work of its own.
