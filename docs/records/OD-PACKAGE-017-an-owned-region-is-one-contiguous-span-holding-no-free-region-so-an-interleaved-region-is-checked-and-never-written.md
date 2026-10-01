---
id: OD-PACKAGE-017
type: decision
title: An owned region is one contiguous span holding no free region, so an interleaved region is checked and never written
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - packages
  - projection
  - ownership
  - ecosystem
relations:
  - target: OD-PACKAGE-003
    type: relates-to
  - target: OD-PACKAGE-004
    type: relates-to
  - target: OD-PACKAGE-005
    type: relates-to
  - target: OD-PROJECT-001
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-AGENT-004
    type: relates-to
---

# An owned region is one contiguous span holding no free region, so an interleaved region is checked and never written

## Question

Two halves of one question were answered hours apart and do not meet.

`OD-PACKAGE-004` version 3 defines a `Composed` asset's owned region "at the granularity
something compares" and works that definition out against this repository's own instance,
`README.md`'s band listing: the zone cell owned, the crate-name cell owned, the set of rows
owned, and the `Owns` column free. Owned region and free region therefore interleave, cell by
cell, inside one artifact.

`nomos-materialization` performs a declared placement and expresses a `Composed` target's
owned region as an `OwnedRegion { opening_marker, closing_marker }`. Its own documentation
states the limit rather than hiding it — "A pair names **one contiguous span**, and that is
the whole of what this maturity expresses" — together with the case it cannot serve, naming
`OD-PACKAGE-004`'s worked instance directly, and the reason it was not taught to: "Teaching
this crate the finer granularity is not the fix. A splicer that understood a table's columns
would be a format-aware mechanism, and the first thing such a thing wants to know is which
kind of package it is serving."

Both statements are careful and neither is wrong. What neither says is which of the two gives.
A placer that wrapped the band listing in a marker pair and asked for it to be written would
have the free region inside the span, and nothing in the record or the mechanism would refuse.
That is a silent loss, which is the property `OD-PACKAGE-004` was amended to end.

## The case, constructed rather than asserted

Measured 2026-09-21 against `README.md` and `nomos-architecture.json` as this record was
written, by re-running `Zone_Row`'s own reading and then applying `Spliced`'s own algorithm from
`crates/substrate/nomos-materialization/src/materializer/composed.rs` to the result.

| | |
|---|---|
| `README.md` | 51,510 characters, 51,680 bytes |
| rows `Zone_Row`'s reading matches | 72 |
| crates `nomos-architecture.json` declares in `members` | 72 |
| characters in the 72 `Owns` cells | 33,102 |
| `Owns` cells that are empty | 0 |
| occurrences of `<!--` anywhere in `README.md` | 0 |

The construction: add a marker pair around the listing, render a replacement from the only
declared source there is — `nomos-architecture.json`, whose five keys are `components`,
`members`, `permits`, `exceptions` and `authorities`, none of them a description — and splice.

| | |
|---|---|
| span the markers would bound | lines 51 to 130, 36,525 characters |
| replacement rendered from `members` | 3,214 characters, 72 rows, every `Owns` cell empty |
| `README.md` after the splice | 18,251 characters |
| characters lost | 33,311 |
| refusal returned by the splice | none |
| rows afterwards | 72, every member still listed at its declared zone, both directions |
| occurrences of `OD-CONTRACTS-001` afterwards | 0, from 1 |

`Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band` stays green across that,
because it compares only the two cells it reads. The one assertion that moves is
`Test_Band_Zero_Should_Be_Described_In_One_Place`, and it moves for the reason version 3
predicted a whole-file constraint would: `README.md`'s single occurrence of `OD-CONTRACTS-001`
sits inside the `nomos-contracts` row's `Owns` cell, so the failure reports that `README.md`
describes band 0 without naming the record, not that 33,311 characters of reasoning were
deleted.

Against `README.md` exactly as it stands, the splice refuses — `Refusal::AbsentRegionMarker`,
the behaviour `Test_A_Table_With_No_Markers_Should_Refuse_Rather_Than_Replace_The_File`
measures. That refusal is about the file not carrying a marker, not about the declaration being
false, and it lasts exactly as long as nobody adds one.

