---
id: OD-LEDGER-041
type: decision
title: Terminal items stay on the board because the claim check reads them, and the cost that was measured is the listing's unbounded default rather than the file
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - repository
  - coordination
relations:
  - target: OD-LEDGER-036
    type: relates-to
  - target: OD-LEDGER-023
    type: relates-to
  - target: OD-LEDGER-020
    type: relates-to
  - target: OD-LEDGER-038
    type: relates-to
  - target: OD-LEDGER-018
    type: relates-to
  - target: OD-LEDGER-009
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-LEDGER-027
    type: relates-to
---

# Terminal items stay on the board because the claim check reads them, and the cost that was measured is the listing's unbounded default rather than the file

## Question

`work/ledger.json` is 7,439,333 bytes and holds 1,483 items, of which 1,464 are in a terminal
state. An external review of `bc0aaac` called this genuine rather than cosmetic bloat, observed
that the original justification for a file was that a human can read its diff, argued that the
property degrades at this size, and proposed splitting active coordination state from an
append-only history of completed work.

No record has asked the question. `OD-LEDGER-036` settles what the ledger is for,
`OD-LEDGER-018` that a commit publishes the board, `OD-LEDGER-023` that what to work on next is
computed from the board rather than read off it, `OD-LEDGER-020` and `OD-LEDGER-038` what a
dependency edge onto an ended item means, and `OD-LEDGER-008` and `OD-LEDGER-009` what a writer
owes a document it reads. None of them asks whether an item that has reached a terminal state
stays in the file every verb parses.

## What Was Measured

At `ee4d6111`, 2026-09-21, against the committed blob rather than the working copy.

**The file is almost entirely terminal.** 1,116 Done and 348 Declined items hold 6.62 MB of the
6.69 MB of item bytes. The nineteen items that are Ready, Claimed, held, waiting or blocked --
the coordination set, the thing the file exists to carry -- hold 0.07 MB. By field, the bytes
are 35.0 per cent `verified`, 26.2 per cent `why`, 18.0 per cent `done_when` and 7.2 per cent
`territory`; the largest single `verified` entry is 2,822 bytes, so no field is unbounded.

**The clone cost is not where the review expected.** The ledger's entire history -- 1,176
committed versions of the file -- packs to 2.87 MiB, against 35.83 MiB for the whole
repository. Git deltas the file well because each commit moves about a hundred lines of it: the
six most recent ledger commits changed between one and 168 lines. So the artifact that is 7 MiB
in the working tree costs about eight per cent of a clone, and the diff-legibility property the
file was justified by is measured intact at the commit level rather than degraded.

**Parsing it is not a cost either.** `nomos work list` parses the whole document, computes
eligibility over every item and renders, in 37 milliseconds.

**The cost that is real is the listing's default, and it is the one an agent pays.**
`nomos work list` with no filter prints one row per item: 1,486 lines and 238,656 bytes, of
which 1,464 rows are items nobody can act on. `AGENTS.md` step 2 sends every session to exactly
that command before it does anything else. So the review's token-consumption claim is correct,
and its cause is not the size of the file but a verb whose default answer to "what work is
available" is the whole history of the board.

**Terminal rows are read by the check that decides whether an item can be claimed.**
`Eligible_Items` filters `document.items` by `Claim_Refusal`, the same function `Claim` and the
listing label already share, and that refusal distinguishes a dependency that is unfinished
from one that was declined -- `DependencyDeclined` names the dependency and its state, which
`OD-LEDGER-020` requires so a dependent is told it is a dead end rather than told to wait. A
Done dependency is read the same way: it is how a dependent becomes claimable at all.

## Decision

**Terminal items stay in `work/ledger.json`. No archive file is created, and no verb changes
under this record.**

Three reasons, in the order they would bite.

**An archived item is indistinguishable from an item that never existed.** The claim check reads
a dependency's terminal state to produce its answer, so a board that has moved terminal items
elsewhere gives one of two wrong answers for every dependent: it refuses an item whose
dependency is satisfied, or -- worse, and the direction this repository writes records about --
it treats an absent dependency as satisfied and grants a claim whose precondition nothing
checked. `OD-LEDGER-038` is the record for what an ended item silently does to its dependents,
and archiving would make that silence structural rather than occasional. The remedy, having
every verb read both files, is not a remedy: it is the same document in two places, which is
what the split was for.

**Two files make validity depend on which one was read.** `OD-LEDGER-009` fixed exactly this
shape once: a document that was valid must not become invalid through no writer's act. A board
whose answers depend on whether the archive was loaded has that defect by construction, and
`OD-LEDGER-018`'s guarantee -- that a commit publishes the board -- would become a guarantee
about a pair of files whose consistency nothing checks. `OD-LEDGER-008` is the third edge: a
build that cannot account for every key refuses to write the document, and that guard is at one
door, `Load`. A second document reachable through a second door is a second place for the same
question to be answered differently.

**The saving does not exist.** Archiving every terminal item removes at most 2.87 MiB from a
35.83 MiB clone, and less than that in practice because the archive itself ships. Against that
it spends the single-board property `OD-LEDGER-036` describes and this repository has twice
paid to keep, most recently when `OD-LEDGER-007` found that two other files were serializing the
board.

**What is owed instead is a bounded default for the listing.** The measured cost is
238,656 bytes of output in which 98.7 per cent of the rows are unactionable, produced by the
command the operating contract tells every session to run first. That is a defect in the verb,
not in the file, and its fix is one this record names rather than makes: `work list` already
takes `--state`, so the increment is to make the default answer the live board and to put the
whole history behind a flag, with `AGENTS.md`'s step 2 reading whichever spelling survives. Its
territory is `crates/substrate/nomos-ledger`, `crates/orchestration/nomos-work-orchestration`,
`crates/host/nomos-cli`'s work module and their tests, plus `AGENTS.md` if the spelling changes,
and it is a capability item rather than a decision, because this record has decided it.

The distinction that makes this the right split of the work: the file is the board, and the
listing is a view of it. `OD-LEDGER-023` already holds that what to work on next is computed
rather than read, and a computation needs the whole board present. A view does not.

## What Would Decide It Differently

Each of these is a number, so that a future session can check it rather than re-argue this.

- **`nomos work list` above 500 milliseconds**, roughly thirteen times today's 37, or any verb
  whose latency becomes visible to a person waiting on it.
- **The ledger's packed history above a third of the repository pack**, against today's eight
  per cent. That would mean git has stopped deltaing the file well, which would itself be
  evidence that the shape of a ledger commit had changed.
- **A live coordination set in the hundreds.** Today it is nineteen. A board whose *live* half
  is large is a different problem from a board with a long history, and an archive would not
  address it.
- **A consumer that must read the board without parsing it whole** -- an editor surface, or a
  transport that serves the board to something that is not this binary. That consumer needs a
  query, and a query is a different mechanism from a second file.

## What This Does Not Decide

Whether the `verified` field should keep a command's output tail at all. It is the largest
single consumer of bytes at 35 per cent, its entries are bounded, and shrinking it is a question
about what a verification record owes a future reader, which `OD-LEDGER-027` owns.

Whether a declined item should be kept forever. This record measures that 348 of them cost
little and are read by the claim check; it does not decide what a deliberate deletion would
mean, and nothing in this repository has needed one.

Whether the board should ever be queryable rather than only loadable. The fourth trigger above
is where that question would arrive, and it belongs to whichever consumer raises it.

## Status

Accepted. No item moves, no file is created, no verb changes, and the one increment this record
names is owed by a separate item.
