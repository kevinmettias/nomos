---
id: OD-HOST-002
type: decision
title: A surface holds no state its canonical services cannot reconstruct
status: accepted
version: 6
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

2. **Capability / provider resolution — a working seam, since `nomos-check-orchestration`.**
   `nomos_capability::Registry`, `Requirement`, `ProviderOffer`, `Resolution`
   (`crates/substrate/nomos-capability/src/registry.rs`, `requirement.rs`,
   `provider_offer.rs`, `resolution.rs`) decide which provider satisfies which capability.
   `nomos_check_orchestration::Registered` and `Resolved_Configuration`
   (`crates/orchestration/nomos-check-orchestration/src/composition.rs`) build this
   registry and hash it into a `ConfigurationId`, callable by any crate that depends on
   `nomos-check-orchestration` rather than only from inside `nomos-cli`. This is the same
   duplication `tests/integration/src/context.rs`'s own rendering of a registry was
   already named against under band 100, where this crate cannot reach it — moving the
   canonical copy did not remove that second one, and this record does not ask it to.

3. **Fact / analysis state — a working seam, over the one input a caller must still
   supply.** `nomos_analysis::Context`, `MemoryFactStore`, and the fact-invalidation
   machinery (`Condensation_Of`, `RematerializationGroup` in
   `crates/substrate/nomos-analysis/src/store.rs:136,171`) hold the facts a check run
   reasons over. `nomos_check_orchestration::Run`
   (`crates/orchestration/nomos-check-orchestration/src/run.rs`) builds this from already-
   walked source and a caller-supplied build variant and hands back a
   `CheckOutcome`, reconstructable by any client that walks the same tree — the directory
   walk itself stays a composition-root concern
   (`nomos-cli::check::sources::Walked`), the same exception
   `nomos-cli::work::Published_Records` already has for a directory listing
   `nomos_platform::FileSystem` has no port for.

4. **Finding / requirement state — a working seam.** `nomos_contracts::Finding`,
   `Applicability`, `EvidenceClass`, `Guarantee`
   (`crates/contracts/nomos-contracts/src/finding.rs:56`, `finding/applicability.rs:33`,
   `finding/evidence.rs:24`, `guarantee.rs:23`) are what `nomos check` produces about a
   requirement. `nomos_check_orchestration::Run` runs `nomos_rules::
   Check_Completeness_Mirrors` over a real `nomos_analysis::Reader` and returns them inside
   `CheckOutcome::Judged`, alongside `Examined` and `Claim`
   (`crates/orchestration/nomos-check-orchestration/src/outcome.rs`) — the roll-up
   judgment a second adapter needs without re-deriving it. `nomos-cli::check::report`
   keeps only the rendering half: `Coverage`, a pure grouping of `findings` for a text
   reader, computed fresh at render time rather than carried as a second fact.

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

9. **Control commands — three of four now route through a seam.** `WorkCommand`
   (`crates/orchestration/nomos-work-orchestration/src/command.rs:15`), `CheckCommand`
   (`crates/orchestration/nomos-check-orchestration/src/command.rs:11`, moved verbatim
   from `nomos-cli::check::command`) and `SpecCommand`
   (`crates/orchestration/nomos-spec-orchestration/src/command.rs:15`, moved verbatim from
   `nomos-cli::spec::command` over four increments — `Profiles`/`Sources`, then
   `Record`/`Table`/`Markdown`, then `Render`/`Freshness` generic over
   `nomos_platform::FileSystem`, then `Preview`/`Commit`) each name a crate a second
   adapter can depend on without also taking `nomos-cli`'s argument parsing, exit codes or
   rendering. `nomos-cli::spec`'s own module doc records the move in full and
   `nomos-cli::spec::Run` now only dispatches to `nomos_spec_orchestration::Run` and
   renders what it returns, the same shape `nomos-cli::work` and `nomos-cli::check`
   already have. `request::Command`
   (`crates/host/nomos-cli/src/request.rs:65`) is the one command group still parsed and
   executed directly inside `nomos-cli`, with no equivalent orchestration crate — a single
   `Submit` verb, thin enough that its transport already calls only canonical services
   (`nomos_spec_orchestration::corpus::Assemble`, `nomos_spec_store::Accept_Submission`)
   and holds no state of its own, but still not callable by a second adapter without
   depending on `nomos-cli` itself to get `Command`, `SubmitRequest` and their parsing.
   Until it has a seam too, this record's second clause — every action is a command
   through a canonical service — is unmet for that one remaining command group. Naming
   that gap is what makes the rule checkable rather than already true by assumption.

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
`nomos-work-orchestration` is the seam for the work group; `nomos-check-orchestration` and
`nomos-spec-orchestration` — each built after this record first shipped, closing families
2–4 and the `CheckCommand` and `SpecCommand` halves of family 9 — are the second and third
demonstration rather than a fourth decision: this record states what any seam must
guarantee once it exists, and does not itself build one. A surface calling through any of
the three seams holds nothing the seam itself cannot regenerate. Building the seams still
missing for families 6–8 and the `request::Command` remainder of family 9 above is future
work this record makes checkable, not work it does.

## Amendment: SpecCommand's Seam Closed; request::Command Is the One Gap Family 9 Still Names

