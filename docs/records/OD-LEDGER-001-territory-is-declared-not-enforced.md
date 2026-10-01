---
id: OD-LEDGER-001
type: decision
title: Territory is declared but not enforced, and nothing yet notices the difference
status: open
version: 5
authority: canonical-normative-record
tags:
  - work-ledger
  - enforcement
relations:
  - target: ARC-SPECDB-001
    type: affects
  - target: OD-LEDGER-004
    type: relates-to
---

# Territory is declared but not enforced, and nothing yet notices the difference

## Question

Every ledger item declares a territory, and claiming refuses an item whose territory
overlaps one somebody already holds. Nothing checks that the work then stayed inside it.

## What Is Actually Enforced

Exclusion between competing claims, and only that. Claiming compares the requested territory
against the territories of items holding a live claim, and refuses on overlap or on an
unanswerable overlap question. That is the whole mechanism.

This paragraph named `Conflicts` until `OD-LEDGER-014` removed it. The mechanism it described
is unchanged and is now reached only through `Claim`, which asks the same question under the
lock; the standalone method had lost its last caller and a record describing a method nobody
can call is a promise the crate no longer keeps.

An agent holding a claim may edit any file in the repository. The territory is a promise
about where it intends to write, and a promise is what this system exists to stop relying
on.

## The Evidence

P2-RECORDS declared its territory as `docs/records`. Completing it required changes to
`nomos-spec-model`, `nomos-spec-store`, `nomos-spec-ingest` and `nomos-spec-bundle` —
four crates, none of them named. The item finished, its predicate passed, and the ledger
reported valid. Nothing was wrong with the work; what is wrong is that nothing could tell.

Two failures are being conflated here and they need separating. One is that the territory
was authored too narrowly, which is an authoring mistake and will recur. The other is that
an authoring mistake of this kind is invisible, which is a design gap.

## It Recurred, Three Times Running

Phase 8 was four items. Three of them worked outside their claimed territory, and the
authoring mistake was the same shape each time — the item named where the *thinking* would
happen and not where the *writing* would.

**P8-COMPOSE** claimed `tests/integration` and wrote `docs/records/OD-ANALYSIS-001`. The
item's own `why` was a finding waiting to happen; that a finding gets written down was not
foreseen by the territory.

**P8-PIN** claimed `nomos-analysis`, `tests/integration` and `docs/records`, and changed
`nomos-lang-rust` — which constructs a `FactKey` and could not survive the field being
removed. The item was authored against the crate that *owns* the type rather than against
the crates that *name* it.

**P8-SECOND-PROVIDER** claimed `crates/languages` and `tests/integration`, and changed the
root `Cargo.toml` and `tests/contract/tests/boundaries.rs`. A new crate cannot join this
workspace without both: one to be built, the other to pass
`Test_Every_Member_Should_Declare_A_Band`.

Every one was caught by the author and named in a commit message. That is not the mechanism
working — it is the mechanism absent and somebody being careful, which is the arrangement
this whole system exists to stop relying on.

The proportion is what makes this worth amending the record for. One instance was an
authoring mistake. Three consecutive instances across items authored by different reasoning
is a pattern, and it says the authoring half is the larger half — the enforcement gap merely
made it invisible, but the check that closes the gap will *fail three items in four* until
the authoring changes too.

## What Would Reduce It Now

Two rules, both checkable by a person writing an item, neither needing the rule engine:

**An item that adds a crate claims the workspace manifest and the band table.** Cargo and
`tests/contract` both require it; no new crate has ever landed without touching both.

**An item that will produce a decision claims the record it will write — by identifier, not
the directory records live in.** Any item whose `why` names an open question or a suspected
defect will produce a record, and "will it?" is answerable when the item is authored rather
than when it finishes. *Which* record is answerable then too: reserve
`docs/records/OD-<AREA>-<NNN>`, the identifier the item will allocate. An item that cannot
say which record it will write has not been thought through far enough to claim.

Reserve the identifier and stop there. The slug on the end of the filename is not knowable
when the item is authored, and a pattern is worse than either — `Territory::Intersect`
answers `Unknown` for a territory carrying one, and `Unknown` refuses.

**An item that will write a canonical record also claims that record's registration file.**
A record carrying `authority: canonical-normative-record` still cannot land alone:
`Test_Every_Canonical_Record_On_Disk_Should_Be_Governing` compares `docs/records` against
`GOVERNING_RECORD_IDS` in both directions, so the record file and its declaration move
together or the workspace goes red. The declaration is now one file per record —
`crates/spec/nomos-spec-store/records/<ID>.record`, named for the identifier the item is
already reserving — so reserve that path and stop there.

This rule used to say the item claims `crates/spec/nomos-spec-store`, the whole crate,
because the declaration was two shared lists in `governing.rs` and a literal count in a test
beside them. That made every record writer exclude every other one, which is the same
failure the previous rule had at directory granularity, arriving one level up:
`OD-LEDGER-007` named the crate a structural serializer for exactly this reason, and twelve
open items had been authored to reserve it. `OD-SPEC-007` dissolved the coupling — the
declaration is per-record, the count assertion is a floor that additions do not touch — so
the reservation is now the registration file. Territories authored before that record still
name the crate; they are history rather than a rule, and they are re-authored by the pass
that empties `KNOWN_SERIALIZERS`.

None is a check and none pretends to be. They are the three cases that have actually
recurred, written down so the next item can be authored past them.

