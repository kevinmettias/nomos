---
id: OD-CONNECTOR-001
type: decision
title: An outward mutation is a command through a canonical service, and absence from the interface is the mechanism rather than a permission check
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ecosystem
  - connectors
  - capability
relations:
  - target: ARC-CONNECTOR-001
    type: relates-to
  - target: OD-SPEC-009
    type: relates-to
---

# An outward mutation is a command through a canonical service, and absence from the interface is the mechanism rather than a permission check

## Question

`ARC-CONNECTOR-001`'s fourth invariant already says a write capability is omitted from a
connector's interface rather than present and permission-checked, and says why: a
deny-checked write method is still a write method, reachable by whoever next changes the
check. What it does not spell out is the structural mechanism that makes omission possible
at all, where that mechanism reaches its own limit, or whether the rule is a fact about
connectors specifically or a shape this workspace already applies elsewhere and simply had
not named for this seam.

The risk is concrete. A vendor SDK hands a connector one client object with read and write
methods sitting side by side — `issue.comment()` next to `issue.get()` on the same value.
The shortest correct-looking way to reflect a Nomos verdict back to a tracker is to call the
write method that is already sitting there in the imported type. Nothing about "omit the
write method" tells an implementer what to write instead, or what happens when the object
that carries both methods is not one this workspace defined and cannot edit.

## Decision

**An outward mutation is a command handed to one canonical service, and a connector never
performs it directly.** This is `OD-SPEC-009`'s shape applied to the opposite direction of
the same seam. `OD-SPEC-009` decided that a submission enters the store through exactly one
accept function and that a form, a CLI verb, an HTTP endpoint or an MCP tool is a transport
onto that door, never a second door of its own — and it did so for the same reason
`nomos-store` already carries one write door per authority (`README.md`, row 12): the number
of doors is a property of the authority being written to, not of how many callers want to
reach it. An outward write onto an external system of record is the same shape mirrored
outward: the canonical service is the one place that constructs the vendor request,
evaluates whether the mutation is sound to send, and executes it; a connector's translation
layer constructs the *content* of a command — the comment text, the target issue, the new
status — and hands it to that service, exactly as a CLI verb constructs a submission and
hands it to `OD-SPEC-009`'s accept function. A connector that calls a vendor's write method
on its own behalf is a second door, in the same sense a transport that persists on its own
behalf is a second door under `OD-SPEC-009` — a defect, not a variant, and this record cites
that decision as the one it is applying rather than restating it.

**The structural rule holds in the strong form: a component receives an interface carrying
only the operations it may perform.** A capability a component may not exercise is not
present on the type it is given, so calling it is not a runtime refusal but a compile
error — there is no method to name. Runtime authorization — a permission check, a role
gate, a scope token evaluated when a call is attempted — is an additional layer on top of
this, never the mechanism itself, for the reason `ARC-CONNECTOR-001` already gives for its
own fourth invariant: a check can be weakened, bypassed, or forgotten by whoever next
touches it, and a method that exists to be checked is a method that exists. This is not a
principle invented for connectors. `crates/substrate/nomos-analysis/src/fact/reader.rs`
already ships it: `FactReader` gives a caller `Get`, `Require`, `Require_Any` and
`Dependencies` — every way to ask the store a question — and no method that writes a fact
back. A caller holding a `FactReader` cannot mutate a materialized fact through it, not
because an attempt is checked and refused, but because the type it was handed has nothing
on it that would perform one. `OD-CONNECTOR-001` names what `FactReader` was already doing
and requires it of a connector's interface too: the observation interface
`ARC-CONNECTOR-001`'s invariant 4 requires is a `FactReader`-shaped trait, not a full
vendor client with a policy wrapped around it.

**Structural absence is achievable wherever this workspace defines the type the caller
holds, and it is not achievable on a type this workspace does not define.** A vendor SDK's
client object is the case that breaks the mechanism: its write methods exist before any
Nomos code runs, sitting on the same object as its read methods, and no trait this
workspace writes can delete a method from a type it does not own. Where the capability
arrives already bundled on a foreign type, the mechanism is not omission but containment,
and it is a weaker guarantee: `ARC-CONNECTOR-001`'s second invariant already requires that
no vendor schema, field name or authentication scheme cross above the translation layer,
and this record extends that same containment to the vendor client object itself — the raw,
write-capable SDK type may not be named, returned, stored or passed above the translation
layer under any circumstance, so that nothing outside it ever holds a value with a write
method reachable through it. What a caller above the seam holds instead is the
`FactReader`-shaped observation interface this workspace defines over that translation
layer, which is where omission becomes possible again because the type is now this
workspace's own. The difference is not stylistic: a Rust method omitted from a trait this
workspace owns cannot be called from outside that trait under any edition of the language,
by construction; a vendor type kept out of every signature above one module is confinement
that a single import statement inside that module's own boundary can still violate, and
which nothing but review, a lint, or a future architectural test catches. Confinement is
what stands in where omission cannot reach, and it is strictly weaker than omission — which
is exactly why the translation layer, and not the connector interface above it, is where
`ARC-CONNECTOR-001` already places the only code permitted to hold the vendor object at
all.