## Two measurements that make the region narrower than "a column problem"

**The listing is two tables, not one.** Lines 51 to 118 carry 66 rows; lines 123 to 130 carry 6
more; and lines 119 to 122 between them are a blank line, two lines of a person's prose — "The
specification system sits beside the kernel rather than above it. It reaches the product only
through a knowledge capability, so nothing in the product may name it." — and another blank
line, followed by a repeated header and separator. `Zone_Row` reads rows in both and the
assertion quantifies over the union, so the owned *set of rows* is already two spans with free
prose between them, before the `Owns` column is considered at all.

**The split between the two tables has no declared source either.** It is not a zone split:
`nomos-spec-orchestration` is a `Specification`-zone crate in the first table while the six
`nomos-spec-*` crates in the second are `Specification` as well. `members` is a flat map from
crate name to zone, so a renderer handed it cannot decide which of the two tables a new
`Specification` member belongs in. Version 3's owned "set of rows" — "add a row for a declared
member" — is therefore not fully derivable from the declaration it is owned against.

## Why a free region inside an owned span is a contradiction, not a granularity problem

The tempting reading is that a span is merely too coarse an instrument for a region this fine.
It is not, and the mechanism's own test suite shows why.

`Spliced` writes `source` into the span and keeps only what lies outside the markers. So for
any byte inside the span to survive, `source` must contain it. `tests/materializing/rows.rs` is
the case built against version 3's row corner, and its fixture is exactly that: the source
constant `ROWS_WITH_ONE_ADDED` carries a row whose third cell reads "What one owns, written
by a person." — the free cell's bytes, reproduced verbatim by the placer so that
`Test_Adding_A_Row_Should_Change_Only_The_Span_The_Markers_Name` can observe them unchanged.

That test is honest about what it measures and its module doc says so. What it demonstrates,
read the other way, is the contradiction: a splice does not preserve free region inside a span,
it requires the placer to re-emit it. And a placer that re-emits the free region is writing the
free region, which `OD-PACKAGE-004` defines as the region where "nothing but its human author
ever writes", and whose overwrite its controls table names as the failure the class exists to
prevent.

So the two are not at different granularities. **Free region inside an owned span is
self-contradictory: either the span's bytes come from the declared source, in which case they
are not free, or some of them do not, in which case the span is not owned.** The span is not
too coarse; the region is not a region.

## Decision

**An owned region is one contiguous span of bytes, and every byte strictly between its markers
is produced by the region's declared source. A candidate region whose owned and free parts
interleave is not an owned region. It is a checked region, and it is not materializable.**

This adds the second half of `OD-PACKAGE-004` version 3's rule rather than replacing it. That
rule — "A `Composed` asset's owned region is exactly what some check actually compares against
a declared source, in the units that check reads" — is the right test for what a *check* holds,
and it is left standing for that. Read as a licence to *write*, it can name a region no write
can honour, because a check reads at whatever granularity it likes and a write is a byte range.
Version 3's own corollary, "State a region in the comparison's units, never the artifact's",
gains a companion: **a region stated in units the target's bytes do not carry — a cell, a
column, the existence of a row — is checkable and is not writable.**

Applied to the instance: `README.md`'s band listing is a checked region. `README.md` is
`Composed` and is **not a materialization target**, now and for as long as the `Owns` column
has no declared source.

### What `OD-PACKAGE-004`'s classification of `README.md` is for, then

It is not spent. `Composed` has carried two modes since version 1, and `README.md` is in the
second by that record's own words: "Where no renderer exists yet for the owned region — this
repository's own case for `README.md` — the equivalent obligation is a bidirectional check
against the declared source, and a person reconciles a failure by hand; the check substitutes
for a render, it does not excuse one." That obligation is live and discharged today by
`Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band`.

What this record changes is the tense. Version 1 read as though the check were standing in for
a renderer that had not arrived yet. Under this decision no renderer is coming for this asset,
so the check is not an interim arrangement and the classification's job is to say which region
a person reconciles and which region nothing may touch — which is exactly what version 3's 218
lines worked out. Its findings were about the reach of a comparison and the entitlement a
classification confers; neither depended on a write ever happening.

