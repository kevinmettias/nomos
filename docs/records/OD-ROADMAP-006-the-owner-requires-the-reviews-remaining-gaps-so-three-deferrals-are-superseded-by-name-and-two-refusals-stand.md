---
id: OD-ROADMAP-006
type: decision
title: The owner requires the review's remaining gaps, so three deferrals are superseded by name and two refusals stand on their measurements
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - roadmap
  - gate
  - workflow
  - analysis
  - rules
relations:
  - target: OD-ROADMAP-001
    type: relates-to
  - target: OD-ROADMAP-005
    type: relates-to
  - target: OD-GATE-022
    type: relates-to
  - target: OD-GATE-030
    type: relates-to
  - target: OD-WORKFLOW-002
    type: relates-to
  - target: OD-WORKFLOW-005
    type: relates-to
  - target: OD-ANALYSIS-009
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
  - target: OD-RULES-022
    type: relates-to
  - target: OD-CAPABILITY-009
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
---

# The owner requires the review's remaining gaps, so three deferrals are superseded by name and two refusals stand on their measurements

## Question

An external architecture review of `dev` at `bc0aaacf` named thirteen core gaps. A triage
measured each against the tree on 2026-09-21: several were stale, two dissolved under
measurement, and eleven were built or decided over that day. `OD-ROADMAP-005` separately
superseded eight deferrals for a different set of the same review's claims and bounded the
override it recorded.

What is left is a list nobody has authorized. Some of it is ordinary work that no record
stands in front of. Some of it cannot be started at all without a record moving, because each
piece sits behind a deferral that was correct when it was written and has never been revisited.
And two of the review's remaining claims are not deferred work at all — they were refused on
measurement rather than on sequencing, which a directive to build does not answer.

The owner has now required the remainder addressed. What that needs from this record is the
same thing `OD-ROADMAP-005` needed: **bounds**. Which deferral each piece supersedes, by clause
and by version. Which pieces have nothing in their way and are listed only so the set is
complete. Which of the review's claims are already answered rather than open. Without them,
each implementation reads as an agent building against an accepted record that says not to, and
a later reader reconciling the same review finds records saying wait and a tree saying
otherwise, with nothing to say which is current.

## What Is Not Being Claimed

**None of the deferrals below was wrong on its own evidence, and this record does not say
otherwise.** `OD-GATE-022` really did find that both cases its own compare doc names are
same-process cases answerable with no store at all. `OD-WORKFLOW-005` really did build a
sequential engine and really did scope it by explicit subtraction. `OD-ANALYSIS-009` really did
find that no caller had asked `Run` to be invoked twice in one process. Those measurements are
left standing and are cited by the items that now supersede them rather than deleted.

**So the override is about *when*, not about whether the reasoning held.** The precedent is
`OD-ROADMAP-001`, which retired a population-of-zero caution for a named cluster on exactly
this footing, and whose own amendment states the scope of that licence precisely: "This record
answers whether a component may be built before anything consumes it."

**A supersession is not a re-measurement.** Every clause moved below was read in the version on
disk at the revision this record was written against, and nothing else in those records is
touched. Where this record states a measurement of its own it re-took it; where it reports what
another record or commit measured, it says whose measurement it is.

## The Decision

**Build the pieces below. Each names the deferral it supersedes, by clause and version, and the
item that acts on it — or states that no record stands in its way.**

### 1. A run history, so a baselined finding can be told from one reintroduced

Supersedes `OD-GATE-022` v1 in two clauses and no others. The decision section's bound on a
first compare increment, "it does not serialize `GateRunResult` and does not persist a run
history", loses its second half only; and the closing clause of that record's "What this record
does not do", "It does not build or schedule a persisted run-history store", is superseded
outright. Everything else in `OD-GATE-022` stands: a compare caller may still re-derive both
runs in one process, and its finding that both motivating cases its own compare doc names are
same-process cases is untouched. `RunId` remains the key that record already named for a store
if one were built; what changes is that one now is.

`OD-GATE-030` v2 is the second consumer that record's own text said it was adding without
scheduling. It names three things that would make continuity provable, in order, and says that
only the third — a history of the states between — separates persistence from reintroduction;
it also says in as many words that `OD-GATE-022` owns that deferral and that it does not
schedule it. This record schedules it.

