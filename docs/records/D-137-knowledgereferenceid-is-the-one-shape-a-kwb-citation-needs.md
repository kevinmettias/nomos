---
id: D-137
type: decision
title: KnowledgeReferenceId is admitted to nomos-contracts as the one shape a KWB citation needs, and nothing yet produces or consumes one
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - contracts
  - ecosystem
  - kwb
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: D-135
    type: relates-to
  - target: D-136
    type: relates-to
---

# KnowledgeReferenceId is admitted to nomos-contracts as the one shape a KWB citation needs, and nothing yet produces or consumes one

## Decision

A new identity type, `KnowledgeReferenceId`, is admitted to `nomos-contracts` (band 0): an
opaque, authored identifier minted by an external knowledge system for one claim,
rationale, or decision, carried by Nomos as a citation without Nomos computing, verifying,
or interpreting it. It is a `Named_Identity`, not a `Digest_Identity`: Nomos does not
compute this value from bytes it holds, so treating it as one of Nomos's own content
digests would claim a verification Nomos cannot perform.

This decision adds the identity type only. No existing type — `Finding`, `EvidenceClass`,
or any rule or correction record — is changed to carry one yet, because nothing in this
workspace currently produces or consumes a citation, and adding an unused field to a
widely-consumed type ahead of a producer is the speculative extension `OD-CONTRACTS-001`
exists to keep out of this crate.

## Rationale

`ARC-ECOSYSTEM-001` states the KWB-to-Nomos boundary as a governed projection: "a
rationale is not enforceable; a contract derived from it is," and the derivation "must be
a recorded step rather than an inference somebody made once." Recording that step requires
naming, at minimum, which KWB-side thing was projected — and that name is the one piece of
vocabulary a peer that never compiles this crate must still agree with Nomos about, which
is exactly `OD-CONTRACTS-001`'s admission test: would a peer be unable to agree with us
without this type? Everything else about a projection — who approved it, what Nomos rule
resulted, when it happened — is Nomos-internal bookkeeping nobody outside Nomos needs to
agree on the shape of, and does not belong in this crate.

`D-135` and `D-136` motivate doing this now rather than waiting for a concrete producer:
Nomos and the forthcoming KWB Rust repository are being designed with each other in mind,
and the projection boundary is exactly the seam vocabulary both sides should agree on
before either builds around a private guess at its shape.

## Consequences

`KnowledgeReferenceId` exists in `nomos-contracts::identity` and is exported from the crate
root, following the same `Named_Identity!` pattern as `RuleId`, `PackageId`, and
`SchemaId`. `tests/contract`'s assertion that this crate depends on `serde` and nothing
else is unaffected, since the new type introduces no dependency. When a concrete producer
exists — a rule authored from a governed projection, or a finding whose evidence traces to
one — the type that carries a `KnowledgeReferenceId` is added to whatever record needs it,
in a separate, later decision, scoped to that actual need.

## Alternatives Considered

Waiting until a concrete Nomos-side consumer exists before naming anything was rejected:
the whole reason this type crosses a product boundary is that both Nomos and the new KWB
repository need to agree on its shape independently, and KWB's own bootstrap (`D-136`) is
happening now — a peer starting its own repository around a private guess at this shape is
a worse outcome than naming the shape one version early.

Reusing an existing identity (`SchemaId`, or a bare `String`) was rejected: a bare `String`
gives a KWB citation and, say, a schema name the same type, so passing one where the other
is expected compiles — exactly the `Waiver { Check: path, Path: check }` failure
`nomos-contracts`'s own `identity.rs` documents as the reason its identities are distinct
newtypes rather than aliases.
