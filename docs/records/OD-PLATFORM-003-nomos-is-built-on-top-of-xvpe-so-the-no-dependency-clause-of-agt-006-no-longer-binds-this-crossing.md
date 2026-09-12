---
id: OD-PLATFORM-003
type: decision
title: Nomos is built on top of XVPE, so AGT-006's no-dependency clause no longer binds this crossing
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - platform
  - architecture
  - traceability
  - agent
relations:
  - target: AGT-006
    type: affects
  - target: OD-PLATFORM-001
    type: relates-to
  - target: OD-EXECUTOR-001
    type: affects
  - target: OD-EXECUTOR-004
    type: affects
---

# Nomos is built on top of XVPE, so AGT-006's no-dependency clause no longer binds this crossing

## Question

`AGT-006` states: *"Neither system shall depend on the other at runtime for its core
function; integration shall occur through neutral versioned contracts and MCP tools."* It
was assessed `Met`, and mechanically enforced by
`Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace`, which failed the build if
any crate's transitive dependencies reached an `xvpe-` prefix outside one named adapter.

That test's own comment recorded the intent precisely: when an adapter arrived it would be
"named here explicitly so that the exception is a decision rather than an oversight."

The decision has now been asked for. Does the no-dependency clause still bind?

## Decision

**No, for the XVPE crossing. Nomos is built on top of XVPE.**

XVPE is the engine. This workspace is an application over it, in the same sense the
knowledge workbench is. The clause was written on the premise of two peer systems that must
not entangle; that is not the relationship these two have.

The clause is **retired for this crossing only**. AGT-006's other two clauses —
integration through neutral versioned contracts, and an MCP tool surface — are unaffected
and remain true: `nomos-contracts` is still band 0 with no dependencies, and `nomos-mcp` is
still a real MCP server over stdio.

**The KWB crossing is untouched.** `Test_No_Crate_May_Name_The_Sibling_Knowledge_Workbench`
still fails the build if any crate reaches a `kwb-` prefix, with an empty
`KNOWLEDGE_ADAPTER`. Nothing here says this workspace may name `kwb-`; both systems being
built on XVPE says nothing about either naming the other.

## What Moved, And Why It Had To

The agent dispatch this workspace had built — a freshly created isolated working directory,
an allow-list granting nothing real rather than a deny-list that is always one release
behind the real capability set, one non-interactive turn, a spend ceiling verified to abort
before the expensive model call, a wall bound that kills, and an answer read from
schema-validated `structured_output` rather than from free text — moved down into
`xvpe-agent-backend-claude-code` and `xvpe-agent-backend-ollama`, over a capability
contract in `xvpe-agent-execution`.

It had to move because a general capability sitting up here was **unreachable by everything
down there**. XVPE's own reference miner had a three-line agent launch with no bound, no
spend ceiling, and inherited rather than captured output — a strictly worse version of
something this workspace had already built well, which it could not reach.

The same argument applies to the process port underneath it, which moved to `xvpe-process`
and `xvpe-process-backend-system`.

## Why The Test Was Deleted Rather Than Widened

Nine crates now reach `xvpe-` transitively: both agent adapters, `nomos-agent-orchestration`,
`nomos-workflow-orchestration`, `nomos-api-transport`, and the three hosts.

Adding nine names to `PLATFORM_ADAPTER` would have left a rule that still *read* as a
boundary while enforcing nothing. That is worse than no rule: the next reader takes it for a
constraint that holds, and the allow-list becomes the place a failing test goes to die. A
retired rule states its own retirement; a gutted one lies.

## What This Costs

**This workspace no longer builds standalone.** It requires XVPE checked out beside it,
because XVPE is not published and the dependency is a path dependency across repositories.
That is a real cost, accepted deliberately rather than discovered later.

## Consequences

- `AGT-006` is assessed `Diverges`, governed by this record.
- `Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` and `PLATFORM_ADAPTER`
  are deleted from `tests/contract/tests/boundaries/graph.rs`, with a comment in their place
  recording what was there and why it is not.
- `nomos-platform-xvpe` exists as the launcher bridge, in the Substrate zone. It is **one
  adapter, not a replacement** for `nomos-platform`'s port: adopting XVPE's launcher trait
  workspace-wide would mean touching 33 implementors and 559 use-sites for no behavioural
  change.