`OD-GATE-030`'s floor does not move and is not up for reinterpretation by the item that builds
this: a finding whose history cannot be established is reported as undetermined and never as
persistent, no occurrence inside an exceeded population is attributed, and the counting bound
the baseline already applies remains a bound on capacity rather than a claim about history.

The item is `P128-A-BASELINED-FINDING-CANNOT-BE-TOLD-FROM-ONE-REINTRODUCED`.

### 2. Branch, join, bounded parallelism, and a definition a run can be replayed against

Supersedes `OD-WORKFLOW-005` v2's "What This Does Not Build" in exactly four clauses: no
immutable published artifacts (`WF-009`); no branch/merge semantics or bounded parallelism
(`WF-010`), whose reason was that `Run` is one ordered sequence; no independently versioned
workflow definitions with pinned historical replay (`WF-011`); and the **cache** half of that
section's `WF-012` clause, so that a runtime may substitute a prior result for a dispatch where
`Cacheability` permits it.

The **cancellation** half of that same clause is not superseded and stays out, for the reason
that clause itself gives: nothing in this workspace can cut a dispatch in flight, and
`CancellationBehavior` is the declaration that would say whether a step even permits it. The
rest of that section is untouched — no deduplication token minted, no compensating step
composed into another step's run, no shared dispatch trait, no `WorkResult` assembly — and so
is everything the record's version 2 amendment already corrected about what the retry, timeout
and compensation runtime closed at `29bc3e20`.

Also supersedes `OD-WORKFLOW-002` v4's "What This Does Not Do" clause, as `OD-WORKFLOW-003`
narrowed it — no execution engine, no `WF-009`, `WF-010` or `WF-011` — to whatever of it
survived `OD-WORKFLOW-005` building the sequential engine. `OD-WORKFLOW-002`'s three named
conditions are **not** retired and none is claimed to have fired: they remain the honest
triggers for the increment after this one, which is exactly the distinction `OD-WORKFLOW-005`'s
own amendment to that record drew between a narrow override and a general retirement.

The item is `P128-THE-WORKFLOW-ENGINE-RUNS-A-LINE-AND-CANNOT-BRANCH-JOIN-OR-REPLAY`.

### 3. A scheduler that runs the waves the correction substrate already computes

Supersedes nothing, and is listed so the set is complete. `OD-ROADMAP-001`'s decision list
already superseded `OD-CORRECTIONS-001`'s conclusion that candidate generation,
classification and ranking, `COR-005`'s rerun-and-compare half and oscillation detection wait
for a real trigger, and the compatibility and wave substrate was built. What is missing is a
consumer: the computation exists and nothing runs it. No record stands in the way of giving it
one.

The item is `P128-NOTHING-RUNS-A-WAVE-SO-THE-CORRECTION-SUBSTRATE-HAS-NO-SCHEDULER`.

### 4. A process that outlives an invocation

Supersedes `OD-ANALYSIS-009` v4's `Decision` clause "no daemon or long-lived-process concept is
scheduled by this record", **and that clause only**. The two clauses standing beside it in the
same sentence — no persistent or cross-invocation fact store, and no caller-supplied-store
parameter — are deliberately not moved here. That record's version 4 amendment narrows what
remains of them to the on-disk half, names the three things that half still owes, and names
`P123-FACT-STORE-SURVIVES-THE-PROCESS-2` as the item that answers it. That item is live and
this record does not reach into it.

The two are not the same artifact and must not be collapsed: the store is the thing that
survives a process, and this is the process that survives an invocation. The item depends on
the store item rather than replacing it.

`OD-ANALYSIS-009`'s four revisit conditions are not retired, and `OD-HOST-002`'s rule that a
resident cache may hold no state its canonical services could not reconstruct is the condition
under which a resident process is admissible at all rather than a detail of how it is built.

The item is `P128-EVERY-INVOCATION-STARTS-COLD-BECAUSE-NOTHING-OUTLIVES-A-PROCESS`.

### 5. Six pieces with no deferral in the way

Each is required work and each is listed only so the set is complete. None supersedes anything,
and each is on the board with its own item, its own territory and its own predicate:

- the digest-keyed fact graph, the owning read on the store's trait surface, the unbounded
  per-key history and the quadratic provider ranking, at
  `P128-THE-FACT-GRAPH-IS-DIGEST-KEYED-TREES-AND-THE-RANKING-IS-QUADRATIC`;
