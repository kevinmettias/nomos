---
id: OD-LEDGER-018
type: decision
title: A ledger commit publishes the board, and a transition is recorded on the item rather than in the message
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - history
  - concurrency
  - territory
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-LEDGER-006
    type: relates-to
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
  - target: OD-LEDGER-009
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
---

# A ledger commit publishes the board, and a transition is recorded on the item rather than in the message

## Question

`work/ledger.json` is one file and every session writes it, so `git add work/ledger.json`
stages whatever other sessions wrote between the diff check and the commit. Nothing is
corrupted and no territory rule is broken — `OD-LEDGER-001`'s territory governs who may write
a path, and this is several sessions writing one path legitimately.

What it produces is a history that under-describes what it publishes. `P10-LEDGER-SWEEP`
asked which of two things closes that: a commit that changes the ledger describes the
transitions it carries, or a commit cannot carry transitions its author did not make.

The second is the one that sounds right. It is refused below, and on correctness rather than
on cost.

## What Was Measured

Twenty-two commits touching `work/ledger.json`, `26a5360` back through `6a585b8`, examined on
2026-08-10 by comparing each commit's ledger against its parent's — every item whose state,
holder, verification or abandonment list moved — against the identifiers its message names.

| | |
|---|---|
| commits examined | 22 |
| messages naming every transition they carry | **16** |
| commits carrying at least one transition the message never names | 6 |
| transitions carried and never named | 9 |

The first number is the one that decides this record and it is the one nobody had counted.
**The practice already works most of the time**, and it works in the body rather than the
subject: a subject names one item because a commit is about one item, and the body names the
rest. What fails is not the convention. It is that the convention has no floor, so a busy
session skips it and nothing says so.

Then the nine, by the state each one landed in:

| Transition carried and unnamed | Landed | Held by | Reserved paths |
|---|---|---|---|
| `P10-SYNTAX-V2` in `c42f93b` | Claimed | `claude-syntax-v2-w1` | 18 |
| `P10-SUBJECT-PROJECTIONS` in `847bdd9` | Claimed | `claude-subject-projections-w18` | 9 |
| `P10-REQUIRED-PROJECTIONS` in `3c9170d` | Claimed | `claude-required-projections-w14` | 5 |
| `P10-STALE-WRITER` in `6a585b8` | Claimed | `claude-stale-writer-w14` | 6 |
| `P10-AGENT-REQUIRED`, `P10-CORPUS-TRACE` in `63d51ee` | Ready | — | 6, 2 |
| `P10-BAND0-ADMISSION`, `P10-SERVICE-SEAM` in `c7082ae` | Ready | — | 6, 7 |
| `P10-SYNTAX-SCHEMA` in `6a585b8` | Ready | — | 9 |

Four of the nine were **live claims**. That is the measurement the second branch has to
survive, and it does not.

## The Decision

### 1. A commit that changes the ledger publishes the board, and is not a change set for one item

The document is global coordination state. Its correct value at any commit is exactly what
every session has written into it, which is what `git add work/ledger.json` already stages.
**The commit is already right; only the message is short.**

So the rule is about what the message says, and it is two clauses:

> A commit that changes `work/ledger.json` names the transitions its author made, and does
> not assert that those are the only transitions it carries.

The second clause is the one with a measured instance behind it. `63d51ee` ends *"Nothing
here is a code change; this commit is the transition"* while carrying `P10-AGENT-REQUIRED`
and `P10-CORPUS-TRACE`, two items another session opened. The first half of that sentence is
true. The second is a claim about content its author did not read, and a false account is
worse than a short one — a reader who trusts it stops looking, which is the one thing an
incomplete index must never cause.

### 2. The isolation branch is refused because it publishes a board that grants held ground

To carry only its author's transition, a verb would have to write the document at `HEAD` plus
its own transition, stage that, and restore the working tree. What it publishes is a board
missing every claim made since the author last pulled.

Measured, that is not hypothetical: four of the nine unnamed transitions landed `Claimed`,
covering **38 reserved paths across four live holders**. A commit that had dropped them would
have published a board on which `crates/rules/nomos-rules`, `crates/spec/nomos-spec-project`
and the rest read as free ground.

That matters because of what the document is for. `OD-LEDGER-001` records that territory is
declared rather than enforced — nothing in the filesystem stops a write into claimed ground,
so the ledger's answer *is* the exclusion. A document that has forgotten four claims does not
merely describe the board wrongly; asked for one of those territories, it grants it. The
branch converts a defect in the history into a defect in the coordination primitive, which is
a strictly worse trade at any rate of occurrence.

It is also the write `OD-LEDGER-008` refuses in general form: a writer that does not
understand a document must not write it. A verb that deliberately drops keys it read is that
writer with intent.

### 3. A transition's evidence belongs on the item, and `OD-LEDGER-006` already put it there

