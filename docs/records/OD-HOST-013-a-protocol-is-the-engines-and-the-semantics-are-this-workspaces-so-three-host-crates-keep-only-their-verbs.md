---
id: OD-HOST-013
type: decision
title: A protocol is the engine's and the semantics are this workspace's, so three host crates keep only their verbs
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - host
  - api
  - mcp
  - lsp
  - architecture
relations:
  - target: OD-PLATFORM-003
    type: relates-to
  - target: OD-HOST-007
    type: affects
  - target: OD-HOST-003
    type: affects
  - target: OD-HOST-010
    type: relates-to
  - target: AGT-006
    type: relates-to
---

# A protocol is the engine's and the semantics are this workspace's, so three host crates keep only their verbs

## Question

`OD-PLATFORM-003` settled that this workspace is an application over XVPE and moved the agent
dispatch down on that basis. It left a test behind it: **what else up here was never this
workspace's?**

Three crates were the obvious candidates and were nearly left alone. `nomos-api-transport`
held a complete JSON-RPC 2.0 implementation, `nomos-mcp` a complete MCP server, and
`nomos-lsp` a complete language server — all three general, none of them about code
conformance, and **none of them with an XVPE consumer asking for them**.

Is "no consumer over there yet" a reason to keep a general capability up here?

## Decision

**No. What decides is what the thing *is*, not who currently calls it.**

A protocol belongs to the engine. What this workspace knows — which verbs it serves, what a
tool means, which severity one of its own rule categories deserves — stays here. The line
runs between *carrying a message* and *knowing what the message says*, and all three crates
had both halves welded together.

The three protocol implementations moved to XVPE on 2026-09-11, into four crates:

| XVPE crate | What it took |
|---|---|
| `xvpe-remote-call` | The contract: `RemoteCallStrategy`, `ToolCatalogStrategy`, the five reserved refusal codes, `ToolDescriptor` / `ToolAnswer` / `ServerIdentity` |
| `xvpe-remote-call-backend-json` | The JSON-RPC 2.0 envelope, the one-object-per-line framing, the tool handshake (`initialize`, `tools/list`, `tools/call`, `ping`), notification suppression, the socket and its accept loop |
| `xvpe-diagnostics` | `SourceDiagnostic`, the four severities, `DiagnosticProviderStrategy` |
| `xvpe-language-server-backend-lsp` | The handshake, workspace-root resolution, `file://` conversion, whole-line spans, and stale-marker clearing |

## What each crate here kept

- **`nomos-api-transport`** — `ServedMethod` (the admitted verbs, which are exactly what
  `ServedMethod::REGISTRY` declares), a parameter type per admitted operation, and
  `NomosApiDispatch`, which is the dispatch into `nomos_api::Handle_*`. It is an
  `xvpe_remote_call::RemoteCallStrategy`.
- **`nomos-mcp`** — `ServedTool`: which tools exist, which is `ServedMethod` projected rather
  than a second list of its own --
  `Test_The_Tool_Registry_Should_Name_The_Same_Operations_As_The_Served_Method_Registry` holds
  the two equal by name and in order -- the sentence each publishes and the JSON Schema each
  accepts. It is an `xvpe_remote_call::ToolCatalogStrategy`, and `src/main.rs`
  hands it to the engine's `Serve_Tools` over this process's own pipes.
- **`nomos-lsp`** — the walk, the judgement, `Diagnostics_For`, `WalkOutward`, and
  `severity::Severity_Of`. That last one is the clearest case for the line this record draws:
  what `GateCategory::Advisory` *deserves* is a fact about this workspace's rule taxonomy and
  no shared vocabulary could know it, which is exactly why `xvpe-diagnostics` declines to
  decide it.

## `OD-HOST-007` is unaffected, and is now cheaper to keep

That record required the repo-tooling exclusion to be **structural rather than advisory**.
It still is, and in one fewer place: `ServedMethod::REGISTRY` is the single list, `nomos-mcp`
projects it rather than keeping a second one, and
`Test_The_Transport_Should_Name_No_Repo_Tooling_Handler` still reads `nomos-api`'s own blessed
surface. Nothing about which handlers are reachable changed.

