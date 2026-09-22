---
id: OD-PROJECT-004
type: decision
title: The tools that govern this repository are separated from the product they govern
status: accepted
version: 2
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
  - target: OD-RULES-020
    type: relates-to
  - target: OD-RULES-024
    type: relates-to
  - target: OD-RULES-028
    type: relates-to
  - target: OD-RULES-029
    type: relates-to
  - target: OD-HOST-006
    type: relates-to
  - target: OD-AGENT-004
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

## Amendment, Version 2: Neither Mechanism Was Built, And The Zone Declaration Is The Guard

Checked 2026-09-21, sixteen days after version 1 landed at `4c18388f`. The two injected-edge
runs below were taken in a detached worktree at `fc941fe2`; every other claim was re-read
against the tree this amendment commits into.

### What version 1 decided was never built

`crates/repo-tooling/` does not exist. The four crates sit exactly where version 1 measured
them: `nomos-ledger` under `crates/substrate/`, `nomos-work-orchestration` and
`nomos-spec-orchestration` under `crates/orchestration/`, `nomos-surface-provenance` under
`crates/host/`. `tests/contract/tests/boundaries/graph.rs` holds eight tests and none of them
names a tooling crate: the vacuity guard, the `nomos-contracts` allowlist, the
knowledge-workbench exclusion, the zone-and-edge check, two that every declared same-zone edge
is a real dependency between two members of one zone, one that every write door is real, and
one that every member declares a band. `P41-REPO-TOOLING-SEPARATION`, the item that produced
version 1, had a `done_when` ending "No crate moves under this item", is Done on exactly that,
and no successor was ever authored for the moves or the assertion. Version 1's "What This Does
Not Do" handed each to "a future item's own territory", and no item took it.

An external review of `bc0aaac` read this record as a built guard. That is the reading this
amendment exists to correct: a record naming a directory and a test a reader cannot find
describes an intention, and it has to say which.

Two of version 1's own citations have also moved under it. The "existing two
sibling-workspace checks" the fifth assertion was to copy are now one.
`Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` does not resolve: it was
retired on 2026-09-10 because this workspace is built on XVPE and naming an `xvpe-` crate
stopped being an exception, and `graph.rs` carries the tombstone where it stood.
`Test_No_Crate_May_Name_The_Sibling_Knowledge_Workbench` is the one that remains. And the
dev-dependency population version 1 measured has changed shape, below.

### What arrived instead, and at what severity

The boundary version 1 wanted a directory to make visible is declared as data at the
repository root, and two readers judge it.

`nomos-architecture.json` is the declaration `OD-RULES-029` decided is read from the
repository under check, and `OD-RULES-024` records as built on 2026-09-14:
`nomos.cap.architecture.declaration`, its contract in
`crates/capabilities/nomos-cap-architecture`, one provider in `nomos-repo-policy`'s
`architecture` module reading that file, and no component name or crate name of this
workspace's surviving in `nomos-rules`. Its `members` place `nomos-ledger`,
`nomos-work-orchestration` and `nomos-surface-provenance` in `Repo Tooling`, and all seven
`nomos-spec-*` crates, `nomos-spec-orchestration` included, in `Specification`. Its `permits`
table admits each of those two zones to exactly two consumers, `Host` and `Verification`, and
no other zone's row names either. So the edge version 1's assertion was to refuse, a product
crate depending on a tooling crate, is refused by the declaration for every product zone at
once; and the family split version 1 measured is healed in the declaration whatever directory
`nomos-spec-orchestration` sits in.

The severity was verified by injection rather than read off `OD-RULES-028`. In a detached
worktree, `nomos-ledger` and `nomos-spec-store` were added to `nomos-model-backend-ollama`'s
dependency table -- an Agent crate, from version 1's own measured population -- and two readers
were run over that tree and none over the shared one:

