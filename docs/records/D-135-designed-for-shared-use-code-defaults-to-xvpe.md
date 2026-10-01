---
id: D-135
type: decision
title: Generic code designed for shared use defaults to XVPE; D-122's proof gate governs only code that started product-specific
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - platform
  - dependencies
  - ecosystem
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: D-130
    type: relates-to
---

# Generic code designed for shared use defaults to XVPE; D-122's proof gate governs only code that started product-specific

## Decision

A subsystem with no software-engineering or knowledge-domain semantics — one whose
definition would be equally correct under a product that had nothing to do with either — is
proposed in XVPE the first time it is written, not after two products have separately
reinvented it. `D-122` (adopted into `ARC-ECOSYSTEM-001`) continues to govern the other
direction unchanged: code that began product-specific and is later suspected of being
generic still needs that proof — two products demonstrating materially identical
domain-neutral semantics — before it moves, because "both products would use it" is the
reuse argument `ARC-ECOSYSTEM-001` already refuses. This decision changes only where
new, deliberately domain-neutral code is authored the first time; it does not weaken the
proof requirement for code moving after the fact.

## Rationale

Nomos and a forthcoming Rust rewrite of KnowledgeWorkbench (`D-136`) are being designed with
each other in mind from the start, which changes the argument `D-122` was answering. `D-122`
guards against inferring genericity from a wish — a mechanism proposed for XVPE because two
products *would* find it convenient, without either having built it. It was never meant to
require two products to independently construct the same execution, storage, or scheduling
substrate on purpose merely to manufacture the proof its gate asks for. Where a subsystem's
definition never mentions a crate, a rule, a finding, a claim, or a concept — content-addressed
storage, task scheduling, serialization, process and telemetry primitives are the concrete
cases motivating this record — writing it twice against two products before naming it XVPE
produces the evidence `D-122` wants by paying the cost `D-122` exists to avoid.

## Consequences

Starting a new subsystem, the question asked first is not "which product owns this" but
"does this subsystem's definition mention Nomos or KWB domain vocabulary." If it does not,
it is proposed in XVPE and consumed by adaptation, per `ARC-ECOSYSTEM-001`'s existing rule
that generic capability is consumed by adaptation and never by extending the generic thing
with product semantics. `D-130`'s constraints on *when* Nomos may actually depend on
anything XVPE holds are unchanged by this decision: no path dependency, a pinned commit SHA
into the quarantined `nomos-platform-xvpe` adapter crate, and no dependency at all before
Phase 5. This record changes only where code is authored; `D-130` still governs when it may
be linked.

## Alternatives Considered

Leaving `D-122` as the sole gate for every case was rejected: applying a
converged-by-accident proof standard to code deliberately co-designed for two products from
the outset treats intentional shared design as though it were opportunistic reuse, which is
a different situation than the one `D-122` was written to police, and the difference is
worth a decision rather than a strained reading of the existing one.
