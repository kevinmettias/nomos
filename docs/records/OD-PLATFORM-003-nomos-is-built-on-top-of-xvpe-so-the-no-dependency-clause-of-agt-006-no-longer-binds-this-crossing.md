---
id: OD-PLATFORM-003
type: decision
title: Nomos is built on top of XVPE, so AGT-006's no-dependency clause no longer binds this crossing
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - platform
  - architecture
  - traceability
  - agent
relations:
  - target: AGT-006
    type: affects
  - target: OD-ROADMAP-005
    type: relates-to
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

## Amendment, Version 2: The Clock Type Comes Back, Because A Port That Cannot Compile Without Its Engine Has The Dependency Backwards

`OD-ROADMAP-005` decision 4 authorizes this and states its own bound: this record is superseded
only where its retirement of `AGT-006`'s no-dependency clause reaches `nomos-platform` itself.
Everything above stands, and the reasoning that produced it was never the objection.

### What was measured

At `53de19a4`, and unchanged at `a50bf332`:
`crates/platform/nomos-platform/src/clock/timestamp.rs` re-exported `xvpe_clock::Timestamp` and
the crate's manifest named `xvpe-clock`. So the ports crate every band above depends on could
not compile without the engine it is a port *to*. An external architecture review of `dev`
called that a reverse boundary leak, and it is one: the seam this crate exists to be had a
dependency running through it the wrong way, and every band above inherited it.

The consumer census over `crates/` and `tests/` is what decided the returning type's surface,
rather than a reading of the one being replaced: 88 files name `Timestamp`;
`Timestamp::From_Unix_Seconds` is called 153 times and `Timestamp::Plus` once as an associated
function; the value methods are `Unix_Seconds` 19 times, `Plus` 10 and `Since` 4; no `const` or
`static` of the type exists, nothing keys a map or a set by it, and no crate implements a trait
for it. `nomos-platform` was the only crate in the workspace naming `xvpe-clock`, and neither
`README.md` nor `nomos-architecture.json` mentioned it, so nothing outside that one manifest
asserted the arrangement being undone.

### What moved, and what did not

The declaration moved, and nothing else. `nomos_platform::Timestamp` is the same path with the
same operations, so no consumer was edited and none learned -- which is what the seam bought on
the way down, bought again on the way back. The ledger's wire format never moved in either
direction: it was always this workspace's own, `timestamp_serde` still holds it, and
`Test_A_Timestamp_Should_Serialize_As_A_Bare_Number_Of_Seconds` still asserts the bytes every
committed ledger carries.

**The crossing is untouched.** It stays adopted and stays pinned at
`82a3c8fccf4ef7f3759f36d3f320a91d0f96341c`, declared once in `[workspace.dependencies]` and
inherited by the eight members that name an `xvpe-` crate, which is one fewer than before.
`nomos-platform-xvpe` is still the adapter and still the narrowest crate that must reach XVPE.
Nothing here makes XVPE a peer again: Nomos remains an application over it, which is what this
record decided and what `OD-ROADMAP-005` decision 4 repeats in terms.

**A conversion in the adapter was considered and refused.** Naming `xvpe-clock` in
`nomos-platform-xvpe` and converting between the two declarations would have kept them joined,
and it was the smaller edit to `Cargo.lock` -- one insertion and one deletion rather than 112
deletions. It buys nothing. No caller converts, and an adapter carrying a dependency and
conversions nothing calls is exactly the claim about the future this crate's own module
documentation refuses. The lock was going to change either way, because it records a per-member
dependency array and a move is an edit to two of them, so the argument from lock stability did
not survive being measured.

### What it cost, which is not what the section above says it costs

**`What This Costs` is wrong, and was already wrong before this amendment.** It says this
workspace no longer builds standalone and requires XVPE checked out beside it, because the
dependency is a path dependency across repositories. Neither clause is true. Every `xvpe-`
dependency is a git source pinned to a revision, so the workspace resolves XVPE from what that
revision pins and needs no sibling directory to build. `AGENTS.md` says so in its operating
hazards, `OD-PLATFORM-004` made the local substitution opt-in and denied it any authority, and
`Test_The_Committed_Lock_Should_Pin_Every_Crossing_Package_To_That_Revision` is what keeps the
committed form honest. The path-dependency arrangement that paragraph describes stopped being
the governing one when the crossing was pinned, and the sentence was left behind.

What the crossing does cost, stated the way that paragraph meant to state it: a build that has
not fetched the pinned revision cannot compile the members that name it, and a revision bump is
a decision taken once in the root manifest rather than a refresh. That is unchanged here.

What *this* change cost is one number. `Cargo.lock` lost 112 lines: the package blocks for
`xvpe-clock` itself and for `web-time`, `js-sys`, `wasm-bindgen` with its three macro crates,
`rustversion`, `slab` and the three `futures` crates, all of which `xvpe-clock` alone pulled
into this workspace. Twelve `xvpe-` packages in the committed lock became eleven, and all
eleven still carry a `git+` source at the one revision the manifests declare.

Checked 2026-09-21 against the crate's manifest and module documentation, the committed lock before and after the change, `cargo tree -p nomos-platform`, and the blessed surface snapshot at `tests/contract/surface/nomos-platform.txt`.
