---
id: OD-CAPABILITY-017
type: decision
title: A bundled provider does not defeat the Rules boundary, because what that boundary forbids is a rule obtaining its own answer
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - layering
  - dependencies
relations:
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-CAPABILITY-015
    type: relates-to
  - target: OD-RULES-020
    type: relates-to
  - target: ARC-CONNECTOR-001
    type: relates-to
---

# A bundled provider does not defeat the Rules boundary, because what that boundary forbids is a rule obtaining its own answer

## Question

`nomos-rules` names `nomos-connector-coderabbit` and `nomos-cap-requirement-trace`. Each of
those crates carries a real provider beside its contract: the first reaches GitHub through
`gh api`, the second reads arbitrary files across the tree.

The dependency model forbids Rules from naming Provider. Both crates are classified Capability
Contract, so the edge is permitted — but the classification is of the *crate*, and the crate
contains a provider. An external review put the worry plainly: the zone label is doing work the
physical dependency does not justify, and `nomos-rules` acquires a compile-time link to
external-system machinery while the declared edge still reads as a prohibition.

Two neighbouring questions are already settled and neither is this one. `OD-CAPABILITY-015`
settles which zone such a crate belongs to, on the criterion that it declares a capability
contract — and `README.md` states in terms that the classification is not made because
`nomos-rules` needs to reach it, which would make the edge vacuous by relabelling whatever a
rule turns out to need. `OD-CAPABILITY-002` settles when a contract earns its own crate, and its
criterion is *provider contention*: the moment a second provider exists, because the first can
change the ceiling, the version or the schema its peer is bound by in a file the peer cannot
open. A second *consumer* is not that, so the argument that `nomos-rules` reading a contract is
itself the second party forcing a split misreads that record, and no split is ordered here.

What is open is narrower: does bundling let a rule reach past a boundary that is supposed to
stop it?

## What Was Measured

Four facts, checked 2026-09-14 rather than reasoned from the shape of the manifests.

**`nomos-rules` calls no provider-side symbol of either crate.** Every use is contract half:
`Capability`, `CONTRACT_VERSION`, `Ceiling`, `Payload_Schema`, `Parse_Payload`,
`Capability_Contract`, and the payload types. `Discover_Workspace` and `Materialize_Workspace`
appear in `checks/requirement_trace.rs` only inside documentation prose describing who produces
the fact, never in code.

**The rule cannot execute anything either provider does.** Both bundled crates reach the outside
world through `nomos_platform`, which is ports — traits, no implementations. From `nomos-rules`
the number of reachable platform *implementations* is zero: no `nomos-platform-std`, no composer.
A rule holding a trait and no implementor of it cannot launch a subprocess, so `gh api` is not
something a rule could run even if it tried to call the code linked beside it.

**The edge that carries them is independently legal.** `Permits` grants `Rules` exactly
`Protocol | Substrate | CapabilityContract`. `nomos-platform` is Substrate, so a rule may name it
outright; it arrives here transitively and `nomos-rules` does not name it directly, but nothing
about that arrival is a permission the zone lacked.

**So nothing crossed that the model did not already allow.** The provider's object code is linked
into the rlib. No capability it needs is reachable, and no symbol of it is called.

## Decision

**Bundling a provider with its contract does not defeat the Rules boundary, and neither crate is
split by this record.**

The reasoning turns on what the prohibition is for. `Rules` may not name `Provider` so that a
rule cannot **obtain its own answer** — so that judgment consumes facts resolved through the
registry, from whichever provider the composition chose, rather than calling one provider
directly and hard-wiring the answer's origin into the judgment. That is a property about how a
verdict is *reached*. It is not a property about which object code shares an rlib.

Measured against that purpose, the property holds. Every answer both rules consume arrives as a
`Fact` resolved through a capability, and the provider code sitting in the same crate is
unreachable to them in the only sense that matters: they neither call it nor could run it.

**A zone classification remains a statement about what a crate declares, not about what its worst
half could do.** `OD-CAPABILITY-015` is unweakened. The alternative — classifying by the most
dangerous thing inside a crate — would make the Capability Contract zone unavailable to any
contract whose single provider happened to live beside it, which is the arrangement
`OD-CAPABILITY-002` licenses on purpose while a capability has one provider.

## What A Future Bundled Contract Must Satisfy

So that this does not have to be re-derived, and so that the permission is bounded rather than
general. A crate carrying both a capability contract and a provider may be named by `Rules` when
all three hold:

1. **The rule consumes the contract half only** — capability identity, contract version, ceiling,
   schema, payload types and payload parsing. It calls nothing that produces a fact.
2. **The provider half cannot act from where the rule sits.** Whatever it needs to reach an
   external system — a launcher, a filesystem, a clock — is a port with no implementation
   reachable from `Rules`. A provider that could act without one is not admissible under this
   record, and bundling it would put real capability inside the Rules zone rather than beside it.
3. **Its presence is licensed by `OD-CAPABILITY-002`**, meaning the capability still has exactly
   one provider. A second provider makes the contract contended, and contention is that record's
   own trigger to extract the contract into its own crate — at which point this question stops
   being asked, because the rule would name a contract crate with no provider in it.

Failing any of the three, the answer is to split the contract out, not to reclassify the crate.

## What Is Owed

**The first clause is the load-bearing one and nothing enforces it.** A future edit to
`checks/review.rs` could call `nomos_connector_coderabbit`'s fetching or translation functions
directly, and the compiler would accept it: the crate is linked, and the symbols are public.
Today's answer rests on a measurement, and a measurement is a fact about one afternoon.

What would hold it is an assertion that `nomos-rules` names only contract-half symbols of a
bundled crate. That is checkable — the contract half is a nameable set — and it is not written
here, because this record's territory is the adjudication and not the guard. It is the same
shape of debt `OD-PLATFORM-004` carried and had paid one item later, and the comparison is
deliberate: a clause that fails silently, in the committed state, is worth a guard rather than a
convention.

## What This Does Not Decide

Whether either crate would be better split anyway, for reasons of taste or of future shape,
rather than because a boundary demands it. Nothing here forbids splitting them; it decides only
that the Rules edge does not compel it.

Nor does it reopen `ARC-CONNECTOR-001`. A connector's evidence class, its read-only posture and
its fixture obligations are that record's, and are untouched.

Checked 2026-09-14 against the bundled crates this adjudication reaches and `nomos-rules`' own edges onto them.
