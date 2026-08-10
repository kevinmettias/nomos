---
id: OD-PROJECT-001
type: decision
title: The repository's README is not the suite's overview, and stays hand-authored
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - projection
  - freshness
  - documentation
relations:
  - target: D-128
    type: relates-to
  - target: D-129
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
---

# The repository's README is not the suite's overview, and stays hand-authored

## Question

`D-128` decides that `README.md`, `ARCHITECTURE.md` and `ARTIFACT_MAP.md` participate in
freshness validation, and that publication tooling shall treat current overview files as
governed freshness-checked outputs. Its rationale is that a valid corpus still misleads
when the maintained overview projections are stale.

Phase 4 shipped fourteen profiles, a renderer per format, a stamp, a sidecar and a
freshness comparison. This repository's `README.md` is produced by none of them, carries
no marker saying so, and is edited by hand — which reads as `D-128` being unmet by the
people who wrote it.

It is not, and the reason is that two different documents wear one name.

## Two Documents, One Name

`D-128`'s `README.md` is a **publication projection of the specification suite**: an
overview of the corpus, generated from it, validated against it. This repository's
`README.md` describes the **workspace**: which crates exist, which band each sits in, how
to run the gate, and why `cargo fmt` is not part of it.

The profiles already ship `D-128`'s documents under names that avoid the collision:

| `D-128` names | This repository ships | Profile |
|---|---|---|
| `README.md` | `README.projection.md` | `github-markdown` |
| `ARCHITECTURE.md` | `spec/architecture.md` | `architecture-document` |
| `ARTIFACT_MAP.md` | nothing | none |

Renaming was the right call. It was made without a record, and that is how the remaining
`README.md` came to look like an output nobody built rather than a different document.

`ARTIFACT_MAP.md` has no profile at all. That is a real gap in `D-128`'s coverage and it is
not this record's question; it is recorded here so it is not discovered a third time.

## The Decision

**This repository's `README.md` is hand-authored and is not a projection output.
`README.projection.md` and `spec/architecture.md` are `D-128`'s documents here, and the
freshness machinery applies to those.**

The criterion, stated so the next document can be judged by it rather than by precedent:

> A file is a projection output exactly when everything it asserts is in the specification
> store.

`README.md` fails that on its face. A projection selects one of ten content kinds —
suites, documents, headings, blocks, rows, nodes, statements, relations, lineage,
omissions — and the band table, the conventions and the gate commands are none of them.
They are facts about `Cargo.toml` and `.github/workflows/gate.yml`. Rendering them would
need a renderer over the workspace, which is a second system wearing the projection
system's name, reachable through the same command, and answering a different question.
`D-129` puts identity in the store; a crate's band is not in the store and putting it
there to justify a renderer would be inventing corpus content to satisfy a file format.

A second reason, independent of the first: the corpus is not in this repository and is on
no CI runner. A `README.md` projected from the store could not be built or checked in the
place a freshness check earns its keep. `OD-GATE-001` records what that absence already
costs sixty-eight other assertions.

## What This Costs

Stated rather than waved at, because the cost is the reason `D-128` exists.

**Nothing checked that `README.md` described this workspace, and it had already stopped.**
At the time of this record the workspace has twenty-two members and the README's two
tables listed eleven. `nomos-spec-project` — the crate that renders the projections this
record is about — was not among them. The README also said "There is no analysis engine
yet" while `nomos-capability`, `nomos-analysis`, `nomos-cap-syntax`, `nomos-lang-rust` and
`nomos-lang-rust-scan` were all workspace members with the vertical slice running over a
real corpus.

That is exactly the drift `D-128` describes: not a corpus defect, an overview that
misleads a reader who has no reason to distrust it.

**So the checkable part is now checked, by a different mechanism than projection.**
`tests/contract/tests/boundaries.rs` already declares every member's band, because a band
is a design decision with nothing in the source to infer it from. The README's tables are
now compared against that declaration in both directions: a member missing from the README
fails, and a README row naming a crate that is not a member fails. A hand edit to a
hand-authored file is caught the same way a hand edit to a generated one is.

**What is still uncaught, and accepted.** The prose is prose. "This repository is at Phase
2 complete" is a sentence about a moving tree that nothing can fail on, and the same holds
for every paragraph that is not a table row. Freshness for prose is what a projection buys
and this file does not buy it. The trade is deliberate: a README that could only be
regenerated on a machine holding the corpus would be worse than a README whose tables are
checked and whose paragraphs are read by a person.

## What Was Built Alongside

`Check` and `Freshness` shipped with Phase 4 and were reachable from the projection crate's
own unit tests and from nothing else. A hand edit to a rendered output was therefore
detectable in principle and detected by nobody — the same shape as the corpus gates that
report `ok` having read nothing.

`nomos spec freshness --into <directory>` runs the comparison over what is on disk, and its
rule about half-present pairs is the part that matters: a body with no sidecar beside it is
a failure, not a skip. Otherwise deleting the sidecar is how a hand edit stops being
caught, and the check would teach that trick to the first person who hit it.

## Status

Closed by `P9-README`.
