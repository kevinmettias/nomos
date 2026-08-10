---
id: OD-LEDGER-017
type: decision
title: Six work-ledger requirements bind this build, and the review that produced them prices each divergence
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - corpus-authority
  - traceability
  - verification
relations:
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-TRACE-002
    type: relates-to
  - target: OD-LEDGER-001
    type: affects
  - target: OD-LEDGER-006
    type: relates-to
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-PACKAGE-002
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
  - target: D-132
    type: affected_by
---

# Six work-ledger requirements bind this build, and the review that produced them prices each divergence

## Question

`WORK-LEDGER-001` through `WORK-LEDGER-006` are `status: normative`,
`authority: canonical-normative-record`, `maturity: accepted`, in
`volume-12.10-4-2-work-ledger-verification-and-decomposition`. They govern the work ledger.
This repository has a work ledger, `OD-TRACE-001` found three of the six diverging by a
single element each, and `OD-TRACE-002` then found that two of those divergences **cannot be
entered in the assessment registry at all** until a record says why — a `Diverges` entry with
no record is refused by the reader, which is `OD-TRACE-001`'s rule enforced rather than
restated.

So the question is not whether to notice. It is which of the six bind, which are met, and for
each one that is not met, what the gap costs and what would make deferring it wrong.

`P10-LEDGER-CORPUS`'s `done_when` refuses a blanket answer, and it is right to: `003` is
already satisfied and `002` has no counterpart in the schema at all, so one verdict covering
all six would be a verdict about none of them.

## What Was Read, And The Document That Changes The Answers

