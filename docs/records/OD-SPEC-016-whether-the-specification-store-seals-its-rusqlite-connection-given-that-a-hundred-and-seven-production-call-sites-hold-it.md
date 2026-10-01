---
id: OD-SPEC-016
type: decision
title: Whether the specification store seals its rusqlite connection, given that a hundred and seven production call sites hold it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - spec
  - storage
relations:
  - target: OD-SPEC-001
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
---

# Whether the specification store seals its rusqlite connection, given that a hundred and seven production call sites hold it

## Question

`SpecificationStore::Connection(&self) -> &Connection` is public, and four crates use it to
write SQL against SQLite's own handle. `OD-SPEC-001` says the test that keeps a store trait
honest is store-backend equivalence — in-memory, SQLite and Postgres yielding the same store
hash over the same corpus — and that test cannot be written while callers hold a `rusqlite`
handle, because a query written against one cannot be satisfied by any other backend.

Postgres itself is not in question; `OD-SPEC-001` declines it and that decline stands. What
is in question is whether the seam that record names can exist at all in the current shape,
and the answer was assumed to turn on a count of call sites.

It does not. The count is the wrong measurement, and this record is mostly about why.

## What Was Measured

Measured 2026-09-13, against the whole workspace.

**The escape, by caller.** 107 production calls to `Connection()` outside the owning crate,
144 counting tests:

| crate | production | tests |
|---|---|---|
| `nomos-spec-bundle` | 52 | 7 |
| `nomos-spec-ingest` | 34 | 21 |
| `nomos-spec-project` | 18 | 5 |
| `nomos-spec-validate` | 3 | 4 |

**The SQL, by statement.** 159 distinct statements over 22 tables — 118 `SELECT`, 46
`INSERT INTO`, 9 `INSERT OR …`, 5 `CREATE TABLE`, 1 `PRAGMA`.

**The handle, by operation.** Five. Every one of those 107 sites reaches the connection for
`prepare` (13), `execute` (7), `query_row` (4), `execute_batch` (3) or `last_insert_rowid`
(1), and for nothing else. There is no sixth.

**Dialect lock-in.** 151 of the 159 statements are plain portable SQL. Eight are not: six
`INSERT OR IGNORE`, one `INSERT OR REPLACE`, one `PRAGMA`.

**Type lock-in.** 84 direct `rusqlite::` mentions across the four crates — `params!` 34,
`Result` 22, `Row` 6, `Error` 5, `Connection` 5, `Error::QueryReturnedNoRows` 4, `FromSql` 2,
`Statement` 2, and one each of `params_from_iter`, `Transaction`, `ToSql`,
`Connection::open_in_memory`.

**Two populations, not one.** `nomos-spec-bundle` is half the escape and is not asking the
store questions. Its own module doc states its contract: "every column of the store reaches
the bundle." It is a schema traversal — a backup format whose job is the database — and it is
checked three ways precisely so no column escapes it. The other 55 sites, in `ingest`,
`project` and `validate`, do ask questions.

## The Decision

**The connection stays public. Sealing it behind named operations is refused. The seam
`OD-SPEC-001` needs is real and buildable, but it is narrow in the type dimension, not the
operation dimension, and it is not paid for now.**

### Named operations is refused, and the number that refuses it is 159

One method per query is 159 methods against a public surface that is 266 lines today. The
store would roughly triple, and more than half of the additions would be `nomos-spec-bundle`'s
schema re-typed as an API: a method per table per direction, whose only caller is the one
crate that already declares its coverage column by column and tests it three ways. That is
not a seam, it is the schema with a second spelling, and a second spelling of the schema is a
second authority over it.

The item that raised this said a store growing one method per query is a different problem.
It is, and 159 is the size of it.

### Why 107 was the wrong number

The call sites do not vary. **Five operations** serve all 107, and four of those five are
`prepare`, `execute`, `query_row` and `execute_batch` — the same four any SQL library exposes
under those or adjacent names. Nothing about the *count* of callers makes a second backend
harder; a thousand `SELECT`s through one `prepare` cost exactly what one does.

What makes a second backend impossible is that those five operations are spelled on a
concrete type. 151 of 159 statements would run unchanged against any SQL backend carrying
this schema. What would not compile is `rusqlite::params!`, `rusqlite::Row` and
`rusqlite::Statement`, at 84 sites.

So the obstacle is 84 type mentions and 8 statements, not 107 call sites — and it is smaller
than the framing suggested, in a different place than the framing pointed.

### What it would cost, in the order it would have to be paid

Recorded so that whoever revives `OD-SPEC-001` does not re-derive it:

1. **The 8 dialect-bound statements become portable.** `INSERT OR IGNORE` and
   `INSERT OR REPLACE` have a portable spelling; the `PRAGMA` is configuration, not a query.
   Smallest of the three, independently worth doing, and the only one that removes a lock-in
   that is invisible today.
2. **The five operations move behind a trait the store owns**, yielding store-owned row and
   parameter types rather than `rusqlite`'s. This is the actual seam, and it is five methods
   wide — not 159.
3. **The 84 `rusqlite::` mentions follow mechanically from 2** and the four manifests drop
   the dependency.

Step 2 is the one with judgment in it. Steps 1 and 3 are mechanical once it is decided.

### Why it is not paid now

`OD-SPEC-001` declines the second backend, and a second backend is the only consumer of this
seam. Building the seam first would be building an abstraction over one implementation and
calling the absence of the second a design — which is the shape this workspace refuses by
default and has refused for this exact record before.

The equivalence test stays unwritten, and this record is what makes that a stated cost rather
than an unnoticed one.

## What This Record Does Not Do

It does not reopen `OD-SPEC-001` or decide anything about Postgres. Its trigger is unchanged:
concurrent authoring across machines.

It does not touch `nomos-spec-bundle`'s design. That crate holding the schema is correct for
what it is, and no seam should try to take it away.

It does not build any of the three steps. What it changes is one thing:
`SpecificationStore::Connection` carried no documentation at all — the widest escape in this
workspace, public, `#[must_use]`, and silent about why it exists. It now says what it is, what
it costs and which record decided that, so the next reader inherits this measurement instead
of taking it again.

## Status

Accepted. The connection stays public; named operations are refused at 159 statements; the
seam `OD-SPEC-001` would need is five operations wide and blocked by 84 type mentions and 8
statements, not by 107 call sites; and it waits on the backend that would consume it.