### Why this rule says "the record" and not "`docs/records`"

Because the first version of it, which said `docs/records`, serialized the entire board.
Every item on an audit ledger names a suspected defect in its `why`, so every item produces
a record, so a rule requiring each of them to reserve one shared directory made every item
exclude every other. Measured on 2026-08-09: one claim, eight refusals, nothing claimable.

That is not a reason to weaken the rule. Under-declared territory was and remains the more
expensive failure — this record exists because of three instances of it. The correction is
to the granularity only, and OD-LEDGER-004 carries the evidence, the controls, and the guard
that keeps it.

## What Would Close It

`nomos.rules.work-ledger`, over a real changeset: the set of paths a holder modified must be
contained in the territory it claimed. Version 4 said this required only "the changeset model
and the rule engine," and named Phase 10 as the point both would exist. Both now exist by
name — `nomos_workspace::WorkspaceChangeSet` and `nomos-rules` with two real rules — and at
Phase 13 that reads as the gate having cleared. Checked directly against the tree, it has
not, on three separate points:

**No producer builds a changeset from what a holder actually did.** The only non-test
`WorkspaceChangeSet` producer in the workspace is `As_One_Checkout`
(`crates/orchestration/nomos-check-orchestration/src/facts.rs`): it wraps every `SourceFile`
a tree walk already read as one `WorkspaceChangeSet::From(ChangeSource::GitCheckout)` —
a full snapshot of the current disk, labeled `GitCheckout` by convention only. Nothing reads
actual git state to build one. "The changeset model . . . exists" was true of the type and
false of the thing this rule actually needs: a changeset scoped to one holder's own edits.

**Nothing records which edits are a holder's own.** `Claim` (`crates/substrate/nomos-ledger/
src/claim.rs`) carries exactly `holder`, `acquired_at`, `lease_expires_at` — no commit SHA,
no range. `VerificationRecord::revision` (`OD-LEDGER-027`) is a single point read at `finish`
time, not a range, and answers "what tree was this checked against," not "what did this
claim change." `OD-LEDGER-027` itself named this directly and declined it on cost grounds,
not on impossibility: "Not a subprocess `git` integration, and not a whole-tree or
territory-scoped digest — both are named above and both were rejected for the concrete
costs stated . . . a later item with a different question . . . may need one of them and
would decide that on its own terms." This is that later item, and its own terms have not
been decided yet — a per-claim start point (or an equivalent way to name "everything this
holder committed") is a ledger schema question this record does not resolve.

**The rule engine's one function shape does not fit this rule's subject.** Every rule
`nomos-rules` ships — `Check_Naming_Convention`, `Check_Completeness_Mirrors`
(`crates/rules/nomos-rules/src/{naming,mirror}.rs`) — has the signature
`fn(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>`: a subject built for
judging source text a capability already read. `nomos.rules.work-ledger`'s subject is a
changeset compared against a territory, neither of which is a `SourceFile` or reaches a
capability's `FactReader` at all. "The rule engine . . . exists" is true of the crate and
does not by itself mean this rule fits the one shape it currently offers; whether it needs a
second composition seam, or the existing one generalizes, is undecided.

None of the three is this record's to resolve — each is its own design question, and forcing
an answer here would be exactly the mistake `D-135` warns building generic machinery from a
wish produces. What this version corrects is the closing condition itself: "the changeset
model and the rule engine" is necessary but was never sufficient, and treating their
existence as the gate clearing is the same shape of premise error `D-130` was found to have
made about XVPE's compile state — true of what was checked, false of what the phase actually
needed.

Until all three are settled, the honest statement is unchanged from version 4: territory
prevents two agents from claiming the same ground, and does not prevent one agent from
working outside its own. Declaring that plainly is worth more than a check that runs
nowhere, because a stated gap can be planned around and a false clean cannot.

## Status

Open, and deliberately not worked around. A partial check — comparing the working tree
against the territory at `finish` time — was considered and rejected: it cannot see edits
already committed, it cannot distinguish an agent's writes from a concurrent one's, and a
check that is wrong in both directions teaches people to ignore the ones that are right.

Amended at version 2 with three further instances and the two authoring rules they suggest.
The gap is unchanged; what changed is the evidence about which half of it is expensive.

Amended at version 3 because the second of those authoring rules, as written, made every
item on the ledger exclude every other one. The rule now reserves the record rather than the
directory. The enforcement gap this record is *about* is still open and still waits on
`nomos.rules.work-ledger`; what version 3 changes is only the granularity of a declaration,
which is the half that was already working.

Amended at version 4 for the same reason one level up: the *third* authoring rule reserved a
whole crate and so re-imposed, on the store, the exclusion version 3 had just removed from
`docs/records`. `OD-SPEC-007` made a record's declaration a file of its own, and the rule now
reserves that file. Again only the granularity moved — the rule still says a canonical record
cannot land without its declaration, because that is what `OD-SPEC-005` was.

Amended at version 5 because version 4's closing condition, checked directly against the
Phase-13 tree, was not actually satisfied by what now exists under those two names. The
changeset type and the rule engine crate are both real, but no changeset producer reads git
history, no claim records which commits it covers, and the rule engine's one function shape
does not take this rule's subject. "What Would Close It" now states the three unresolved
questions those checks actually found, instead of the two-item list that reads as already
satisfied. The enforcement gap itself is unchanged; what changed, again, is the evidence
about how far from closed it still is.
