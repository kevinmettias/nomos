---
id: OD-CONTRACTS-001
type: decision
title: Band 0 admits what crosses a boundary, and a domain-local concept stays in its domain
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - contracts
  - layering
  - protocol
  - admission
relations:
  - target: OD-CONTRACTS-002
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-AGENT-001
    type: relates-to
---

# Band 0 admits what crosses a boundary, and a domain-local concept stays in its domain

## Question

`nomos-contracts` is band 0. Every other crate may depend on it and it may depend on nothing
but `serde`, and `Test_Contracts_Should_Depend_On_The_Allowlist_And_Nothing_Else` holds that
line. What no test and no authority states is the other direction: **what may be added to it.**

The crate holds ten modules. Nothing in it is wrong. That is the reason to settle the question
now rather than after a finding shape, an architecture delta or an agent type has been admitted
on the strength of a sentence nobody chose.

## What Was Measured

Three files described band 0 and they did not agree.

| Where | What it said |
|---|---|
| `README.md`, band table | Protocol truth. Depends on `serde` and nothing else. |
| `crates/contracts/nomos-contracts/src/lib.rs` | the only authoritative statement of Nomos **protocol** semantics |
| `Cargo.toml`, members comment | The only authoritative statement of Nomos **semantics** |

The widest of the three is the one in the workspace manifest, and it is the copy nothing reads
back. `AGENTS.md` says that when two authorities disagree the mechanical one wins and the
disagreement is a defect worth an item. Here the mechanical one is also the narrow one, so the
disagreement had no mechanical resolution: the wide sentence was free to stand.

"A statement of Nomos semantics" is not a criterion. Every concept in the product is a statement
of Nomos semantics — a finding, a rule package, a correction plan, a run request, an agent
capability. The argument for the eleventh module was already written, and it was written in a
comment that no test parses.

## The Decision

**A type is admitted to band 0 when it crosses a subsystem, process or plugin boundary and the
parties on both sides need one stable shared representation of it. Everything else stays in the
domain that owns it.**

Two halves, and the second does the work. The first alone would admit anything that *could* be
shared; the requirement is that something on the other side of a boundary really must agree
about this type in order to speak to Nomos at all.

The test is not "is this important" or "is this general". It is: *would a peer that never
compiles this crate — a knowledge service in another language, a client in TypeScript, a
platform in another Rust workspace — be unable to agree with us without it?* The module
documentation already gives the reason this matters: a dependency added here makes the protocol
Nomos-shaped and forces those peers to vendor a Rust crate. A type admitted here does the same
thing to the vocabulary.

What a domain-local concept does instead is stay in the crate that owns it, and cross a boundary
only as a projection. `SyntaxPayload` is the worked example: it is a real shared representation,
it is read by more than one provider, and it lives in `nomos-cap-syntax` at band 23 rather than
in band 0, because the parties that must agree about it are the providers of one capability
rather than every peer that speaks to Nomos. `OD-CONTRACTS-002` decided the same shape for a
capability contract — it is not the property of the provider that answers it, and it is also not
protocol truth.

The honesty vocabularies are the reason band 0 exists and they satisfy the criterion exactly.
`Applicability`, `EvidenceClass`, `PeerAvailability`, `GateCategory` and `Assurance` each answer
a question a peer must be able to answer in the same words, or the absence of knowledge reads as
a statement that all is well on one side of the boundary and not the other.

## The Statement Lives In One Place

The criterion is stated here, and the three files that described band 0 now route to it rather
than restate it. `OD-AGENT-001` records why that is the right shape for this repository: a
summary of a checked file is an unchecked copy of it, and this workspace has already paid for
one.

`crates/contracts/nomos-contracts/src/lib.rs` carries the operative sentence, because that is
where an author adding an eleventh module is already reading. `README.md` keeps the mechanical
claim its band table can be checked against and names this record. `Cargo.toml` makes no
ownership claim at all.

## What This Binds

An addition to `nomos-contracts` must be justifiable by the criterion above, and the
justification belongs in the change that makes it rather than in a comment discovered later.

`Test_Band_Zero_Should_Be_Described_In_One_Place` in `tests/contract/tests/boundaries.rs` holds
the second half: the ownership phrase may appear in exactly one of the three files, and that
file must cite this record. It is a phrase check and it is deliberately narrow — it cannot tell
whether a *new* type belongs, and no test can. What it can do is stop the wide sentence from
being restated somewhere nothing reads, which is how this defect arrived.

## What This Does Not Do

It does not remove anything from band 0. All ten modules satisfy the criterion, and a record
that both stated a rule and applied it retroactively would be two decisions wearing one
identifier.

It does not make admission mechanical. A test that counted the modules would be a declared
universe needing a mirror, and a test that pattern-matched type names would refuse the next
honest addition for its spelling. The criterion is for a person and for the review of a change,
and this record is what that review cites.

It does not decide where a rejected concept goes. That is a question about which crate owns a
responsibility, and `ARC-ECOSYSTEM-001` routes it.

## Controls

| Alternative | Why not |
|---|---|
| delete the `Cargo.toml` sentence and state nothing | an unstated rule admits everything the wide one did, which is the defect with its evidence removed |
| let the widest statement stand | every concept in the product is a statement of Nomos semantics, so it is not a criterion |
| enforce admission with a test over the module list | a declared universe needs a mirror, and a census cannot answer whether a type crosses a boundary |
| put the criterion in `README.md` only | the author adding a module is reading `lib.rs`, and a rule read after the change is a rule that did not apply |

## Status

Accepted. Band 0 is described once, the description is a criterion rather than a claim of
importance, and the two files that restated it now route to it.
