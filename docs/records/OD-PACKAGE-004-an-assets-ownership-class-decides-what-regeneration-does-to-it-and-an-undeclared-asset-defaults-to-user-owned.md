---
id: OD-PACKAGE-004
type: decision
title: An asset's ownership class decides what regeneration does to it, and an undeclared asset defaults to user-owned
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - packages
  - projection
  - freshness
  - ownership
  - ecosystem
relations:
  - target: OD-GATE-005
    type: relates-to
  - target: OD-PROJECT-001
    type: relates-to
  - target: OD-PROJECT-002
    type: relates-to
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-PACKAGE-003
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-AGENT-004
    type: relates-to
---

# An asset's ownership class decides what regeneration does to it, and an undeclared asset defaults to user-owned

## Question

This repository already generates files into itself and already has three different
relationships to the results, and nothing names the difference between them.

`diagrams/relations.mmd` and `spec/domain-specification.md` are rendered from the record set
and belong to the renderer outright. `AGENTS.md` states the obligation directly: "Changing a
governing record leaves every committed projection stale, and the gate fails on them
[...] a commit touching `docs/records/` changes both whether or not it meant to." `OD-GATE-005`
settles who may write them — nobody's territory, rendered from a tree constructed to hold
exactly the record set the commit publishes — and `OD-PROJECT-002` is why these two and not a
third: `diagram-set` and `domain-specification` are the required set, each for a reason argued
and priced there.

`README.md` looks the same at a glance and is not. `OD-PROJECT-001` found it hand-authored
where `D-128` expected a projection, and the discovery was drift: eleven crates listed against
twenty-two in the workspace, and a paragraph claiming no analysis engine existed while five of
its crates were already members. The repair did not turn `README.md` into a projection —
`D-128`'s actual documents were renamed to `README.projection.md` and `spec/architecture.md`,
both `GeneratedOwned` in this record's terms — it left `README.md` as a single file carrying
two different kinds of content at once: a band/crate table now checked by
`tests/contract/tests/boundaries.rs` in both directions, and prose that carries no check at
all.

`.claude/settings.local.json` is the third kind, and it is protected rather than generated.
`CLAUDE.md` states it plainly: "The file .claude/settings.local.json is personal and stays out
of git." Nothing in this workspace writes it, and `OD-PACKAGE-003`, measuring the same
directory, records that it "carries only `permissions`, and is itself personal and untracked
per `CLAUDE.md`."

Three relationships, one convention distinguishing them today — which prose file a reader
happened to open — and that stops being survivable the moment a package materializes files it
did not write into a repository it has never seen, which is exactly the shape
`OD-PACKAGE-003` names for a future `IntegrationPackage` and a future `ProjectionPackage`. The
failure is asymmetric: refusing to regenerate a file that only the renderer owns costs a stale
projection, and the gate already catches that. Regenerating a file that is not the renderer's
costs the user's own work, with no diff to recover it from, on every machine the mistake runs
on.

## The Three Classes

| Class | What regeneration does | What a conflict does |
|---|---|---|
| `GeneratedOwned` | Overwrites the file unconditionally. The renderer is the sole author of every byte in it, so there is no partial write and nothing to merge. | `nomos spec freshness` reports the conflict before any overwrite happens, and keeps apart two causes that share exit code 8: `stale` (the store moved under an unchanged body) and `edited` (something changed the body itself, so its digest no longer matches its sidecar). Both resolve the same way — re-render and take the renderer's bytes. A hand edit is not preserved, because the instant it landed it was already a defect, not a competing claim this record has to arbitrate. `OD-GATE-005`'s controls table states the same rule for the diagram specifically: "hand-edit `relations.mmd` to add the missing edges" leaves `freshness` reporting `edited` rather than `stale`, and the fix in both cases is the same render. |
| `Composed` | Regeneration is scoped to the region tied to a declared source of truth, and never touches the region that is not. Where a renderer exists, it writes only the owned region and leaves the rest byte-for-byte untouched. Where no renderer exists yet for the owned region — this repository's own case for `README.md` — the equivalent obligation is a bidirectional check against the declared source, and a person reconciles a failure by hand; the check substitutes for a render, it does not excuse one. | A conflict in the owned region fails loudly — a test failure, a gate step gone red — rather than being silently overwritten or silently passed. The fix is always to correct the region against its declared source, never to blank the file or accept the drift. There is no such thing as a conflict in the free region, because nothing but its human author ever writes there, and no check watches it. |
| `UserOwned` | Never happens, under any circumstance. No package, no installer, no renderer, no profile writes this path. | Undefined, because nothing else ever holds a competing claim on the bytes to conflict with. A mechanism about to write a path it cannot place in `GeneratedOwned` or `Composed` must treat it as this class and refuse the write outright. |

