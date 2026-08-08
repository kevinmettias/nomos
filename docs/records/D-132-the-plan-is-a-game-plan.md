---
id: D-132
type: decision
title: The plan is a game plan, so a count that disagrees with the corpus is superseded, not corrected
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - preservation
  - measurement
  - lineage
relations:
  - target: D-120
    type: affects
  - target: OD-SPEC-002
    type: affects
  - target: ARC-SPECDB-001
    type: affects
---

# The plan is a game plan, so a count that disagrees with the corpus is superseded, not corrected

## Context

`D-120` says game-plan notes are lineage and never sole authority. It was written about the
two transcripts the v14 corpus carries. This build has a third: the approved plan document,
which is the same kind of artifact — a reasoning trace that motivated the work, not a
reading taken from the archives.

The distinction stopped being theoretical. Every figure the plan states about the corpus is
a line or heading count under a looser definition than the thing it names, and the pattern
held for every family measured:

- `282 markdown table rows` is the pipe-line count. 24 of them are delimiters.
- `6 code blocks` is the fence-line count. Three blocks, counted at both ends.
- `30 canonical domain models` is the pipe-line count of the table that lists them. 28 are models.
- `8 roadmap release definitions` is the milestone count. The eighth is Foundation 0; there is no Release 8.
- `8 worked scenarios` is the appendix-G section count. Four are scenarios.
- `1,632 catalog entities` and `64 v15 records` are simply older than the files they describe: 2,619 and 66.

The first two were settled individually in `OD-SPEC-002`. Settling the seventh, and the
eighth, one record at a time is the argument had repeatedly rather than once.

## Decision

**D-120 extends to the plan document.** It is lineage: authoritative about intent, never
about the corpus.

Two rules follow, and both are mechanical rather than advisory.

**1. Where the plan and the corpus disagree, the corpus wins and the plan's figure is
recorded, not deleted.** The register at `tests/corpus/families/counts.json` carries the
measured value, the extraction that produced it, the corpus it was measured over, and the
plan's figure marked `superseded` with a reading of what that figure actually counted. A
number that is quietly replaced leaves the next reader unable to tell a correction from a
transcription error, and leaves the plan looking wrong where it was merely counting
something else.

**2. Every count Nomos asserts names its unit and its corpus.** A count without an
executable extraction is not a measurement, so each register entry has an extractor in
`family_counts.rs` and an entry with none fails the suite. The sentence in the register is
that extractor's reading, not a substitute for it.

## Consequences

The register is the single home for these numbers. A restoration, a regression report or a
projection quotes it rather than re-deriving, and the store's own row census answers the
same three questions — pipe lines, authored rows, data rows — as three queries over a typed
column rather than one number and two subtractions.

One figure survives as unreproducible and is recorded that way. No extraction over v14.36
yields the plan's **54 service descriptions**: section 6 of volume 02 holds 56 headings at
every depth, 51 of them leaves, 40 of those naming a `Service`, and the section's subsystem
table adds 29 rows. Naming it unreproducible costs nothing; choosing whichever definition
happened to reach 54 would have produced a measurement that agrees with an expectation
instead of with the corpus, which is the failure this record exists to prevent.

This does not license editing the plan. The plan is the approved statement of intent and
stays as written; the disagreement lives in the register, where it is data.

## What this is not

Not a second home for `OD-SPEC-002`, which decided what the regression headline reports for
two specific families and remains the authority on that. This decides the rule that made
both of those the same finding, so the next one is resolved by rule instead of by argument.