- a second interchange format beside SARIF, at
  `P128-A-JUDGMENT-LEAVES-IN-ONE-FORMAT-AND-EVERY-OTHER-CONSUMER-IS-UNSERVED`;
- a compiler-backed provider for C#, at
  `P128-C-SHARP-IS-READ-ON-ITS-FACE-AND-NO-COMPILER-ANSWERS-FOR-IT`;
- tool providers for Go, at
  `P128-GO-HAS-A-PARSER-AND-A-MANIFEST-READER-AND-NO-TOOL-SPEAKS-FOR-IT`;
- a first verb for a repository adopting this tool, at
  `P128-A-REPOSITORY-ADOPTING-NOMOS-HAS-NOTHING-TO-RUN-FIRST`;
- a rendering of the effective policy's provenance, at
  `P128-THE-EFFECTIVE-POLICY-KNOWS-WHAT-DECIDED-EACH-FIELD-AND-NO-HOST-SAYS-SO`.

### 6. One remaining gap is a question rather than a piece

Whether a rule can be authored without writing Rust is the review's largest remaining claim
about the rule tier, and it is a decision rather than an implementation: it needs a
measurement against the rule population that now exists, and the review itself also said not to
invent a rule intermediate representation before real rules require one. This record does not
answer it. `P128-OD-RULES-034-WHAT-A-RULE-AUTHORING-SURFACE-IS-AT-SEVENTY-RULES` is where it is
answered.

`nomos-platform` compiling without XVPE is also on this board and is **not** authorized here:
`OD-ROADMAP-005` decision 4 already authorizes and bounds it, and restating it would create a
second authority for one piece.

## The Two Refused On Measurement Are Decided, Not Superseded

Both of these were refused because somebody measured the thing and found the gap answered, not
because a sequencing condition had not arrived. Superseding a deferral that does not exist
would produce an item to build an artifact with nothing for it to do, and **a directive to
build does not make an inert artifact useful.** So each is decided here, with its reason.

### The demand planner: the requirement is satisfied because the gap is answered, not open

`OD-RULES-009`'s latest round found that a planner today "would schedule an ordering that does
not exist, over a choice that has no alternatives, using cache state nothing consults", called
that speculative rather than deferred infrastructure, and named three things that would change
it. Those three were checked 2026-09-14. **They are re-checked here at `66292b7f` rather than
quoted, because a quoted trigger is worth nothing without the check.** None has fired:

- **No fact family's production depends on another family's output.** The only reads of the
  store anywhere under `nomos-check-orchestration`'s materialization are two currency checks,
  each asking whether the fact about to be written is the one the store is already serving under
  that identity. No section takes another section's fact as its input, so there is still no
  order to get wrong.
- **No capability has two installed providers whose choice is not decided by the requirement.**
  The composition root declares fourteen capabilities against seventeen offers. Twelve
  capabilities have exactly one offer each. Syntax has three: the Rust parser strictly dominates
  the Rust scanner on every axis they differ on, and the Go provider is partitioned by subject
  through a path recognition the composition root computes before anything is digested into a
  `SubjectId` and attaches to the requirement, which is `OD-CAPABILITY-009`'s answer rather than
  a choice anybody schedules. Dependency has two, and the second never clears the requirement's
  own floor, so ranking never sees two comparable offers. Two further providers have been built
  since the last check — a C# syntax provider and a compiler-backed Rust provider — and neither
  is offered by any composition root, an uncomposed state this workspace already declares and
  guards rather than leaves silent.
- **No measured cost makes skipping an undemanded family worth deciding rather than deriving.**
  Demand is still a union over `nomos_rules::DESCRIPTORS` that reads no cache state, no provider
  cost and no structure between families. The only measurement taken in this area since the last
  check moved the cost the other way rather than up: `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY`
  records, in its own commit message, that every non-syntax family now proves its fact current
  before filing it, where the eleven non-syntax families previously wrote unconditionally.

So the owner's requirement is **already satisfied for this gap**, because the gap is answered
rather than open. What the review wanted from a planner — that materialization does only the
work the selected rules demand, from one authority rather than a hand-written second one — is
what `Demanded_Families` does by derivation. Building a planner beside it would add a mechanism
with no ordering to sequence, no alternative to arbitrate and no cache state to consult, and it
would be a second statement of the rule-to-fact relation for the first to drift against, which
is the defect that removal was made to fix.

