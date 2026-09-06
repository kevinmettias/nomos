---
id: OD-PROJECT-004
type: decision
title: The tools that govern this repository are separated from the product they govern
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - project
  - layering
  - repository
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-LEDGER-036
    type: relates-to
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-PACKAGE-015
    type: relates-to
---

# The tools that govern this repository are separated from the product they govern

## Question

`README.md` already marks four crates `[repo tooling]` — `nomos-ledger`, `nomos-work-
orchestration`, `nomos-spec-orchestration` and `nomos-surface-provenance` — and states
plainly that the mark "does not move a crate, does not change a dependency, and is not
itself a decision." `OD-LEDGER-036` went further for the ledger specifically, settling
*ownership* (bootstrap machinery, not a Nomos product feature) while explicitly declining
to reopen *location*: "It does not move any file, rename any crate... this record answers
the ownership question... without reopening the location question it explicitly declined
to answer." `ARC-ECOSYSTEM-001` named the same three subsystems — the specification
system, the work ledger, and the contract-test/surface-snapshot/gate-derivation tooling —
and said in advance that nobody may cite their location as proof of ownership, without
deciding what their location should be.

Naming a distinction and enforcing it are different acts. The risk `OD-PACKAGE-015`'s own
why already states for crate boundaries generally applies here specifically: a distinction
that costs nothing to cross gets crossed without anyone deciding to.

## What Was Measured

**The four marked crates sit in three different directories, interleaved with product.**
`nomos-ledger` is under `crates/substrate/`, beside `nomos-capability` and `nomos-analysis`.
`nomos-work-orchestration` and `nomos-spec-orchestration` are under `crates/orchestration/`,
beside `nomos-check-orchestration` and `nomos-gate-orchestration` — real product surfaces.
`nomos-surface-provenance` is under `crates/host/`, beside `nomos-cli` and `nomos-api`. A
reader cannot tell repo tooling from product by directory today; the README mark is the
only signal, and it is prose, not a boundary.

**The specification family already solved this for itself, partially.** Five of its six
crates — `nomos-spec-model`, `nomos-spec-store`, `nomos-spec-bundle`, `nomos-spec-ingest`,
`nomos-spec-validate`, `nomos-spec-project` — live under `crates/spec/`, physically apart
from product. `nomos-spec-orchestration`, the family's own orchestration layer, does not:
it was placed under `crates/orchestration/` instead, grouped by pattern (orchestration
crates sit together) rather than by subject (spec crates sit together), splitting one
family across two directories for a reason nothing records.

**No enforced test exists for either claim.** `tests/contract/tests/boundaries/graph.rs`
already enforces two structurally identical claims — `Test_Only_The_Platform_Adapter_May_
Name_The_Sibling_Workspace` and `Test_No_Crate_May_Name_The_Sibling_Knowledge_Workbench`,
both an allowlist checked against `Workspace::Transitive_Dependencies` — for the sibling
XVPE and KWB workspaces. Grepping the same suite for `nomos_spec` or a repo-tooling name
finds only band-table entries. README's own claim for the specification family — "It
reaches the product only through a knowledge capability, so nothing in the product may
name it" — and the identical claim this record would make for repo tooling are both
unenforced prose today.

**The gap is not hypothetical: three real product crates already depend on repo tooling
for one type that does not require it.** `Territory` is defined in `nomos-scope-
verification` (band 19, the crate `OD-LEDGER-037` extracted specifically so it would be
"ledger-agnostic"), and `nomos-ledger` re-exports it. `nomos-agent-executor-claude-code`,
`nomos-model-backend-ollama` and `nomos-workflow-orchestration` — three genuine product
crates — each carry `nomos-ledger` as a **dev-dependency** solely to reach `Territory` in
their own tests (`use nomos_ledger::Territory;`, five call sites across the three crates),
when `nomos_scope_verification::Territory` would give them the identical type without
naming repo tooling at all. This is exactly the "architectural attention" failure this
item's own why names: nothing broke, nothing shipped wrong, and three crates still reached
for the tool instead of the primitive beneath it, because nothing made the difference cost
anything to cross.

