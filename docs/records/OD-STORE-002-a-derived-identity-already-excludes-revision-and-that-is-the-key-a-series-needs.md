---
id: OD-STORE-002
type: decision
title: A derived identity already excludes revision, and that is the key a series across revisions needs
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - identity
  - history
  - fact-store
  - reproducibility
relations:
  - target: OD-MODEL-002
    type: relates-to
  - target: OD-ANALYSIS-001
    type: relates-to
  - target: OD-GATE-002
    type: relates-to
---

# A derived identity already excludes revision, and that is the key a series across revisions needs

## Question

`crates/contracts/nomos-contracts/src/identity.rs`'s module doc names history as one of
five comparative features that killed the prototype, all for the same reason: every one of
them was a join with no key. Every check in this workspace inspects one tree, and nothing
compares a run against the runs before it — including the one case where both sides of that
comparison are already committed: `tests/contract/surface/<crate>.txt` holds a derived
snapshot of every crate's exported declarations, `OD-GATE-002` made re-blessing it a
deliberate act, and every public API change in this workspace's history is therefore sitting
in git, unqueried, next to the question of whether it shipped with a governing record.

The reason that query has never been written is not effort. It is that the temporal subject
was never decided. `OD-ANALYSIS-001` and `P8-PIN` removed a workspace state from `FactKey`
deliberately, which settles what a fact's identity does *not* include, but settles nothing
about what happens when the workspace state a fact was measured against is not the current
one. Three things are undecided going in: whether a fact from an earlier revision is
retained anywhere, what addresses one entity's story across many revisions, and what makes
two facts from different revisions the *same* fact rather than two unrelated ones that
happen to compare equal by accident.

## What Is Already Decided, Read Correctly

Nothing new is required to answer any of the three. The codebase already contains the
answer, stated twice, independently, at two different layers, and this record's only work
is to name the pattern and make it binding rather than incidental.

**`crates/substrate/nomos-analysis/src/fact/materialized.rs`.** `MaterializedFact::snapshot`
carries the `SnapshotId` a fact was measured against, and its own doc comment says why it is
there and why it is not part of `FactKey`: "provenance, not identity… deliberately not
restamped when a fact is reused: a reused fact is not a repeated observation." `FactKey`
carries `subject: SubjectId` and `semantic_inputs: InputDigest` instead — bytes drawn from
what the fact is about, not from which tree it was read from.

**`crates/kernel/nomos-model/src/identity/composite_identity.rs`.** `SourceProvenance`
carries a `revision: String` field, documented as "the revision it was read at," and
`CompositeIdentity::Entity_Id` does not hash it: the `Digest_Of_Parts` call feeds in
`language`, `qualified_name`, `signature`, the structural fingerprint and
`provenance.repository`, and stops there. `Test_Identity_Should_Not_Depend_On_Location`
changes `provenance.revision` between two constructions and asserts the resulting
`EntityId`s are equal. That assertion is not incidental; it is the same claim
`OD-MODEL-002` makes generally — a derived identifier "is a function of the corpus and
nothing else… run it twice over an unchanged corpus and the bytes fed to the hash do not
move" — read one layer down, at the one entity family the crate ships a concrete
construction for.

Both layers carry the revision as data beside the identity and exclude it from the identity
itself, for the same reason `identity.rs`'s own module doc gives for excluding machine- and
run-local inputs generally: an identifier answers "which thing," never "when was it looked
at." That is already the key `identity.rs` says history needs. A derived identity, taken as
`OD-MODEL-002` defines it, is *already* invariant across a change of revision for an
unchanged subject — not because this record adds that property, but because deriving rather
than minting was always going to have that property, and nobody had connected it to the
"join with no key" the module doc names as history's failure mode until this item asked the
question directly.

## The Rule

> **Facts from a prior revision are not retained.** The live fact store
> (`MemoryFactStore`) is in-process and generation-scoped: an entry is materialized,
> optionally invalidated at a later generation within the same run, and nothing in it
> survives past the process that built it or reaches back across a git revision boundary. A
> claim about a prior revision is not served from a Nomos store. It is reconstructed —
> today, by checking out that revision and rematerializing over it; for the worked case
> below, directly from git, because the evidence committed there already *is* the
> reconstruction.
>
> **A series across revisions is addressed by the same derived identity `OD-MODEL-002`
> already assigns the entity family, not by a new construct.** Two observations of the same
> semantic address, taken on two different revisions, already carry the same `SubjectId` or
> `EntityId` today, because the derivation excludes revision by the rule stated above. That
> identity is the series key. Nothing under this record computes a new one.
>
> **Two facts from different revisions are the same fact when their derived identity
> agrees and their recorded revision differs — never on equal identity alone and never on
> equal revision alone.** Equal identity with equal revision is one fact observed twice, not
> a series. Equal revision with different identity is two different facts about the same
> tree, which is what the fact store already handles.
>
> **A derived identity that bakes in the attribute a comparison exists to observe is the
> wrong grain for that comparison's series key, and this is a decided tradeoff already
> visible in the model that carries it, not a defect this record is finding.**
> `CompositeIdentity::Entity_Id` folds `signature` into the same digest as `qualified_name`,
> gated by `IdentityPolicy::distinguish_overloads` — and the policy's own test names the
> cost: an identity that "churns whenever a parameter type is reformatted." A series meant to
> observe *whether* a signature changed cannot use the identity that changes the instant it
> does; it must use the coarser coordinate held fixed underneath — language, qualified name,
> declaring repository — and treat the folded-in attribute as the value being tracked at
> each point in the series, not as part of the key that addresses the series. This is the
> same discipline `identity.rs` already applies once, generally; a series key applies it a
> second time, locally, to whichever attribute the comparison is about.