## Where The Classification Lives

**The asset does not declare its own class.** A generator that writes the wrong thing is the
exact failure this record exists to prevent, and a tag living inside the very bytes that
generator is about to overwrite is authored by the same actor whose mistake it would need to
catch. Self-attestation checks nothing here; it only adds a second sentence for the same
mistake to get wrong.

**The class is declared by whatever places the asset — the package, or a manifest once one
exists — not by the target file.** `OD-PACKAGE-003`'s placement table already carries the
shape this needs: each surface an `IntegrationPackage` or `ProjectionPackage` touches is
recorded there as placed, merged, or declared-only, together with its owner. Ownership class is
one more fact of that same kind, recorded once, next to the path it governs, by the thing doing
the writing — not discoverable by reading the target file in isolation.

This repository builds neither package nor manifest today. `OD-PACKAGE-001` found `PackageKind`
with no consumer and no package on disk anywhere in this workspace, and `OD-PACKAGE-003`
confirmed the same absence for every integration surface this repository already carries: every
one of them is "hand-placed, without a package or an installer having produced any of them."
So nothing here is classified by a manifest yet, and this record's own table below is, for now,
the only place the four real assets' classes are written down. That is consistent with both
records this one routes through rather than a gap introduced here.

**An asset with no recorded class defaults to `UserOwned`, and the write is refused.** The
default has to point one direction, and the direction is fixed by the asymmetry the question
section states: a `GeneratedOwned` file wrongly left alone is a stale projection the gate
already catches on the next commit that touches it. A `UserOwned` file wrongly overwritten is
gone, with nothing to diff it against, and an installer that gets this wrong gets it wrong
identically on every machine it runs on. Refusing costs a rerun; overwriting costs work that
cannot be recovered. The safe direction is the one whose mistake is cheap, and that is refusal.

## The Four Assets This Repository Already Carries

| Asset | Class | Basis |
|---|---|---|
| `diagrams/relations.mmd` | `GeneratedOwned` | Rendered by `nomos spec render --profile diagram-set`, one of the two profiles `OD-PROJECT-002` requires. `OD-GATE-005` decision 1: "owned by nobody" — no item's territory, and a hand edit is caught as `edited` and lost on the next render. |
| `spec/domain-specification.md` | `GeneratedOwned` | Rendered by `nomos spec render --profile domain-specification`, added to the required set by `OD-PROJECT-002` for the same reasons and under the same regeneration/conflict contract as the diagram; both are committed as a body plus a `.nomos-projection.json` sidecar. |
| `README.md` | `Composed` | `OD-PROJECT-001`: hand-authored workspace description carrying one owned region — the band/crate table, checked bidirectionally against `tests/contract/tests/boundaries.rs` — and one free region, the prose, that carries no check. Discovered exactly because it was believed to be `D-128`'s projection output and had silently drifted while nobody watched either region; the actual `D-128` documents were split out as `README.projection.md` (`github-markdown` profile) and `spec/architecture.md` (`architecture-document` profile), both `GeneratedOwned` and outside this table only because `done_when` names the four assets already generated or protected, not every projection this repository ships. |
| `.claude/settings.local.json` | `UserOwned` | `CLAUDE.md`: "The file .claude/settings.local.json is personal and stays out of git." Nothing in this workspace writes it today; `OD-PACKAGE-003` measured the same file and found it carrying only `permissions`. A package or installer that writes it is wrong regardless of what it would have written. |

## What This Costs If Left As Read Today

Nothing changes at HEAD for any of the four assets: the diagram and the specification render
exactly as `OD-GATE-005` and `OD-PROJECT-002` already require, `README.md` keeps the checked
table and free prose `OD-PROJECT-001` left it with, and `.claude/settings.local.json` stays
untracked and unwritten. What changes is that a future package or installer — the
materialization `OD-PACKAGE-003` describes and this repository does not yet build — has three
named behaviors to implement and a stated default for the fourth case, an asset it does not
recognize, instead of inventing regeneration and conflict handling per asset the way three
different conventions already exist here by accident.

## What This Is Not

**Not a manifest, and not the first consumer of one.** `OD-PACKAGE-001`'s four-step path to a
first manifest reader is untouched; this record states what a class would mean and where it
would be declared, and does not build the declaration mechanism, a `PackageKind` consumer, or a
`ProjectionPackage` that writes a `Composed` file's owned region automatically.

