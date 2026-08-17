---
id: OD-PACKAGE-004
type: decision
title: An asset's ownership class decides what regeneration does to it, and an undeclared asset defaults to user-owned
status: accepted
version: 2
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

## Status

Accepted. Four real assets are classified against the model; a fifth kind of generated file, if
this repository grows one, is measured against this table rather than invented against a blank
page. Amended to version 2 by `P13-PACKAGE-REFINE`, which noted `GeneratedOwned`'s
single-producer framing as an open question for a future multi-producer asset, without
changing the three classes or any of the four current assets' classifications.
