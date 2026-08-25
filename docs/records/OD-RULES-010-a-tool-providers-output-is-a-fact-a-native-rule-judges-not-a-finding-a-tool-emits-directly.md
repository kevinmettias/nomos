---
id: OD-RULES-010
type: decision
title: A ToolProvider's output is a fact a native rule judges, not a Finding a tool emits directly
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - rules
  - packages
  - capability
  - evidence
relations:
  - target: OD-RULES-001
    type: relates-to
  - target: OD-RULES-004
    type: relates-to
  - target: OD-PACKAGE-011
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
---

# A ToolProvider's output is a fact a native rule judges, not a Finding a tool emits directly

## Question

`nomos_contracts::PackageKind::ToolProvider` ("an external analysis or transformation
tool, wrapped behind a capability") is one of twelve still-unconsumed package kinds --
`package.rs`'s own doc says plainly that no installable unit of any of those twelve kinds
exists in this workspace, and none is written to make them look used. Whether the first
real `ToolProvider` -- a wrapper around `cargo clippy`, the workspace's own obvious first
candidate -- should let the tool's own verdict become a `Finding` directly, or should
treat the tool's output as raw material a native `nomos-rules` function then judges, is
undecided, and the two shapes are not interchangeable: one widens `Finding`'s own
attribution scheme, the other does not.

## What Was Measured

Verified directly against the real code: every `Finding` this workspace produces today
traces to exactly one `nomos_contracts::RuleId` (`finding.rs`) -- the rule that judged it.
There is no second attribution field, and no type anywhere names a tool as the entity that
rendered a verdict. `nomos_contracts::EvidenceClass::Authoritative` ("Definitional -- true
because the system defines it so") is declared and has zero real usages anywhere in the
workspace; every real `Finding`/`MaterializedFact` produced today carries `Verified`
(a provider's own subprocess output, e.g. `nomos-lang-rust-cargo`'s `cargo metadata`
materialization) or `Derived` (a native rule's judgment over a materialized fact).

`Check_Dependency_Direction` (`crates/rules/nomos-rules/src/dependency.rs`) is the
worked precedent for the only shape that exists today: it calls `FactReader::Require`
against a capability, and on success judges the decoded payload and emits a `Finding` at
`Applicability::Supported`, `EvidenceClass::Derived`. `nomos-lang-rust-cargo`
(`crates/languages/nomos-lang-rust-cargo/src/metadata.rs`) is the worked precedent for a
subprocess-backed provider: it runs an external tool (`cargo metadata`) through
`nomos_platform::ProcessLauncher`, parses its output, and returns facts for the
composition root to materialize into the store via `nomos_capability::Registry::
Declare_And_Offer` -- it does not itself construct a `Finding`, `Applicability`, or any
judgment. `cargo clippy`'s own diagnostics are, unlike `cargo metadata`'s package graph,
already a rendered verdict about specific code -- level, message, location -- which is
what makes this question real rather than a restatement of the existing precedent: a
`ToolProvider` wrapping clippy could either mirror `nomos-lang-rust-cargo`'s shape (an
inert fact) or skip straight to what `Check_Dependency_Direction` produces (a judged
`Finding`), and nothing already decided which.

`OD-PACKAGE-011` v3 and `OD-WORKFLOW-002`/`OD-WORKFLOW-003` each already declined a
structurally identical move elsewhere: widening a closed vocabulary or a type's shape to
serve exactly one real case, before a second one exists to prove the wider shape is
actually needed rather than merely convenient for the first. Widening `Finding`'s
attribution scheme for one tool, and giving `Authoritative` its first use in the same
stroke, is the same move.

## The Decision

The first real `ToolProvider` materializes its tool's output as a fact behind a real
capability contract, the same shape `nomos-lang-rust-cargo` already establishes for a
subprocess-backed provider, stamped `EvidenceClass::Verified` (the tool ran and answered),
never `Authoritative`. A native `nomos-rules` function then reads that fact through
`FactReader::Require` and emits `Finding`s from it at `EvidenceClass::Derived`, the same
`Require`-then-judge-then-emit shape `Check_Dependency_Direction` already uses. `Finding`
gains no new attribution field. This decision's own first increment mirrors
`nomos-lang-rust-cargo` paired with `nomos_capability::Registry::Declare_And_Offer` for the
provider half, and `Check_Dependency_Direction`'s own trace for the rule half -- no new
mechanism, only a new capability and a new provider and rule offering against it.

## What This Does Not Do

It does not widen `Finding` to carry a tool's own identity as a second attribution axis
beside `RuleId`, and it does not give `EvidenceClass::Authoritative` a real consumer.
Both would be justified the moment a second real `ToolProvider` needs to report a verdict
`Finding` cannot already represent honestly through a judging rule -- not before, the same
"no invented shape ahead of a real second case" discipline `OD-PACKAGE-011` and
`OD-WORKFLOW-002`/`003` already apply. A native rule that does nothing but relay a tool's
own diagnostic 1:1, rather than add real judgment of its own, is accepted as an honest
consequence of this decision, not hidden as a defect: the rule's floor is "this tool
already decided," stated as its own `Requirement` the way every existing rule states its
own, and `Check_Dependency_Direction`'s own precedent already accepts a rule whose
"judgment" is largely pass-through when its capability's own provider already resolved the
hard part.

It does not pick which real tool the first `ToolProvider` wraps, name the capability's own
identifier, or build any code. Those are the first increment's own questions, not this
record's.

It does not decide anything about `MetricProvider`, `RepositoryProvider` or
`RuntimeProvider` -- the other package kinds `PackageKind::Hosts_Foreign_Code` groups
beside `ToolProvider`. Each is a distinct capability shape with its own real first case to
measure against, not a case this record's reasoning is assumed to generalize to
automatically.

## Status

Accepted. Two real `ToolProvider`s now exist on this shape: `nomos-lang-rust-clippy`
(`nomos.cap.lint.diagnostics`) and `nomos-lang-rust-deny` (`nomos.cap.dependency.policy`),
both `Require`-then-judge-then-emit through a native rule at `EvidenceClass::Derived` over a
`Verified` fact, neither needing a tool-identity attribution axis or a real
`EvidenceClass::Authoritative` consumer. The second instance confirms this decision rather
than reopening it.
