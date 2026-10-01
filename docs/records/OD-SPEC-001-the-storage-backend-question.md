---
id: OD-SPEC-001
type: decision
title: Whether the specification store gains a second backend, and what would decide it
status: open
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - storage
relations:
  - target: ARC-SPECDB-001
    type: affects
---

# Whether the specification store gains a second backend, and what would decide it

## Question

SQLite is the authoritative local store. A Postgres backend is specified but not built.
Two things are undecided: whether it is ever built, and whether the ledger and the
specification store stay separate.

## Current Position

SQLite only, with a second backend behind a feature if one is ever needed. The portable
authority is the JSONL bundle, not the database file, so a backend change does not change
what is committed to git.

The ledger and the specification store stay two stores with one lock implementation. The
ledger is a JSON file whose readability under `git diff` is load-bearing for the
multi-agent workflow; merging it into a database would trade that for a uniformity nothing
currently needs.

## What Would Decide It

Concurrent authoring across machines is the only requirement that forces a second backend.
SQLite has no `SKIP LOCKED`; the portable equivalent is an immediate transaction plus an
insert that does nothing on conflict, where zero affected rows means "held, skip it". That
is one interface with two implementations, and it works locally. It stops working when two
authors are not on the same filesystem.

The test that keeps the trait honest is store-backend equivalence: in-memory, SQLite and
Postgres must yield the same store hash for the same corpus. Without it the trait grows
per-backend semantics and the second backend becomes a second authority.

## Status

Open. Recorded here so the position is legible rather than implied by the absence of a
Postgres crate. Revisit when concurrent authoring is actually attempted across machines,
or at Phase 10 with a measurement over a real corpus.