### The expression candidate, and why it is declined in both its shapes

The second candidate is that the intent gains a way to express a non-contiguous region. It has
two shapes and neither survives.

**Format-aware.** An intent that said "column three of each row is free" would carry a unit only
a format defines. The mechanism honouring it would have to parse the target to find the unit,
so its vocabulary would close over target formats — a Markdown table, a YAML block, a TOML key
— and each package kind arrives with the formats its surfaces use. The mechanism would then
grow once per package kind, which is precisely `OD-PACKAGE-003`'s test: the mechanics "refer to
nothing that knows what a crate, a rule, a gate, or a peer connection is". The materializer's
author declined on this ground and the decline is upheld here.

**A set of spans.** The cheap generalization — a non-contiguous region as a list of contiguous
spans — is not format-aware and still does not help, for three measured reasons. It does not
address the contradiction above, since each span in the list must still hold no free region, so
the `Owns` column forces one span per owned cell rather than one per table. Those markers must
each occur exactly once in the target, which `Sole_Occurrence` requires, so they would have to
be keyed per row and the target would carry 144 of them inside its own cells. And a row that
does not exist yet carries no markers, so adding one would require the mechanism to write
markers — which `OwnedRegion` states it never does: "The markers themselves are preserved."
A generalization that buys nothing for the one instance and costs the marker-preservation
invariant is not the answer.

## What refuses, and where

Four answers, ordered by strength, because naming only the strongest would repeat the mistake.

**1. The declaration refuses, by being unformable.** A `Composed` intent must carry an
`owned_region` — `Refusal::UndeclaredOwnedRegion`, exercised by
`Test_A_Composed_Target_Declaring_No_Region_Should_Refuse` — and under this decision a placer
may declare one only for a span it has shown closed under the declared source. For
`README.md`'s listing that showing fails, measured: 33,102 characters in 72 cells against a
declaration holding no description, plus two lines of prose and a table split inside the same
span. There is no region, so there is no intent. The refusal is that the declaration cannot be
formed, not that a mechanism would decline it.

**2. The closure test is a procedure, and it is performable today.** Render from the declared
source and compare byte for byte against the candidate span. Identical, and the span is closed.
Different, and the difference is either staleness or free bytes, and a placer that cannot say
which does not have a region. It costs one render and one comparison, needs no new mechanism,
and is `OD-PACKAGE-004` version 3's third question — "Construct the disagreement" — asked of a
span instead of a check. It was run for `README.md` above and the answer is published rather
than left for the next reader to derive.

**3. What must not be mistaken for the guard.** `Refusal::AbsentRegionMarker` fires today only
because `README.md` carries no `<!--` at all. It refuses an unprepared file, not a false
declaration, and it disappears the instant somebody adds a marker pair. The standing instruction
this leaves is concrete: **`README.md` is not to be given region markers.**

**4. What does not refuse, and why that is a price rather than a defect.** `Materialize` cannot
refuse this and will not be taught to. It holds three things — the target's previous contents,
the two markers, and the source — and no function of those three separates "the owned region is
stale" from "the span holds bytes the source cannot produce", because both present as the source
differing from the span. Telling them apart requires knowing the target's format. So the absence
of a mechanism-side refusal is the cost of `OD-PACKAGE-003`'s test being passed, and it is named
here rather than left to be discovered by whoever loses a column.

That cost has a residue this record does not close: nothing checks that a placer performed the
closure test. A placer that declares a span it never closed still loses bytes at exit 0. What
changes is that the loss now has a name and an owner — a false closure claim, made at the
declaration — instead of being a behaviour nobody had written down, and that the one asset for
which the claim is known false is named. Building the check belongs with the first `Composed`
intent, which does not exist; see below.

## Surviving `OD-PACKAGE-003`'s test

The test is that the mechanism must not know what a crate, a rule, a gate or a package kind is.
This answer adds nothing to the mechanism at all: no field on `MaterializationIntent`, no
variant on `Refusal`, no reading of the target beyond the two markers it already matches.
`Materialize` remains a byte-range splicer that cannot tell a table from a YAML block, and the
obligation this record creates lands on the placer, which already knows its own format because
it renders the source. A second package kind that declares a `Composed` intent changes nothing
here.