## What A Temporal Claim Is Evidence Of, And What It Is Not

A claim computed over accepted revisions — a trend, a recurrence, a "this kept happening" —
is `EvidenceClass::Derived` at best: "computed from other facts by a deterministic rule. No
stronger than its inputs," per `crates/contracts/nomos-contracts/src/finding/evidence.rs`.
It is never `Verified` or `Observed`, both of which that same enum reserves for a claim about
one tree checked by a mechanism that would have caught its negation. A history is a
computation over a sequence of past `Verified` or `Observed` claims, and `Weaker_Of` already
states the arithmetic: a derivation over its inputs is never stronger than the weakest of
them, and a trend inherits nothing but what accepting each revision along the way already
established.

The distinction has one operational consequence, and it is the one `done_when` asked this
record to state rather than leave implicit: **a temporal claim must never gate the change
that produced the observation the trend depends on.** A live check on the current tree is
evidence the tree is or is not in a state; a trend across ten accepted revisions is evidence
about a pattern across those ten revisions, and treating it as if it were evidence about the
eleventh — the one being proposed right now — lets a historical inference block a change on
evidence about *other* changes, none of which is the one in front of the reviewer. A
regression-across-revisions report, a co-change signal, a recurring finding: every one of
these is a `Derived` claim to be read alongside a live check, never substituted for one, and
never wired to a mechanism — a gate, a lint, an auto-block — that a `Verified` claim alone is
entitled to drive.

## The Worked Case

`tests/contract/surface/<crate>.txt` is already, without any change this record makes, a
committed time series. `Surface::package` (`tests/contract/src/surface.rs`) names the crate;
that name is the series' address, human-authored and stable across every revision that does
not rename the crate itself — the same shape `identity.rs` gives a `Named_Identity`, whether
or not it is wrapped as one. `Surface::declarations` is the value at one point in that
series, and `OD-GATE-002`'s re-blessing mechanism — `NOMOS_SURFACE_BLESS` rewrites the file
and then fails — is what turns a change of that value into a commit somebody had to make on
purpose.

The comparison this item was opened to make possible is now expressible entirely in terms
already committed:

> For a crate `C` and a commit range, does the blob at `tests/contract/surface/C.txt` differ
> between the range's endpoints, and does any commit in that same range touch
> `docs/records/`? A `yes` to the first and `no` to the second is a public API change with no
> accompanying record.

Both sides of that join are git history and nothing else: `git log -p -- tests/contract/surface/C.txt`
answers the first, `git log --name-only <range> -- docs/records/` answers the second, and
the series key that makes "the same crate's surface" a well-formed subject across the range
is `C` itself — the file's own name, exactly as this record's rule says a series key must
be: the coordinate held fixed, not the value being watched for change. Writing the query
that runs this join is not this record's territory; naming the key it needs, so that writing
it is a query and not a research question, is.

## What Was Considered And Rejected

**A third identity shape for "series identity," alongside `Digest128` and the
authored-string shape `identity.rs` already names.** Rejected for the reason `OD-MODEL-002`
gives generally: deciding an identity question once per family, instead of once here, is how
it ends up answered differently for each one. The two existing shapes, read with revision
excluded, already serve; a third shape would duplicate what they already do rather than add
anything they cannot.

**Retaining facts from prior revisions in the live store, keyed by revision, so a series
query reads from Nomos rather than from git.** Rejected because it duplicates a durable
history git already is. `MemoryFactStore` is in-process by construction and nothing in this
item's territory changes that; the worked case above is evidence that the duplication is not
needed even for the concrete comparison this item exists to unlock.

**Treating a `Derived` temporal claim as equivalent to a live check once enough revisions
agree.** Rejected in the section above. No number of accepted revisions promotes a trend to
evidence about the revision that has not been accepted yet; `EvidenceClass` already has no
path from `Derived` to `Verified` that does not go through checking the thing itself.

## What This Does Not Decide

Not the query implementation, the storage of its result, or which corpus-gated test would
run it. This record names the key; building on it is further work, reachable now that the
join has one.

Not a general history feature for every entity family in one motion. The rule above applies
per family as it is reached, the same way `OD-MODEL-002`'s derivation obligation does — this
record settles the shape the answer takes, not every family's answer at once.

Not a change to `FactKey`, `MaterializedFact`, `CompositeIdentity`, `SourceProvenance`, or
`IdentityPolicy`. Every one of them already has the shape this record describes; none of
them is edited by it.

## Status

Accepted. It states that prior-revision facts are not retained by the live store, that a
series across revisions is addressed by the entity family's existing derived identity read
at the coordinate that excludes the attribute under observation, that two such facts compare
as the same fact on agreeing identity and differing revision, that a temporal claim is
`Derived` evidence and never a substitute for a live check, and it works the public-API/
no-record comparison through concretely as the case both sides of which are committed today.
