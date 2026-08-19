---
id: OD-LEDGER-032
type: decision
title: An assertion over live coordination state is honest when that state is empty, and proves its teeth on a constructed subject
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - testing
  - gate
  - vacuity
relations:
  - target: OD-LEDGER-004
    type: relates-to
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
  - target: OD-LEDGER-003
    type: relates-to
---

# An assertion over live coordination state is honest when that state is empty, and proves its teeth on a constructed subject

## Question

`OD-LEDGER-004` decided that the acceptance suite for `P10-RECORD-LOCK` reads the repository's
own `work/ledger.json` rather than a fixture, and the reasoning is right: "the defect was never
in the comparison code, which had always compared paths correctly, so a fixture proving two
invented territories disjoint would have passed on the day the board was fully blocked." The
defect lived in what the items *said*, so the items are the subject.

Anti-vacuity guards were then added on top, each demanding the board hold at least two open
record-writing items. Nobody asked what those guards say about a board with nothing open.

A board with nothing open is not an edge case. It is what finished work looks like, and it is
what CI checks out, because the ledger is a file in the tree.

## What Was Measured

Every commit's own committed `work/ledger.json` was read and its open items whose territory
reserves anything under `docs/records` were counted, over the last thirty commits of `dev`:

| Open record writers in the committed ledger | Commits |
|---|---|
| 0 or 1 | **26 of 30** |
| 2 | 3 of 30 (`ed03cf6`, `1fa8a8c`, `ce76c96`, inside seven minutes on 2026-08-18) |

Seven assertions in `crates/substrate/nomos-ledger/tests/records_do_not_serialize` fail below
two. Reproduced on a clean tree at `4823e31`: `7 passed; 7 failed`, every failure of the form
*"fewer than two open items reserve a record, so the concurrency this file measures cannot be
observed; got 0"*.

The gate's `Test` step is `cargo test --workspace`. So that step has been red on nearly every
commit of this branch, and red for something no commit contains — a property of which sessions
happened to be mid-work when the commit was made. A gate answering a question about the
committer's calendar is not a gate, and one that is red for reasons unrelated to the change
trains everybody to stop reading it.

Separately measured while closing this, because it changes the shape of the answer: with
*one* open record writer, `Undeclared_Serializers` reported that writer's entire territory —
eight paths, including the record registration and the seven test files the item was itself
editing — as structural serializers. Below two, the search does not merely weaken; it inverts.

## The Decision

An assertion whose subject is live coordination state stays an assertion, is honest when that
state is empty, and proves it can fail on a subject built for the purpose rather than by
requiring the live state to be populated.

Three shapes follow, and each guard takes exactly one.

| Shape | Applies when | Here |
|---|---|---|
| **Honest assertion** | the claim is true of an empty subject | `Test_No_Open_Item_Should_Reserve_The_Whole_Record_Directory`, `Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer`, `Test_Every_Universal_Reservation_Should_Be_Declared` |
| **Constructed control** | the claim's teeth must be shown | `Test_An_Item_Reserving_The_Whole_Record_Directory_Should_Be_Found`, `Test_An_Undeclared_Serializer_Should_Be_Found`, `Test_Two_Items_Writing_One_Record_Should_Still_Be_Refused` |
| **Report, not assertion** | the figure is worth knowing and is not a property of the code | `Test_A_Run_Should_Report_Whether_The_Board_Is_Parallel` |

"No open item reserves the whole record directory" is *true* of a board with no open items.
"Every record writer can be claimed at once" is true of a board with one and of a board with
none. Reporting those as failures is not rigour; it is the assertion answering a different
question from the one it states.

A control shares the search the assertion makes rather than reimplementing it beside it —
`Directory_Reservers` and `Undeclared_Serializers` are each called by both — so the control
exercises the code that will actually be wrong when it is wrong.

## The Precedent This Follows

`tests/contract/tests/rule_contract_citation.rs` already does exactly this: the real records
are checked, and `Test_A_Mismatched_Version_Should_Fail_The_Comparison` writes a fixture record
under a scratch root to prove the comparison fires. The real subject and the adversarial
subject are different subjects, and demanding that the real one be adversarial is what breaks.

`OD-LEDGER-004` also already reconstructs the pre-fix authoring in memory — the board as it was
on the day the defect existed — precisely to show the acceptance test cannot pass vacuously.
That reconstruction is this record's shape, applied once, before anybody named it.

## What This Does Not Say

It does not say a test may pass over an absent subject. `OD-GATE-001` governs that and it is
not weakened here: the corpus gates report an absence rather than swallowing it, and
`AGENTS.md` states plainly that a test which cannot find its corpus passes having read nothing.
The distinction is that an *absent* subject means the check could not run, while an *empty*
subject means the check ran and the answer is that there is nothing wrong. A board at rest is
the second. What is forbidden is not knowing which of the two you have, which is why every
honest assertion above is paired with a control that fails.

It does not weaken `OD-LEDGER-004`. Where the real board is the subject, the real board is
still read. `Two_Record_Writers` stopped reading it only because both of its callers overwrite
both territories before contesting anything — they never read what the real items reserved, so
borrowing them bought nothing and cost a green tree at rest.

It does not touch `OD-LEDGER-007`'s debt register or `KNOWN_SERIALIZERS`, which is empty and
stays empty. Nothing here declares a serializer to silence a search.

## What Was Considered And Rejected

**Lower the threshold from two to one.** Rejected on the measurement: one writer inverts the
universal search into eight false positives. Two is not the problem; reading a number that
changes without the tree changing is.

**Skip the guards when the board is at rest.** Rejected, and it is the tempting one. A skip is
indistinguishable at the console from a pass, which is the class `OD-GATE-001` exists for. A
constructed control runs every time and costs nothing.

**Make the guards conditional on an environment variable.** Rejected for the same reason plus
one more: the variable would be unset in CI, which is the one place the answer matters.

**Delete the guards.** Rejected. The file exists because a property nobody exercised turned out
not to hold; the guards are how it stays exercised. They are moved onto subjects that exist,
not removed.

**Keep the ledger out of the repository so CI cannot read it.** Rejected as far larger than this
question and contrary to `OD-LEDGER-001` — territory is declared in a file everyone can read,
and that file being in the tree is what makes the declaration reviewable.

## What Holds It

The suite itself: seven assertions that were failing now pass, and three constructed controls
prove the searches still find what they are for. Measured both directions rather than argued —
with the closed defect reintroduced (one open item reserving `docs/records`),
`Test_No_Open_Item_Should_Reserve_The_Whole_Record_Directory` and
`Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` both fail, and the first names the
offender.

What does **not** hold it should be said plainly. Nothing stops the next author from adding an
assertion over the live board with a population precondition on it, and the 26-of-30 number is
the kind that only gets measured when somebody goes looking. This record and the doc comments
beside each guard are the whole of the defence.

## Status

Accepted. `P13-QUIESCENT-BOARD-RED` carries it. The gate's `Test` step being red for reasons
unrelated to the commit is `OD-LEDGER-003`'s scoping gap seen from the other side: `work finish`
runs the item's own predicate, so a workspace-wide failure no item's predicate names can persist
across many green finishes — measured on 2026-08-19 at a full day for an unrelated failure, and
at nearly the whole branch for this one.
