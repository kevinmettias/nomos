---
id: OD-SPEC-005
type: decision
title: Six governing records were never in the store, and the store's own validation had never seen them
status: closed
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - governing-records
relations:
  - target: D-129
    type: affects
  - target: ARC-SPECDB-001
    type: affects
---

# Six governing records were never in the store, and the store's own validation had never seen them

## Question

`D-129` decides that the specification store holds identity and markdown is an editing
surface. `Seed_Governing_Records` embeds this repository's own records and writes them in,
and `GOVERNING_RECORD_IDS` lists what a seeded store must contain.

Nine ids were listed. Fifteen records exist under `docs/records`, every one of them declaring
`authority: canonical-normative-record`.

## What Was Found

Six governing records had never been in the store: `OD-ANALYSIS-001`, `OD-STORE-001`,
`OD-CAPABILITY-001`, `OD-CAPABILITY-002`, `OD-GATE-001` and `OD-LEDGER-002` — every record
the product phases produced. Two of them decide how the store and the analysis kernel
behave. They were written as files, cited in commit messages and in each other, and read by
people; the store knew nothing about any of them.

The check that existed compared `GOVERNING_RECORD_IDS` against a seeded store, which is the
direction that cannot fail: the list and the seed are the same list. Nothing compared either
against `docs/records`.

## What Seeding Them Immediately Caught

This is the part worth the record. Three defects had been sitting in committed, cited
records, and all three surfaced within a minute of the store first reading them.

**Three records' titles did not match their headings.** `Parse_Record` refuses a record whose
front-matter `title` disagrees with its `# ` heading, and `OD-LEDGER-002`,
`OD-CAPABILITY-002` and `OD-GATE-001` each had a heading that was a truncation of its title.
Written by hand, reviewed, committed, and wrong.

**Three used a relation type this build's vocabulary does not contain.** `relates-to` is not
in `RELATION_TYPES`, so the foreign key refused the edge. The relation had been in those
records' front matter since they were written, meaning nothing anywhere had ever checked
that a record's declared relations are relations this system can hold.

**A count assertion was a literal.** `Test_The_Records_Should_Be_Present_As_Disposed_Content`
asserted nine source documents, which was the number of records at the time somebody wrote
it. It now asserts `GOVERNING_RECORD_IDS.len()`, because a count that has to be edited every
time a record is added is a count that will eventually be edited to whatever makes the test
pass.

Every one of those is the same shape as the finding itself: a mechanism that works, in front
of an input it had never been given.

## The Decision

**A record that claims canonical normative authority is seeded, and that is checked against
the directory rather than against the seed.**

`Test_Every_Canonical_Record_On_Disk_Should_Be_Governing` reads `docs/records`, takes every
file declaring `authority: canonical-normative-record`, and asserts two containments in both
directions: no such file is missing from `GOVERNING_RECORD_IDS`, and no seeded id lacks a
file. It guards against vacuity by failing when the directory yields nothing, because every
assertion in it iterates over that set.

`relates-to` is added to the seed relation vocabulary as its own inverse, since it is
symmetric. The alternative — rewriting those three relations as `affects` — was rejected as
false: `OD-CAPABILITY-002` borrows `OD-STORE-001`'s criterion and does not affect it, and a
wrong edge in the graph this system exists to keep honest is worse than a vocabulary one term
short. It remains a seed term; `ADR-ARTIFACT-GRAPH-002`'s vocabulary arrives with the corpus
and supersedes the whole table.

## What This Says About D-129

The store now holds every governing record. It holds them because `include_str!` embeds the
files and a test compares the list to the directory — which means **the files are still the
substrate and the store still holds a copy.**

That is the honest reading, and it is why `D-129` was amended at version 2 rather than
declared satisfied. The round trip that record decides — read a record out as markdown, write
the edit back as a transaction with a mandatory preview — does not exist. `P9-AUTHORING`
carries it.

Until then the arrangement is: files are authored, the store is seeded from them, and a check
holds the two together. That is strictly better than six records the store had never seen,
and it is not what `D-129` decided.

## Status

Closed by `P9-PHASE-GAP`. Three controls confirmed red: a canonical record on disk that
nothing seeds, a seeded id with no file behind it, and a records directory that is not there
— the vacuity guard, without which the whole check passes over an empty set.
