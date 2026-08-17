---
id: OD-HOST-002
type: decision
title: A surface holds no state its canonical services cannot reconstruct
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - cli
  - orchestration
  - state
  - architecture
relations:
  - target: OD-HOST-001
    type: relates-to
---

# A surface holds no state its canonical services cannot reconstruct

`OD-HOST-001` gave `nomos-work-orchestration` a seam so a second adapter could call the
work group without duplicating platform choice, verb execution or rendering. It settled
the duplication. It did not settle what a second adapter is allowed to keep once it has
called through that seam, and the gap is invisible today only because `nomos-cli` is a
CLI that exits after every verb — nothing has ever had a process lifetime long enough to
accumulate anything.

An editor session, a long-lived service, or a web view does not exit between calls, and a
process that does not exit is cheap to let accumulate things that are expensive to ask
for again: a resolved provider selection, an in-memory work graph, a connector session, a
computed trace. Each is individually reasonable and collectively becomes a second copy of
the system's state that no other client can read and no test can address. The failure
mode is not corruption; it is that the official surface quietly becomes the only complete
one, and every other client — a script, a peer implementation, a CI step, an agent —
sees a subset.

## The decision

**No surface holds state that a canonical service cannot reconstruct.** A canonical
service here means a library crate any client can call directly — `nomos-ledger`,
`nomos-work-orchestration`, `nomos-capability`, `nomos-analysis`, `nomos-contracts`,
`nomos-spec-store`, and their kind — not a rendering one particular client already
produced. Operationally:

- every piece of state a surface displays must be obtainable, by any client, by calling a
  canonical service — not only by asking the surface that first computed it;
- every action a surface can take must be issued as a command through a canonical
  service, not executed by logic that lives only inside the surface.

A surface that cannot name which service it would call to reconstruct a fact it is
currently showing is holding privileged state, whether or not that state happens to be
correct today.

This does not forbid a surface from remembering anything — see "What is not privileged
state" below.

## The state families this applies to

Verified against the current codebase, not assumed from vocabulary. Each is real
independent of whether a seam exists for it yet; where none exists, this record names the
gap rather than closing it.

1. **Work / board state — the one family with a working seam.** `nomos_ledger::
   LedgerDocument` (`crates/substrate/nomos-ledger/src/store/ledger_document.rs:12`) is
   the on-disk board; `nomos_work_orchestration::WorkOutcome`, `BoardView`, `ShowView`
   (`crates/orchestration/nomos-work-orchestration/src/outcome.rs:19,31,45`) are the
   reconstructable shape `OD-HOST-001` built the seam for. `nomos-cli::work::Run`
   (`crates/host/nomos-cli/src/work.rs:39`) holds nothing of its own: it calls `Run`
   fresh, renders the returned `WorkOutcome`, and exits. This is the demonstration the
   other families are held to.

2. **Capability / provider resolution — no seam yet.** `nomos_capability::Registry`,
   `Requirement`, `ProviderOffer`, `Resolution` (`crates/substrate/nomos-capability/src/
   registry.rs`, `requirement.rs`, `provider_offer.rs`, `resolution.rs`) decide which
   provider satisfies which capability. Today only `nomos-cli::check::composition::
   Registered` and `Resolved_Configuration`
   (`crates/host/nomos-cli/src/check/composition.rs:15,59`) build this registry and hash
   it into a `ConfigurationId`; no crate outside `nomos-cli` can obtain the same resolved
   registry. A second surface has two options — reimplement `composition.rs`, or accept
   whatever `nomos-cli` computed as fact — and both are the failure this record forbids.

3. **Fact / analysis state — per-invocation only today, and only because the CLI
   exits.** `nomos_analysis::Context`, `MemoryFactStore`, and the fact-invalidation
   machinery (`Condensation_Of`, `RematerializationGroup` in
   `crates/substrate/nomos-analysis/src/store.rs:136,171`) hold the facts a check run
   reasons over. `nomos-cli::check::facts::Prepare`
   (`crates/host/nomos-cli/src/check/facts.rs`) builds this fresh per call and drops it
   when `check::Run` returns. It is stateless across invocations only because `nomos-cli`
   happens to exit; a long-lived surface running this same code path in place, rather
   than through a callable service, reaches the identical failure with a longer
   lifetime.