Family 9 above, as first written, named `SpecCommand` and `request::Command` together as
the two command groups with no orchestration crate. `SpecCommand`'s seam has since closed,
over four increments, none of which touched this record: `nomos-spec-orchestration`
answers all nine verbs (`Profiles`, `Sources`, `Record`, `Table`, `Markdown`, `Render`,
`Freshness`, `Preview`, `Commit`), and `nomos-cli::spec.rs`'s own module doc states the
move in full, including the reasoning `Render`, `Freshness`, `Preview` and `Commit` earned
genericity over `nomos_platform::FileSystem` where `Profiles`, `Sources`, `Record`,
`Table` and `Markdown` did not. This record's text describing that gap as still open was
stale against a codebase that had already closed it.

`request::Command` was not touched by that work and is not stale: it remains one verb
(`Submit`), parsed and dispatched entirely inside `nomos-cli::request`, with no crate a
second adapter could depend on to reach it without also taking `nomos-cli`. It is the one
piece of family 9 this record still names as open.

## Amendment: GateCommand's `run` Verb Is a Closed Seam Family 9 Never Named

Family 9 above, as first written and as the amendment before this one left it, enumerates
`WorkCommand`, `CheckCommand`, `SpecCommand` and `request::Command` — four command groups,
naming three seams closed and one still open. It never named `GateCommand` at all: the
survey predates `nomos-gate-orchestration` having any `run` computation to seam.
`GateCommand` already existed as vocabulary for `plan`, which needed no seam of its own —
`plan`'s whole computation fit inside `nomos-gate-orchestration` from the day that crate
shipped, the same reason family 9's original four did not include it.

`run` was different from the day it was given a real body. `nomos-gate-orchestration` and
`nomos-check-orchestration` were both band 40, and a band may not depend on its own band
(`tests/contract/tests/boundaries/graph.rs`), so the walk-judge-reduce composition for
`GateInvocation::Run` lived in `crates/host/nomos-cli/src/gate/run.rs` instead — a second
adapter wanting `gate run` without depending on `nomos-cli` would have had to re-derive it,
the same duplication `OD-HOST-001` closed for the work group before this record ever
shipped. `P13-GATE-RUN-SEAM-CRATE` closed it the identical way: `nomos-gate-orchestration`
moved to band 41, above `nomos-check-orchestration`'s band, so it may depend on it, and
`nomos_gate_orchestration::Run_Gate`
(`crates/orchestration/nomos-gate-orchestration/src/run_gate.rs`) now composes
`nomos_check_orchestration::Run` and this crate's own `Disposition` into a `GateRunResult`,
generic over `nomos-platform`'s traits the same way `nomos_check_orchestration::Run` and
`nomos_work_orchestration::Run` already are. `P13-GATE-RUN-SEAM-CLI` migrated the caller:
`crates/host/nomos-cli/src/gate.rs`'s `GateInvocation::Run` arm now walks the tree and
reads the host build variant — the composition-root role `check.rs` already keeps for the
same reason — and hands both to `Run_Gate`, and `gate/run.rs` no longer exists.

This does not change the amendment before it: `request::Command` is still the one command
group with no orchestration crate at all. `GateCommand` was never that — it is the case
family 9 simply forgot to list, closed before this record ever had to call it open.

## Amendment: `PackageKind` Has One Real Consumer Now

Family 6 above, as first written, said `PackageKind` "is declared and deliberately
unconsumed — no manifest reader exists yet, by the type's own documentation." That has not
been true since `P13-PACKAGE-GENERIC-CORE` (`OD-PACKAGE-007`): `nomos-package`'s reader
(`crates/packages/nomos-package/src/reader.rs`) resolves a manifest's `package_kind` field
against this enum and returns `ManifestError::WrongPackageKind` for anything other than
`PackageKind::LanguagePackage` — this enum's first real consumer. The type's own doc
comment (`crates/contracts/nomos-contracts/src/package.rs:57-89`) states the change in
full and was corrected once already, by `P13-PACKAGEKIND-CONSUMER-STALE`, a correction this
record never picked up.

The other fifteen kinds remain exactly the open condition family 6 described:
`RulePackage`, and — named since family 6 was first written — `ModelBackendPackage` and
`AgentExecutorPackage` (`OD-PACKAGE-010`'s first manifest maturity for model selection,
which gives neither package kind a real backend or executor implementation). Each still
gains its consumer the way `LanguagePackage` did: something reads a declared manifest of
that kind and refuses one it cannot resolve. Until then, family 6's original point holds
for those fifteen unchanged — package state is reconstructed through that reader once it
exists, not accumulated inside whichever surface implements resolution first.

## Amendment: Family 3's Directory-Listing Clause Was Overtaken By OD-PLATFORM-002

State family 3 says the walk "stays a composition-root concern
(`nomos-cli::check::sources::Walked`), the same exception `nomos-cli::work::Published_Records`
already has for a directory listing `nomos_platform::FileSystem` has no port for". The port has
one: `OD-PLATFORM-002` added `FileSystem::Read_Directory` on 2026-09-05
(`P41-PLATFORM-DIRECTORY-ENUMERATION-3`, `093a0e4e`).

**The family's own claim is unaffected.** What family 3 asserts is that a `CheckOutcome` is
reconstructable by any client that walks the same tree, and that the walk is a composition-root
concern. Both are still true, and the surviving reason is that `Read_Directory` is one level by
`OD-PLATFORM-002`'s own floor while a source walk is recursive -- not that no operation exists.
`nomos-cli::check::sources::Walked_Sources` carries that reason at the site.

Corrected under `P72-STALE-PLATFORM-DIRECTORY-CLAIM-2`, alongside fourteen other sites and two
other records carrying the same overtaken clause. `OD-HOST-001`'s own amendment has the
measured population and what the staleness cost.
