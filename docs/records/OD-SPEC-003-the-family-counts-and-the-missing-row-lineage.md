---
id: OD-SPEC-003
type: decision
title: The family counts are line counts, and lineage cannot reach a table row
status: open
version: 1
authority: canonical-normative-record
tags:
  - preservation
  - restoration
relations:
  - target: ARC-SPECDB-001
    type: affects
  - target: OD-SPEC-002
    type: affects
---

# The family counts are line counts, and lineage cannot reach a table row

## Question

P3-FAMILIES restores the families v15.0 destroyed. Two things block it, and neither is
answerable from the repository alone.

## One: lineage cannot reference a table row

The item's done-when requires the canonical domain model to come back as one table block
plus a concept node per row, **each with a lineage row to its originating row**. The
`lineage` table carries `source_block_uid` and `source_heading_uid` and nothing for a
table row. P3-ROWS added `source_table_rows` but did not extend `lineage` to reach one,
because its own done-when did not ask for it.

So a concept node can today be lineaged to the block holding the table, not to the row it
came from — which is precisely the "one canonical source, two projections, never two
authorities" property the item exists to establish.

The fix is a migration adding `source_table_row_uid` to `lineage` and rebuilding
`lineage_unique` to include it. That is `nomos-spec-store/src/schema.rs`, and
P3-FAMILIES declares only `crates/spec/nomos-spec-ingest` and `tests/corpus/families`.
Recorded rather than done, because four instances of territory drift are already on
OD-LEDGER-001 and a schema change is not the fifth to take quietly.

## Two: every family count in the plan is a line count

Measured against `01_authoring/domain_volumes`:

| Family | Plan | Measured | Definition used |
| --- | --- | --- | --- |
| Canonical domain model | 30 | 28 | 30 pipe lines less the header and the delimiter |
| Roadmap releases | 8 | 7 | headings `### Release 1` through `### Release 7` |
| Worked scenarios | 8 | 4 | headings `### G.n End-to-end scenario`, G.2 through G.5 |
| Service descriptions | 54 | 39 | headings ending in the word Service |

The domain model figure is the same arithmetic as OD-SPEC-002's: 30 counts the header and
the delimiter, 28 is the models. It lands directly in the item's done-when, which says the
30 domain models resolve individually. The models are enumerable and the plan's named
examples all check out — `WorkspaceContext`, `FindingGeometry` and
`MetricTradeoffProjection` are all present — so only the count is in question.

The other three are held more loosely. The extraction definitions above may be too narrow;
services in particular may also be described in tables this did not count. They are
recorded as "by this definition" rather than as refutations.

## What Would Close It

1. Whether to split a `P3-ROW-LINEAGE` item ahead of P3-FAMILIES, or widen P3-FAMILIES's
   territory to include the store schema. The split is recommended: it is the same shape
   as P3-ROWS and keeps the store's write door under one item at a time.
2. Whether the done-when asserts the measured 28 or the plan's 30, per OD-SPEC-002's
   unresolved question, which this is now the second instance of.
3. Whether the remaining families need their own measurement pass before restoration
   asserts a count for any of them.

## Status

Open. P3-FAMILIES is unstarted and no code was written for it, because restoring a family
against a count nobody has settled is how a measurement gets bent to match an expectation.
