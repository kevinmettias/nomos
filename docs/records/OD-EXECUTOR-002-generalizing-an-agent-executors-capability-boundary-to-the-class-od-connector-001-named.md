---
id: OD-EXECUTOR-002
type: decision
title: Generalizing an agent executor's capability boundary to the class OD-CONNECTOR-001 named and OD-EXECUTOR-001 explicitly declined to decide
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - executor
  - security
  - contracts
  - architecture
relations:
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-CONNECTOR-001
    type: relates-to
  - target: ARC-CONNECTOR-001
    type: relates-to
  - target: OD-SPEC-009
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
---

# Generalizing an agent executor's capability boundary to the class OD-CONNECTOR-001 named and OD-EXECUTOR-001 explicitly declined to decide

## Question

`OD-CONNECTOR-001` names four boundary shapes and decides none of them: "an executor invoking
a subprocess, a plugin loaded into this workspace, an agent given tool access, or a transport
under `OD-SPEC-009`," stating "each is a different boundary with its own shape" and that
"deciding all four by extension from the connector case would repeat the mistake `ARC-
CONNECTOR-001` already named and refused to make." `OD-EXECUTOR-001` later decided exactly one
of the four — "an agent given tool access," for the real Claude Code `AgentExecutor` — and said
so explicitly in its own "What This Record Does Not Do": "It does not decide whether a plugin,
a transport, or any executor other than the one dispatching `TaskEnvelope`/`WorkResult` carries
the same rule."

Three of the four remain unmeasured. An external architecture review names this directly: one
real instance of the class was decided, not the class itself. This record measures each of the
other three against real code at this repository's current state, the same rigor `OD-EXECUTOR-
001` applied to the one it decided, rather than deciding by analogy from that single instance —
the exact failure mode `OD-CONNECTOR-001` itself named and this record is written to avoid
repeating in the other direction.

## What Was Measured

### An executor invoking a subprocess, for a fixed, non-agentic purpose

This is not hypothetical. `nomos_platform::ProcessLauncher` — the same trait `nomos-agent-
executor` dispatches Claude Code through — has three other real callers today, verified
directly:

- `nomos-lang-rust-cargo::Discover_Workspace` (`crates/languages/nomos-lang-rust-cargo/src/
  metadata.rs`) runs `cargo metadata --format-version 1 --no-deps --all-features` over the
  repository root, invoked from `nomos-check-orchestration::Materialize_Dependencies` on every
  `nomos check` and `nomos gate run`.
- `nomos-ledger::finish::running::Runnable_Predicate` (`crates/substrate/nomos-ledger/src/
  finish/running.rs`) runs an item's own `VerificationPredicate` argv — `cargo test`, `cargo
  build`, or whatever a `work add --` call named — as part of `nomos work finish`.
- `nomos-surface-provenance` (`crates/host/nomos-surface-provenance/src/evaluate.rs`) runs `git`
  to read this repository's own commit history.

Every one of these three shares a property none of `OD-EXECUTOR-001`'s rule addresses because
none of its four conditions apply to them: **the command line is fully determined by this
workspace's own code (or, for `nomos-ledger`, by an argv a session already authored into a
shared, reviewed ledger file) before the process is ever launched, and the invoked program is
not handed a goal it interprets and decides its own actions from.** `cargo metadata`'s argv is a
compile-time constant plus the root path; `git`'s argv is the same; a ledger predicate's argv is
whatever `work add` stored, chosen by a session and visible to every other session and to
review before it is committed. None of the three ever receives natural-language content, a
`TaskEnvelope.goal`, or any input an adversary distinct from this workspace's own contributors
could shape. `OD-EXECUTOR-001`'s rule — isolate the working directory, deny every tool, read
structural denial rather than self-report — exists because Claude Code is asked to interpret a
goal and is capable of choosing actions its invocation does not enumerate in advance; the risk
the rule closes is that latitude. A fixed-argv invocation of a deterministic program has no such
latitude to close: there is no decision for the process to make that its own argv does not
already make for it.

**The rule for this shape is different, and it is not new: this workspace already trusts fixed
invocations of its own build tooling everywhere else.** `cargo build`, `cargo test`, `cargo
clippy` and the gate's own predicates all run as ordinary local commands with the ambient
permissions of the machine running them — the same trust level this repository already extends
to every other tool its own CI and its own contributors already depend on to build and verify
it. Requiring `OD-EXECUTOR-001`'s isolation for `cargo metadata` would mean isolating it from
the very repository root it exists to read, which is not a stricter boundary — it is a
different function.

**The decision: `OD-EXECUTOR-001`'s structural-absence rule governs a subprocess given an
open-ended goal it interprets (an agentic invocation). It does not govern, and does not need to
be extended to, a subprocess invoked with a fixed, fully-predetermined argv and no interpretive
latitude (a deterministic invocation) — that shape is already governed by the same ordinary
build-tooling trust this workspace extends to every dependency its own build already requires,
and no further rule is needed until a real deterministic invocation is shown to accept
attacker-influenced argument content, which none of the three named above does.**

### A plugin loaded into this workspace

Checked directly, not assumed: a workspace-wide search for `libloading`, `dlopen`, `Library::
new`, `.dll` and `.so` as a runtime-loaded artifact returns no matches anywhere under `crates/`.
Every "plugin" this workspace has — `nomos-lang-rust`, and the language/rule/package provider
shape `.claude/skills/nomos-add-plugin` describes for a future second one — is an ordinary Rust
crate, compiled and statically linked into the same binary as the code that calls it. There is
no separate process, no separate address space, and no ABI boundary of any kind between a
"plugin" and the composition root that calls it — `OD-CONNECTOR-001`'s own reasoning already
names why this differs in kind from a connector: "a plugin's ABI is not a translation layer
between a foreign schema and a canonical one." A capability boundary, in the sense `OD-EXECUTOR-
001` and `ARC-CONNECTOR-001` both use the term, restricts what a separately-invoked, separately-
trusted thing may do; a statically-linked crate is not a separately-trusted thing, it is this
program.

**The decision: this shape has no real referent today, and the question `OD-CONNECTOR-001`
deferred does not yet have a case to decide against.** This is not an extension of `OD-EXECUTOR-
001`'s rule by assumption — it is the opposite: measuring finds no process boundary here to
extend anything to. The trigger that would fire it is named, not left implicit: a plugin
mechanism that loads code across a real process, address-space, or ABI boundary — a `.so`/
`.dll` loaded at runtime, a WASM sandbox, or a plugin invoked as its own subprocess — rather
than a Rust crate linked into this binary. Should one arrive, it decides this question the way
`OD-EXECUTOR-001` decided its own: measured against the real mechanism, not inferred from this
record by analogy.

### A transport under `OD-SPEC-009`

`OD-SPEC-009` already states a rule for every submission-intake surface — CLI, API, MCP or form
— shaped identically to the write-omission mechanism `OD-CONNECTOR-001` and `OD-EXECUTOR-001`
both use: "No surface is that door. A form, a CLI verb, an HTTP endpoint and an MCP tool are
*transports*: each constructs the typed submission from whatever it collects, hands it to the
accept function, and renders the verdict it gets back. A transport that validates, defaults or
persists on its own behalf is a second write door and is a defect, not a variant." That is the
same shape as `OD-CONNECTOR-001`'s "the enforcement is the absence of the capability, not a
permission check performed when [it] is attempted": a transport never holds the capability to
write on its own account, structurally, the same way a connector's interface never holds the
vendor's write methods and an executor's invocation never holds a granted tool.

`OD-CONNECTOR-001` itself flagged the gap precisely: "an MCP transport already answers to
`OD-SPEC-009` on the inbound side and has not been asked an outbound question." Checked
directly against `OD-SPEC-009`'s own text: its rule is not direction-scoped — "no surface is
that door" binds every transport's own persistence capability regardless of which direction
data is said to move, because a submission transport that persisted directly would be writing
outward from the surface's own code exactly as an outbound mutation would. There is no second,
undecided "outbound" question left over — `OD-SPEC-009`'s single-accept-function rule already
closes it, for the two real transports this workspace has (`nomos-cli`, `nomos-api`) and for any
future one, the same way it already closes the inbound question it was written for.

**The decision: `OD-SPEC-009` already answers this. No new rule is written; this record
supplies the citation connecting the two, which neither record made on its own.**

## What This Record Does Not Do

It does not change `OD-EXECUTOR-001`'s rule, its scope, or the invocation it governs. The Claude
Code `AgentExecutor` is unaffected.

It does not build a plugin sandbox, a WASM host, or any dynamic-loading mechanism. It states the
condition that would make the deferred plugin question real, and builds nothing toward it ahead
of that condition, the same restraint `OD-HOST-004` and `D-135` both already name for this
workspace.

It does not add enforcement to `nomos-lang-rust-cargo`, `nomos-ledger`, or `nomos-surface-
provenance`. Their existing, unrestricted `ProcessLauncher` use is the shape this record found
correct, not a gap it closes.

It does not revisit `OD-SPEC-009`'s own rule or extend it — it cites what that record already
decided rather than restating or amending it.

It does not name every possible future subprocess shape this workspace could ever add. The
criterion it states — agentic (interprets a goal, decides its own actions) versus deterministic
(fixed argv, no interpretive latitude) — is what a new subprocess caller checks itself against;
one that is agentic needs `OD-EXECUTOR-001`'s own rule applied to it directly (or a successor
record, if its shape genuinely differs), not an inference from this one.

## Status

Accepted. `OD-CONNECTOR-001`'s four-item deferred list is now fully measured: one decided by
`OD-EXECUTOR-001` (agent given tool access), one decided here as already covered by an existing
record (`OD-SPEC-009`'s transport), one decided here as governed by ordinary build-tooling trust
rather than `OD-EXECUTOR-001`'s rule (deterministic subprocess), and one found to have no real
referent yet (plugin), with its firing condition named. Revisit the plugin case when a real
out-of-process plugin mechanism is proposed or built; revisit the deterministic-subprocess case
if any of the three named callers, or a future one, is ever given attacker-influenced argument
content.
