---
id: OD-EXECUTOR-006
type: decision
title: A CodeRabbit-style review adapter takes ToolProvider/connector shape, not AgentExecutor shape
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - executor
  - connector
  - capability
  - review
relations:
  - target: OD-EXECUTOR-005
    type: relates-to
  - target: OD-LEDGER-037
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: ARC-CONNECTOR-001
    type: relates-to
  - target: OD-CONNECTOR-001
    type: relates-to
  - target: OD-CONNECTOR-002
    type: relates-to
---

# A CodeRabbit-style review adapter takes ToolProvider/connector shape, not AgentExecutor shape

## Question

An external architecture review named CodeRabbit as an obvious next integration once
`nomos-cap-lint`'s tool-provider seam and `nomos-agent-contracts`' executor seam were both
real, without asking which of the two shapes it should take. `nomos-agent-executor-claude-code`
and `nomos-agent-executor-ollama` are the workspace's only two `AgentExecutor`-shaped crates
today (`OD-PACKAGE-013` reclassified Ollama as a `ModelBackendPackage`, leaving exactly one
real `AgentExecutor`); `nomos-cap-lint` plus `nomos-lang-rust-clippy`/`nomos-lang-rust-deny`
are the workspace's real `ToolProvider` instances; `nomos-connector-github` is a real, separate
third shape, a connector under `ARC-CONNECTOR-001`. A CodeRabbit adapter could plausibly be
built as any of the three, and nothing before this record asked which.

Two live decisions sit adjacent to this question and are checked rather than assumed:
`OD-LEDGER-037` (whether `nomos-agent-contracts`' reuse of `nomos-ledger`'s `Territory`/
`VerificationPredicate` needs its own product-owned types) and `OD-EXECUTOR-005` (whether the
`--backend` flag dispatching an `AgentExecutor` and a `ModelBackend` means `OD-EXECUTOR-004`'s
shared-trait trigger has fired). Neither touches `TaskEnvelope`'s or `WorkResult`'s shape, and
`OD-LEDGER-037`'s own question is a future relocation of two field types, not a change to
either. A third `AgentExecutor` implementation started today reads the identical
`TaskEnvelope` regardless of how either question resolves, so this record does not wait on
them.

## What Was Measured

**What `AgentExecutor` shape actually requires.** `nomos-agent-contracts` exports no trait —
`TaskEnvelope`, `WorkResult`, `NomosResolvedChangeContext`, nothing else. Both real
implementations independently supply their own free function,
`Execute<P: ProcessLauncher>(&TaskEnvelope, &P) -> Result<AgentExecutionOutcome,
AgentExecutionError>`, each with its own outcome and error types, each doc comment stating
plainly that no shared trait exists. `AgentExecutor` shape means: accept a bounded task
description, act on the repository on the caller's behalf (possibly choosing tools and
models internally), and return a structured account of what happened. It is a shape for
*doing work*.

**What `ToolProvider` shape actually is.** `OD-RULES-010` states it precisely: a tool's
output becomes a fact a native rule judges, never a `Finding` the tool emits directly.
`nomos-lang-rust-clippy::provider::Materialize_Workspace` runs `cargo clippy` and produces
`DiagnosticsFact`s at `EvidenceClass::Verified`, filed under a `FactKey`; `nomos_rules::
Check_Lint_Diagnostics` reads that fact and relays it as `Finding`s at `EvidenceClass::
Derived`. It is a shape for *establishing evidence about code that already exists*, with
Nomos keeping the judgment.

**What CodeRabbit actually produces.** A CodeRabbit review returns comments about an existing
diff — findings a third party made about code, not new code and not an action taken on the
repository. It does not choose tools, does not act with `scope`/`prohibited_changes`
boundaries the way `AgentExecutor` shape requires, and has nothing resembling `TaskEnvelope`'s
input contract: there is no task to hand it, only a diff to hand it and a review to get back.
That is evidence about code, produced externally, exactly `ToolProvider`'s own shape — not a
bounded actor doing work on Nomos's behalf.

