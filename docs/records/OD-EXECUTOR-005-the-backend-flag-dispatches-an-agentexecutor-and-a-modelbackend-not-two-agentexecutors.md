---
id: OD-EXECUTOR-005
type: decision
title: The --backend flag dispatches an AgentExecutor and a ModelBackend, not two AgentExecutors, so OD-EXECUTOR-004's shared-trait trigger has not fired
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - executor
  - model
  - contracts
relations:
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-EXECUTOR-004
    type: relates-to
  - target: OD-PACKAGE-013
    type: relates-to
---

# The --backend flag dispatches an AgentExecutor and a ModelBackend, not two AgentExecutors, so OD-EXECUTOR-004's shared-trait trigger has not fired

## Question

`OD-EXECUTOR-004` declined to build "a dispatch trait generic over more than one" backend,
naming its own re-fire condition precisely: `OD-EXECUTOR-001`'s restraint "holds until a real
caller needs to choose." `nomos agent execute`/`judge-role`'s `--backend` flag
(`P14-TRACKB-CLI-AGENT-BACKEND-FLAG`) is a real caller that chooses, today, between
`nomos_agent_executor_claude_code::Execute` and `nomos_agent_executor_ollama::Execute` — a
plain `match` with zero shared trait, its own doc comment stating so explicitly
(`crates/host/nomos-cli/src/agent.rs`'s `Dispatch`: "The two crates share no trait —
`OD-EXECUTOR-001`/`OD-EXECUTOR-004` both decline to invent one ahead of a real need — so this
match is the entire dispatch"). Read at face value, this is the trigger firing: a real caller
now dispatches between two implementations sharing an identical `Execute<P: ProcessLauncher>`
signature, near-verbatim-duplicated `Isolated_Working_Directory`/`Command_For`/
`Require_Clean_Exit` logic, and independently declared, structurally similar outcome and error
types.

`OD-PACKAGE-013` changes the premise this question was read against: it finds
`nomos-agent-executor-ollama`'s real mechanism is `ModelBackendPackage`'s shape, not
`AgentExecutorPackage`'s. If that classification is right, `--backend` is not choosing between
two `AgentExecutor`s at all — this record checks what that does to `OD-EXECUTOR-004`'s trigger
before any trait is designed against the wrong premise.

## What Was Measured

**`OD-EXECUTOR-004`'s trigger names a choice between backends of the same kind.** Its own text:
"a real caller needs to choose" between `AgentExecutor` implementations — the same restraint
`OD-EXECUTOR-001` states as "a dispatch trait generic over more than one" `AgentExecutor`.
Nothing in either record contemplates a caller choosing between an `AgentExecutor` and a
`ModelBackend` under one flag; that shape did not exist when either was written.

**Under `OD-PACKAGE-013`, there is exactly one real `AgentExecutor` in this workspace today:**
`nomos-agent-executor-claude-code`. Ollama's crate, real and dispatched through the identical
CLI flag, is a `ModelBackendPackage` instance wearing an `AgentExecutor`-shaped function
signature — `Execute<P: ProcessLauncher>(&TaskEnvelope, &P) -> Result<...>` — because it was
built to satisfy `OD-EXECUTOR-004`'s (correctly measured, wrongly classified) capability
boundary, not because it is structurally the same kind of thing Claude Code's crate is.

**The structural duplication `OD-EXECUTOR-004`'s trigger would be evidence of — two
`AgentExecutor`s sharing real, extractable behavior — has a different, weaker explanation
once the premise is corrected.** `Isolated_Working_Directory`, `Command_For`'s subprocess
shape, and `Require_Clean_Exit` are duplicated because both crates dispatch a subprocess
through the same `nomos_platform::ProcessLauncher` seam, not because both are `AgentExecutor`s.
A `ModelBackend` and an `AgentExecutor` invoked as local subprocesses would share exactly this
much structure regardless of which package kinds they are — process launch, working-directory
isolation, exit-code discipline are properties of `ProcessLauncher`-based dispatch, not
properties `AgentExecutorPackage` specifically confers. `nomos-lang-rust-cargo` and
`nomos-lang-rust-deny` — two `ToolProvider`s, an entirely different package kind — share the
identical `Require_Clean_Exit` shape with both agent crates, which is evidence for a possible
future `ProcessLauncher`-dispatch convenience shared across *all* subprocess-based providers,
not evidence specific to `AgentExecutor`.

**No second real `AgentExecutor` exists to justify an `AgentExecutor` trait today.** With
Ollama correctly read as a `ModelBackend`, `OD-EXECUTOR-004`'s named trigger — a real caller
choosing between two `AgentExecutor`s — has not fired. What `--backend` actually demonstrates
is a real caller choosing between a `TaskEnvelope`-consuming `AgentExecutor` and a
model-request-issuing `ModelBackend`, presented as if they were peers under one CLI flag and
one Rust function shape. That is a real question, but a different one than the trigger names.

## The Decision

**`OD-EXECUTOR-004`'s shared-`AgentExecutor`-trait trigger has not fired.** No `AgentExecutor`
trait is built. The plain `match` in `crates/host/nomos-cli/src/agent.rs::Dispatch` stays
exactly as it is with respect to `AgentExecutor` abstraction — `OD-EXECUTOR-001`'s restraint
continues to hold, because a second real `AgentExecutor` still does not exist.

**The real question `--backend` raises is a naming and framing one, not an abstraction one: it
presents a `ModelBackend` as a peer `AgentExecutor` choice.** That is `OD-PACKAGE-013`'s
territory to name as a follow-on correction — a `--backend` flag (or a split
`--executor`/`--model-backend` pair) that reflects what each value actually dispatches to,
once `nomos-agent-executor-ollama` is renamed and reclassified — not this record's to build.

## What This Record Does Not Do

It does not decide whether a `ModelBackend` trait, or a lower-level shared
`ProcessLauncher`-dispatch convenience beneath both package kinds, is warranted — that
question was not asked here and has its own, separate evidence (`nomos-lang-rust-cargo`,
`nomos-lang-rust-deny`, both agent crates) that a future record can measure on its own terms,
should a second real `ModelBackend` or a third `ProcessLauncher`-dispatched crate arrive.

It does not change `nomos agent execute`/`judge-role`'s CLI surface, rename
`nomos-agent-executor-ollama`, or touch `crates/host/nomos-cli/src/agent.rs`'s dispatch. It
answers only whether the trigger fired, and names the real correction as a follow-on's
territory, alongside the rename `OD-PACKAGE-013` already named.

## Status

Accepted. `OD-EXECUTOR-004`'s shared-`AgentExecutor`-trait trigger has not fired; the evidence
that appeared to fire it was a `ModelBackend` misclassified as a second `AgentExecutor`.
Revisit if a second real `AgentExecutor` — not a `ModelBackend` — is ever dispatched alongside
Claude Code's.
