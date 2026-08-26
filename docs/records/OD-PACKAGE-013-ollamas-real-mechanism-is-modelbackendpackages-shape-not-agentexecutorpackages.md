---
id: OD-PACKAGE-013
type: decision
title: Ollama's real mechanism is ModelBackendPackage's shape, not AgentExecutorPackage's, measured against OD-PACKAGE-010's own two definitions
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - package
  - model
  - agent
  - executor
relations:
  - target: OD-PACKAGE-010
    type: relates-to
  - target: OD-PACKAGE-012
    type: relates-to
  - target: OD-EXECUTOR-003
    type: relates-to
  - target: OD-EXECUTOR-004
    type: relates-to
---

# Ollama's real mechanism is ModelBackendPackage's shape, not AgentExecutorPackage's, measured against OD-PACKAGE-010's own two definitions

## Question

An external architecture review, checked against this workspace's own records rather than
accepted on read, named `nomos-agent-executor-ollama` — landed as "the second real
`AgentExecutor`" (`P14-TRACKB-AGENT-EXECUTOR-OLLAMA-FIRST-INCREMENT`) — the biggest
conceptual drift in the track that built it: mechanically it is "Nomos builds a request →
local inference runtime → model response," the review's own description of a direct
`ModelBackendPackage`, not "Nomos hands a `TaskEnvelope` to something that may internally
choose tools and models," its description of `AgentExecutorPackage`.

`PackageKind`'s own doc comments (`crates/contracts/nomos-contracts/src/package.rs:115-118`)
state the same distinction in this workspace's own words: `ModelBackendPackage` is "a model
inference backend"; `AgentExecutorPackage` is "an agent execution backend." Neither
`OD-EXECUTOR-003` nor `OD-EXECUTOR-004` — the two records that built and then measured
Ollama's capability boundary — asked which of these two kinds it actually is. Both inherited
"AgentExecutor" from `OD-EXECUTOR-001`'s framing by analogy, the same failure mode
`OD-EXECUTOR-004`'s own text names and avoids for capability-boundary questions but did not
apply to classification. `OD-PACKAGE-010` and `OD-PACKAGE-012` — the two records that define
`ModelBackendPackage`/`AgentExecutorPackage` and how routing between them should integrate —
existed before Ollama's crate was written and were not consulted.

## What Was Measured

Read directly, `crates/agent/nomos-agent-executor-ollama/src/lib.rs`, not assumed from its
crate doc alone.

**`Command_For` builds exactly one shape of request: a fixed model, a fixed subcommand, and
one string.** `Command_For` (`lib.rs:134-142`) constructs
`["ollama", "run", MODEL, task.goal]` — `MODEL` a hardcoded constant
(`qwen2.5-coder:7b`), never resolved from a catalog, selection policy, or caller choice. This
is a model inference request assembled by Nomos, dispatched to a local inference runtime —
`ModelBackendPackage`'s own shape, verbatim.

**Every field a `TaskEnvelope`-carrying `AgentExecutor` would need to honor is read, and
explicitly not honored.** The crate's own module doc (`lib.rs:22-30`) states plainly:
`task.goal` is read; `scope`, `prohibited_changes`, `available_tools`, `knowledge_context`,
`applicable_rules` and `effort` are "accepted and ignored." There is no tool-use loop
(`OD-EXECUTOR-004`'s own measurement: without `--experimental`, absent here, "no tool-use
loop of any kind"), no MCP configuration surface, no scope-honoring mechanism of any kind —
not a governed-but-narrow `AgentExecutor`, but the complete absence of the capability class
`AgentExecutorPackage` names. `OD-EXECUTOR-004`'s own words, read again with this question in
mind rather than the capability-boundary question it was written to answer: "Ollama is a bare
model runner, not a project-aware coding agent." That sentence is `ModelBackendPackage`'s own
doc comment, restated independently by the record that built this crate.

**Nothing about the `TaskEnvelope` parameter itself changes the classification.** Both
`Execute` functions in this workspace — `nomos_agent_executor_claude_code::Execute` and
`nomos_agent_executor_ollama::Execute` — take a `TaskEnvelope`. Taking that type as an input
parameter is not what makes something an `AgentExecutorPackage`; what a `ModelBackendPackage`
and an `AgentExecutorPackage` each *do* with it is. Claude Code's own crate honors
`--allowedTools`, `--strict-mcp-config`, `--max-budget-usd` and reports structural tool
denials (`OD-EXECUTOR-001`) — real evidence of an agent execution backend that may internally
choose tools. Ollama's honors none of it, by its own admission, and forwards one field.

## The Decision

**`nomos-agent-executor-ollama` is a `ModelBackendPackage` instance, not an
`AgentExecutorPackage` instance.** The crate's real, measured mechanism — a fixed local model,
one text field forwarded, every agent-shaped field of `TaskEnvelope` explicitly ignored, no
tool-use or MCP surface of any kind — matches `PackageKind::ModelBackendPackage`'s own
definition and does not match `PackageKind::AgentExecutorPackage`'s. `OD-EXECUTOR-004`'s
measurement of its capability boundary stands unchanged; what this record corrects is the
class that measurement was filed under, not the measurement itself.

This does not indict `OD-EXECUTOR-003`/`OD-EXECUTOR-004`'s own reasoning — both correctly
measured what Ollama's real invocation is permitted to do. It indicts only the unexamined
premise both inherited: that a second backend reachable through `nomos agent execute` must be
a second `AgentExecutor`, rather than a question `OD-PACKAGE-010`/`OD-PACKAGE-012` — already
on record, never checked — existed to answer.

## What This Record Does Not Do

It does not rename the crate, move its code, change `nomos agent execute`'s CLI surface, or
touch `nomos-agent-executor-claude-code`. Naming what Ollama actually is and building the
correction that follows are different acts, the same distinction `OD-EXECUTOR-004` itself
drew between measuring a boundary and building the crate that would be bound by it. A
follow-on item's territory: a rename (`nomos-agent-executor-ollama` →
`nomos-model-backend-ollama`, or whatever a real `ModelBackendPackage` naming convention
settles on), and reconciling `nomos agent execute --backend`'s CLI framing, which currently
presents Ollama as a peer choice to Claude Code under one flag — see `OD-EXECUTOR-005`, which
measures what that flag is actually choosing between now that this record answers what Ollama
is.

It does not decide whether `ModelBackendPackage`'s first manifest maturity
(`OD-PACKAGE-010`) or its routing-integration constraint (`OD-PACKAGE-012`, integrate through
`nomos_capability`, not a privileged layer) should now be built against a real backend. It
names that Ollama is the real second case those two records were written ahead of, not what
either should build from it.

It does not decide Codex's or Gemini's classification. `OD-EXECUTOR-004` found both blocked
for reasons outside this workspace's control; this record answers nothing about either.

## Status

Accepted. Ollama's real, measured mechanism is `ModelBackendPackage`'s shape; the crate and
its CLI framing remain unrenamed pending a follow-on correction item.
