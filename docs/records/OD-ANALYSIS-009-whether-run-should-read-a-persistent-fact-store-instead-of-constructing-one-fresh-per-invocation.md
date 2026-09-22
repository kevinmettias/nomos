---
id: OD-ANALYSIS-009
type: decision
title: Whether nomos-check-orchestration's Run should read a persistent fact store instead of constructing MemoryFactStore fresh per invocation
status: accepted
version: 5
authority: canonical-normative-record
tags:
  - analysis
  - incrementality
  - check-orchestration
  - architecture
relations:
  - target: OD-ANALYSIS-001
    type: relates-to
  - target: OD-ANALYSIS-002
    type: relates-to
  - target: OD-ANALYSIS-008
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-003
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
---

# Whether nomos-check-orchestration's Run should read a persistent fact store instead of constructing MemoryFactStore fresh per invocation

## Question

An external architecture review (read against this workspace rather than accepted on read)
names `nomos-check-orchestration::run::Run`'s `let mut store = MemoryFactStore::New();` as
evidence that `nomos-analysis`'s incremental substrate — generations, invalidation,
dependency-aware recomputation — is architecturally proven but never operationalized:
despite that machinery being real, "the normal user-facing check path still creates a fresh
fact store for every invocation." It names a persistent incremental daemon or service as one
of the largest remaining gaps between what this workspace's substrate can do and what `nomos
check` actually does today. `ARC-ROADMAP-001` separately lists "analysis + incremental fact
infrastructure" in Nomos Core's near-term tier. Whether either of those is new evidence that
`Run` should stop owning its own store's construction, or a restatement of scope
`ARC-ROADMAP-001` already named without ordering, was unmeasured before this record — the
same question `OD-RULES-009` asked of the same review's `RunPlanner` claim, asked here of its
persistence claim.

## What Was Measured

**The review's claim is accurate.** Verified directly against the real code at this record's
own HEAD: `crates/orchestration/nomos-check-orchestration/src/run.rs:51` constructs
`MemoryFactStore::New()` fresh inside `Run`, on every call, with no parameter through which a
caller could supply one instead.

**The incremental machinery this store would reuse is real, not aspirational.**
`OD-ANALYSIS-001` closed a bug that defeated cross-edit reuse entirely (a workspace-snapshot
component in `FactKey` that recomputed 9,440 facts per keystroke over a six-file corpus);
`OD-ANALYSIS-002` built the dependent half of invalidation; `OD-ANALYSIS-008` (open,
narrowed) tracks one remaining internal cost in that same machinery.
`tests/invalidation_order.rs`, `tests/recomputation_equivalence.rs` and
`tests/fact_identity/*` exercise generation-based reuse and dependency-aware invalidation
directly, and pass.

**But nothing in this workspace has ever asked one `MemoryFactStore` to survive across two
separate calls to `Run`.** A workspace-wide search for `MemoryFactStore::New()` finds
twenty-three call sites: one in production code (`run.rs:51`) and twenty-two in test modules,
each constructing its own store, using it within one test function, and dropping it. None is
held by anything that outlives one call. `nomos-cli`'s two composition roots that reach
`Run` — `check::Run` and `gate::run` — each call it exactly once per process, and
`main.rs` returns one `std::process::ExitCode` per invocation with no loop, no server and no
state that survives past `return`. The "fresh store per invocation" cost the review measures
is therefore not currently a same-process repeated-computation waste; it is the ordinary cost
of a fresh OS process on every shell invocation of `nomos check`, which passing a
caller-supplied store into `Run` would not by itself change — nothing would exist to hold
that store between one shell invocation and the next unless a long-lived process held it, and
no such process, nor any vocabulary for one, exists anywhere in this workspace. A search of
`docs/records` for "daemon", "watch mode" and "persistent workspace" finds nothing proposing
one.

**A persistent store would be licensed, if a real caller ever needed one — but none does
yet.** `OD-HOST-002`'s cache/privileged-state test draws the line a persistent `FactStore`
would have to clear: losing it must cost latency, not information. It would clear that line
(a dropped store is rebuilt from source, not lost data), so nothing forbids building one.
`OD-HOST-003` goes further and already reasons about exactly this shape for a long-lived
editor process, settling that a resolved capability registry or fact set held across such a
process's lifetime is a cache under `OD-HOST-002`'s test, not privileged state — real
precedent for what the first caller of a persistent store would look like, reasoned about
before that caller exists rather than invented to justify building the store first.

**Naming a category as near-term is not the same as ordering this increment inside it.**
`ARC-ROADMAP-001` lists "analysis + incremental fact infrastructure" in Nomos Core's
near-term tier, but its own fourth "What This Record Does Not Do" clause states plainly that
it "does not order the near-term tier internally... a separate judgment, informed by this
record but not fixed by it." `OD-RULES-009` already drew this same distinction against the
same review's `RunPlanner` claim, over the same record; it applies identically here.

## Decision

**Declined to build now.** `Run` keeps constructing `MemoryFactStore::New()` internally; no
persistent or cross-invocation fact store, no caller-supplied-store parameter, and no
daemon or long-lived-process concept is scheduled by this record.

This is not a verdict that persistence is the wrong eventual shape. `ARC-ROADMAP-001` already
names the category as near-term, and `OD-HOST-002`/`OD-HOST-003` already establish that a
persistent store would be licensed as a cache rather than forbidden as privileged state when
one is built. It is a verdict that no caller anywhere in this workspace has yet asked `Run`
to be invoked more than once within one process's lifetime, so there is no real
repeated-computation cost to fix today and no real caller's shape to design a
caller-supplied-store seam from — the same "wait for a real case, not the population of
zero" reasoning this workspace applies elsewhere (`OD-PACKAGE-006`, `OD-PACKAGE-008`,
`OD-RULES-007`, `OD-RULES-008`). `OD-ROADMAP-001`'s override does not reach this question: it
names one explicit cluster — `AgentExecutor`/`ModelBackend`/`RulePackage`/corrections — and
this is not in it.

## What Would Decide It

- **A real long-lived caller.** Most plausibly the editor surface `OD-HOST-003` already
  reasons about, or any IDE/LSP-shaped client, that would invoke `Run` (or something
  functionally like it) more than once within its own process lifetime — the concrete case
  that would turn a per-call `MemoryFactStore::New()` into a measured cost rather than a
  theoretical one.
- **A workflow engine or a Gate phase concept** (`OD-WORKFLOW-002`'s still-unfired third
  trigger) that runs multiple analysis passes within one execution and would benefit from
  sharing a store across them.
- **Measured evidence from a real corpus-scale workload** — an edit-then-recheck run timed
  against a fresh-store baseline — the same evidentiary bar `OD-ANALYSIS-001` and
  `OD-ANALYSIS-008` both hold themselves to rather than arguing from complexity alone.
- **`ARC-ROADMAP-001`'s near-term tier being internally sequenced** by a later decision that
  specifically orders "analysis + incremental fact infrastructure" ahead of the items beside
  it.

## A Direct Override Built The First Real Increment, Narrower Than Any Trigger Above

None of the four triggers named above had fired on their own by 2026-08-26. What changed is
not evidence — no editor surface, no workflow engine, no measured corpus workload, no
internal sequencing of `ARC-ROADMAP-001`'s near-term tier arrived. What changed is that the
user directly and explicitly overrode this record's "wait for a real caller" conclusion,
the same shape `OD-ROADMAP-001` already used for a different cluster of decisions. This is
not `OD-ROADMAP-001` reaching a question its own text already said it does not reach — this
record's own "Decision" section above already measured that and it is still true, unchanged
by this section. It is a second, distinct override, over this specific question, recorded
here rather than folded into `OD-ROADMAP-001`'s text or assumed to already be covered by it.

**What was built** (`P14-ANALYSIS-009-STORE-WORKSPACE-REUSE-FIRST-INCREMENT`):
`nomos_check_orchestration::Run` now takes `workspace: &mut Option<Workspace>` and
`store: &mut MemoryFactStore` as parameters instead of constructing a
`nomos_workspace::Workspace` and a `MemoryFactStore` internally on every call. `Ingested`
(`crates/orchestration/nomos-check-orchestration/src/facts.rs`) reuses an existing
`Workspace` when one is handed in, via `Option::get_or_insert_with`, rather than always
starting from `Workspace::Empty`. Both of this crate's real callers —
`nomos-cli::check::Run` and `nomos-gate-orchestration::run_gate::Judged` — construct a fresh
`Workspace` (`&mut None`) and a fresh `MemoryFactStore` for every call, unchanged: each is
still one process per invocation, exactly the finding "What Was Measured" made above, so
nothing about *their* behavior is different after this increment. What is different is that
`Run` no longer forces every caller to.

A new test,
`nomos_check_orchestration::tests::Test_A_Store_And_Workspace_Reused_Across_An_Edit_Agrees_With_A_Clean_Recomputation`,
is the first caller this workspace has ever had that reuses either object across two calls.
It carries one `Workspace` and one `MemoryFactStore` across a call that edits one source and
leaves another alone, and checks the result against an independent third call that
recomputes the post-edit tree from nothing. Two things are proven, both new:

- The claim and the findings agree between the reused-state call and the from-nothing call —
  `IncrementalResult(S) == CleanRecomputation(S)`, the literal invariant `crates/substrate/
  nomos-analysis/tests/recomputation_equivalence.rs` already proved over synthetic `FactKey`s
  built by hand, exercised here for the first time through this crate's own real composition
  (the real syntax provider, the real `Workspace`, `Run`'s real ingestion and judging) rather
  than bypassing it.
- The reused `Workspace`'s generation genuinely advances across the two calls
  (`Workspace::Apply`'s own diff — an untouched file is `Redundant`, an edited one is
  `Modified` — is what makes the second call a real second generation, not the fresh-store-
  every-time behavior computing the identical generation twice).

**What this does not close.** `Materialize_Syntax` and its sibling materialization steps
still re-derive every source's fact on every call, whether or not `store` already holds a
live one for that identity — this increment proves reuse is *safe*, not that anything is
*skipped*. `IncrementalResult` above costs exactly what `CleanRecomputation` costs. And no
long-lived process exists anywhere in this workspace that could hold a `Workspace` and a
`MemoryFactStore` across two *shell* invocations — `nomos check` and `nomos gate run` remain
one process each, so the first trigger above ("a real long-lived caller") has still not
fired; this increment is what a real caller of that kind would need to exist, not that
caller itself.

## Amendment (P40-FACT-STORE-PERSISTENCE-2): The First Trigger Fired, And It Asks For Something Narrower Than This Record's Subject

**Both halves of this section were corrected at version 4, and its citations are dated
history rather than live pointers.** `crates/host/nomos-lsp/src/server.rs` no longer exists
-- the server mechanics moved into `xvpe-language-server-backend-lsp` on 2026-09-10 -- and
the per-call construction this section calls "a real, concrete, immediately buildable gap"
was built. Read
`Amendment (P123-OD-ANALYSIS-009-SENDS-A-READER-TO-A-DELETED-FILE-AND-TO-BUILD-WHAT-EXISTS)`
below before relying on either half. Nothing here is rewritten or deleted: this section was
true at the revision it measured, and the correction is only checkable against it if it
stands.

`P40-FACT-STORE-PERSISTENCE-2` re-measured all four triggers directly against the tree
rather than trusting this record's own prior "unfired" verdict, the same discipline
`OD-WORKFLOW-002` already modeled for the workflow tier.

**Trigger 1 has fired, at the letter and largely in substance.** `crates/host/nomos-lsp`
exists now and did not when this record's "What Was Measured" section searched for a
long-lived caller. Its `Run_Server` (`crates/host/nomos-lsp/src/server.rs:32`) opens a
JSON-RPC connection and loops (`Serve`, line 93) for the life of an editor session, calling
`Recheck_And_Publish` on every `didOpen` or `didSave` -- exactly "a real long-lived caller
... that would invoke `Run` more than once within its own process lifetime," the concrete
shape this record's first trigger named and attributed to `OD-HOST-003`'s editor surface
before either was real.

**But read past the letter, `nomos-lsp` asks for the increment this record already declined
to build in its own first amendment, not the one this section's subject is about.**
`Recheck_And_Publish` (`server.rs:122`) constructs `let mut workspace = None;` and
`let mut store = nomos_analysis::MemoryFactStore::New();` fresh, inline, on every call --
discarding, on every keystroke's save, the exact reuse `P14-ANALYSIS-009-STORE-WORKSPACE-
REUSE-FIRST-INCREMENT` (recorded above) already built and proved safe:
`nomos_check_orchestration::Run` has taken a caller-supplied `workspace: &mut
Option<Workspace>` and `store: &mut MemoryFactStore` since that increment shipped, and
`nomos-lsp` is the first real caller with a process lifetime long enough to hold either
across two calls, but does not. That is a real, concrete, immediately buildable gap -- and
it is answered entirely by wiring one already-decided shape into one composition root's own
loop state. It needs nothing this record's subject asks for: no serialization format, no
on-disk store, no key that survives a build-variant or contract-version change, and no
eviction policy, because nothing here is asked to outlive the process. `nomos-lsp`'s own
process still ends when the editor closes it, same as every other caller in this workspace,
and nothing measured shows that boundary needs crossing.

**Triggers 2 through 4 remain unfired, re-verified directly.** `nomos-workflow-orchestration`
(`P40-WORKFLOW-*`) now exists and its own `Run` (`crates/orchestration/nomos-workflow-
orchestration/src/run.rs:46`) does take a whole `&[WorkflowStepPlan]`, but its one real
caller, `nomos-cli`'s `workflow.rs`, composes exactly one step per invocation by its own
design -- `Run`'s own doc comment states plainly that "a caller composing more than one step
links `nomos-workflow-orchestration` directly," and grepped directly, nothing does. Trigger
2 is unfired in substance, the same "satisfied only at the letter" gap `OD-WORKFLOW-004`
already named for a different condition, except here not even the letter is satisfied: no
real plan with two analysis-bearing steps has ever been composed. No corpus-scale timing
comparison exists anywhere in this workspace for trigger 3. `ARC-ROADMAP-001`, re-read in
full, still states its near-term tier is "not... a validated ordering," unchanged, so
trigger 4 is unfired. `OD-ROADMAP-001`'s cluster-scoped override, re-checked, still names
`AgentExecutor`/`ModelBackend`/`RulePackage`/corrections and nothing about analysis
persistence, so it still does not reach this question.

**Decision, unchanged in substance, sharpened in reason.** No real caller anywhere in this
workspace needs a `FactStore` to survive its own process's exit, so this record continues to
decline building one, its keying and its eviction/invalidation lifecycle now. What changed
is that the "no real long-lived caller" half of that reasoning is no longer the load-bearing
half -- a real one now exists -- and the record now rests on the narrower, better-supported
finding that the caller which exists needs in-process reuse of already-built machinery, not
cross-process persistence of new machinery. `P40-FACT-STORE-PERSISTENCE-2`'s territory does
not reach `nomos-lsp`, so this amendment records the gap rather than closing it; wiring
`nomos-lsp`'s own loop to hold one `Workspace` and one `MemoryFactStore` across its calls is
real, disjoint, next work, left for a ledger item scoped to that crate.

## Amendment (P123-OD-ANALYSIS-009-SENDS-A-READER-TO-A-DELETED-FILE-AND-TO-BUILD-WHAT-EXISTS): Both Halves Of The Trigger-1 Amendment Went Stale, One By A Move And One By Being Built

The amendment above was written against a tree in which `crates/host/nomos-lsp` held a
server and did not hold its own reuse. Neither is true now. Both halves were re-measured
directly against `9f76f2f6` rather than carried over from that section's text, and this
amendment corrects the citation and marks the ordering argument spent. It decides nothing
about persistence: the `Decision` section above is untouched, and so is that section's own
restatement of it.

This follows `OD-ANALYSIS-007`'s amendment rather than a rewrite -- the stale sentences are
quoted so the correction can be checked against them, and the paragraphs carrying them are
left standing -- and keeps `ARC-ROADMAP-001`'s convention of naming each amending item in
`Status`.

**First half: the path went stale because the code moved, not because it was wrong when it
was written.** The section above cites `Run_Server` at
"`crates/host/nomos-lsp/src/server.rs:32`", its loop at "`Serve`, line 93", and
`Recheck_And_Publish` at "`server.rs:122`". There is no `server.rs` under
`crates/host/nomos-lsp/src`, and none of those three functions exists anywhere in this
workspace; the crate's files are `build_variant.rs`, `file_diagnostic.rs`, `lib.rs`,
`location.rs`, `main.rs`, `nomos_diagnostic_provider.rs`, `severity.rs`, `sources.rs` and
`walk_outward.rs`. The crate's own module doc dates and explains the move rather than
leaving a reader to infer it: under "What changed on 2026-09-10", "the whole server is
XVPE's now" -- the handshake, the workspace root, the capability declaration, the `file://`
URI conversion, the whole-line spans and the stale-marker clearing an editor needs all moved
down into `xvpe-language-server-backend-lsp` over the `xvpe-diagnostics` provider contract,
because "Nomos is an application over that engine, and a general capability sitting up here
was unreachable by everything down there". What stayed is what was always this workspace's:
the walk, the judgement, the severity each of its own rule categories deserves, and what a
finding carries. `NomosDiagnosticProvider` is, in that doc's own words, "the one seam, and
`src/main.rs` hands it to the engine".

So the long-lived caller trigger 1 fired on is still real and is still this crate -- one
editor session across many `didOpen` and `didSave` notifications -- but the name to read it
at is `NomosDiagnosticProvider::Diagnose`, not `Run_Server` or `Recheck_And_Publish`. The
citation is the defect and the finding is not: an amendment that sends a reader to a deleted
file reads as a live pointer into code, which is worse than saying nothing, while a path
citation is dated history the moment the tree moves -- `OD-SPEC-017` decided exactly that
about path citations, which is why nothing mechanical caught this one and an amendment had
to.

**Second half: the gap that section calls immediately buildable is built, so its argument
for building it first is spent.** What it says: `Recheck_And_Publish` "constructs `let mut
workspace = None;` and `let mut store = nomos_analysis::MemoryFactStore::New();` fresh,
inline, on every call -- discarding, on every keystroke's save, the exact reuse
`P14-ANALYSIS-009-STORE-WORKSPACE-REUSE-FIRST-INCREMENT` (recorded above) already built and
proved safe", and calls that "a real, concrete, immediately buildable gap ... left for a
ledger item scoped to that crate". Measured now:
`crates/host/nomos-lsp/src/nomos_diagnostic_provider.rs:55`-60 declares
`workspace: Option<Workspace>`, `store: MemoryFactStore` and
`reassessment: nomos_check_orchestration::RuleReassessmentCache` as struct fields of
`NomosDiagnosticProvider`. The two initializers that section quotes survive, in `New` at
lines 66-72, where they run once per provider rather than once per call. The type's own doc
is explicit about which question that answers: its first line calls the three "the three
things it keeps between being asked twice", and its "What it keeps between calls" heading
says they are "reused across every judgement rather than rebuilt per call", citing this
record's second amendment as the concrete case its first trigger was written for.

Two ledger items closed it, in this order:

- `P40-LSP-STORE-AND-WORKSPACE-REUSE` (Done, commit `7596982f`, 2026-09-06) grouped the
  published set, the `Workspace` and the `MemoryFactStore` into one value the server owned
  and threaded into every call it made to `Run` -- precisely the "real, disjoint, next work"
  the section above left for an item scoped to this crate. When the server moved to XVPE
  four days later, that reuse landed on `NomosDiagnosticProvider`'s own fields, which is why
  the closure is invisible from the path that section cites.
- `P105-THE-LSP-KEEPS-THE-REASSESSMENT-CACHE-AND-PROVES-THE-REUSE-AT-ITS-OWN-BOUNDARY`
  (Done, commit `0c3b1013`) added the third field and made the rule-level skip reachable at
  all: `Run` builds a reassessment cache fresh for its own call and drops it at the end by
  construction, so every composed rule re-runs on every call for every caller, and a caller
  wanting the skip has to keep the cache itself. An editor is the caller that can.

The reuse is proven at that boundary rather than asserted there.
`Test_A_Second_Judgement_Should_Re_Derive_A_Fact_Only_For_The_Source_That_Moved` holds that a
second judgement re-derives a fact only for the source whose bytes moved,
`Test_A_Reusing_Provider_Should_Answer_What_A_Fresh_One_Answers_Over_The_Same_Tree` holds
that keeping the three does not change the answer, and
`Test_A_Cold_Judgement_Should_Equal_What_The_Unreassessed_Run_Path_Produces` holds the
reassessing path against the plain one -- the same
`IncrementalResult(S) == CleanRecomputation(S)` invariant
`Test_A_Store_And_Workspace_Reused_Across_An_Edit_Agrees_With_A_Clean_Recomputation` proves
one layer down, now proven at the caller that actually reuses.

**Why a closed gap needs recording rather than tidying.** The section above does not merely
mention that gap; it rests its conclusion on it. Its argument is an ordering one -- the
nearer in-process work "needs nothing this record's subject asks for: no serialization
format, no on-disk store, no key that survives a build-variant or contract-version change,
and no eviction policy", so that is the work to do and this record's subject is not. The
argument is now **spent**: the nearer work is done and nothing waits behind it. A spent
ordering argument left standing in a governing record reads as a live reason to wait, which
is the whole defect corrected here. What does not follow is any conclusion about
persistence, in either direction -- an argument being spent removes a reason to wait and
supplies no reason to build.

**What remains genuinely unbuilt is this record's own subject, the on-disk half.** Measured
at `9f76f2f6`, three things, none of which the in-process closure above touches:

- **No store survives process exit.** `crates/substrate/nomos-analysis/src/fact_store.rs`'s
  `FactStore` trait has exactly one implementor in this workspace, `MemoryFactStore`, and it
  is `BTreeMap`s in memory. The provider's own doc says the same of what it keeps: "Reused,
  not persisted -- all three still end when this process does. Nothing here asks any of them
  to survive past that."
- **No key has ever had to survive the binary that wrote it.** `FactKey` already carries
  `contract_version` and `variant: BuildVariantId` among its nine components, so within one
  process a changed contract version or build variant yields a different key and costs a
  recompute rather than serving a stale fact. What does not exist is any serialized form of
  that key or of a fact, anything that validates one read back from a process that is not
  this one, or any decision about what a stored entry means when the reading binary is not
  the writing binary, or when `FactKey`'s own shape moved between the write and the read.
  That is the keying question this record's subject asks, and it is untouched.
- **No retention rule bounds the per-key history.** `MemoryFactStore` holds a `Vec` of
  entries per key digest and every `Materialize` pushes another one onto it; nothing trims,
  ages or evicts, and the process's own exit is the only bound. A store that ends with the
  process can be built that way. One that outlives it cannot.

**That question belongs to a forthcoming item, and this amendment is not it.**
`P123-FACT-STORE-SURVIVES-THE-PROCESS-2` builds the on-disk store and amends the `Decision`
section; it is not on the board while this correction is open, because an amends reservation
on this record is exclusive and this item holds it, and it is authored once this lands.
Until that item decides otherwise, the `Decision` section above is this record's position,
unchanged and deliberately not reweighed here.

## Amendment (P123-FACT-STORE-SURVIVES-THE-PROCESS-2): The On-Disk Half Is Built, And What A Store Means To A Build That Did Not Write It Is Now Decided

**This supersedes the `Decision` section above for the on-disk half, and for nothing else.**
That section declined "no persistent or cross-invocation fact store, no caller-supplied-store
parameter, and no daemon or long-lived-process concept"; the middle clause was already
overtaken at version 2, and this amendment overtakes the first. The third is untouched: no
daemon, no long-lived process and no host wired to read or write a persisted store arrives
with this, because which hosts may share one is a question about where a cache lives and it
belongs to `P128-EVERY-INVOCATION-STARTS-COLD-BECAUSE-NOTHING-OUTLIVES-A-PROCESS`. The
`Decision` text is left standing rather than rewritten, the amendments at versions 3 and 4
with it, so what changed is checkable against what it changed from.

**What decided it is the requirement, not a trigger.** No new measurement arrived: triggers
2 through 4 are where version 4 left them, and the surviving half of trigger 1 — a caller
needing its `Workspace`/`MemoryFactStore` to survive its own process's *exit* — is still not
a caller that exists. What exists is the work: this record's own subject is a Required
capability item on the board, and the item that builds a process outliving an invocation is
queued behind it and will read what this writes. Version 4's correction is what makes that
honest rather than circular. It marked the ordering argument for doing the nearer in-process
work first **spent**, and said in the same breath that spending it "removes a reason to wait
and supplies no reason to build". The reason to build is this item, recorded here as one,
the same shape the first override took at version 2 rather than a trigger being read
generously.

**What was built.** `nomos_analysis::MemoryFactStore` gained
`Write_To_Directory(&self, directory: &Path)` and
`Read_From_Directory(directory: &Path, understood_schemas: &[SchemaId])`
(`crates/substrate/nomos-analysis/src/fact/memory_fact_store.rs`), both delegating to
`crates/substrate/nomos-analysis/src/fact/memory_store/persistence.rs`, a child of the store's
own module because it reads and rebuilds fields that are nobody else's business. The
directory holds one file, `fact-store.json`, written under a scratch name and renamed into
place so an ordinary write cannot tear it. It carries the store's keys, the retained entry of
each key's history, the dependents graph, and the count of writes the store has taken. It does
not carry the `DependencyPropagation` strategy: a strategy is the composing build's choice,
not state a previous process measured. It *does* carry the dependents graph rather than
leaving it to be derived from the entries, because `Materialize` adds an edge per dependency
and never removes one, so deriving it would drop edges the writing store still held and a
reloaded store would invalidate less than the one that wrote it.

**Two versions, kept apart, because they move for different reasons.** `FORMAT_VERSION` is
the written form's own; `Understood_Key_Shape` is the key's, and it is not a hand-maintained
number — it is `Component::All()`'s labels, this crate's declared mirror of `FactKey`'s
fields, which `Component::Label`'s exhaustive match and
`crates/substrate/nomos-analysis/tests/fact_identity/key_identity.rs` already hold in step
with the key. A component added, removed or renamed therefore changes the shape by itself,
and **every store written under the old one is refused whole**. That is the decision this
item exists to make: a key whose components moved does not mean what its bytes say, so there
is no partial read and no per-entry rescue, because a rescue would decide silently what a
component's disappearance meant. A moved shape costs a recomputation, deliberately, and no
migration path is offered. Both checks run over the whole file before one entry is built, so
"wholesale" is structural rather than promised.

**What a build must refuse rather than believe** is `nomos_analysis::PersistenceError`
(`crates/substrate/nomos-analysis/src/persistence_error.rs`): a file that cannot be written
or read, one that is corrupt or truncated, a foreign format version, a foreign key shape, a
key this build addresses by a different digest than the file records, and a payload schema
the reading build did not declare. Every variant names the file and `File()` returns it
without a fallback, because a refusal a reader cannot locate is a rumour. The payload
question is answered by the reader and not by this crate, which holds payload bytes
opaquely: `understood_schemas` is what the composing build says its providers can interpret,
and anything else is refused rather than handed on to something that would read the bytes as
a schema they were not written in.

**The per-key history now has a stated retention rule, and it is one entry.**
`RETAINED_HISTORY_ENTRIES` and `MemoryFactStore::Push_Entry` carry it at the one place a
history grows. Every answer the store gives reaches its entry through `Latest`, which is
`history.last()` — `Current` through `Lookup`, `Historical`, `Dependencies_Of`,
`Superseded_At`, `Live`, `Refuse_Backdated`, and the invalidation walk's
`Is_Already_Invalidated` and `Try_Invalidate_One` — so an entry behind the newest is
unreachable rather than merely unused. The historical read is not the exception it sounds
like: supersession is recorded *on* the entry it superseded, as that entry's own
`invalidated_at` and `cause`, so what `Historical` needs is exactly what is kept. A store
that ends with its process could afford the rest; one that outlives it cannot, because an
unbounded history is an unbounded file.
`Test_A_Trimmed_History_Should_Answer_What_A_Single_Write_Answers` and
`Test_A_Trimmed_History_Should_Invalidate_And_Read_Historically_Like_A_Single_Write` are the
sufficiency evidence: a store that wrote one key four times answers every read the store
offers exactly as one that wrote the same final fact once, before and after an invalidation.
`P128-THE-FACT-GRAPH-IS-DIGEST-KEYED-TREES-AND-THE-RANKING-IS-QUADRATIC` asks for a retention
rule too; this is the one it adopts rather than a second to mint.

**Proven, not asserted.** `Test_A_Reloaded_Store_Over_An_Unchanged_Tree_Should_Agree_With_A_
Clean_Rebuild` and `Test_A_Reloaded_Store_Rematerialized_After_An_Edit_Should_Agree_With_A_
Clean_Rebuild` are `tests/recomputation_equivalence.rs`'s own invariant —
`IncrementalResult(S) == CleanRecomputation(S)`, over a graph with a real derived hop whose
payload is a function of its upstream's — asked of a store that crossed a process boundary.
`Test_A_Reloaded_Store_Should_Invalidate_Exactly_What_The_In_Memory_Store_Would` compares the
whole `InvalidationReport` of a reloaded store against the store that wrote it, and
`Test_A_Reloaded_Store_Whose_Edge_Was_Never_Recorded_Should_Serve_A_Stale_Value` is the
control that keeps the fixture from agreeing for the wrong reason. They live inside the
crate rather than in `tests/`, because this item's verification predicate runs the library's
own tests and a proof it does not run is not a proof it has.

**Two dependency decisions, both narrow.** `serde` and `serde_json` are declared in the root
manifest's workspace table already and inherited with `workspace = true`; no crate was added
to the lock, only an edge. They are used through private mirror types in
`persistence.rs` rather than derived onto this crate's public vocabulary, so the written form
can move without a field rename becoming a silent format change and `Serialize` stays off the
published surface — `tests/contract/surface/nomos-analysis.txt` shows the difference. And the
store names `std::fs` rather than `nomos_platform::FileSystem`, which `nomos-ledger` takes
for the file it owns: the port's surface is text-only and has no operation that creates a
directory, a store written *to a directory* must create the one it is given, and
`nomos-platform` was not this item's territory, so widening the port here would have decided
a platform question inside an analysis one. Which filesystem a persisted store should be
handed is the same question as which host composes one, and it is left where the rest of
that question already is.

## Status

Accepted, amended a second time. Trigger 1 (a real long-lived caller) has fired with the
arrival of `nomos-lsp`, but what it asks for is the in-process reuse `P14-ANALYSIS-009-
STORE-WORKSPACE-REUSE-FIRST-INCREMENT` already built, not the cross-process, on-disk
persistence this record's own subject is about -- so the decision to decline stands, now for
a narrower and more precise reason than "no real caller exists." Triggers 2 through 4 remain
unfired, re-verified directly rather than assumed unchanged. Revisit on `nomos-lsp` (or any
other caller) needing its `Workspace`/`MemoryFactStore` to survive its own process's exit
rather than merely to be held across calls within one process, on a real multi-analysis-pass
workflow plan being composed by a real caller, on a corpus-scale timing measurement, or on
`ARC-ROADMAP-001`'s near-term tier being internally sequenced.

Amended a third time, to version 4, by
`P123-OD-ANALYSIS-009-SENDS-A-READER-TO-A-DELETED-FILE-AND-TO-BUILD-WHAT-EXISTS`, which
corrected both halves of the trigger-1 amendment above rather than deleting them. Its
citations to `crates/host/nomos-lsp/src/server.rs` name a file that no longer exists,
because the server mechanics moved into `xvpe-language-server-backend-lsp` on 2026-09-10
and `NomosDiagnosticProvider` is the seam that remains; and the in-process reuse gap it
called immediately buildable was built, by `P40-LSP-STORE-AND-WORKSPACE-REUSE` and then
`P105-THE-LSP-KEEPS-THE-REASSESSMENT-CACHE-AND-PROVES-THE-REUSE-AT-ITS-OWN-BOUNDARY`, so
its ordering argument for closing the nearer gap first is spent rather than live. That
correction leaves the decision exactly where it was: the on-disk half -- a store that
survives process exit, a key that survives the binary that wrote it, a retention rule over
a per-key history -- remains unbuilt and is not reweighed here, and
`P123-FACT-STORE-SURVIVES-THE-PROCESS-2` is the forthcoming item that answers it. The
revisit conditions above are unchanged; what the correction sharpens is that the surviving
half of trigger 1 is only the one they already name, a caller needing its
`Workspace`/`MemoryFactStore` to survive its own process's exit rather than merely to be
held across calls within one process.

Amended a fourth time, to version 5, by `P123-FACT-STORE-SURVIVES-THE-PROCESS-2`, which
built the on-disk half and decided the keying question this record's subject asks. A
`MemoryFactStore` can now be written to a directory and read back by another build; the
written form carries its own version and, separately, the key shape it was written under;
a store written under a `FactKey` shape or a format version the reading build does not know
is refused **whole** rather than partly read, a payload schema the reading build did not
declare is refused rather than misread, and an unreadable, corrupt or truncated file is
reported as such, never served, by a refusal that names the file. The per-key history gained
a stated retention rule of one entry, proven sufficient because every read the store offers
reaches `Latest`. The `Decision` section's first clause is superseded by that amendment and
its third is not: no host reads or writes a persisted store yet, and which may share one
stays `P128-EVERY-INVOCATION-STARTS-COLD-BECAUSE-NOTHING-OUTLIVES-A-PROCESS`'s question.
Triggers 2 through 4 are unchanged and unfired, and nothing here re-weighs them: what
decided this was the requirement, recorded as one.