## The Decision

**The four `[repo tooling]` crates move to a new top-level directory, `crates/repo-
tooling/`, naming the distinction physically rather than only in README prose.**
`nomos-ledger` and `nomos-work-orchestration` (repository coordination, no existing
family) and `nomos-surface-provenance` (a standalone report, no existing family) go there.
`nomos-spec-orchestration` goes to `crates/spec/` instead, rejoining the five siblings it
was split from — its subject is the specification family, and `crates/spec/` is already
that family's home; `crates/repo-tooling/` would only re-create the split this record
measured, one directory over. Both moves keep every crate's band, every dependency edge,
and every public type exactly as they are: this is a `path` change in the workspace
manifest and a `git mv`, nothing else.

**It stays one Cargo workspace, not two.** A second workspace buys none of the three
things `OD-PACKAGE-015` asks a boundary to earn: no crate here is independently versioned
or published, no isolation is enforced that a directory plus the test below does not
already give, and `nomos-cli`/`nomos-api` — the two real hosts that legitimately wire both
product and repo-tooling verbs into one binary — would still declare the identical `path`
dependency they declare today, just across a workspace boundary instead of a directory
one, at the cost of a second `Cargo.lock`, a second `cargo build` invocation for CI to
know about, and no dependency this repository does not already have a mechanism to check.

**A product crate is prevented from depending on repo tooling by a fifth graph
assertion, the same shape as the existing two sibling-workspace checks.** An allowlist
(`nomos-cli`, `nomos-api`, and any crate whose own `root` sits under `crates/repo-
tooling/`) is checked against every other member's `Transitive_Dependencies`; any of the
four repo-tooling crate names found outside that allowlist fails the assertion, naming
which crate reached across the line and which tooling crate it named — the identical
report shape `Test_No_Crate_May_Name_The_Sibling_Knowledge_Workbench` already gives.
Unlike `Test_Dependencies_Should_Run_Strictly_Downward`, this check does **not** exempt
dev-dependencies. The band check's own reasoning for exempting them — "a low crate's tests
may reasonably use a higher-level fixture" — is about a layer legitimately reaching for
its own kind of fixture; the measurement above is a different shape, three crates
reaching into repo tooling's own internals in their tests when a substrate primitive
already gives them the type without naming the tool. A dev-dependency on repo tooling is
exactly the failure this record exists to make visible, not a case to carve out of it.

## What This Does Not Do

**No crate moves under this record.** It names the target directory, the one exception
(`nomos-spec-orchestration` to `crates/spec/`, not `crates/repo-tooling/`), the choice to
stay in one workspace, and the shape of the enforcement test; carrying the moves out,
writing the fifth graph assertion, and fixing the three crates found reaching into repo
tooling from a dev-dependency are each a future item's own territory, scoped against this
record's findings rather than re-deriving them.

It does not reopen `OD-LEDGER-036`'s ownership question. That record decided the ledger is
bootstrap machinery, not a product feature; this record answers the location question that
record explicitly left open, for the ledger and its three `[repo tooling]` siblings
together, without disturbing what either record decided about ownership.

It does not decide the specification family's own remaining gap — no enforced test exists
for README's "nothing in the product may name it" claim about `nomos-spec-*` as a whole
either, only measured here as a parallel instance of the same missing mechanism. This
record closes the repo-tooling case its own why names; the specification family's
identical gap is left exactly as open as this record found it, for whichever item takes it
up by name.

It does not decide whether `crates/repo-tooling/` should someday become its own workspace,
should the population or a real independent-versioning need change what `OD-PACKAGE-015`'s
test would find. Nothing measured here shows that need existing today.

## Status

Accepted. Four crates named for a new directory, one of the four redirected to an
existing family directory instead, one Cargo workspace kept, and one graph assertion's
shape stated; no crate moves and no test is written here.