**Not a change to any of the four assets or the checks over them.** `tests/contract/tests/boundaries.rs`,
`nomos spec freshness`, and the render commands are unchanged. This record documents the
ownership each asset already has and states the class model that generalizes it; it does not
rename a file, move a check, or add one.

**Not a claim that every future generated asset fits exactly one of three classes forever.**
The three named here are the classes this repository's own four assets exhibit, per
`done_when`. A class this repository has not yet produced an instance of is a question for
whichever record adds the instance, not one this record forecloses.

**Left open:** `GeneratedOwned`'s current wording — "the renderer is the sole author of every
byte" — is framed around this repository's own single-producer case. A future generated asset
built from more than one declared source, or through a producer pipeline, would need that
framing generalized toward "the declared producer(s) hold exclusive authority over the
canonical content" without weakening the unconditional-overwrite behavior this record actually
requires. Nothing here has that shape yet, so nothing is changed; noted for whichever record
adds the first instance that does.

## Controls

| Weakening | What it produces |
|---|---|
| let an asset declare its own class | a generator that writes the wrong thing also gets to say it was right to, which protects nothing |
| default an undeclared asset to `GeneratedOwned` | the first installer bug overwrites a user's personal file with no diff to recover it from, on every machine it runs on |
| treat `Composed`'s free region as `GeneratedOwned` | prose a person is expected to edit gets silently overwritten the next time anything regenerates the file |
| treat `Composed`'s owned region as `UserOwned` | the drift `OD-PROJECT-001` found in `README.md`'s tables recurs, uncaught, because nothing compares it to its source again |
| collapse `stale` and `edited` into one conflict outcome for `GeneratedOwned` | a correct file rendered over a moving store and a file someone typed into report identically, which is the confusion `OD-GATE-005` already separated and this record must not re-merge |

## Amendment, Version 3: A Composed Asset's Owned Region Is Declared At The Granularity Something Compares

The table above classifies `README.md` as `Composed` and names its regions as "the band/crate
table, checked bidirectionally against `tests/contract/tests/boundaries.rs`" and "one free
region, the prose, that carries no check". That sentence is true about the intent and wrong
about the reach. The table it names has three columns; the check it names reads two of them.

### What the check actually reads

`Zone_Row` in `tests/contract/tests/boundaries/readme.rs` is the whole of the reading, and it
is eighteen lines:

```rust
fn Zone_Row(architecture: &ArchitecturePayload, line: &str) -> Option<(String, String)>
{
    let mut cells = line.split('|').map(str::trim);
    if cells.next() != Some("")
    {
        return None;
    }

    let (Some(zone_text), Some(name)) = (cells.next(), cells.next())
    else
    {
        return None;
    };
    let component = architecture.components.iter().find(|component| return *component == zone_text)?;
    let name = name.strip_prefix('`').and_then(|rest| rest.strip_suffix('`'))?;

    return Some((name.to_owned(), component.clone()));
}
```

Three calls to `cells.next()`: one for the empty string before the opening delimiter, one for
the zone, one for the backticked crate name. The iterator is then dropped and the row is
returned as a two-tuple, so no caller could reach a third cell even if it wanted to. Everything
built on that reading inherits its reach.
`Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band` asserts three things over those
pairs — no crate listed twice, every declared member listed, every listing a real member at its
declared component — and all three are quantified over `(crate, zone)` alone.

Measured on 2026-09-21 at `53de19a4`, by re-running `Zone_Row`'s own reading over `README.md`
against `nomos-architecture.json`:

| | |
|---|---|
| rows the reading matches | 71 |
| crates `nomos-architecture.json` declares in `members` | 71 |
| cells those rows carry | 213 |
| cells the check compares | 142 — the zone and the crate name |
| cells the check never reads | 71 — the `Owns` column |
| text in the unread column | 4,227 words, 31,715 characters, roughly 366 rendered lines at 95 columns |
| rows whose `Owns` cell is empty | 0 |
| longest single `Owns` cell | 1,304 characters, `nomos-mcp` |

A third of the table's cells and the overwhelming majority of its bytes are inside the region
this record called owned and outside everything that holds it.

### The description column has no declared source, and that is not an oversight of the check

