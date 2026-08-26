---
id: OD-LEDGER-037
type: decision
title: Whether nomos-agent-contracts' reuse of nomos-ledger's Territory and VerificationPredicate needs its own product-owned types
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - agent
  - contracts
  - architecture
relations:
  - target: OD-LEDGER-036
    type: relates-to
  - target: OD-LEDGER-001
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# Whether nomos-agent-contracts' reuse of nomos-ledger's Territory and VerificationPredicate needs its own product-owned types

## Question

An external architecture review named `nomos-agent-contracts`'s dependency on `nomos-ledger`
a real boundary violation: `TaskEnvelope.scope`/`.prohibited_changes`
(`crates/agent/nomos-agent-contracts/src/task_envelope.rs`) are `nomos_ledger::Territory`;
`NomosResolvedChangeContext.permitted_scope`/`.required_verification`
(`change_context.rs`) are `nomos_ledger::Territory` and `nomos_ledger::VerificationPredicate`;
`WorkResult` (`work_result.rs`) also imports `VerificationPredicate`. The review's reasoning:
`nomos-ledger` is repository coordination tooling, not a product concept, so a product-level
agent-safety contract depending on it looks backward — `nomos-ledger` should adapt product
primitives, not the other way around.

`OD-LEDGER-036` (accepted one day before this review, citing this same review by name)
already settled the half of this question it was written to settle: `nomos-ledger` and
`nomos-work-orchestration` are repository bootstrap machinery, not a Nomos product feature,
and do not converge toward `TaskEnvelope`/`Workflow`/`AgentExecutor`. Its own "What This
Record Does Not Do" section is explicit that it leaves the type-reuse question open: it names
`nomos-agent-contracts`'s `TaskEnvelope`/`WorkResult` as "declared bundling" — a description,
not a verdict — and does not move a file, rename a crate, or decide whether `Territory`/
`VerificationPredicate` should stay shared or be split. This record answers the question
`OD-LEDGER-036` left standing.

## What Was Measured

**The dependency and the two type imports are real**, confirmed directly:
`crates/agent/nomos-agent-contracts/Cargo.toml` names `nomos-ledger` a direct dependency;
`task_envelope.rs` imports `Territory`; `change_context.rs` imports `Territory` and
`VerificationPredicate`; `work_result.rs` imports `VerificationPredicate`.

**Neither type is ledger-shaped once read on its own terms.** `Territory`
(`crates/substrate/nomos-ledger/src/territory.rs`) is a set of path patterns with
`Covers`/overlap logic built for one job: deciding whether two ledger claims conflict. Nothing
about a set of path patterns is specific to work-ledger coordination — `TaskEnvelope.scope`
uses exactly this shape to say what an agent may touch, a genuinely product-level concept with
no ledger claim anywhere near it. `VerificationPredicate`
(`crates/substrate/nomos-ledger/src/verification/predicate.rs`) is a program-and-argv pair the
ledger runs to decide whether an item is done; `NomosResolvedChangeContext.required_verification`
uses the identical shape to say what an agent's change must pass. Both types are already
general-purpose in everything but name and package — they do not encode a ledger claim, a
lease, a holder, or anything else specific to `work/ledger.json`'s coordination job.

**No band rule is violated, and no test would catch a change either way.** `nomos-ledger` is
band 20; `nomos-agent-contracts` is band 36 — strictly downward, per README's mechanically
enforced rule. `README.md`'s `[repo tooling]` marker on `nomos-ledger` is descriptive, per
`OD-LEDGER-036`'s own text, not a dependency-direction ban. This means the current dependency
is not a defect a test will surface on its own; it is a design-clarity question this record
answers by argument, the same way `OD-LEDGER-036` answered ownership by argument rather than
by a mechanical check finding a violation.

**The cost of the status quo is legibility, not correctness.** A reader of
`nomos-agent-contracts`'s public surface who has not read `OD-LEDGER-036` sees a product
safety contract importing types from a crate README marks `[repo tooling]`, and has no way to
tell, from the import alone, that the types themselves are ledger-agnostic — only that they
happen to live there because `nomos-ledger` built them first. `OD-LEDGER-001` already
establishes that territory is declared, not enforced, which is a property of the *mechanism*
`Territory` implements, unrelated to which crate owns its Rust definition.

## The Decision

**The types are ledger-agnostic and belong to `nomos-agent-contracts`'s own dependency level,
not `nomos-ledger`'s — but this record does not move them now.** Extracting `Territory` and
`VerificationPredicate` (or product-owned equivalents `nomos-agent-contracts` would define and
`nomos-ledger` would then depend on instead) is a real, warranted correction in direction. It
is not built here for the same reason `OD-PACKAGE-007`'s generic-core split was built only once
a second language made the Rust-specific shape concretely wrong rather than merely aesthetically
displeasing: moving two widely-used types out from under a crate with a real, working consumer
(`nomos-work-orchestration`'s claim/overlap logic) and into a new home is a mechanical,
multi-file change with real regression surface, not a naming correction like `OD-PACKAGE-007`'s
own rename was. Naming the shape now, so a future session does not have to re-derive it, is
this record's job; making the move is a follow-on's.

**The shape a follow-on item would build:** `Territory` and `VerificationPredicate` (recognizable
by their real current field shapes, unchanged) move to a crate at or below
`nomos-agent-contracts`'s own band — either a new lightweight crate both `nomos-ledger` and
`nomos-agent-contracts` depend on, or directly into `nomos-contracts` if their shape is judged
general enough to sit beside `RunId` and the other identity/contract primitives already there.
`nomos-ledger` re-exports or wraps them for its own claim/overlap logic exactly as
`nomos-lang-rust-package` re-exports `nomos-package`'s domains today (`OD-PACKAGE-007`) — a
behavior-preserving move, not a redesign.

## What This Record Does Not Do

It does not move `Territory`, `VerificationPredicate`, or any code. `nomos-agent-contracts`
keeps depending on `nomos-ledger` exactly as it does today until a follow-on item performs the
move this record names.

It does not reopen `OD-LEDGER-036`'s ownership decision. `nomos-ledger` remains repository
bootstrap machinery; this record's finding — that two of its types are themselves
ledger-agnostic — is consistent with that decision, not a correction to it.

It does not decide whether `ScopeConstraint`/`ArtifactScope`/`MutationBoundary`/
`VerificationRequirement` — the specific names an external review proposed — are the right
names or the right granularity for the extracted types. That is design work for whichever item
performs the move, informed by `Territory`'s and `VerificationPredicate`'s real current shapes
rather than invented ahead of them.

## Status

Accepted. The two types are ledger-agnostic and a follow-on extraction is warranted; the move
itself is not performed by this record.
