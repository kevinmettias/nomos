---
id: OD-PROJECT-003
type: decision
title: Any committed projection carries the re-render tax, not only a required one
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - projection
  - freshness
  - gate
  - publication
relations:
  - target: OD-PROJECT-002
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
---

# Any committed projection carries the re-render tax, not only a required one

## Question

`OD-PROJECT-002` requires `diagram-set` and `domain-specification`, and declines to require
`html-site` and `traceability-matrix` because a third and fourth required projection would put
a re-render obligation on every future record author. That reasoning is stated in terms of
`--require`. It leaves open whether an *unrequired* projection, rendered once and committed
anyway, would be free of the same obligation.

It was measured rather than assumed, because the two are not the same question:
`freshness --into .` is what `.github/workflows/gate.yml` runs, and it is not scoped to
`--require`. It reports on every projection output it finds on disk, required or not.

## What Was Measured

`html-site` renders without a corpus, is one of the two `OD-PROJECT-002` names as cheap and
deliberately unrequired, and was the test case.

Rendered once against a seeded-only store (no corpus — CI's condition) and left untouched,
`nomos spec freshness --into .` reports it `current` and exits `0`. Edited by one byte and
checked again, it reports `edited: the stamp declares … and the file hashes to …` and exits
`8` — with no `--require html-site` anywhere in the invocation.

`mcp-resource`, a corpus-backed profile, was rendered once against a store seeded with the real
v14 corpus, then checked with the corpus absent (CI's actual condition on every run). It
reports `this store is not whole` and exits `6`. Absence of `mcp/specification.json` would have
been silent under `OD-PROJECT-002`'s own rule — `freshness` treats a build root holding a
subset as normal. **Presence** of a corpus-backed output on a corpus-less runner is not treated
the same way as absence; it is treated as drift, and drift always fails.

## The Consequence

`OD-PROJECT-002`'s stated cost of requiring a third projection —

> an author who renders three of five, commits, and reddens the gate for everybody behind them

— is paid the moment a projection is **committed**, not the moment it is **required**.
`--require` only changes what happens when an output is *absent*. It has no effect on what
happens when a committed output is *present and stale*, and staleness is unconditional: the
gate's `freshness` step checks whatever is on disk, and a stale or unrebuildable output fails
it regardless of whether that output's name was ever typed after `--require`.

A corpus-backed profile makes this worse than an ordinary stale file: CI can never rebuild it
(no corpus is ever set there), so committing one is not a re-render obligation somebody might
occasionally forget — it is a permanent, unrecoverable drift the moment any governing record
this repository's part of the store contributes to changes under it.

## The Decision

**No profile beyond the two `OD-PROJECT-002` already requires is committed to this repository,
including the two it names as renderable-but-not-required.** `html-site` and
`traceability-matrix` remain buildable on demand and stay out of the tree; the fourteen
corpus-backed profiles remain local, on-demand renders against a corpus set by `--corpus` or the
three environment variables, exactly as `nomos spec sources` already reports them.

This does not amend `OD-PROJECT-002`'s required set. It closes the gap that record left open:
whether the *unrequired* two were a cheaper, safe alternative to committing. They are not
cheaper — committing either one reopens the same obligation `OD-PROJECT-002` priced and
declined to pay, by a mechanism that record did not name.

## Controls

| Weakening | What it produces |
|---|---|
| commit `html-site` or `traceability-matrix` as a convenience artifact | a re-render obligation on every governing-record commit, with no `--require` line naming it as the cause of a red gate |
| commit a corpus-backed profile's output because it rendered cleanly once, locally | a permanently stale committed file, since CI never has a corpus to rebuild it with |
| read `OD-PROJECT-002`'s two-profile required set as the full list of what is safe to commit | this record, which found the third case that set never covered |

## Status

Accepted. The full eighteen-profile catalogue remains available through `nomos spec render`
and `nomos spec sources`; this record is why only two of the eighteen outputs are ever
checked in.
