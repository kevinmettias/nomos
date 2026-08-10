---
id: OD-LEDGER-016
type: decision
title: A record identifier and the file it names are one subject
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-001
    type: affects
  - target: OD-LEDGER-004
    type: relates-to
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-LEDGER-011
    type: relates-to
  - target: OD-SPEC-006
    type: relates-to
---

# A record identifier and the file it names are one subject

## Question

`OD-LEDGER-001`'s authoring rule tells an item to reserve the record it will write by
identifier: `docs/records/OD-<AREA>-<NNN>`, not the directory records live in.
`P10-RECORD-LOCK` re-authored the whole board that way, and the board has been authored that
way ever since — every open item reserves an identifier.

An identifier is not a path. Nothing is ever at `docs/records/OD-LEDGER-006`; the file is
`docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md`.
Containment compares them by appending a separator, deliberately, so that `crates/a` does
not swallow `crates/abc`:

```rust
right.starts_with(&format!("{left}/")) || left.starts_with(&format!("{right}/"))
```

Neither is a prefix of the other *at a separator*, so the two are siblings and compare
`Disjoint`. Reserving a record's identifier reserved the identifier and nothing else.

## What That Cost

Two things, and the second is the one that makes it a defect in the mechanism rather than a
gap in a convention.

**An item amending an existing record excluded nobody from the file it edits.** The record
is on disk, another item may be editing it, and the reservation names something adjacent to
it. This was found while designing `P10-LAPSE-TAKEOVER`, which has to amend `OD-LEDGER-006`
and `OD-LEDGER-009` — both of which exist — and discovered that reserving their identifiers
reserved neither. It was worked around there by hand, by writing out both full filenames.

**The two spellings did not exclude each other.** So the hand-written workaround did not
work either: an item naming the file and an item naming the identifier are, to the ledger,
two items with nothing in common. Two agents could hold one record between them and both be
told the territory was disjoint — the single failure this ledger exists to prevent.

## What The Guard Could Not See

`Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` reduces every open record writer to
the record paths it reserves and claims all of them at once through a real ledger. Stems do
that correctly: distinct identifiers are distinct paths, and they contribute no exclusion.
The property is true, it was proved, and it is a property of *identifiers*.

It was read as a property of *records*, and it never was one. That reading is why this
survived `P10-RECORD-LOCK`, `P10-SEEDING-SERIALIZES` and `P10-SURFACE-GRAIN`: three items
looked straight at the record reservations and each of them was measuring a set of names
rather than a set of files.

### What `P10-RECORD-LOCK` Actually Bought

That two items writing two different records stop excluding each other — which was real, was
the thing it named, and remains true; what it did not buy, and never claimed to, is that two
items writing *one* record exclude each other, because the identifier it moved everybody
onto was not the file anybody would write.

## Decision

The identifier and the file fold onto one subject, in `Normalize_Path`, scoped to
`docs/records`. A path of the form `docs/records/<IDENTIFIER>-<slug>.md` reduces to
`docs/records/<identifier>` before its identity is computed.

The relation is **identity, not containment**. `docs/records/OD-LEDGER-006` does not contain
the record; it *is* the record, named the only way it can be named before its slug has been
written. Saying so in normalization rather than in `Contains_Or_Equals` is the whole of this
decision, and three things follow from it.

**`Contains_Or_Equals` stays a pure path relation.** `a/b` contains `a/b/c` and nothing
else, decided from the text with no filesystem access and no knowledge of what any directory
is for. `OD-LEDGER-001` describes territory that way and it is still true.

**The identity form is fixed too.** `Subject_Of` — and so `Territory::As_Subject_Set`, and
so anything that ever compares territories as digests — now answers that the two spellings
are one subject. A rule written into containment would have left the digest form still
saying they are unrelated, and left a second answer in the codebase waiting to be believed.

**The authoring rule needs no amendment.** `OD-LEDGER-001` says reserve the identifier and
stop there, because the slug is the writing and the writing is not knowable when the item is
authored. That reasoning was always correct. What was missing was not a better rule; it was
a mechanism that made the rule mean what it says.

### The Two Alternatives, And Why Not

**Teach containment that a stem covers a name.** Same effect on claims, and it says
something false: it makes the identifier the broader of two paths, so every derived
"which of these is the container" tie-break — `Shared_Paths`, `Covers`, the narrower-name
choice inside `Intersect` — starts reporting the identifier as a directory that has a file
in it. It also leaves `As_Subject_Set` wrong. The cost is not that it is harder; it is that
it puts the special case in the layer that is supposed to know nothing.

