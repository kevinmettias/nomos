---
id: OD-LEDGER-004
type: decision
title: An item reserves the record it will write, not the directory records live in
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - enforcement
relations:
  - target: OD-LEDGER-001
    type: affects
  - target: OD-LEDGER-002
    type: relates-to
---

# An item reserves the record it will write, not the directory records live in

## Question

`work claim` refuses on territory overlap, and a directory contains its files. Every open
item reserved `docs/records`, so every item excluded every other item. Is that an authoring
problem, a resolution problem, or a mechanism problem?

## The Fourth Instance, Measured

P10-RECORD-LOCK recorded three observations from 2026-08-09. Here is a fourth from the same
day, and it is the widest: **one claim blocked the entire board.**

At unix 1786313158 a holder claimed P10-FIRST-CHECK, whose territory included
`docs/records`. Probed at 1786313364, every one of the remaining eight items refused:

```text
P10-RECORD-LOCK        refused: P10-FIRST-CHECK overlaps territory held by claude-first-check
P10-SPEC-DETERMINISM   refused: (same)
P10-ABANDON-REASON     refused: (same)
P9-README              refused: (same)
P9-PUBLIC-API          refused: (same)
P9-FALLBACK            refused: (same)
P9-AUTHORING           refused: (same)
P9-DEPENDS-ON          refused: (same)
```

Eight for eight, on one shared path, while `work list` reported all eight as `ready` — the
separate defect P9-DEPENDS-ON names. The item that exists to fix this was itself among the
refused, which is the part worth recording: **the defect protected itself.** The board could
not be unblocked from the board.

## It Is An Authoring Problem

`Territory::Intersect` decides containment textually, on normalized paths, with no
filesystem access. `Contains_Or_Equals` appends a separator before comparing, so `a/b`
contains `a/b/c` and does not contain `a/bc`. Two consequences, and the second is the
finding:

- `docs/records` contains `docs/records/OD-ANYTHING.md`. One directory claim excludes every
  record anybody might write.
- **Two distinct paths under `docs/records` are already `Disjoint`.** Nothing groups
  siblings. The granularity needed to hold two record-writing items at once was already
  implemented and already tested.

So no comparison code changed. What changed is what the items say.

## What Is Reserved Is The Identifier

An item reserves `docs/records/OD-LEDGER-004`, not
`docs/records/OD-LEDGER-004-the-record-directory-is-the-lock.md`.

That is deliberate, and it is the one place this repair is weaker than it looks. The
identifier is knowable when an item is authored; the slug on the end of the filename is not,
and reserving a guess would be reserving the wrong path. A pattern was rejected outright:
`Territory::Intersect` answers `Unknown` for any territory carrying one, and `Unknown`
refuses — a pattern would restore the total block by a different route.

The consequence, stated plainly: this reserves a **name**, not a decision. Two items could
still each allocate `OD-LEDGER-007` independently, and the collision would surface as a
merge conflict rather than as a refused claim. That is a real regression against the
directory claim, which made the collision impossible by making concurrency impossible. It is
the right trade — a name collision is visible, recoverable and rare — but it is a trade and
not a free win.

Since OD-LEDGER-001 already records that territory is declared and not enforced against what
a holder actually writes, reserving the identifier is the honest granularity rather than a
shortcut: nothing was ever checking the filename.

## How Much This Actually Buys

Not what the first analysis assumed. Nine items were open, which is 36 pairs, and all 36
were excluded. Recomputing every pair with `docs/records` removed and the same containment
rule applied:

```text
pairs that become concurrent: 14    pairs still blocked: 22    total: 36
```

The record directory was **the unique universal edge** — the only path every item shared,
and therefore the only single change that turns a *complete* conflict graph into a sparse
one. It was not the only edge. Twenty-two pairs stay excluded on genuine code overlap:

- `crates/host/nomos-cli` — 8 blocked pairs
- `crates/spec` and `crates/spec/nomos-spec-store` — 6
- `tests/contract` — 5
- `crates/substrate/nomos-ledger` — 4

Recording the number rather than the impression is the point. A repair described at a scale
it does not reach is the error OD-LEDGER-001 was amended to correct.

## The Same Defect, One Level Down

The hub list is not a separate problem. `crates/spec`, claimed whole by P9-AUTHORING,
contains three spec crates and so excludes three items that touch different ones.
`crates/host/nomos-cli` is claimed by five items that touch different commands inside it.

The general shape is *claim the directory, not the artefact*, and the record directory was
merely its most complete instance, because a record is the one artefact every item produces.
Naming it here is deliberate, so the next person to widen a territory to a parent directory
can find the reason not to. Whether the hub crates deserve the same treatment is a question
for a later item on its own evidence, not something to fix speculatively.

## What Produced It

Not an authoring mistake. OD-LEDGER-001's second authoring rule said an item that will
produce a decision claims `docs/records`, and it was written to stop territory being
*under*-declared — the failure it had just seen three times running. On an audit ledger
every item's `why` names a suspected defect, so every item produces a record, so the rule
that fixed under-declaration guaranteed total exclusion instead.

Both halves were right in isolation. The rule was right that the record must be reserved and
wrong about the granularity. It is amended at version 3 rather than replaced.

## The Guard

`crates/substrate/nomos-ledger/tests/records_do_not_serialize.rs`, six tests, asserting over
the repository's own `work/ledger.json` rather than over a fixture. That choice is the whole
design: the defect was never in the comparison code, which had always compared paths
correctly, so a fixture proving two invented territories disjoint would have passed on the
day the board was fully blocked.

- Two open items that write different records are claimed concurrently, through the real
  ledger, against a copy of the real board.
- No open item reserves the bare directory. **Confirmed red:** restoring `docs/records` to
  one item — P10-LAUNCHER-PIPE — fails this test and names it. This one matters most,
  because control 2 below means the repair is not incremental.
- The pre-fix authoring is reconstructed in memory and asserted to offer *no* concurrent
  pair, so the acceptance test above cannot pass vacuously.

Claims are exercised against a copy in a temporary directory. A test that claimed on the
real ledger would take territory from whoever was working the repository while it ran.

## The Controls

A repair that merely stopped refusing would be worse than the defect.

1. **Two items that would genuinely write the same record are still refused**, with the
   holder named. Not a blanket exemption for `docs/records`.
2. **One straggler still claiming the directory blocks everyone.** Containment is doing its
   job. The operational corollary is that this fix could not be applied incrementally; it
   had to reach all nine open items in one pass, and the guard above is what keeps it there.
3. **`docs\Records\OD-LEDGER-004` still collides with `docs/records/OD-LEDGER-004`.** Case
   folding and separator unification survive the finer grain, so it does not reintroduce the
   two-spellings hole `Normalize_Path` exists to close.

## Status

Accepted. Nine open items re-authored, OD-LEDGER-001 amended to version 3, and the guard
above keeps the board from serializing again without somebody being told which item did it.

One thing worth admitting, because it is the same defect this record is about. The territory
of P10-RECORD-LOCK was authored too narrowly: it claimed the record it would write and not
`crates/spec/nomos-spec-store`, and a canonical record cannot land without also being seeded
there. The claim was widened openly while held, with no other claim live, rather than the
edit being made quietly outside the declared territory — which nothing would have stopped,
and which is exactly what OD-LEDGER-001 says nothing can see. It produced a third authoring
rule, now recorded there.

The `Done` items were left as they were authored. They hold no claim and exclude nobody, so
their territories are history rather than reservations, and rewriting them would be editing
the record of what was actually claimed.
