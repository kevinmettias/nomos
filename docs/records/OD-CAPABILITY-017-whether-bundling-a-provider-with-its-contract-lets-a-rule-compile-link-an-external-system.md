---
id: OD-CAPABILITY-017
type: decision
title: A bundled provider does not defeat the Rules boundary, because what that boundary forbids is a rule obtaining its own answer
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - capability
  - layering
  - dependencies
relations:
  - target: OD-ROADMAP-005
    type: relates-to
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

## Amendment, Version 2: The Owner Required One Of The Two Crates Split, And Every Measurement Above Still Holds

`OD-ROADMAP-005`, decision 5, supersedes this record's closing sentence — "neither crate is
split by this record" — for one of the two crates it reaches.
`nomos.cap.review.finding`'s contract now lives in `nomos-cap-review-finding`, and
`nomos-connector-coderabbit` keeps the provider. `nomos-cap-requirement-trace` is untouched
and is still bundled.

Nothing measured above is withdrawn, and saying which part moved is the whole content of
this amendment. A reader arriving at this record from the tree would otherwise find a
decision not to split beside a split, with nothing to say which is current.

### What still stands, re-checked rather than assumed

All four measurements, re-run on 2026-09-22 against the tree immediately before the split:

- **`nomos-rules` called no provider-side symbol of either crate.** Still true and now
  mechanical for the crate that is still bundled:
  `Test_Rules_Should_Name_Only_The_Contract_Half_Of_A_Bundled_Crate` is green, and it was
  green with `nomos-connector-coderabbit` in its population on the commit before this one.
  The split was not repairing a violation. There was none.
- **No platform implementation was reachable from `Rules`, so the provider could not act
  from where a rule sat.** Unchanged: `nomos-rules` still names no `nomos-platform-std` and
  no composer, and the split neither added nor removed one.
- **The edge that carried the crates was independently legal.** Unchanged.
- **So nothing crossed that the model did not already allow.** Unchanged.

`OD-CAPABILITY-002`'s criterion is also intact, and this is the easiest thing here to read
wrongly: the criterion is provider **contention**, and it has not fired.
`nomos.cap.review.finding` has exactly one provider today, the same one it had when this
record was written. The contract did not earn a crate under that record; it was given one
under a different authority.

### What the override actually changed, which is the ground and not the finding

The external review's objection was never that a rule could act through the provider linked
beside it. This record measured that and found it could not, and that finding is the one the
override leaves alone. The objection was that a **generic rule layer had to spell a vendor's
crate name** to obtain a generic capability: `nomos-rules` wrote `nomos_connector_coderabbit`
to reach `nomos.cap.review.finding`, so a reader of the Rules zone met `CodeRabbit` there.

This record answered that with "the Rules boundary is about how a verdict is reached, not
about which object code shares an rlib," which remains correct about the boundary and does
not answer the naming complaint at all. The owner treated the naming complaint as a
sequencing question — build the split now rather than wait for contention — which is the
owner's to decide, and recorded the bound in `OD-ROADMAP-005` rather than leaving six
implementations reading as sessions building against accepted records.

### What moved, exactly

| went to `nomos-cap-review-finding` | stayed in `nomos-connector-coderabbit` |
|---|---|
| `CAPABILITY`, `SCHEMA`, `CONTRACT_VERSION`, `Capability`, `Payload_Schema`, `Ceiling`, `Capability_Contract` | `PROVIDER`, `Declared_Guarantee`, `Provider_Offer` |
| `FindingPayload`, `PayloadRefusal`, `ReviewFindingId`, `Encode_Payload`, `Parse_Payload` | `Fetch_Review_Comment`, `Translate_Review_Comment`, `Materialize_Review_Comment`, `Fact_Of`, `ReviewFindingFact`, `FactContext` |
| | `Sample_Review_Comment_Response`, `ReviewFindingProduction`, `Review_Comment_Identity` |

The right-hand column is `OD-CAPABILITY-002`'s own division applied unchanged: a
`ProviderId` is a provider's own name, a `Guarantee` is its own claim, a recorded fixture is
its own evidence, and a `Strategy` declaration is a promise about its own repetition. The
codec crossed with the payload types because every extracted `nomos-cap-*` crate in this
workspace already holds both halves of its codec — `nomos-cap-lint` and `nomos-cap-dependency`
are the precedent, and splitting a writer from its reader would put the agreed wire shape in
two crates.

