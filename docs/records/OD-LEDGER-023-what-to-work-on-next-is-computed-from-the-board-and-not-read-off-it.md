---
id: OD-LEDGER-023
type: decision
title: What to work on next is computed from the board, and not read off it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - agent-harness
relations:
  - target: OD-LEDGER-005
    type: relates-to
  - target: OD-LEDGER-017
    type: relates-to
  - target: OD-AGENT-001
    type: relates-to
---

# What to work on next is computed from the board, and not read off it

## Question

Every ledger verb answers *may I have this one* — `Claim`, `Renew`, `Take_Over`, `Decline`
and the refusal each can give. None answers *which one*. `AGENTS.md`'s loop said "Read the
board. Pick an item," and the only thing shaped like selection was `work list`'s state
column, which `README.md` already warns means nothing seconds after it is printed. So the
choice of what to work on next was made by whichever session read the listing, and two
sessions reading the same board could differ with nothing recording that they had.

## What Was Measured

Two incidents, not a hypothetical cost.

`P10-DECLINE-VERB`'s own rationale records that one subject became three items —
`P10-REQUIRABLE-DECLARED`, `P10-REQUIRABLE-FLOOR` which delivered the work, and
`P10-REQUIRABLE-SET` which redid work already landed — because the reasons that would have
prevented the duplication existed as abandonments on the item and sat behind `work show`
while the state column still said `Ready`. A session spent a full run duplicating what was
already committed.

Separately, three sessions independently reserved `docs/records/OD-LEDGER-020` and two
reserved `OD-LEDGER-021`, each authored by a session reaching for the next free `OD-LEDGER`
number off the same snapshot. `work add` refuses a duplicate item id and nothing else, so
five reservations of two identifiers were recorded before anybody compared them —
`P11-SELECTION-AUTHORITY`'s decline is the record of that collision, and it is a smaller
waste than the first incident but the same shape: a choice made from a snapshot by whoever
was reading it, with no authority above the ledger to make it once.

## The Decision

### 1. `Eligible_Items` computes the real answer, once

`nomos_ledger::store::refusal::Eligible_Items(document, now)` filters `document.items` to
exactly those `Claim_Refusal(document, &item.id, now) == None` — the same function `Claim`
and `Listing_Label` already share, so a third opinion about which items are actually
takeable does not get a chance to disagree with the other two. This is the same discipline
`OD-LEDGER-005` names for `Claim_Refusal` itself, applied over the whole board instead of
one item.

### 2. The order is id, because nothing else is written down yet

`OD-LEDGER-017` deferred a `priority` field rather than add one that would satisfy the
letter of `WORK-LEDGER-001` while ageing worst of the three ranking keys the corpus named,
and left the other two — how many descendants an item unblocks, and conflict risk — as
derivable from `depends_on` and `territory` rather than built. Building that derivation is
not this record's subject. `Eligible_Items` sorts by `ItemId` — lexicographic, stable, and
the one key every item already carries that two sessions reading the same board are
guaranteed to compare identically. It is the tie-break for as long as no ranking exists to
replace it, not a ranking of its own.

### 3. The answer surfaces on `work list`, not a new verb

`WorkCommand` and its parser (`crates/host/nomos-cli/src/work/command.rs`,
`crates/host/nomos-cli/src/work/parse.rs`) are outside this item's territory — a
dedicated `nomos work next` would need both, and the item that reserved this record
reserved only `crates/host/nomos-cli/src/work.rs`. `List`, the function behind the existing
`list` verb, is inside that file, so the eligible set's first item is printed as a `next:`
trailer line on an unfiltered `work list` — the "read the board" call the loop's step 2
makes. Filtered calls (`--state lapsed` and the like) do not carry it: a filter asks for one
bucket's rows, and a summary naming an item outside that bucket would contradict the filter
it is appended to. This is a narrower surface than a verb of its own would be, and is
recorded as a cost rather than presented as the ideal shape — see below.

### 4. `AGENTS.md` routes through it

Step 2 of the loop no longer says "pick." It names `nomos work list`'s `next:` line as the
computed answer, claim what it names, or add an item if it names none.
`tests/contract/tests/agent_harness.rs` asserts both halves: that the contract no longer
carries the literal instruction to pick, and that it names the mechanism that replaced it.

## What Was Considered And Rejected

**A dedicated `next` verb.** The better shape — a single unambiguous answer instead of a
line appended to a general listing — and rejected only on territory, not on merits. Blocked
by `command.rs` and `parse.rs` sitting outside what this item may touch; a future item
narrow enough to reserve both, or a widened reissue of this one, can still build it without
this record's answer changing, because `Eligible_Items` is what a dedicated verb would call
first.

**A `priority` field, or a derived ranking from `depends_on`/`territory`.** Out of scope by
`OD-LEDGER-017`'s own terms: the corpus wants a ranking, a bare integer would satisfy the
letter of the requirement while buying the key that ages worst, and building the derived
version is its own item's worth of work, not a byproduct of closing this one.

**Sorting `work list`'s existing rows.** This item's own `done_when` excludes it by name: a
sorted snapshot is the same snapshot `README.md` already warns means nothing seconds after
it is read, because sorting does not compute eligibility — it only reorders labels that
were already correct. `Eligible_Items` re-derives eligibility from `Claim_Refusal` over
every item; the `next:` line is that computation's output, not a presentation of rows
`work list` was already printing.

## Controls

| Weakening | What it produces |
|---|---|
| read `state == Ready` instead of `Claim_Refusal` | the exact defect `Claim_Refusal`'s own doc comment names: a listing calling a held item `ready` |
| sort `work list`'s rows instead of computing `Eligible_Items` | the presentational fix this item's `done_when` names and rejects |
| document order instead of `ItemId` order | two sessions computing the same eligible set and reading it in different orders, because `add`'s insertion order is not written down anywhere a second session can reconstruct |
| leave `AGENTS.md` saying "pick" | the mechanism exists and the contract still tells every session to do the thing the mechanism replaces |

## Status

Closed by `P11-NEXT-WORK`.