- `nomos check` over that root reported two findings, `[Advisory] dependency-direction`
  naming `nomos-model-backend-ollama (Agent) depends on nomos-ledger (Repo Tooling)` and the
  same sentence for `nomos-spec-store (Specification)`, in a run of 463 findings whose own
  summary line reports none of which can fail a build, and exited 0. `OD-RULES-028`'s Advisory
  is still true, and it is now a `GateCategory::Advisory` literal in the rule's `violations`
  module rather than a table entry.
- `cargo test -p nomos-contract-tests --test boundaries` over that tree failed:
  `Test_Dependencies_Should_Run_Strictly_Downward` panicked with
  `nomos-model-backend-ollama (Agent) depends on nomos-ledger (Repo Tooling).` and the process
  exited 101. That test reads the same file through `Declared_Architecture`, so `gate.yml`'s
  Test step, which runs the workspace test suite, and its Boundaries step, which runs the
  contract crate's, both go red on the edge.

The worktree was reset afterwards. Nothing was injected into the shared tree.

**Blocking in effect, Advisory by name.** A product-to-tooling dependency edge cannot merge,
because the gate fails through its Test and Boundaries steps. What is Advisory is the rule's own
report of it in `nomos check`, which is the arrangement `OD-RULES-028` recorded for the
`Backend` zone and which applies to every zone crossing alike, because one rule judges every
row of `permits`. Whether `dependency-direction` should itself be Blocking is a question about
that rule's contract for every zone at once, and this record does not decide it for one.

**Today no product crate crosses either line.** Every manifest under `crates/` outside
`crates/spec/` and `crates/host/` was read for a `nomos-spec-` dependency: only
`nomos-spec-orchestration`'s own manifest names a sibling, and neither
`nomos-check-orchestration` nor `nomos-gate-orchestration`, the crates behind `nomos check` and
`nomos gate`, names one at all. Outside `crates/host/`, the only dependency edge into a Repo
Tooling crate is `nomos-work-orchestration` onto `nomos-ledger`, the declared same-zone
exception. The boundary holds in fact and is guarded by the declaration.

### The dev-dependency clause is withdrawn

Version 1 refused to exempt dev-dependencies, on the measurement that three product crates
named `nomos-ledger` as a dev-dependency only to reach `Territory`, which
`nomos-scope-verification` gives them. Both readers exempt dev-dependencies by name --
`Is_Dev_Dependency` in the rule and `Is_Not_Dev` in the contract crate's workspace reader --
and the population has changed shape under the clause. `nomos-agent-executor-claude-code` and
`nomos-model-backend-ollama` each now carry
`Test_The_Ledger_Territory_A_Task_Envelope_Carries_Is_The_Same_Territory_A_Real_Ledger_Claims`,
which builds a real ledger, saves a real item and claims it, to prove that the scope a
`TaskEnvelope` carries is the shape the ledger's exclusion model claims and not merely the same
type name. That dev-dependency is the seam test's subject, and the substrate primitive cannot
stand in for it. `nomos-workflow-orchestration`'s own test module is the one site where version
1's finding still holds -- it imports `nomos_ledger::Territory` where
`nomos_scope_verification::Territory` is the same type -- and it is a fixture spelling with no
shipped edge behind it, not a defect a record owes an item for.

So version 1's sentence that a dev-dependency on repo tooling is exactly the failure this
record exists to make visible is withdrawn. The failure this record existed to make visible was
a shipped edge nothing refused, and that is closed. A test exercising the tool it is tested
against is the reason both readers exempt dev-dependencies, and two of version 1's three cases
are now that.

### The directory move is retired

