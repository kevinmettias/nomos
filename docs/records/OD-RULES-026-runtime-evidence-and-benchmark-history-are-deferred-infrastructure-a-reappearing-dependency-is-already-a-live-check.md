---
id: OD-RULES-026
type: decision
title: Runtime evidence and benchmark history are deferred infrastructure; a reappearing dependency is already a live check
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - history
  - runtime
  - dependency
  - fact-store
relations:
  - target: OD-STORE-002
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-GATE-015
    type: relates-to
  - target: OD-RULES-003
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
---

# Runtime evidence and benchmark history are deferred infrastructure; a reappearing dependency is already a live check

## Question

`P42-RUNTIME-AND-HISTORY-RULES` claimed every rule in this workspace judges the tree as it
is now, and named three worked examples of what would be different: a declared
zero-allocation path judged against runtime allocation evidence, a performance requirement
judged against benchmark history, and a previously removed dependency reappearing as an
architectural regression. Its own `why` stated the fact substrate's generations are "most
of what a historical claim needs." Its territory was one file,
`crates/rules/nomos-rules/src/checks/runtime_history.rs`. Whether any of the three is a
real, narrow, reproducible increment — rather than a request to invent a category of
infrastructure this workspace has no instance of — is the question this record answers.

## What was measured

**The item's own premise about generations is substantially overstated.**
`nomos_contracts::GenerationId` (`crates/contracts/nomos-contracts/src/digest128/generation_id.rs`)
is a bare `u64` with no serialization anywhere in its file. `MemoryFactStore`
(`crates/substrate/nomos-analysis/src/fact/memory_fact_store.rs`) holds every entry in
in-process `BTreeMap`s with no disk I/O anywhere in the type. Generations exist to let one
run's own invalidation walk know which of its own already-computed facts are stale after a
mid-run edit; nothing survives past the process that built it. `docs/records/OD-STORE-002`
already states this independently and predates this item: `MemoryFactStore` "is in-process
and generation-scoped" and nothing in it "survives past the process that built it or
reaches back across a git revision boundary." The fact substrate supplies none of what a
historical claim needs; it supplies the opposite guarantee.

**Runtime allocation evidence has no observation mechanism to build on, and collides with
an already-drawn boundary.** `nomos_platform::ProcessLauncher::Run`
(`crates/platform/nomos-platform/src/process_launcher.rs`) runs a command to completion and
captures its output — one-shot, no attach, no stream, no instrumentation hook. Every
provider crate under `crates/languages/` (`nomos-lang-rust`, `-scan`, `-cargo`, `-clippy`,
`-deny`, `-compiler`, `-go`, `-go-modules`) either parses source, reads a manifest,
consults the compiler for resolved semantics, or relays another static tool's report —
none executes the judged repository's own binary or tests and observes it running. No
allocator counter, profiler, or `valgrind`/`heaptrack` integration exists anywhere in this
tree. Beyond the absence of any starting point, `docs/records/ARC-ROADMAP-001` already
places "runtime/debug intelligence" and "DBG (debugger integration)" under "Deferred
consuming systems," not Nomos Core's near-term boundary — this example asks for work this
workspace has already decided comes later.

**Benchmark history has no first-class concept to build on.** `Cargo.lock` carries no
`criterion` dependency, and nothing in the workspace references a `target/criterion/`
convention or any benchmark-results format. There is no persisted-history mechanism to
read from and none to extend; building one means inventing both a benchmarking convention
and a results store with nothing here to build on.

