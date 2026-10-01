---
id: OD-SPEC-002
type: decision
title: The regression headline counts lines, not blocks, and 24 of them are separators
status: closed
version: 2
authority: canonical-normative-record
tags:
  - preservation
  - regression
relations:
  - target: ARC-SPECDB-001
    type: affects
---

# The regression headline counts lines, not blocks, and 24 of them are separators

## Question

The plan fixes the v14.36 to v15.0 headline as `282 table_row, 6 code_block` disappeared,
and states that if the archaeology does not produce it, the archaeology is wrong. Measured
against the real archives, both numbers are line counts, and the block-level and row-level
counts they stand in for are different numbers.

## The Measurement

Over `01_authoring/domain_volumes`, the ten documents the block manifest covers:

| Counted as | Value |
| --- | --- |
| Lines beginning and ending with `\|` | **282** |
| Table separator lines (`\| --- \|`) among them | 24 |
| Header and data rows, the ones carrying content | **258** |
| Tables | 24 |
| Code fence lines | **6** |
| Code blocks | **3** |

So `282` is `258` content rows plus one separator per table, and `6` is three fenced blocks
counted at both ends. v14's own manifest agrees with the block reading: it records 2533
blocks, of which the tables are 24 prose blocks and the code is 3 blocks, and P1-GATE
reproduces every one of those hashes.

## Why It Matters

A separator line is table structure, not authored content. Reporting 282 lost rows
overstates the loss by 24 rows that never said anything, and reporting 6 lost code blocks
double-counts 3. The preservation ledger tracks blocks and, once rows are first class,
rows — neither of which is a line. An archaeology built to reproduce `282` exactly would
have to count separators as content to hit the number, which is the sort of accommodation
that makes a measurement agree with an expectation instead of with the corpus.

This is the same class of finding as the catalog count: the plan says 1,632 entities and
the file holds 2,619. The plan's numbers are the estimates that motivated the work, not
readings taken from the archives.

## What Would Close It

A decision on which of these P3-REGRESSION's acceptance asserts:

1. The block and row counts — 24 tables, 258 content rows, 3 code blocks — as the headline,
   with the line counts reported alongside as the plan's basis so the two are reconcilable.
2. The plan's line counts verbatim, which requires the report to count separator lines and
   fence lines as content.

The first is recommended. It is what the store can express, it is what the manifest already
agrees with, and the second cannot be stated in the block model without redefining a row.

Either way the substantive claim the plan makes is unaffected and confirmed: every table
and every code block in the ten domain volumes is absent from v15.0, which retains 11 table
rows tree-wide and no code blocks at all.

## Resolution

Closed by P3-REGRESSION, which asserts the first option. The regression headline states the
block and row counts and quotes the counts register for both readings, so `282` and `6`
appear as the plan's basis rather than as the report's claim, and the report never counts a
separator line as content to reach a number.

The substantive claim held and became more exact. All ten domain volumes are absent from
v15.0 and no path survived at its old location, so every row and every code block went with
the document carrying it — the loss is of documents, and the rows are lost because of it.
The reading of v15.0's own retained table stands as recorded here: 11 authored rows, which
this record names correctly and which the header row kind now resolves as 1 header and 10
data rows.

## Status

Closed. Recorded before P3-RESTORE built the report it feeds, because the two readings
differ in what the ingest has to record per row, and discovering that after the fact would
have meant rebuilding it.