Version 1's reason for `crates/repo-tooling/` was that a reader cannot tell repo tooling from
product by directory, that the README mark is the only signal, and that prose is not a
boundary. The second half is no longer true. The zone declaration is the boundary, read by the
rule and by the contract tests, and `OD-RULES-020` had already said the zone does not follow the
directory: if that record's own move happens, the crates land where it already put them. A
directory would now be a third copy of a membership `nomos-architecture.json` holds and two
readers check, and the one copy nothing compares against the declaration -- a crate placed
under `crates/repo-tooling/` and zoned elsewhere would contradict it silently. `OD-AGENT-004`
is the record for what an unchecked copy of a checked fact does. The same holds for
`nomos-spec-orchestration` and `crates/spec/`: the declaration already places it with its
family.

Neither move is owed, and no follow-up item is named. Carrying either out is a `git mv` and a
`path` change that leaves `nomos-architecture.json` untouched -- permitted at any time,
required by nothing, and if ever made, the declaration and not the directory remains what the
gate reads. The trigger to reopen this is the one version 1 named and left unmet: a real need
for the tooling to become its own workspace, which is `OD-PACKAGE-015`'s test and not a
directory question.

### The Specification zone is repository tooling, with a named route out

The seven `nomos-spec-*` crates are repository tooling in `OD-LEDGER-036`'s sense, on that
record's own evidence class. No product surface depends on any of them; every real consumer is
this repository authoring and projecting its own governing records -- `nomos-cli`'s `spec`
verbs, `nomos-api`'s specification handlers that `OD-HOST-006` found to be the same seam
exercise `OD-LEDGER-036` found for the ledger, `gate.yml`'s required-projections step, and the
contract tests -- and the `permits` table admits them to the same two zones it admits Repo
Tooling to. `README.md` marks `nomos-spec-orchestration` `[repo tooling]` and says of the family
that nothing in the product may name it; the declaration is that sentence as data.

They are not a product capability awaiting a caller, and the distinction decides what would
change this answer. `ARC-ECOSYSTEM-001` names the family's mature home as KWB, reached from here
through a knowledge capability, which is already how the product is permitted to see it. The
route by which the specification reaches the product is therefore a capability contract and a
provider -- a `Provider`-zone crate naming a `nomos-spec-*` crate, which is a new row in
`permits`, decided in `nomos-architecture.json` when that provider exists -- and not a product
crate depending on these seven. No such contract exists: none of the thirteen `nomos-cap-*`
crates is a knowledge capability, and none names a `nomos-spec-*` crate.

The shared trigger stays shared. `OD-HOST-006` bound `nomos-spec-orchestration`'s exposure
through `nomos-api` to the trigger `OD-LEDGER-036` stated -- `nomos-api` becoming an externally
consumed surface -- and `OD-HOST-007` discharged it against the transport crate, where every
specification handler is excluded by assertion. This record adds nothing to that trigger. It
records that the ownership answer the trigger guards covers all seven crates, not only the one
README marks.

### What version 2 changes, and what it does not

The decision's title stands: the tools that govern this repository are separated from the
product they govern. What changed is how -- by a declaration two readers judge, not by a
directory and a fifth assertion -- and that both of version 1's mechanisms are retired rather
than pending. No crate moves, no test is written and `nomos-architecture.json` is not edited by
this amendment.

`README.md`'s sentence that `OD-PROJECT-004` decided where these four belong physically
describes version 1 and is stale from this amendment on, as its statement that `nomos-rules`
declares a zone table already was. Both are `README.md`'s own, held by another item's territory
at the time of writing, and are named here so the next README item finds them rather than
re-derives them.

## Status

Accepted at version 1, amended at version 2.

Version 1 named four crates for a new directory, redirected one of the four to an existing
family directory instead, kept one Cargo workspace, and stated one graph assertion's shape; no
crate moved and no test was written there.

Version 2 records that neither the directory nor the assertion was ever built, that
`nomos-architecture.json`'s `permits` is the guard in effect and is Blocking through the gate
while Advisory in the rule's own report, retires both of version 1's mechanisms rather than
leaving them pending, withdraws the dev-dependency clause, and answers that all seven
`Specification`-zone crates are repository tooling under `OD-LEDGER-036`'s shared trigger.
