---
id: OD-RULES-024
type: decision
title: Architecture drift and representation leakage both need a fact this workspace does not yet produce
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
relations:
  - target: OD-RULES-020
    type: relates-to
  - target: OD-RULES-023
    type: relates-to
  - target: OD-GATE-002
    type: relates-to
---

# Architecture drift and representation leakage both need a fact this workspace does not yet produce

## Question

A person required a second pair of declared-architecture rules beside write authority and
ownership: architecture drift — declared topology against the observed call, data and
dependency graph, judged for *shape* rather than the *direction* `OD-RULES-020`'s zones
already judge — and representation leakage — a provider-specific type reaching a consumer
promised only the capability contract. The item's own territory named one new file,
`crates/rules/nomos-rules/src/checks/architecture_drift.rs`, for both. Whether that is
buildable needed checking the same way `OD-RULES-023` checked write authority and ownership,
rather than assumed from the first rule's shape.

## What Was Measured

**Every capability this workspace has today was read, not assumed.** Ten capability
contracts exist under `crates/capabilities/`: dependency edges, controlflow reachability,
lint diagnostics, four policy families (dependency, naming, limits, scripting, goals,
words), and syntax items. `nomos.cap.dependency.edges` — the fact `Zone_Of`/`Permits`/
`SAME_ZONE_EDGES` and now `WRITE_DOORS` all read — is package-level: `DependencyPayload {
package, edges: Vec<DependencyEdge> }`, a Cargo dependency, nothing finer. `nomos.cap.
controlflow`'s own contract is narrower still: whether a control-flow path from a
fact-read `Err` reaches a `Finding` construction inside one function, not a cross-module or
cross-crate call graph. No capability here answers a call graph, a data-flow graph, or any
topology finer than the eleven zones `ZONES` already declares. `SAME_ZONE_EDGES` is the
finest declared adjacency this workspace has.

**Architecture drift has nothing to be judged against.** "Shape not direction" presupposes
an observed graph finer than package-level dependency edges — module reaches module, type
flows into type, function calls function — and this workspace materializes none of them.
The gap is the identical shape `OD-RULES-023` found for ownership: a real reading is asked
for, and the one fact this crate can reach today (`nomos.cap.dependency.edges`) cannot carry
it. Building a rule against it now would mean silently narrowing "call, data and dependency
graph" down to whatever the existing package-level fact happens to expose, which is not the
property that was asked for — the same false-coverage failure `OD-COMPLETENESS-001` and
`OD-RULES-023` both already declined to ship.

**Representation leakage is a real, checkable property — and this workspace is clean against
it today, which is not the same as ready to check it mechanically.** Every public signature
in `nomos-rules` and `nomos-check-orchestration`'s own surface snapshots was checked for a
provider-specific type — `DiscoveredPackage`, `MetadataError`, `FactContext`, or any
`nomos_lang_*` path — reaching a consumer promised only a capability contract. None does:
`RunContext`'s own public fields name only `nomos_workspace::Workspace`, `nomos_platform::
{FileSystem, ProcessLauncher}` and `nomos_analysis::MemoryFactStore`, all Substrate, never
Provider. The real case is clean the same way `nomos-store`'s write-authority case was true
before anything checked it.

But the mechanism to catch a future leak mechanically is not ready the way write authority's
was. `tests/contract/src/surface.rs` deliberately does not resolve where a bare type name in
a signature is defined — `OD-GATE-002` decided that on purpose, no rustdoc resolution — and
`nomos.cap.syntax.items`'s own `PayloadItem` carries `kind`, `visibility`, `qualified_name`
and a coarse shape (arity, field count), never a signature's own referenced type identifiers.
The capability-to-provider pairing this rule would need to judge against exists only as
literal `registry.Declare`/`registry.Offer` calls in `nomos-check-orchestration`'s own
`composition.rs` — not a fact `nomos-rules` (Rules zone, forbidden by `Permits` from naming
Provider at all) can read. A rule reading provider identity from inside the Rules zone would
also be the exact boundary crossing `Permits` exists to forbid, which this record's own
measurement surfaces as a design question nobody has answered yet: what fact carries
"provider-specific type" without the checking rule itself having to see the Provider zone.

## The Decision

**Neither rule is built here.** Both need a real fact this workspace does not yet produce,
not a design choice a rule's own implementation could make on the way past. Architecture
drift needs a call-graph or data-flow capability decided and provided before any rule can
read it — a materialization question of the same weight `OD-RULES-010`'s `ToolProvider`
instances answered for lint and dependency-policy facts, not a detail to invent inside
`architecture_drift.rs`. Representation leakage needs either `nomos.cap.syntax.items`'s own
payload schema widened to carry a signature's referenced types, or a new fact naming which
crate is the provider of which capability in a form `nomos-rules` may read without crossing
into the Provider zone itself — `OD-GATE-002`'s own no-resolution stance is not reversed by
this record, only named as the reason the mechanism is not ready.

**The clean baseline for representation leakage is worth recording now, so a future rule is
measured against a known starting point rather than an assumed one.** Zero provider-specific
types were found reaching a consumer's public surface as of this record. A rule built once
the fact exists should expect to compose against that baseline, the same way `OD-RULES-023`
recorded `nomos-store`'s single real write door before anything checked it.

## What This Record Does Not Do

**No code changes here**, the same way `OD-RULES-023` made none. It does not design the
call-graph or data-flow capability architecture drift would need, nor the widened syntax
payload or provider-identity fact representation leakage would need — both are real
capability-design questions, each the size `OD-RULES-010` and `OD-CAPABILITY-010` were, not
answered by naming that they exist.

It does not withdraw the property either rule is for. Both remain real: architecture drift
is a gap `OD-RULES-020`'s zones do not close, and representation leakage is the mechanism
that keeps a capability contract from being a boundary "only by intention," in the original
item's own words. Neither is declined as unwanted; both are undecided as unready.

It does not reduce the original item's two-rule scope to one. Unlike `OD-RULES-023`, where
write authority was ready and only ownership was not, this record found neither half
buildable against an existing fact — the split there does not repeat here because there is
nothing on this record's own side of it to build yet.

## Status

Accepted. Architecture drift and representation leakage are both real properties this
workspace does not yet have a fact to check them against. A capability or fact decision is
the next step for each, named here rather than improvised inside a rule's own
implementation; representation leakage's clean baseline is recorded for whenever that
decision lands.
