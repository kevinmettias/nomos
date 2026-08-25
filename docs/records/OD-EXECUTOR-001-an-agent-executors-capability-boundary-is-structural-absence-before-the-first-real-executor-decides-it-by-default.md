---
id: OD-EXECUTOR-001
type: decision
title: An agent executor's capability boundary is structural absence, before the first real executor decides it by default
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - executor
  - security
  - contracts
relations:
  - target: OD-CONNECTOR-001
    type: relates-to
  - target: ARC-CONNECTOR-001
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
  - target: OD-CORRECTIONS-001
    type: relates-to
---

# An agent executor's capability boundary is structural absence, before the first real executor decides it by default

## Question

`OD-CONNECTOR-001` names its own scope precisely: "It does not decide whether an executor
invoking a subprocess, a plugin loaded into this workspace, an agent given tool access, or a
transport under `OD-SPEC-009` carries the same write-omission requirement, and it
deliberately declines to extend the rule to any of them here... Each is a different boundary
with its own shape." It also names the specific failure mode this record exists to close:
"the write-omission rule is assumed to already cover executors, plugins, agents or
transports" — "a boundary with its own shape is governed by an analogy instead of by its own
record."

A first real `AgentExecutor` is about to be built: `nomos-agent-contracts::TaskEnvelope` in,
`WorkResult` out, dispatched to a real subprocess — Claude Code, invoked non-interactively —
rather than a stub. Nothing in this workspace names what that subprocess is permitted to do
before this record, and the first implementation would decide it by being the only thing
that has, the same shape `ARC-CONNECTOR-001` named for connectors before any existed.

## What Was Measured

**`TaskEnvelope` already carries capability-boundary-shaped fields, and nothing reads them.**
`scope: Territory`, `prohibited_changes: Territory`, and `available_tools: Vec<CapabilityId>`
are real, typed fields. Verified directly: a workspace-wide search for `TaskEnvelope` finds
exactly one production construction site — its own crate's test — and no consumer anywhere
reads `.scope`, `.prohibited_changes`, or `.available_tools` off a real value. A caller
filling these fields in expresses an intent nothing is yet obligated to honor.

**`WorkResult`'s only channel into a mutation has no real caller either.** `plan:
CorrectionPlan` is the sole field through which an agent's output could ever change this
workspace; every other field (`claims`, `tests`, `requested_verification`, `assumptions`,
`unresolved_questions`) is report-only. `nomos-corrections`'s own stage/validate/commit chain
is what would apply a `CorrectionPlan`, and `OD-CORRECTIONS-001` already measured that chain
has zero real callers anywhere in this workspace. So today, even a maximally adversarial
`WorkResult` cannot mutate this workspace through the typed path this crate defines — the
door an agent's output would have to walk through to write anything does not open for anyone
yet. The risk this record actually addresses is different and prior to that: what capability
the *subprocess itself* is granted at invocation, independent of what its typed response
later claims, and independent of whether `WorkResult.plan` is ever read.

**The real CLI's non-interactive flags were checked directly, not assumed.** `claude --help`
confirms a non-interactive invocation (`-p`/`--print`) supports `--allowedTools`/
`--disallowedTools`, `--strict-mcp-config` (ignore ambient MCP configuration), and an
`--output-format json` single-turn response. None of these is the default. An invocation
built without them inherits whatever tool permissions the *launching* directory's own
`.claude/settings*` and `CLAUDE.md` auto-discovery grant — for this repository, this
session's own permissive settings, were a subprocess launched from this working tree.

## The Rule

**An executor's capability boundary is structural absence at the invocation site, not a
permission check inside the invoked process, and not trust in what the process is asked to
do.** The same mechanism `OD-CONNECTOR-001` already states for a connector's interface — "the
enforcement is the absence of the capability, not a permission check performed when [it] is
attempted" — applies here, one layer further out: the invocation itself must omit every
capability beyond producing a text response, so there is nothing for a compromised or simply
instruction-following process to reach for, regardless of what the prompt asked it to do.

Concretely, until this record or a successor names a real need otherwise, an invocation of a
real agent executor:

- launches from a freshly created, empty, isolated working directory — never this
  repository's own tree, never any directory carrying its own `.claude/settings*` or
  `CLAUDE.md` — so no ambient permission grant or auto-discovered project instruction can
  reach it;
- passes no MCP configuration and sets `--strict-mcp-config`, so no MCP-provided tool exists
  to be reached;
- denies every built-in tool explicitly, not a subset left unlisted — naming the full current
  builtin set is defense in depth beneath the isolated-directory guarantee above, not a
  substitute for it — and never sets `--dangerously-skip-permissions`,
  `--allow-dangerously-skip-permissions`, or a `--permission-mode` of `bypassPermissions`,
  `acceptEdits`, or `auto`;
- requests `--output-format json` over one `--print` turn: one request, one response, no
  session state, no follow-up turn where an earlier refusal could be renegotiated.

`TaskEnvelope.available_tools` is not yet a way to grant any of this back. Until a real
`CapabilityId` names a tool this boundary actually permits, and something enforces the field
at the invocation this record governs, an empty structural boundary is what every invocation
gets — the same "nothing enumerated, nothing granted" reading `available_tools: vec![]`
already has as a value.

## What This Record Does Not Do

It does not decide whether a plugin, a transport, or any executor other than the one
dispatching `TaskEnvelope`/`WorkResult` carries the same rule. `OD-CONNECTOR-001`'s own "each
is a different boundary with its own shape" stands; this record answers only the question it
named for an agent executor specifically.

It does not build a permission-granting mechanism. `TaskEnvelope.available_tools` staying
unenforced is a real gap this record does not close — when a real task needs a real tool,
naming what enforces that field and how a grant is checked is a later record's question,
measured against a real need rather than designed ahead of one.

It does not touch `nomos-corrections`'s own stage/validate/commit chain, `OD-CORRECTIONS-001`,
or any `AGT-*` contract type. It adds one constraint at one seam: what the subprocess a
`ProcessLauncher`-based executor starts is permitted to do, decided before its own response is
ever read rather than inferred from what that response later claims.

## Status

Accepted.
