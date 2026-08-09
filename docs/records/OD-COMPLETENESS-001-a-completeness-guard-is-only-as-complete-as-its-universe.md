---
id: OD-COMPLETENESS-001
type: decision
title: A completeness guard is only as complete as the universe it quantifies over
status: open
version: 1
authority: canonical-normative-record
tags:
  - verification
  - completeness
relations:
  - target: OD-GATE-001
    type: relates-to
  - target: OD-LEDGER-001
    type: relates-to
---

# A completeness guard is only as complete as the universe it quantifies over

## Question

Three completeness checks in this workspace compared one side against the other and never
the reverse. All three were found by accident, each was repaired where it was found, and
the shape was never named — so the third took a hunt to find and a fourth would be
unguarded.

`P9-ONE-DIRECTION` was opened to name the shape and enforce it. This record is the naming
half. The enforcement half is not built yet, and the record says so rather than implying a
check that does not run.

## The Shape, Which Is Not The One The Item Assumed

The item describes the defect as a containment asserted in one direction and not the other.
That is true of two of the three instances and not of the first, and the difference matters
because a rule written for the two-containment reading would not have caught it.

**A completeness assertion quantifies over a universe. When that universe is itself
declared rather than derived, the assertion is only as complete as the declaration — and
comparing the declaration against reality is the other direction.**

That single statement covers all three.

## The Three Instances

**`Assert_Complete`** in `nomos-spec-bundle` counts every row in the store against every
record in the bundle, per table, and fails on inequality. The count comparison is already
symmetric; nothing about it is one-directional. What made it half a check is the universe:
it iterates `Table::All()`, a list kept beside the enum. A migration adding a table without
adding it there leaves that table outside the guard entirely, and the guard passes by not
looking. A new *column* passed for the same reason on a different axis, until
`Assert_Columns_Covered` was written.

**`Assert_Landed`** in the importer is the mirror of `Assert_Complete` across the round
trip, and inherits the same universe. Its own doc comment calls it "the mirror of the
exporter's completeness guard" — accurate about the axis it mirrors, and silent about the
one both of them share.

**`GOVERNING_RECORD_IDS`** was compared against a seeded store and never against
`docs/records`. The seed and the list are the same list, so the comparison could not fail.
Six governing records — among them `OD-STORE-001` and `OD-ANALYSIS-001`, which decide how
the store and the analysis kernel behave — declared canonical normative authority, were
cited in commits and in other records, and never reached the store. Closed by
`Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`, whose own doc comment calls
itself "the direction nothing checked".

## The Workspace Had Already Written This Down

`Test_Every_Table_In_The_Schema_Should_Be_Declared`, in `nomos-spec-store`, carries this in
its doc comment:

> The other direction, which is the one that bites. `Table::All()` is a list kept beside the
> enum, and a migration that adds a table without adding it here leaves that table out of
> every completeness guard built on `All()` — the bundle's row count, its column coverage,
> the import's landed check — all of which then pass by not looking.

The rule was correctly stated, in the right words, next to the one check that enforces it,
and it stayed a comment on a single test rather than becoming a property of the workspace.
That is the same failure one level up from the one it describes: a statement that is true,
written down, and load-bearing for four other guards that do not reference it.

Recorded here because the next person to add a completeness guard will read this directory
and will not read that doc comment.

## The Rule

**A completeness guard must say what universe it quantifies over, and a declared universe
must have a check comparing it against the reality it claims to enumerate.**

Two corollaries worth stating, because both were violated by real code above:

- A guard whose universe is derived — read from the schema, walked from a directory,
  resolved from `cargo metadata` — owes nothing further on this axis. The cost is paid by
  deriving.
- A comparison whose two sides come from the same source cannot fail. `GOVERNING_RECORD_IDS`
  against a store seeded from `GOVERNING_RECORD_IDS` is not a check, and it reported success
  for as long as it existed.

## What Would Close This

A test that fails when a completeness guard is added without its mirror. The intended
mechanism is `tests/contract`, which watches the workspace from outside and links almost
nothing.

Discovery can be mechanical and classification cannot. A scanner that fires only when both
operands of a negated containment are collections — bound in the same function, or an
upper-case constant — reduces 39 raw sites to 15 candidate functions, of which 4 already
check both directions, 1 is a genuine pair split across two functions, and the rest are
deduplication, filtering and accumulation rather than comparison. Telling those apart needs
types, and `tests/contract` deliberately has none. So the enforcement is the shape
`OD-GATE-001` already uses: derive the candidates, declare the classification in a table,
and check the two against each other so the table cannot go stale in the direction that
flatters.

Not built here. `P9-ONE-DIRECTION` carries it.

## Status

Open. The shape is named and the three instances are analysed; nothing yet fails when a
one-directional guard is added. Closed when `P9-ONE-DIRECTION` lands the enforcement, at
which point this record should be amended to say what the check actually holds — including
whether all three instances above fail it as originally written, which is the item's own
bar and is not met by this record alone.