`nomos-architecture.json` holds five keys — `components`, `members`, `permits`, `exceptions`
and `authorities` — and `members` is a flat map from crate name to zone name. There is no
per-crate description in it, nor in any other declaration this repository carries. The `Owns`
column is therefore not a restatement of a declared source that somebody forgot to compare; it
is the only statement of what it says. `OD-PROJECT-001` kept this file hand-authored "on the
condition that the part of it a machine can check is checked", and the part a machine can check
is exactly the part `nomos-architecture.json` declares.

That settles what went wrong above, and it is narrower than it looks. `Composed` scopes
regeneration "to the region tied to a declared source of truth, and never touches the region
that is not" — so a column with no declared source was never in the owned region by this
record's own definition. Version 1 did not widen the class. It named the region in the units of
the artifact, a table, rather than in the units of the comparison, a cell, and a table is wider
than its checked columns.

### The column is not entirely unconstrained, and what constrains it is not a description check

Two assertions read `README.md` as one string rather than as a table, and reach the third
column incidentally:

- `Test_Band_Zero_Should_Be_Described_In_One_Place`, in
  `tests/contract/tests/boundaries/band_zero.rs`, requires `README.md` to contain
  `OD-CONTRACTS-001` and not to contain the phrase `authoritative statement of Nomos`.
  `README.md` names that record **exactly once**, and the one occurrence is inside the `Owns`
  cell of the `nomos-contracts` row.
- `Test_The_Readme_Should_Not_Relist_A_Vocabulary_It_Routes_To` requires five words to be absent
  from the file entirely.

So a mechanism that rewrote the `Owns` column would redden a band-zero check, and would be told
it had described band 0 in the wrong number of places rather than that it had deleted 31,715
characters of reasoning. A whole-file phrase constraint is not a region check: it cannot name
which region broke it, it happens to cover one of seventy-one cells, and its failure reports a
different defect from the one that occurred. It is a reason to preserve the column, not evidence
that the column is watched.

The near miss worth naming is `tests/contract/tests/boundaries/transport_registry.rs`. Its
reasoning leans on the four `[repo tooling]` marks that live in `Owns` cells —
`Test_The_Transport_Should_Name_No_Repo_Tooling_Handler` quotes the mark in its own doc and in
its failure message — and it never opens `README.md` at all. Those marks are cited by a check
and read by none.

### Decision: the `Owns` column is free region

Reclassified, not deferred, and not left owed. `README.md`'s owned region is the zone cell, the
crate-name cell and the set of rows. The `Owns` column is free region, on the same footing as
the prose around the table.

| Part of the table | Region | What a regenerating mechanism may do |
|---|---|---|
| column 1, the zone | owned | write it, from `members` in `nomos-architecture.json` |
| column 2, the backticked crate name | owned | write it, from the same declaration |
| the set of rows | owned | add a row for a declared member and remove a row for a crate that is gone — the two directions `Assert_Every_Member_Is_Listed` and `Assert_Every_Listing_Is_A_Member` already compare |
| column 3, `Owns` | free | preserve it byte for byte, including the four `[repo tooling]` marks. Never write it, never normalize it, never reflow it. |

Adding a row is the corner where the two regions meet, and it has one honest answer: a
mechanism adding a row writes the two owned cells and leaves the `Owns` cell empty for a person,
because it has nothing to write there and inventing a description is the failure this record
exists to prevent. An empty `Owns` cell leaves every check green, since no check reads it, so it
is a marker for a person rather than a defect a gate will report. That is the cost of this
decision stated rather than hidden.

### Why reclassifying protects the column rather than abandoning it

The objection to calling it free is that it tells a later generator the column does not matter.
It says the opposite. In the class table above, free region is the region "nothing but its human
author ever writes"; owned region is the region regeneration writes. The dangerous state is the
one this amendment corrects — a column named owned, so a mechanism believes it is entitled to
write it, and compared by nothing, so the loss is silent and every check stays green. Naming it
free withdraws the entitlement, which is the stronger of the two protections and the one the
evidence supports: two items were editing these descriptions by hand while this amendment was
being written, which is what a maintained free region looks like from outside.

What the decision gives up is real and is not hidden. Nothing will detect an `Owns` cell that
has become false, so a description can drift the way `OD-PROJECT-001` found the whole table had
drifted, and no run will say so. That cost is what HEAD already carries; this amendment neither
creates nor closes it. What it ends is the false statement that something is watching.

### What would move the column back to owned

One thing, and it is stateable: a declared source of truth for a crate's description, outside
`README.md`, that something compares the column against in both directions. Naming that source
is a design question rather than a detail, which is why it is not named here as owed work. The
content is 4,227 words of prose carrying emphasis, backticked identifiers and record citations,
so a declaration holding it would be either the same prose in a second file — the second
authority `OD-AGENT-001` and `OD-AGENT-004` refuse, able to disagree with the copy nobody reads
— or a shorter derived fact, which would not be this column. Until somebody answers that, free
is the truthful classification and owed-with-no-comparator is not.