`OD-ROADMAP-001` does not license it. That record's own amendment says what its licence answers:
whether a component may be built before anything **consumes** it. A planner's problem is not a
missing consumer; it is a missing decision. Those are different populations of zero and only the
first is covered.

`OD-RULES-009`'s three conditions stand exactly as that record states them, this record retires
none of them, and none of its earlier rounds is reopened.

### Rules loaded from package files: the requirement is satisfied for the same reason

`OD-RULES-022` decided that composition resolves a **declaration** against a linked
**implementation**, that the two are different artifacts owned by different layers, and — in its
own words — that "a manifest cannot conjure a function, and this record does not pretend
otherwise." Measured at `66292b7f` rather than assumed: the declaration side is built and
derives every rule package from `nomos_rules::DESCRIPTORS`; the resolution step is built and
refuses in both directions, an unmatched mechanical declaration and an unmatched registration
alike; and no caller anywhere in this workspace reads a rule-package manifest from disk, nor is
there one on disk to read.

So a rule package file would buy one of two things and neither is the thing the review wanted.
For a rule this build links, it is a second source of declarations beside the one every side
already derives from — precisely the duplicated authority this repository files records about.
For a rule this build does not link, it is a declaration with no function behind it, which
resolution refuses if it claims to be mechanical, and which contributes no finding to a
deterministic run if it declares itself model-judged. The second is a truthful thing for a
declaration to be, and it is inert.

The owner's requirement is therefore **already satisfied for this gap too**. The question that
is genuinely open is not where a declaration is stored but whether a rule can be authored
without writing Rust, and that is section 6's question, answered by
`P128-OD-RULES-034-WHAT-A-RULE-AUTHORING-SURFACE-IS-AT-SEVENTY-RULES` and not here.

## What This Does Not Do

**It builds nothing.** Every piece above is an item on `work/ledger.json` carrying its own
territory and its own verification predicate. This record moves no code.

**It does not reopen `OD-ROADMAP-005`'s eight pieces**, which are decided, bounded and in
flight, and it does not restate the one of them that is also on this board.

**It does not touch `ARC-ROADMAP-001`'s deferred tier.** Atlas, architecture discovery and
feature topology stay where that record puts them, and nothing here is evidence about any of
them.

**It does not retire a trigger in any record it names.** `OD-WORKFLOW-002`'s three conditions,
`OD-ANALYSIS-009`'s four revisit conditions and `OD-RULES-009`'s three all stand as their
records state them. A superseded clause is a clause this record moves past; a trigger is a
measurement that has not fired, and the two are not the same thing.

**It does not widen a superseded clause into its record.** Each supersession above is bounded to
the sentence quoted, in the version stated, and a piece that turns out to need a second clause of
the same record is a new question rather than an extension of this one.

**It does not leave a superseded record to correct itself.** Each record named above stays false
in one clause until an item amends it, and — measured on the board at the time of writing — none
of the three items in sections 1, 2 and 4 reserves any path under `docs/records/` or
`crates/spec/nomos-spec-store/records/`, so none of them can make that repair inside its own
territory. The amendment is therefore a follow-on item that reserves the record, authored by
whoever lands the piece. That is not a novelty: it is the shape
`P123-OD-WORKFLOW-005-SAYS-THE-RETRY-AND-COMPENSATION-RUNTIME-IS-UNBUILT-AND-IT-LANDED` already
took, for a record whose building item reserved no record territory and whose claimant could
measure the staleness and not repair it.

**It does not promise an order.** These pieces contend for shared files in a tree several
sessions work at once. Which lands first is a coordination outcome, and an item blocked on a
peer's claim waits rather than reaching in.

## Consequences

The items named above are on the board as required work, each carrying its own territory,
predicate and `done_when`. Three of them are unblocked by this record landing. Two of the
review's remaining claims are closed here as answered rather than scheduled, and the reason is
recorded so a later reconciliation of the same review finds a decision rather than an omission.

## Status

Accepted. The bound is the enumeration above: four pieces against a named clause of a named
record at a stated version, six with nothing in their way, one question routed to its own
decision item, and two claims closed as answered rather than open — with every measurement
those records made left standing, and the two re-measurements this record needed taken rather
than quoted.