The declined candidate is the one that fails the test, and it fails it at the first step rather
than eventually: a mechanism asked to honour "column three is free" must first ask what kind of
document it is looking at.

## Is this urgent

**No. It is latent, and the affected asset is one nothing places.** Measured across the whole
tree:

- `OwnershipClass::Composed` appears nowhere outside `nomos-materialization`'s own sources and
  tests. No package, manifest or fixture declares a `Composed` intent.
- No manifest, fixture or source names `README.md` as a materialization target.
- The only manifest on disk is
  `crates/packages/nomos-integration-package/tests/fixtures/nomos.integration.self.json`. It
  declares four intents — `AGENTS.md`, `CLAUDE.md`, `.claude/skills` and
  `.github/workflows/gate.yml` — declares no ownership class for any of them, and therefore
  resolves all four to `UserOwned`, which
  `Test_The_Self_Fixture_Should_Resolve_To_The_Four_Placed_Rows` asserts. `Materialize` refuses
  that whole run with `Refusal::UserOwnedTarget` before reading a byte.

So nothing is blocked today. This record is a guard placed before the first `Composed`
declaration rather than a repair of a live one, and a reader should treat it as the
precondition on that work rather than as an incident.

One adjacent gap, noted and deliberately not folded into this decision: directory sources are
out of scope in `nomos-materialization` and refuse as `Refusal::UnreadableSource`, and the self
fixture's `.claude/skills` intent names a directory. That refusal is unreachable for the fixture
as it stands, because `Admissible` refuses `UserOwnedTarget` before the source is read. It is a
separate question for whoever gives that fixture a real class.

## What this costs

**`README.md`'s band listing gets no renderer.** Adding a crate stays a hand edit in two
places, reconciled by `Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band` going red
until they agree. That is what HEAD already does; what this record adds is that the arrangement
is settled rather than pending.

**It is settled, not permanent.** The condition version 3 already named is the same one that
would reopen this: a declared source of truth for a crate's description, outside `README.md`,
compared in both directions. Given one, the span becomes closed under its source, the region
stops interleaving, and the asset becomes materializable under this record's own rule with
nothing here amended. The table split measured above would have to gain a declared source too.

**A placer's closure claim is unchecked.** Stated above, and left to the first `Composed`
intent rather than pre-built against a population of zero declarations.

**No code, check or test is changed by this record**, and none is owed by it. The three
ownership classes, the default to `UserOwned`, the four assets' classifications and every
refusal `nomos-materialization` already performs stand exactly as written.

## Controls

| Weakening | What it produces |
|---|---|
| let a placer declare a span it has not shown closed under the declared source | the measured case above: 33,311 characters gone, no refusal, and the only check that moves reports a different defect |
| teach the splicer a target's format so a per-cell region can be written | a mechanism whose first question is which package kind it serves, which is the crossing `OD-PACKAGE-003` runs backward |
| read `Refusal::AbsentRegionMarker` as the guard against a non-contiguous declaration | the guard evaporates the moment somebody prepares the file, and preparing it looks like progress |
| generalize `OwnedRegion` to a list of spans and call the interleave solved | each span still holds free region, the markers multiply into the cells, and adding a row needs the mechanism to write a marker it promises to preserve |
| keep `README.md` classified as awaiting a renderer | the entitlement version 3 withdrew from the `Owns` column is restored to the whole listing, and the next placer reads it as work to do |
| treat this as urgent and build the closure check now | a guard shaped against zero declarations, landing before the first real one says what shape it needs |

## Status

Accepted. `OD-PACKAGE-004`'s rule for what a check owns is unchanged and gains its companion
for what a mechanism may write; `nomos-materialization`'s contiguous-span maturity is ratified
as the definition rather than recorded as a limitation; and `README.md` is a checked `Composed`
asset that nothing materializes. No code changed and none is owed. The first package to declare
a `Composed` intent owes the closure check named under "What refuses, and where", and the
question of a declared source for a crate's description remains open exactly where
`OD-PACKAGE-004` version 3 left it.