A presence check — every row's `Owns` cell is non-empty — is cheap, is available, and is
deliberately not required here. It would catch a wholesale blanking and nothing else, and a
check that cannot tell a true description from a placeholder must not be presented as one that
watches the column. If it is ever built, it belongs to a `Composed` mechanism's own safety rail
rather than to this record's ownership table, and it does not make the column owned.

### The general rule the instance illustrates

**A `Composed` asset's owned region is exactly what some check actually compares against a
declared source, in the units that check reads. A region named owned and compared by nothing is
free in effect whatever a record says about it, because the only thing "owned" ever buys is a
mechanism's permission to write.**

A later `Composed` asset is classified by applying that rather than by copying `README.md`'s
answer. Three questions, in order, asked of the candidate region at the granularity a mechanism
would write it:

1. **Name the declared source.** Which file, and which key in it, holds the truth this region
   restates? If the answer cannot be given as a path and a field, the region is free and there
   is nothing further to ask.
2. **Read the comparison rather than its description.** Which test opens both the region and
   that source and fails when they disagree? Read the function body. The owned region ends
   exactly where the reading stops — `Zone_Row`'s third `cells.next()` is that boundary here,
   and this record, the test's own module doc and the test's name all described a wider one.
3. **Construct the disagreement.** Make the region wrong on purpose and watch the comparison go
   red. A comparison that stays green over a deliberately wrong region was not comparing that
   region. `Test_A_Stated_Count_That_Disagrees_Should_Be_Read_As_A_Different_Number` is what
   this looks like when it is built in rather than performed once and forgotten.

A region that fails any of the three is free region, and a mechanism must not write it.

Two corollaries the instance also pays for:

- **State a region in the comparison's units, never the artifact's.** "The table" was the error;
  "the zone cell and the crate-name cell" is the same claim at a granularity that cannot quietly
  widen. A file, a table or a section is almost always wider than whatever compares it.
- **A whole-file constraint is not a region check.** It binds a free region without owning it,
  it cannot say which region broke it, and a mechanism writing the owned region can break it
  from outside. `band_zero.rs` over `README.md` is the worked case.

### Controls

| Weakening | What it produces |
|---|---|
| name a `Composed` asset's owned region by the artifact it sits in — the table, the section, the file | the region is wider than the comparison again, and the difference is exactly the part a mechanism may destroy with every check green |
| read a whole-file phrase constraint as a check over a region | a column believed watched by an assertion that reaches one of its seventy-one cells and reports a different defect when it fires |
| let a mechanism fill a new row's `Owns` cell rather than leaving it empty | a description nobody wrote, indistinguishable from seventy-one somebody did, in the one column no check can tell apart |
| leave a column named owned until a check for it exists | the state this amendment corrects, held open for as long as there is no source of truth to build the check against, which is indefinitely |

### What this does not change

No code, no check and no test is touched by this amendment, and none is owed by it. The three
classes, the default to `UserOwned` and the other three assets' classifications stand as
written; `README.md` is still `Composed`, and what moves is where the seam inside it falls.
`OD-AGENT-004`'s decline of generated README tables is strengthened rather than reopened: the
owned cells are checked *because* they are authored, and the free column cannot be generated at
all, having no declared source to generate from. A mechanism that reaches this table therefore
checks it — comparing the two owned cells and the row set against `nomos-architecture.json`, and
leaving the `Owns` column alone — rather than writing it.

## Status

Accepted. Four real assets are classified against the model; a fifth kind of generated file, if
this repository grows one, is measured against this table rather than invented against a blank
page. Amended to version 2 by `P13-PACKAGE-REFINE`, which noted `GeneratedOwned`'s
single-producer framing as an open question for a future multi-producer asset, without
changing the three classes or any of the four current assets' classifications. Amended to
version 3 by `P125-A-COMPOSED-FILES-OWNED-REGION-IS-AMBIGUOUS-BY-COLUMN`, which narrowed
`README.md`'s owned region from the band/crate table to the zone cell, the crate-name cell
and the set of rows — the units `Zone_Row` actually compares — reclassified the `Owns`
column as free region, and stated the general rule that a `Composed` asset's owned region
is declared at the granularity something compares. No class, no default and no other
asset's classification changed, and no check was built.