**A previously-removed dependency reappearing is not a gap. It is already a live,
gate-composed check, discovered under a different name.** `BaselineDebt`/`BaselinePolicy`
(`crates/orchestration/nomos-gate-orchestration/src/policy/baseline_debt*.rs`) was the
expected template — a committed baseline compared against current state — but
`docs/records/OD-GATE-015` states directly that "No CLI flag or configuration file
constructs a `BaselineDebt` yet"; it is an in-memory tolerate-list with no persistence, not
the pattern this example needs. What already exists instead: `deny.toml`'s `[bans]` section
carries a committed, rationale-documented `deny = [...]` list (`wgpu`, `egui`, `winit`,
`ash`, `naga`) — a deliberate "this must never come back" declaration. `nomos-lang-rust-deny`
runs `cargo deny check bans` (network-free, deterministic: reads only `Cargo.lock` and
`deny.toml`) and materializes its violations as `nomos.cap.dependency.policy` facts.
`crates/rules/nomos-rules/src/checks/policy.rs`'s `Check_Dependency_Policy` already relays
every violation as a `Finding`, and it is already composed into a real run —
`crates/orchestration/nomos-check-orchestration/src/run_context.rs`'s `ComposedRule { id:
DEPENDENCY_POLICY, ... }`. If any banned crate reappeared in the resolved graph today, an
ordinary `nomos check` run would already report it, reproducibly, with no network call and
no git query.

**What this exposes is that the third example, examined honestly, was never actually about
history.** The mechanism above never asks "did this reappear" — it asks "is a banned name
present now," a static comparison between the current resolved graph and a currently
committed declared list. That is the identical declared-constraint-vs-observed-fact shape
`dependency-direction` and `write-authority` already prove, not evidence "from a run or
from the past" as the item's own `why` distinguishes it. The only way to make this example
genuinely about history — an automatic diff against git's own record of what was removed
and when — is exactly the live `git log`/`git diff` query `OD-STORE-002` and
`crates/host/nomos-surface-provenance` already decline to wire as a gate input, for the
reproducibility reason both give: `.github/workflows/gate.yml`'s `actions/checkout` sets no
`fetch-depth`, so CI runs the default shallow checkout, and a live history query inside a
gate rule would be exactly as non-reproducible as those records already warn.

## The verdict

None of the three worked examples is a real, narrow, reproducible increment buildable
today. Two (runtime allocation evidence, benchmark history) require inventing an entire
evidence-capture category this workspace has no instance of, and the first of those two
additionally reaches into territory `ARC-ROADMAP-001` already deferred on purpose. The
third (a reappearing dependency) is not a gap at all — it is already shipped as
`dependency-policy`, composed and gate-reachable, and on inspection its evidence was never
actually historical; forcing a second, weaker, hand-rolled version of it under this item
would manufacture the appearance of new work without delivering what the item's own `why`
asked for: evidence from a run or from the past, rather than from current source.

## What this record does not do

It does not close the door on runtime or historical evidence permanently.
`ARC-ROADMAP-001` already names a path for runtime/debug intelligence, on its own
schedule, once Nomos Core's near-term tier is further along. A benchmark-history
capability becomes reachable the moment this workspace adopts a benchmarking convention
for its own purposes, at which point a comparable `WholeWorkspace` fact and rule, shaped
like `dependency-policy`, is a real next step. Nothing here forecloses a future, explicit
decision to accept `git log` as a CI-time input once a non-shallow checkout or an
equivalent reproducibility fix exists; `OD-STORE-002`'s own rule already gives the shape
such a decision would need to satisfy.

It does not touch `crates/rules/nomos-rules/src/checks/policy.rs`, `deny.toml`, or any
composed rule. Nothing described as already built here needed a change to be found; this
record only names what already exists and traces the item's own examples against it.

## Status

Accepted. `P42-RUNTIME-AND-HISTORY-RULES`'s single-file territory
(`crates/rules/nomos-rules/src/checks/runtime_history.rs`) is not built. Its own premise
about the fact substrate's generations is corrected against the real code and against
`OD-STORE-002`, which already settled the question before this item was written. Two of
its three worked examples need infrastructure categories with no instance in this
workspace; the third is already a live, composed, reproducible check
(`dependency-policy`/`cargo deny check bans`), whose evidence, read honestly, was never
historical in the first place.