## Scope

This record's scope is connectors. It does not decide whether an executor invoking a
subprocess, a plugin loaded into this workspace, an agent given tool access, or a transport
under `OD-SPEC-009` carries the same write-omission requirement, and it deliberately declines
to extend the rule to any of them here. Each is a different boundary with its own shape — a
subprocess is not a vendor SDK object with methods bundled by a client library, a plugin's
ABI is not a translation layer between a foreign schema and a canonical one, an MCP transport
already answers to `OD-SPEC-009` on the inbound side and has not been asked an outbound
question — and deciding all four by extension from the connector case would repeat the
mistake `ARC-CONNECTOR-001` already named and refused to make for connector addressing:
settling a scheme for a case that is not in front of anyone yet. Whichever of those boundaries
is built first is where that boundary's own record decides whether this rule reaches it,
citing this one as the precedent it is applying, the same way this record cites
`ARC-CONNECTOR-001`'s fourth invariant rather than restating it.

## What This Binds

Every connector's observation interface — the generic connector substrate and the
vendor-to-canonical translation layer together, as `ARC-CONNECTOR-001` scopes them — carries
only the operations a caller above the seam may perform. An outward mutation is never one of
them; it is a command constructed by the translation layer and handed to the one canonical
service that performs it, in the same relationship a transport under `OD-SPEC-009` has to
the accept function.

The vendor's own client object, and any type carrying its write methods, may not be named,
returned or held by anything above the translation layer that defines it.

## What This Does Not Bind

It does not name the canonical service, define its interface, or say which crate holds it.
That is implementation the first connector's translation layer is checked against, the same
way `ARC-CONNECTOR-001` left the quarantine crate's name to whoever adopts one.

It does not decide whether an executor, a plugin, an agent or a transport carries the same
rule. See Scope, above.

It does not touch `ARC-CONNECTOR-001`'s first three invariants, or reopen the crossings
`ARC-ECOSYSTEM-001` governs. Both are unchanged.

## Controls

| Weakening | What it produces |
|---|---|
| the connector calls the vendor's write method directly, behind a permission check | a write method that exists to be checked, reachable by whoever next weakens the check — exactly what `ARC-CONNECTOR-001`'s fourth invariant already refuses |
| the connector's interface exposes the raw vendor client above the translation layer | confinement is skipped, and the write method the interface omitted is reachable anyway through the object that carries it |
| the canonical service is treated as a convenience the connector may bypass for "simple" mutations | the second door `OD-SPEC-009` already named the failure mode for, mirrored outward |
| the write-omission rule is assumed to already cover executors, plugins, agents or transports | a boundary with its own shape is governed by an analogy instead of by its own record |

## Conflicts With Existing Decisions

`ARC-CONNECTOR-001` is untouched. Its fourth invariant is applied in its strong form and
grounded in a mechanism and a precedent it did not name; the invariant itself is not
reopened or reinterpreted.

`OD-SPEC-009` is untouched. Its one-door-many-transports shape is applied to the outward
direction of the same third crossing `ARC-CONNECTOR-001` names; the intake seam it governs
is unchanged.

## What This Record Does Not Do

No canonical service is built, and no connector interface is defined. No executor, plugin,
agent or transport question is answered. This record does not claim the write-omission rule
is enforced mechanically anywhere yet — only that it is now a rule a connector's interface
and its translation layer are checked against, with a named mechanism, a named limit, and a
named precedent, rather than a discouragement without a structural reason.

## Status

Closed by `P12-WRITE-CAPABILITY`, which named the mechanism `ARC-CONNECTOR-001`'s fourth
invariant assumed without naming: omission where this workspace owns the type, containment
where it does not. It discharges the mechanism question for connectors; whether an
executor, a plugin, an agent or a transport carries the same rule is left to that
boundary's own record, citing this one as precedent rather than inheriting it.
