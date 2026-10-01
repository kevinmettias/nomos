---
id: OD-CONTRACTS-004
type: decision
title: TaskEnvelope gains an effort field, so an agent executor can read one real model-routing signal
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - agent
  - contracts
  - model-route
relations:
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-PACKAGE-011
    type: relates-to
  - target: OD-PACKAGE-012
    type: relates-to
---

# TaskEnvelope gains an effort field, so an agent executor can read one real model-routing signal

## Question

`MODEL-ROUTE-001` requires that "gates, phases, workflows, rules/checks, and agent task
classes shall be able to reference a `ModelExecutionProfile` for every agent-assisted
operation." `nomos_model_package::ModelExecutionProfile`, its `EffortLevel` and
`ModelSelector` components, and `MODEL-ROUTE-015`'s `EffortMappingRecord` all exist as
declared vocabulary, with zero real consumers anywhere in the workspace (`OD-PACKAGE-011`
v3). `nomos_agent_contracts::TaskEnvelope` -- the one existing, `Serialize`/`Deserialize`
"what an agent-assisted operation is handed" type `AGT-001` requires -- has no field for
any of it, and `nomos-agent-executor::Command_For` dispatches every task through one
hardcoded Claude Code invocation with no model or effort choice at all. Whether
`TaskEnvelope` is the right place for a first real reader of this vocabulary, and whether
building one now is premature ahead of a second real executor, is this record's question.

## What Was Measured

Verified directly against the installed CLI, not assumed: `claude --help` documents a
real `--effort <level>` flag, accepting `low`, `medium`, `high`, `xhigh`, `max`.
`nomos_model_package::EffortLevel` (`MODEL-ROUTE-004`) closes a six-value canonical
enumeration: `BackendDefault`, `Minimal`, `Low`, `Medium`, `High`, `Maximum`. The two
vocabularies do not line up one-to-one: `claude`'s `xhigh` has no `EffortLevel`
counterpart, and `EffortLevel::Minimal` has no native control below `claude`'s own `low`.

`OD-PACKAGE-011` v3 and `OD-PACKAGE-012` v2 already close the larger question this record
does not reopen: no routing/selection taxonomy exists yet (`MODEL-ROUTE-038`-`049`), and
building one is premature while exactly one real executor (`nomos-agent-executor`,
dispatching to Claude Code alone) exists -- inventing a selection policy over a single
candidate would invent a taxonomy nothing yet needs, the same "no real second case" finding
`OD-PACKAGE-011` already made for the fuller shape. Reading one already-specified field,
`EffortLevel` alone, is a different, much narrower act: the field is closed and
corpus-fixed, and `claude --effort` is a real, present control it maps onto today, not a
resolver this record would have to invent.

`TaskEnvelope` is the correct place for this field rather than a bare function parameter to
`Execute`: `TaskEnvelope` is this workspace's one type a peer executor -- API-hosted,
subscription-agent, human, or recorded-replay, `WF-006`'s own list -- would receive across
a wire boundary, per its own doc's framing as "the contract a peer executor must agree with
Nomos about." A function parameter invisible to that type would never reach a real
out-of-process peer at all, only this one in-process caller.

## The Decision

`TaskEnvelope` gains an eighth field, `effort: EffortLevel`, alongside `AGT-001`'s own
seven. `nomos-agent-executor::Command_For` reads it and appends `--effort <value>` to the
real `claude` invocation: `Minimal` and `Low` both map to `low` (`Minimal` an
approximation, `MODEL-ROUTE-015`'s own `MappingQuality::Approximate` shape, since `claude`
has no distinct control beneath `low`); `Medium`, `High` and `Maximum` map to `medium`,
`high` and `max` respectively; `BackendDefault` omits the flag entirely rather than passing
a value naming "the default," which is this crate's own behavior for every caller that
predates this record. `claude`'s `xhigh` stays unreachable from `EffortLevel` -- this
record does not fold it into `high` or `max` to manufacture a use for it.

This is a correction, not a fresh architectural decision: it gives an already-specified
requirement (`MODEL-ROUTE-004`) its first real reader, the same shape `OD-CONTRACTS-003`
already used for `WorkResult.plan`.

## What This Does Not Do

It does not build `ModelSelector`, any resolver, or wire `nomos-model-package` into
`nomos-agent-executor` beyond this one field. `OD-PACKAGE-011`/`012`'s finding that
selection is premature with one real executor is unchanged and unrevisited here.

It does not emit a real `MODEL-ROUTE-015` `EffortMappingRecord`. That type already exists
for exactly this kind of mapping and is the natural next increment once a caller needs the
resolved-mapping evidence returned, not merely the flag sent -- naming it here rather than
building it now, the same "found but not yet needed" discipline this workspace already
applies elsewhere.

It does not retroactively require every existing caller to change beyond adding the one new
field. Verified directly: the only production construction sites for `TaskEnvelope` in this
workspace before this record are this crate's own test and `nomos-cli::agent`'s CLI verb.
The latter is not this record's own territory -- `crates/host/nomos-cli/src/agent.rs` is
held by a concurrent claim (`P14-CLI-AGENT-JUDGE-ROLE`) building an unrelated feature in the
same file -- so its two literals were given the same one-line, purely additive fix in the
shared working tree, coordinated directly with that session rather than committed here; it
lands with whichever commit publishes that item's own work, not this record's.

## Status

Accepted. `nomos-agent-executor-claude-code::Command_For` reads `TaskEnvelope.effort` and
appends `--effort` exactly as decided -- `Minimal`/`Low` collapsing to `low`,
`Medium`/`High`/`Maximum` mapping straight across, `BackendDefault` omitting the flag --
tested for all six variants including the deliberately unreachable `xhigh` case.
`ModelSelector` and a real `MODEL-ROUTE-015` `EffortMappingRecord` remain exactly as unbuilt
as "What This Does Not Do" already said.