## What this closed that was not the point

**The framing was written twice.** `nomos-api-transport` and `nomos-mcp` each carried their
own line loop and a byte-identical `UNRENDERABLE` constant, because the two surfaces sat side
by side with no shared floor under them. There is one of each now, one repository down.

**A call was travelling as a wire line between two crates in one process.** `nomos-mcp`
reached its sibling by re-serializing every `tools/call` into a synthetic JSON-RPC line and
handing it to that crate's own parser — a round trip through a wire format neither side was
reading off a wire, which existed only because the two had no contract to meet at. They have
one now, and a call is a call.

## What it cost

**`nomos-lsp` no longer names a protocol library.** `lsp-server` and `lsp-types` are gone from
this workspace's manifest entirely. That is a real reduction and also a real coupling: the
editor surface now cannot be built without XVPE checked out beside this repository —
`OD-PLATFORM-003` already accepted that for the workspace as a whole, and this extends it to
the one host that could previously have been built alone.

**Three public surfaces changed and were re-blessed.** The wire types
(`WireRequest`/`WireResponse`/`WireError`), `Answer`, `Serve`, `Serve_Connection`,
`PROTOCOL_VERSION`, `ServedTool::Listing`, `FileDiagnostic` and `Run_Server` are all gone from
this workspace's exports.

## A guard this repaired

`Test_Every_Declaration_Should_Be_Held_To_It_By_The_Harness` failed on the three new
`impl Strategy` blocks, because `tests/contract/src/fact_domain.rs` matches that text without
being able to tell `xvpe_primitives::Strategy` from `nomos_contracts::Strategy` — two traits
that share a name and three axes. The scanner now reads which vocabulary a file speaks from
its own imports.

Fixing it exposed a second defect nobody had noticed: the scanner only matched a bare
`impl Strategy for X`, so **every generic declaration had been invisible to it since it was
written** — `nomos-platform-xvpe`'s own `impl<Launcher: ProcessLauncher> Strategy for
XvpeLauncher<'_, Launcher>` among them. Both directions of that completeness check had been
quietly under-quantified.

## Consequences

- The three crates are adapters over engine protocol surfaces. A change to the wire, the
  framing, the handshake or the editor loop is made in XVPE, not here.
- `lsp-server` and `lsp-types` are removed from `Cargo.toml`'s workspace dependencies.
- `tests/contract/src/fact_domain.rs` scopes its scan to this workspace's own determinism
  vocabulary, and sees generic implementations.
- `tests/contract/tests/boundaries/mcp_registry.rs` still holds: `nomos-mcp` depends on
  `nomos-api-transport` and no other workspace member.

## Amendment, Version 2: Two Kept-Surface Entries Named A Size And A Type That Are Both Wrong

Version 1's account of what each crate kept said `ServedMethod` is "the four admitted verbs"
and `ServedTool` is "which four tools exist", and named the transport's dispatch
`NomosApiService`. The dispatch is `NomosApiDispatch` and always was, so that half was wrong
when written. The two counts were right when written and were made false by `f0052c39`, which
admitted `nomos.check.run` and took both registries to six.

The division this section draws is untouched: the transport keeps the verbs and the dispatch,
`nomos-mcp` keeps the catalogue, `nomos-lsp` keeps the judgement. What is removed is the size,
which this record never needed and which no test renders back into it. The entries now name the
registry that declares the set, and cite
`Test_The_Tool_Registry_Should_Name_The_Same_Operations_As_The_Served_Method_Registry` as the
mechanical authority for the second registry being a projection of the first rather than a copy
that can drift from it.

Amended by `P123-FOUR-HOST-RECORDS-FROZE-A-REGISTRY-SIZE-INTO-PROSE-AND-A-SIXTH-OPERATION-MADE-THEM-FALSE`.