**Why it is closer to a connector than a same-band `ToolProvider`.** `nomos-lang-rust-clippy`
runs a local subprocess the caller's own `ProcessLauncher` controls end to end. CodeRabbit is
a genuine external peer system reached over its own API/webhook surface, carrying vendor
identity and vendor-shaped payloads that need translation before Nomos can read them —
`nomos-connector-github`'s own shape, not `nomos-lang-rust-clippy`'s. `ARC-CONNECTOR-001`
already binds this shape with four invariants, and `OD-CONNECTOR-001`/`OD-CONNECTOR-002`
already settle two of the mechanics a review connector would need regardless of which peer:
an outward mutation (posting a reply, resolving a thread) is a command through a canonical
service, never a raw peer write, and identity/absence/replay are governed the identical way
`nomos-connector-github` already proves end to end, with a real fixture-below-translation
test and one opt-in live-network test. A CodeRabbit adapter's own fact contract would sit
beside its one provider, per `OD-CAPABILITY-002`'s real criterion, the same way `nomos.cap.
connector.artifact` does today for `nomos-connector-github` — extracting a shared contract
ahead of a second connector needing it would be designing from a population of one.

## The Decision

**A CodeRabbit-style review adapter takes connector/`ToolProvider` shape: it establishes
review evidence as a fact, and a native Nomos rule judges it into a `Finding`. It does not
take `AgentExecutor` shape.** The concrete mechanism is `nomos-connector-github`'s own
precedent, not `nomos-lang-rust-clippy`'s: a peer system reached over a real transport,
requiring vendor-to-canonical translation and its own identity/evidence handling under
`ARC-CONNECTOR-001`, rather than a local subprocess a `ProcessLauncher` runs directly. What
it shares with `nomos-lang-rust-clippy` is the boundary that actually matters here —
`OD-RULES-010`'s fact-not-finding split — not the transport.

This is not a preference between two equally valid shapes. `AgentExecutor` shape presumes a
bounded actor that does work; CodeRabbit does not do work on the repository, it produces
evidence about work already done. Building it as a third `AgentExecutor` would mean
inventing a `TaskEnvelope` with nothing for CodeRabbit to read and a `WorkResult` describing
an action that never happened, the same undifferentiated-bucket failure `OD-PACKAGE-003`
names for a package kind carrying content that belongs to a different kind entirely.

## What This Record Does Not Do

It does not build `nomos-connector-coderabbit` or any crate. No new capability contract is
declared, no band is reserved, and no `Cargo.toml` or `README.md` entry changes. It does not
decide CodeRabbit's own review-comment schema, its authentication mechanism, or which of its
API surfaces (REST, webhook, GitHub Check) a real adapter would read — those are the first
real adapter's own measurement, the same way `nomos-connector-github`'s own `fetching.rs` and
`translation.rs` measured GitHub's issue shape rather than this record inventing one in
advance of a subject.

It does not decide whether a review connector's fact contract is `nomos.cap.connector.
artifact` reused, or a `nomos.cap.review.*` contract of its own — that is a second-party
question `OD-CAPABILITY-002`'s criterion answers only once a second connector exists to
compare against the first, exactly as `nomos-connector-github`'s own contract.rs notes for
itself.

It does not reopen `OD-LEDGER-037` or `OD-EXECUTOR-005`, and nothing in either record's own
question bears on this one: both are about `AgentExecutor`-shaped crates' internal typing and
CLI framing, and this record's answer is that CodeRabbit is not `AgentExecutor`-shaped at all.

It does not decide Clay's shape. An executor that genuinely takes a `TaskEnvelope` and acts
on a repository — choosing tools, respecting `scope` and `prohibited_changes` — would be
measured against `AgentExecutor` shape the way this record measured CodeRabbit against it and
found no match; a future record makes that measurement when a real candidate exists to check,
not this one in advance of it.

## Status

Accepted. Names the shape a future CodeRabbit-style adapter takes; builds nothing.