This is what makes the first branch sufficient rather than resigned, and it is the reason
this record does not end in a plea for diligence.

`OD-LEDGER-006` decided the general form: *a transition is not persisted at all — it happens,
it produces a new state, and whatever it was carrying is gone unless something on the item
was given the job of holding it.* It then gave abandonment that job, as a list, so that an
item abandoned twice says so.

The consequence for this question is exact. `verified` holds the predicate's result,
`abandoned` holds every deliberate release with its reason, and the takeover list holds every
lapse. **A reader asking what happened to an item asks the item.** The commit message is an
index over a file's change, and the thing it indexes is not the authority for anything.

So an unnamed transition costs a worse index. It does not cost a fact. Any transition whose
evidence a reader will actually need is one that should have been given a home on the item —
and if some future transition has no such home, `OD-LEDGER-006` is the record it fails
against, not this one.

### 4. Why this disagrees with `OD-GATE-005`, which was asked the same question

`P10-DIAGRAM-OWED` was asked whether a commit that changes records must re-render
`diagrams/relations.mmd`, and answered that it must: render from the record set your commit
publishes. Here the answer is that the author names their own transitions and claims nothing
further. Two answers to one shape of question, and the difference is what the artifact is.

| | `diagrams/relations.mmd` | `work/ledger.json` |
|---|---|---|
| authored or derived | derived from the record set | authored, by every session at once |
| has one computable correct value | yes | no — its value *is* what was written |
| can the committed file be wrong | yes, and CI computes the verdict | no |
| what the obligation can be | an action, checked | a discipline, reviewed |

`OD-GATE-005` requires an action because the artifact can be wrong and something can say so.
This record requires a discipline because the artifact cannot be wrong and nothing could say
so. That is the same asymmetry its decision 4 drew between the lint step and the freshness
step — one command reads a tree it is about, the other reads a tree it is not — applied to
artifacts rather than to commands.

## What Was Considered And Rejected

**A gate step comparing the message against the ledger diff.** Mechanically available: the
diff is in the commit and the identifiers are literals. Refused because of what it would
produce. It forces an author to name items they cannot explain, so messages grow a list of
identifiers and answer less than before — and `P10-LEDGER-SWEEP`'s own complaint is that the
history is where a reader asks what a change was *for*. A check that is satisfied by a list
of ids satisfies the letter of this record and defeats its purpose. It also cannot be fixed
after the fact: a message is fixed at commit time and CI's verdict arrives after.

**One file per item, so that `git add` carries exactly one transition.** This is the honest
form of the second branch and it fails on a different record. Exclusion is computed across
every item at once, and a directory has no atomic read: a claim would consult *N* files while
a peer writes some of them, so whether the board refuses depends on when it was read.
`OD-LEDGER-009` decides that a document's validity must not depend on when it is read, and
this arrangement makes that question unanswerable rather than answering it. `OD-LEDGER-007`
priced the adjacent cost — files that serialize the board — and this would be that shape
multiplied by the item count.

**A hazard line in `AGENTS.md` instead of a record.** `P10-LEDGER-SWEEP` refused this in
advance and was right to: the question is which of two designs is correct, and an
operating-hazard line states a habit without the argument that makes it the right one.
`OD-AGENT-001` is the standing reason. If a routing line is ever wanted it is one line and it
cites this record; it is not written here, because the harness is another item's territory.

## What This Does Not Do

- It does not add a check, a verb, or a flag. `P10-LEDGER-SWEEP` reserves two record paths
  and nothing else, which is the correct territory for the answer this record reached and
  would have been the wrong territory for the other one.
- It does not make ledger commits atomic per item, and does not leave that door open: the
  arrangement that would achieve it is rejected above by identifier.
- It does not tell an author to read the whole diff before committing. It tells them not to
  describe what they did not read. Those differ in cost by an order of magnitude and only the
  second is load-bearing.
- It does not touch `git add -A`, which `AGENTS.md` already forbids for a different and
  stronger reason. Staging the ledger explicitly is correct and remains so; what it carries
  is not the author's to prevent.

## Controls

| Weakening | What it produces |
|---|---|
| commit only the author's own transition | a published board missing four live claims across 38 reserved paths — measured over 22 commits, not argued |
| check the message against the diff at the gate | messages that list identifiers their author cannot explain; the index lengthens and answers less |
| one file per item so a commit carries one transition | exclusion computed over a directory with no atomic read, which is the question `OD-LEDGER-009` forbids being unanswerable |
| assert "this commit is the transition" without reading the diff | `63d51ee`: a false account, which stops the reader who trusts it |
| say nothing and let the diff speak for itself | the measured rate: 6 commits in 22 carrying 9 unexplained transitions, and no floor under it |

## Status

Closed by `P10-LEDGER-SWEEP`.