The six requirement artifacts under
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/nomos-spec-internal-artifacts/01_authoring/artifacts/requirements/`,
each restated verbatim in `01_authoring/markdown_volumes/12-12-roadmap-quality-decisions-and-traceability/index.md`
at line 128 and following.

And one document that was not read when `P10-LEDGER-CORPUS` was authored, which is where most
of this record comes from: **`00_overview/WORK_LEDGER_REVIEW.md`, 182 lines.** It is the review
that produced all six requirements, and it names its own subject on line 3:

> Source reviewed: `docs/nomos-spec-internal-artifacts/01_authoring/work_ledger/tasks/`

That directory exists in the corpus and holds **216 task files** today. Its schema is
`nomos.work-ledger.task.v1`; `TASK-0001` carries `"work_item_id": "F3"`, `"priority": 2`,
`"state": "done"`, a `territory` array of globs, `depends_on`, `blocks`, and a
`verified.command`. A Go tool — `go run ./tools/work-ledger` — is its write path.

**This matters for every verdict below.** The six requirements are not abstract statements
about work ledgers in general. They are a review of *this product's own previous work ledger*,
written by somebody who had parsed 182 items of it and counted what the model actually
delivered. Every one of the six has a measured origin in that document, and four of them come
with the review's own numbers on how well the reviewed ledger honoured its own rule.

## All Six Bind, And That Is Settled Before The Individual Verdicts

`OD-PACKAGE-001`'s shape is available here and is deliberately **not** used. That record could
read `ARCH-001` as not reaching this build, because "independently versioned is a property of a
package and this build has no package" — the requirement's subject was absent.

No such reading is available for any of these six. The reviewed artifact is this product's
work ledger and `work/ledger.json` is this product's work ledger; the second is a Rust rebuild
of the subsystem the first is a Go implementation of. The subject is present, and it is the
same subject.

So **"does not reach this build" is refused as a verdict for all six**, and it is refused here,
once, rather than being separately declined six times below. What remains per requirement is:
met, or diverging with a price.

Stated plainly because it is the load-bearing move in this record and the cheapest thing here
to get wrong in the convenient direction.

## WORK-LEDGER-003 — Met

> A non-declined item should declare a verification predicate before work starts. Prose-only
> `done_when` text is planning guidance, not executable completion evidence.

The review's origin for it, from Testing Status Lessons:

> Open work needs predicates earlier. `done_when` is useful prose, but the sweep identified 52
> open items with no machine-executable predicate.

and, from Recommended Next Changes:

> Require `verification.command` or `verified.self_reported_reason` when new non-declined items
> are created.

Measured on this board, at the commit this record lands on: **101 items, 0 without a
verification predicate.** Not 0 among the open ones — 0 among all of them. The reviewed ledger
had 52 of 182 open items with no predicate; this one has none.

The distinction the requirement's second sentence draws is also structural here rather than
observed. `VerificationPredicate`'s own doc comment says it "is also a *predicate*, not a
description. `done_when` on the item is prose for a human", `LedgerItem::done_when` says
"Paired with — never a substitute for — `verification`", and `Is_Runnable` refuses an empty
argv as "a field somebody filled in to satisfy a schema". `work finish` keeps *no predicate at
all* as its own answer, distinct from the predicate failing.

`Met`, and satisfied more strongly than a `should` asks for.

**What is not claimed.** Nothing *enforces* the predicate at `add` time — `verification` is
`Option`, and `work add` accepts an item with no `--` clause. The 0 is a property of how this
board has been authored, not a guard. That is the honest limit, and it is not upgraded to a
guard here: `WORK-LEDGER-003` is a `should`, the review's own remedy was to *require* it at
creation, and turning `Option` into a required field is a schema change this item's `done_when`
sends to a follow-on.

## WORK-LEDGER-005 — Diverges by one cause, and the cause is declined

> Blocked work shall classify the blocker as dependency, decision, territory mismatch,
> external resource, needs-split, stale probe artifact, or other typed cause rather than
> relying only on prose notes.

`Blocker` in `crates/substrate/nomos-ledger/src/item.rs` declares six, in exactly that order,
with matching semantics. `StaleProbeArtifact` is absent, and before this record the string
"stale probe" appeared in this repository only inside `P10-LEDGER-CORPUS`'s own `why`.

`P10-LEDGER-CORPUS` names this the strongest evidence in the item and the cheapest thing to get
wrong. It is answered from the sentence the requirement was derived from —
`WORK_LEDGER_REVIEW.md`, line 71, quoted in full:

> `blocked` currently covers dependencies, decisions, territory mismatches, stale probe
> artifacts, and external resource limits. Those should be typed as `blocked_by_item`,
> `blocked_by_decision`, `blocked_by_territory`, `blocked_by_external`, `needs_split`, or
> `remove_probe_artifact`.

**The seven causes are an inventory, not a design.** The first clause enumerates what the
reviewed ledger's free-prose `blocked` field was *observed to already contain* across 31
blocked items. The requirement types that inventory. Five of the causes are generic — any board
can have a dependency, a decision, a territory mismatch, an external resource, an item too
large to claim. "Stale probe artifact" is the one that names a specific artifact class, because
the reviewed ledger had blocked items waiting on one.

Two further facts from the same sentence:

- The suggested type name is **`remove_probe_artifact`** — an *action*, imperative, unlike the
  five `blocked_by_*` siblings. The reviewer's own naming treats it as a chore to discharge
  rather than a state to wait in.
- `stale probe artifact` occurs **nowhere else in the entire corpus.** Measured across the v14
  artifact tree, every domain volume, the machine catalog, the retrieval chunks and the
  published HTML: every occurrence is a restatement of `WORK-LEDGER-005` itself or this one
  review line. No requirement defines a probe artifact, no volume describes one, and no other
  requirement under `01_authoring/artifacts/requirements/` contains the word "probe" at all.

So the cause has no definition in the corpus outside the ledger whose prose it was lifted from.

### The nearest thing this build has is not a blocker

The honest test is not "does the word appear here" but "can an item on this board be blocked by
a stale generated artifact". The candidate is real and worth naming rather than waving past: a
committed projection going stale. `diagrams/relations.mmd` is derived, a record commit makes it
stale, and `P10-DIAGRAM-OWED` existed because the gate was red for eleven commits over exactly
that.

**It is still not a `Blocker`, and the reason is structural rather than terminological.**
`Blocker`'s doc comment is "Why an item cannot be worked on", and the prototype's lesson
recorded there is that a typed cause exists so a query can answer "what is waiting on a person
versus waiting on a dependency". Every one of the six declared causes names something *outside
the holder's own hands*. A stale projection is inside them: `OD-GATE-005` decision 3 settles
that the obligation is to render from the record set your commit publishes, in a tree the
author constructs rather than waits for. It never stops an item being worked, it stops a commit
being clean, and the remedy needs no second party. Recording it as a blocker would put a chore
into the field that answers "who am I waiting for", which is the queryability the requirement
exists to buy.

### The decision, and what would reverse it

**`StaleProbeArtifact` is declined, not missing.** `Blocker` declares six of
`WORK-LEDGER-005`'s seven typed causes deliberately.

`WORK-LEDGER-005` is not violated by the decline, and this is the reason it is a decline rather
than a deferral. The requirement's obligation is *type the blocker rather than relying only on
prose notes*, and its own list ends "**or other typed cause**". `Blocker::Other { detail }` is
declared, so every blocker this build can have is expressible as a typed cause. What a missing
named variant costs is queryability for that one cause — and a cause with no referent here is a
query nobody can ask.

**What makes this wrong:** an item on this board having to wait on *somebody else* regenerating
a derived artifact — a corpus re-ingest, a bundle re-export, a projection only another holder's
territory can re-render. That is a blocker in the sense `Blocker` means, it is a stale artifact,
and the variant is owed the day it happens. Until then, `Other { detail }` carries it and the
detail says what.

**What this is not:** it is not a claim that six of seven is the right count in general, and not
a licence to drop a seventh member of some other corpus enumeration. `OD-CONTRACTS-002` reached
the opposite verdict on the structurally identical finding — `Applicability` missing one of
`CHK-003`'s seven reporting categories — and built the seventh, because *that* referent existed.
Two of `OD-TRACE-001`'s three single-element divergences were derived the same way and are
being answered in opposite directions, on the referent and nothing else.

## WORK-LEDGER-001 — Diverges by `priority`, deferred, and the follow-on is ranking

> A work-ledger item shall be addressable as an individual artifact with stable id, title,
> state, priority, rationale, territory, dependencies, blockers, claim lease, completion
> condition, verification predicate, and recorded completion evidence.

`LedgerItem` carries `id`, `title`, `state`, `why`, `territory`, `depends_on`, `blocked`,
`claim` (which holds `acquired_at` and `lease_expires_at`), `done_when`, `verification` and
`verified`. `priority` is absent, and the string appears nowhere in `nomos-ledger` or
`nomos-cli`.

**One correction to `OD-TRACE-001`, which recorded "twelve of `WORK-LEDGER-001`'s thirteen
fields".** The statement's list is **twelve** comma-delimited elements, not thirteen; counted
against them, `LedgerItem` carries eleven. Thirteen is reachable only by reading "claim lease"
as two addressable things, which is defensible — `Claim` does carry both the claim and its
expiry — and then twelve are carried. Either segmentation gives the same divergence, **exactly
one element, and it is `priority`**, which is why the count did not mislead anyone. It is
corrected here because this record is what the assessment registry will cite, and a cited count
that cannot be reproduced from the sentence sends its next reader looking for a thirteenth noun
that is not there.

### What the corpus wanted `priority` for

Not addressability. From the review's Target Coordination Model:

> Dependency selection should operate on a graph. A task is ready only when its prerequisites
> are complete, its required tools are available, and its expected impact has no exclusive
> conflict with live work. **Ranking can start deterministic: priority first, then number of
> downstream descendants unblocked, then conflict risk.**

`priority` is the **first of three ranking keys** for automated selection over a 182-item graph.
The other two are *derivable from what this build already has*: "downstream descendants
unblocked" is a traversal of `depends_on`, and "conflict risk" is a function of `Territory`,
which this ledger already computes exactly for exclusion.

That reframes the gap. Adding a `priority` field alone would satisfy the letter of
`WORK-LEDGER-001` while buying **the weakest third** of what the corpus wanted the field for —
and it is the third that is a number somebody types and nobody maintains, against two that are
computed and cannot go stale.

### The price, measured

101 items: 88 `Done`, 9 `Ready`, 3 `Claimed`, 1 `Declined`. `nomos work list` prints all 101 in
file order with no ranking, and this is the surface an agent picks work from. So the cost is
paid in two places. An agent choosing an item reads titles and `why` prose to guess importance,
and the ordering that results is whatever the file happens to be in. And the listing is a
context cost on every session that reads the board — the same concern the review names for its
own ledger: "the task-record set is large enough that agents should usually read generated
per-item, state, feature, testing, or summary projections instead of loading all task files".

**Deferred.** `P10-LEDGER-CORPUS` is explicit that this is not closed by adding fields, and the
follow-on it sends this to is therefore **ranking, of which `priority` is one key** — not a bare
field. A `priority` integer landed on its own would be the third of three that ages worst,
landed first, with nothing computing the two that do not.

**What makes the deferral wrong:** the board acquiring enough concurrent holders that two agents
routinely pick the same next item, or the `Ready` count growing to where reading the titles is no
longer how anyone chooses. Nine `Ready` items is browsable; ninety is not.

## WORK-LEDGER-004 — Does not bind yet, and the schema is already shaped to violate it

> Completed work shall distinguish recorded verification at completion time from later
> reverification sweeps. A failed or timed-out sweep shall not erase the original completion
> evidence, but it shall create a visible regression or verification-gap record.

Its origin is a specific incident, from the review's Verified Findings:

> `go run ./tools/work-ledger -verify-all` was attempted during this migration review and
> exceeded the 120 second review budget. The done records are therefore recorded as previously
> verified by command, not freshly reverified by this migration pass.

and the remedy, from Testing Status Lessons:

> Verification needs two timestamps or statuses: `recorded verified` for the command run at
> close time, and `reverified` for later regression sweeps.

**This build has no reverification sweep.** Nothing re-runs a `Done` item's predicate, so
nothing today can erase completion evidence, and the requirement's obligation has no occasion
to be discharged or breached. That is why the verdict is not `Diverges`: there is no second
verification to fail to distinguish from the first.

**But `LedgerItem::verified` is a single `Option<VerificationRecord>`, and that is the shape
`OD-LEDGER-006` already named as a defect one scale down.** That record made `abandoned` a list
because "an item abandoned twice was abandoned twice, and keeping only the latest discards the
earlier reason"; `displaced` is a list for the same stated reason. A sweep writing `verified`
would overwrite the record written at completion — precisely the loss `WORK-LEDGER-004`'s second
sentence forbids, produced by a field shape rather than by a mistake in the sweep.

**Decision: the requirement binds the sweep, and the constraint is recorded now rather than
discovered then.** Whoever builds a reverification sweep must make the completion record and
the sweep record distinct before running one — `verified` becoming a list, or a sibling field,
which is that item's decision and not this one's. A sweep built over the current `Option` is a
defect this record has already described, and the point of writing it down before the sweep
exists is that the cheap moment is now.

## WORK-LEDGER-002 — Diverges wholly, deferred, and it is the ledger's unit of decomposition that differs

> Feature work shall be decomposed into implementation, unit-test, integration-test, and
> e2e-test dimensions. A feature is complete only when every dimension is done or explicitly
> declined with a reason.

There is no counterpart in the schema at all. `LedgerItem` has no `feature` and no dimension
vocabulary.

The reviewed ledger's shape, measured: a task record carries a `feature` field and a `dimension`
field, and the four dimensions are **four separate task files**, not four fields on one —
`TASK-0077--W2-ledger-testing-implementation.md` through
`TASK-0080--W2-ledger-testing-e2e-tests.md`, with `"dimension": "implementation"`,
`"unit-tests"`, `"integration-tests"`, `"e2e-tests"`. One of them records
`"done_when": "nothing: this dimension was declined"`.

### The review's own numbers on it

| Measure, from `WORK_LEDGER_REVIEW.md` | Count |
|---|---:|
| Items | 182 |
| Items with a declared feature dimension | 4 |
| Features represented by four dimensions | 1 |

> Testing status is present but underused. Only one feature uses the four-dimension model.

One feature in 182 items, and that feature is the ledger's own testing — the review names it
`W2-ledger-testing`, and its own `why` in the corpus says "dogfooding the four-dimension model
on the ledger itself". Its outcome: "implementation and unit-test dimensions are explicitly
declined, while integration and e2e dimensions are done."

This is recorded because it is the strongest evidence available about what the requirement costs
to honour, and it points the uncomfortable way: the ledger that *generated* this requirement
honoured it once, on itself, and the one use spent two of four dimensions on declining them.

**It does not make the requirement non-binding.** A normative accepted requirement is not
weakened by its author's own build being behind it — `D-132` puts the corpus above the plan, and
nothing in this record's authority chain lets adoption statistics outrank a `shall`.

### Why it is deferred rather than built

The unit differs. This ledger decomposes by **territory**, and that is load-bearing rather than
stylistic: territory is what mutual exclusion can be proven on, and `OD-LEDGER-001` records that
the whole point of a claim is keeping two authors out of one file. A feature with four testing
dimensions cuts *across* territory — the four `W2-ledger-testing` task files above would be four
items whose territories overlap by construction, which on this board is a set of claims that
refuse each other. Adopting `WORK-LEDGER-002`'s decomposition is therefore not a field addition;
it is a second decomposition axis alongside the one exclusion depends on.

The requirement's *substance* — "the ledger's strongest rule is that implementation-only
completion is not feature completion" — is met by a different mechanism. An item cannot reach
`Done` by assertion: `work finish` runs the derived gate step and the item's own predicate and
records `Done` only if both exit zero, and `GateOutcome` exists so that an item finished before
the gate was derived cannot read as though it had been checked.

**The price, stated rather than smoothed:** nothing on this board records that integration or
e2e coverage was *considered*. An item whose predicate is a unit-test invocation and one whose
predicate genuinely reaches an end-to-end path are indistinguishable from the outside, and a
dimension nobody thought about reads exactly like one deliberately declined. That is the same
class of defect as `OD-GATE-001`'s skipped test reporting `ok` and `P9-SKIP` before it, and this
record does not pretend otherwise.

**What makes the deferral wrong:** a `feature` grouping arriving on this board for any other
reason, or an item finishing `Done` whose predicate is later found not to have reached the layer
its `done_when` claimed. Either one makes the four dimensions cheap to add and the absence
expensive to keep.

## WORK-LEDGER-006 — Diverges, and it splits: two axes are reachable, two fall behind 002

> The operational ledger may remain a locked write surface, but the suite shall expose generated
> projections by item, state, feature, and testing status for review and agent context.

The first clause is honoured. `work/ledger.json` is a locked write surface — `OD-LEDGER-015`
put the lock on the three verbs that change the board after finding they read and wrote without
it.

The second is absent. **Eighteen projection profiles exist** — `api-documentation`,
`architecture-document`, `contract-yaml`, `diagram-set`, `domain-specification`,
`feature-design`, `github-markdown`, `html-site`, `implementation-context-pack`,
`mcp-resource`, `offline-bundle`, `release-specification`, `requirement-catalog`,
`subject-contract`, `subject-dossier`, `subject-model`, `subject-report`,
`traceability-matrix` — and every one of them projects the specification store. None reads
`work/ledger.json`. The string does not occur anywhere under `crates/spec/`.

The requirement's purpose, from the review, is agent context rather than review convenience:

> The task-record set is large enough that agents should usually read generated per-item, state,
> feature, testing, or summary projections instead of loading all task files.

> Generated item JSON gives token-efficient access without creating a second source of progress
> truth.

**The four axes do not have one verdict, and that is the finding.**

`item` and `state` are reachable now. `nomos work list` already answers both, and
`P10-SUBJECT-PROJECTIONS` landed the ability to point a profile at one subject. What is missing
is that the ledger is not a projection *subject*: the profile engine renders the specification
store, and `work/ledger.json` is not in it. That is a real architectural question — a second
substrate for a projection subject — and it is a follow-on item rather than a field.

`feature` and `testing status` are **not reachable, and they fall with `WORK-LEDGER-002`.** A
projection by feature presupposes a feature, and a projection by testing status presupposes the
four dimensions. Deferring `002` defers exactly half of `006`, and the two must move together or
not at all.

**Deferred, split.** The cost of the `item`/`state` half is the one this record already priced
under `WORK-LEDGER-001`: every session that reads the board reads all 101 items, because there
is no generated view narrower than the whole listing. The cost of the other half is
`WORK-LEDGER-002`'s cost, counted once.

**What makes the deferral wrong:** the board reaching a size where `work list` is itself the
context problem the review describes. At 101 items it is the largest single artifact a session
reads to orient, and it grows monotonically because `Done` items are never removed.

## Summary Of The Six

| Requirement | Verdict | Where it stands |
|---|---|---|
| `WORK-LEDGER-001` | Diverges by one element | `priority` absent. Deferred; the follow-on is ranking, of which priority is the weakest of three keys and the only one not already derivable. |
| `WORK-LEDGER-002` | Diverges wholly | No feature, no dimensions. Deferred: the decomposition axis conflicts with territory, which exclusion depends on. Substance met by predicate plus derived gate step; the cost is that a dimension nobody considered reads as one declined. |
| `WORK-LEDGER-003` | **Met** | 0 of 101 items without a predicate, against the reviewed ledger's 52 of 182 open. Not enforced at `add` time, and that is stated rather than claimed. |
| `WORK-LEDGER-004` | Does not bind yet | No sweep exists. The constraint on building one is recorded now: `verified` must stop being a single `Option` before anything re-runs a predicate. |
| `WORK-LEDGER-005` | Diverges by one cause | `StaleProbeArtifact` **declined**, referent absent, `Other` carries it, and the trigger that would reverse it is named. |
| `WORK-LEDGER-006` | Diverges, split | Lock half honoured. `item`/`state` deferred behind the ledger becoming a projection subject; `feature`/`testing status` fall with `002`. |

Six verdicts, five distinct. No blanket answer, which is what `P10-LEDGER-CORPUS` refused.

## What Holds It

**`WORK-LEDGER-005`'s answer is at the site**, in `item.rs`'s test module, in the shape
`OD-PACKAGE-002` used for `CORPUS_TAXONOMY` and for the reason `D-134` put a universe beside its
mirror: the next reader of `Blocker` must not have to re-derive the comparison from a corpus that
is not on their machine.

`CORPUS_CAUSES` transcribes `WORK-LEDGER-005`'s seven causes in the order the requirement names
them, each paired with the `Blocker` variant that carries it — or `None` for the one declined
above. A row is a claim about the corpus rather than a local preference, and its doc comment says
so.

`Test_The_Declared_Causes_Should_Be_The_Corpus_Causes_Minus_The_Declined_One` walks the six
declared variants, asserts each serializes as the name the table transcribes at the position the
table gives it, and asserts the occupied positions are exactly the table's non-`None` rows. A
relabelling, a reordering, a dropped variant and an added one all move it.

`Test_The_One_Declined_Cause_Should_Be_The_Stale_Probe_Artifact` pins *which* row is empty, so
the decline cannot be silently relocated to a different cause. A different row going empty is a
different decision and needs a different record.

A seventh variant is held by the compiler rather than by an assertion. `Corpus_Position` is an
exhaustive match, so adding `StaleProbeArtifact` — or anything else — makes the test module fail
to build, beside the table and this record's identifier, rather than adding a cause the
assertions would never visit. That is the mechanism, and its limit is the same one
`OD-PACKAGE-002` stated: it forces the author to arrive at the table, and it cannot force them to
argue honestly once there.

`LedgerItem`'s own doc comment names `priority` as the absent element and points here, so the
`WORK-LEDGER-001` verdict is also readable at the type rather than only in this file.

## What This Does Not Do

**It adds no field and changes no schema.** `SCHEMA_VERSION` does not move and the item field
count stays at 13. `P10-LEDGER-CORPUS` bounded this deliberately, and the reason is territorial
as well as editorial: the exhaustive undeclared-key probe lives in
`crates/substrate/nomos-ledger/tests/exclusion_holds.rs`, which open items already reserve.

**It writes no assessment entry.** `tests/contract/requirements/` is `OD-TRACE-002`'s registry
and is not in this item's territory. What this record supplies is the thing that registry is
waiting for — `OD-TRACE-002` states that `WORK-LEDGER-001` and `WORK-LEDGER-005` "cannot be
entered until `P10-LEDGER-CORPUS` writes the reasons" — so the two entries are now authorable and
a follow-on item carries them. Until they land,
`Test_Every_Divergence_Should_Name_A_Governing_Record` stays vacuous over the committed set, which
`OD-TRACE-002` already recorded as its own consequence.

**It does not re-verify the four requirements already assessed.** `EVID-001`, `CAP-002`,
`CAP-003` and `CHK-003` are `OD-TRACE-001`'s and `OD-CONTRACTS-002`'s subjects and are untouched.

**It does not make a `Met` entry self-verifying.** `WORK-LEDGER-003`'s verdict rests on a count
taken by hand against this board today. `OD-TRACE-001`'s semantic-drift limit applies unchanged:
a `Met` whose site still exists and whose behaviour stopped satisfying the requirement reads
exactly like one that did not.

**It does not reach the other 357 requirements.** Six of 363 are answered, and `OD-TRACE-001`
chose a floor over a count precisely so that this is a readable state rather than a failure.