4. **Finding / requirement state.** `nomos_contracts::Finding`, `Applicability`,
   `EvidenceClass`, `Guarantee` (`crates/contracts/nomos-contracts/src/finding.rs:56`,
   `finding/applicability.rs:33`, `finding/evidence.rs:24`, `guarantee.rs:23`) are what
   `nomos check` produces about a requirement. They are consumed only inside
   `nomos-cli::check::report` and `vacuity.rs` today; no other crate can ask "what does
   this repository's rule engine currently say about this subject" without running
   `nomos check` itself.

5. **Evidence — two vocabularies at two layers.** `EvidenceClass` above is the
   contracts-layer provenance strength attached to a `Finding`. `nomos_model::EvidenceRef`
   and the evidence-claim vocabulary (`crates/kernel/nomos-model/src/evidence/
   reference.rs:5`, `evidence/claim.rs`) are a separate, kernel-layer pointer to
   supporting evidence. A surface reconstructs each through the canonical crate that owns
   its layer; neither is memoized in place of calling that crate.

6. **Package state.** `nomos_contracts::PackageKind`
   (`crates/contracts/nomos-contracts/src/package.rs:82`) is declared and deliberately
   unconsumed — no manifest reader exists yet, by the type's own documentation. Named
   here so that when a reader is built, package state is reconstructed through it rather
   than accumulated inside whichever surface happens to implement resolution first.

7. **External / connector state — anticipated, not yet real.** No `Connector` type
   exists anywhere in this workspace's code. `ARC-CONNECTOR-001`, `OD-CONNECTOR-001` and
   `OD-CONNECTOR-002` describe the intended external-system substrate, and
   `PackageKind::IntegrationPackage` reserves its packaging slot. Recorded here, ahead of
   the first connector, so its session state is decided by the substrate that will own
   it rather than by whichever surface implements the first connector against a live
   external system.

8. **Decision / record state.** No `DecisionTrace` type exists. The actual decision
   trail is the governing-record corpus itself — records under `docs/records/` carrying
   `type: decision`, read through `nomos-spec-store` and `nomos-spec-project`.
   `DecisionGap` (`crates/spec/nomos-spec-model/src/failure/decision_gap.rs:10`) is the
   adjacent but different concept: an open question blocking a submission, not a history
   of decisions already taken. A surface showing "why was this decided" reads the store
   again; it does not cache a rendered history of records it has shown once.

9. **Control commands.** `WorkCommand`
   (`crates/orchestration/nomos-work-orchestration/src/command.rs:15`) is the one command
   vocabulary that already routes through a seam. `SpecCommand`
   (`crates/host/nomos-cli/src/spec/command.rs:11`), `CheckCommand`
   (`crates/host/nomos-cli/src/check/command.rs:6`) and `request::Command`
   (`crates/host/nomos-cli/src/request.rs:65`) are parsed and executed directly inside
   `nomos-cli` today, with no equivalent orchestration crate. Until each has one, this
   record's second clause — every action is a command through a canonical service — is
   unmet for three of the four command groups. Naming that gap is what makes the rule
   checkable rather than already true by assumption.

## What is not privileged state

Caches, view state and selections are not privileged state, and the difference is
reconstructibility, not memory.

A **cache** holds an answer a canonical service already gave and can give again.
Dropping it costs latency, not information: the surface asks the service again and gets
the same answer, because the service — not the cache — is still the source of truth.

**View state** — which panel is open, a scroll position, an expand/collapse toggle — and
**selection** — a cursor position, a highlighted item, an unsent draft command — are not
answers to a domain question at all. No canonical service could hold them, because they
describe the surface's own presentation rather than the system's state, and no other
client would ever need them to answer the question this surface is answering.

The test that separates the two: *if this surface exits right now, is any fact lost that
no other client — a script, a CI step, a peer implementation — could get back by calling
the same canonical service?* Cache, view state and selection all answer no. A resolved
provider registry kept only in memory instead of recomputed, a work graph edited in place
instead of re-read through the seam, a connector session whose negotiated state exists
nowhere else — these answer yes, and are exactly the privileged state this record
forbids.

## What this constrains

This record adds no seam and reopens none. `OD-HOST-001` decided that
`nomos-work-orchestration` is the seam for the work group; this record states what any
seam — that one, or one not yet built for capability resolution, fact state, findings,
evidence, packages, connectors, or the record store — must guarantee once it exists: a
surface calling through it holds nothing the seam itself cannot regenerate. Building the
seams still missing for families 2–4 and 6–9 above is future work this record makes
checkable, not work it does.