**Require the full filename once the file exists, and check it.** The check is buildable —
for every open item reserving `docs/records/<ID>`, fail if a matching file is on disk — and
it fails for a reason nothing fixes: at the moment an item is authored to *create* a record,
no filename exists to require. So the identifier has to stay reservable regardless, which
means both spellings stay in use, which means they have to fold anyway. The rule would then
be a second thing to remember on top of a mechanism that already worked, and it would put
the burden back on authoring — which is exactly where it already failed, twice, in the same
week.

## Blast Radius

`Territory::Intersect` is the one comparison every claim decision rests on, so what newly
overlaps has to be enumerated rather than assumed.

Newly overlapping: within `docs/records` only, a record's identifier and its file, and two
filenames carrying the same identifier. Nothing else moves. Paths outside the directory
reach `Record_Identifier_Form`, fail its first step and are returned unchanged, so every
existing subject identity outside `docs/records` is the identity it was.

The general version of this rule would be a catastrophe and is worth naming so nobody
reaches for it later: "a name contains the names that extend it with a hyphen" makes
`crates/nomos-spec` contain `crates/nomos-spec-model`, and the repository serializes
completely. That is the case containment appends a separator to avoid, and
`Test_A_Sibling_With_A_Shared_Prefix_Should_Not_Be_Contained` is the standing guard on it.

The scoping is therefore not a convenience. `docs/records` is the one directory where the
identifier-to-filename relation is written down rather than inferred: `OD-SPEC-006` makes it
the authoring substrate and nothing else, and every canonical record's registration under
`crates/spec/nomos-spec-store/records/<ID>.record` states the mapping as a `path:` line.

Three nearest misses are constructed as tests rather than reasoned about:

- `docs/records/OD-LEDGER-001` against `docs/records/OD-LEDGER-0011-a-much-later-record.md`.
  The ordinal is a whole component and `0011` is not `001`, so the hundred-and-first record
  is not the first. A rule written with `starts_with` would have folded them.
- `docs/records-archive/OD-LEDGER-001-an-old-copy.md` against `docs/records/OD-LEDGER-001`.
  A directory whose *name* extends the record directory is not inside it.
- `tests/fixtures/case-001` against `tests/fixtures/case-001-expected.md`. Outside the
  record directory an ordinal in a filename means nothing, and the two stay two subjects.

A fourth was found by the tests rather than by reasoning, which is the reason to write them
first: `2026-08-09-notes.md` reaches its first all-digit component at `08` and folded onto
`docs/records/2026-08`, so two unrelated notes from one month would have shared a subject.
An identifier now has to begin with a word.

The residual risk is over-folding and not under-folding: a file in `docs/records` whose name
happens to fit the grammar without being a record folds onto a name it does not mean, and
two items serialize over a record neither is writing. That is the same trade case folding
already makes, and it falls the same way — over-folding costs throughput, under-folding
costs an edit, and an edit does not come back.

## What Was Re-Authored, And What Was Not

Two open items had been spelling record filenames out by hand to compensate for this, and
both are back on the single rule: `P10-LAPSE-TAKEOVER` on `OD-LEDGER-006` and
`OD-LEDGER-009`, `P10-REFUSAL-SURFACE` on `OD-LEDGER-001`. `P10-FACT-BYPASS` carries the
same shape for `D-134` and was held at the time; a claimed item's territory is not somebody
else's to move, and under this decision both spellings mean the same thing anyway, so it
costs nothing to leave.

Re-authoring is not what closes this. Every item on the board could be spelled correctly
today and the next item to amend a record would spell it the way `P10-LAPSE-TAKEOVER` first
did, because that is the spelling the rule invites. What closes it is that both spellings
now name one subject, and a test says so against every record in the directory — with each
record's identifier read from its own front matter, so the two sides of that comparison stay
independently authored in the sense `OD-SPEC-007` settled.

## Status

Accepted. The enforcement gap `OD-LEDGER-001` is *about* — nothing checks that a holder
wrote inside its territory — is untouched and still waits on `nomos.rules.work-ledger`. What
changed is that a reservation now denotes the thing it was always meant to denote.

There is an irony here and it is only worth a sentence: the registration files are named by
identifier too, so this record is registered by `records/OD-LEDGER-016.record` — which is a
real path, resolves, and needs none of this, because there the identifier *is* the filename.