### The identity constructor went with the provider, not with the type it returns

`ReviewFindingId` is a payload field, so the type is the agreement's. Its constructor was
not. `Of_Review_Comment` joined a repository and GitHub's permanent comment id with a
`review-comment` label — GitHub's addressing scheme, spelled out. A contract crate carrying
it would be one vendor's convention wearing the agreement's name, which is the defect this
piece exists to remove, one level down from the crate name.

So it is `nomos_connector_coderabbit::Review_Comment_Identity` now, and the contract carries
an opaque identity that a second provider over a different review system could mint its own
way. This was visible in the tree before the split and is worth recording because it is the
kind of thing a crate-level move leaves behind: `nomos-rules`' own test fixture built its
sample payload by calling `ReviewFindingId::Of_Review_Comment(...)` — a generic rule's test
minting a GitHub identity. It spells the literal now, and could not do otherwise:
`Permits` forbids `Rules` from naming `Provider`, which is where the constructor lives.

### No re-export, and convenience was not a good enough reason for one

`nomos-connector-coderabbit` re-exports no part of the contract. A re-export would keep
`nomos_connector_coderabbit::Capability()` and `nomos_connector_coderabbit::FindingPayload`
spelling correctly, so a consumer could go on naming the vendor crate for a generic
contract — which is precisely the thing the review objected to and the only thing this
piece removes. A compatibility shim here would be the defect preserved behind a `pub use`,
and the migration it would spare is four call sites in two crates, both of them in this
workspace, both changed in the same commit. `nomos-lang-rust-clippy` does not re-export
`nomos-cap-lint` either, and nothing has ever wanted it to.

### The three conditions, and what became of each

The conditions this record set for a **bundled** crate are unchanged and still govern
`nomos-cap-requirement-trace`. For `nomos-connector-coderabbit` they no longer apply,
because it is no longer bundled, which this record's own third condition anticipated in
terms: contention "is that record's own trigger to extract the contract into its own
crate — at which point this question stops being asked." The trigger that fired was a
different one; the consequence is the one predicted.

The consequence is also stronger than the conditions were. `nomos-connector-coderabbit` is
Provider zone now — `OD-CAPABILITY-015`'s criterion is what a crate *declares*, and it
declares no contract — so `Permits` forbids `Rules` from naming it at all, and the first
condition is enforced for this crate by the dependency model rather than by a text scanner.
The debt this record recorded as owed, and
`tests/contract/tests/boundaries/bundled_contract_half.rs` paid, is what held that line in
the meantime.

### The guard's population fell to one, and it is not retired

`HALVES` named two crates and now names one. Dropping the row rather than editing it is
deliberate: a row for a crate that is not bundled is a classification outliving its subject,
which is the staleness `Test_Every_Module_Of_A_Bundled_Crate_Should_Be_Classified` exists to
refuse in the other direction.

A population of one is a guard with a subject; a population of zero would be
`OD-COMPLETENESS-001`'s shape — a check quantifying over an empty universe and passing
having judged nothing. That was the risk worth naming, so both assertions now check the
population is non-empty and that `nomos-rules` still depends on each crate named, rather
than trusting that somebody would notice. `Test_A_Rule_Naming_A_Fact_Producing_Symbol_Should_Be_Rejected`,
the negative control, no longer spells a crate name at all: it reads the first row of
`HALVES` and takes its own literals from what that crate re-exports, so the next such move
cannot leave a control passing about a crate nobody bundles.

Retiring the module was considered and rejected on two counts. It would leave this record's
first clause unenforced for `nomos-cap-requirement-trace`, which is still bundled and still
named by `nomos-rules`; and the day a second bundled contract arrives, the guard would have
to be written again out of this record, which is the cost `OD-PLATFORM-004`'s comparison
above was about.

Checked 2026-09-22 against the split as landed: the workspace compiles, the contract crate
carries no provider symbol, `Test_A_Capability_Id_Should_Be_Written_In_One_Crate` still
finds `nomos.cap.review.finding` in exactly one crate's `src`, and
`Test_Every_Crates_Public_Surface_Should_Match_Its_Snapshot` holds both crates' surfaces to
their committed snapshots.
