---
id: OD-MODEL-001
type: decision
title: A claim and a fact address a path by one rule, and the record-identifier fold stays with the ledger
status: closed
version: 1
authority: canonical-normative-record
tags:
  - model
  - ledger
  - subject
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
  - target: OD-SPEC-006
    type: relates-to
---

# A claim and a fact address a path by one rule, and the record-identifier fold stays with the ledger

## Question

Three places in this workspace reduced a repository path to a `SubjectId`, and all three
had written the reduction out longhand:

- `nomos_ledger::Subject_Of`, addressing the territory an item **claims**;
- `tests/integration/src/corpus.rs`, addressing a **fact** about a corpus file;
- `crates/host/nomos-cli/src/check.rs`, addressing a **fact** from the composition root.

The spelling rule in all three was byte-for-byte identical: trim, unify separators, drop
empty and `.` segments, lowercase. The ledger then applied one more step the other two did
not — folding a record filename onto the record identifier it carries.

The corpus copy carried a doc comment saying the duplication was "worth converging behind
one home the moment a third caller appears". `OD-RULES-001` made `check.rs` that third
caller, and it said so in its own comment while declining to do anything about it.

Converging is not a move of one function. Two of these compute what a *fact* is about and
one computes what a *claim* is about, and the question underneath is whether those are the
same kind of address.

## What Was Actually Wrong

**Three copies of a rule that decides identity.** Not three copies of a convenience. If two
of them drift, a composition root files facts under subjects a rule computes differently,
and every affected fact reads as *unavailable* rather than as one function being wrong.
That failure is silent in exactly the way this system exists to prevent.

**The existing comments made the duplication look considered.** Each site explained that
importing one of the others would "couple what a fact is about to what a claim is about".
That reasoning was right about the *record fold* and wrong about everything else, and
because it was stated once and copied twice it read as settled.

## The Answer

**The spelling rule is one rule, and it lives in the kernel.**
`nomos_model::Normalize_Path` and `nomos_model::Subject_Of_Path` are the single home, beside
`SubjectSet` and `Intersection` — which are in the kernel for the same reason, recorded in
that module's own documentation: three subsystems at three levels need to ask one question,
and a type defined at any one of those levels cannot be used by the others.

A claim and a fact *do* ask the same question of a path spelling. "Which file is this?" has
one answer, and `src/Main.rs` against `src/main.rs` is one file whether the asker is
reserving it or measuring it. Keeping two copies did not preserve a distinction; it
preserved the *option* of a distinction nobody wanted, at the price of a real drift risk.

**The record-identifier fold stays with the ledger, and composes on top.**
`nomos_ledger::Normalize_Path` now calls the kernel's and applies
`Record_Identifier_Form` to the result. Its public signature is unchanged and its behaviour
is unchanged, which is the whole of what a reader needs to check.

The fold is not a fact about what a path spells. It exists because an item must reserve a
record *before the file exists*, so the identifier is the only name two items can both write
down in advance — `OD-LEDGER-001`'s authoring rule, and `OD-SPEC-006` makes `docs/records`
the one directory where the identifier-to-filename relation is written down rather than
inferred. That is a fact about coordinating unwritten work.

So the two subject rules differ on exactly one shape of path and agree everywhere else, and
the difference is now visible at one site instead of being the residue of two copies.

## Why The Composition Runs Ledger-Over-Kernel

The obvious alternative was one function with a parameter — `Normalize_Path(path, folding)`
— so that the caller "asks for" the fold. It was rejected on two counts.

**It teaches the kernel what a decision record is.** Band 1 would then carry
`docs/records`, the identifier grammar, and the ordinal rule. A fact about a record file
would become one flag away from being a fact about the identifier instead, and the flag
would be at every call site rather than at one.

**The direction is what makes the difference safe.** A subsystem may add a rule of its own
on top of the kernel's; the kernel may not host every subsystem's rule and hope each caller
selects correctly. The first is composition and the second is a switch.

The cost is that `nomos_ledger::Subject_Of` and `nomos_model::Subject_Of_Path` are two
functions of the same shape that disagree on one input. Both now say so at their
definitions and name this record, and the harness asserts the direction of the difference
rather than assuming it.

## What Was Considered And Rejected

**Importing the ledger's into the two fact producers.** The item's `done_when` refuses it
outright, and correctly: it would have made every fact about a file in `docs/records` a fact
about the identifier instead, so editing one record would invalidate facts about another.
The corpus test named below is the guard against someone doing it later.

**Recording that the three are deliberately different and leaving them.** Available, and
dishonest — they were not different. Two of them were identical and the third differed by a
step neither of the others wanted. Writing "these are deliberately three" would have
converted an accident into a decision.

**Putting the home in `nomos-contracts` beside `SubjectId`.** The rule needs
`Content_Digest`, which is in the kernel, and `nomos-contracts` sits below it. Following
`SubjectId` down would have meant moving the digest too.

## What Holds It

- `crates/kernel/nomos-model/src/path.rs` — the spelling assertions, including the negative
  control that different paths stay different subjects, and
  `Test_A_Record_Filename_Should_Not_Fold_Onto_Its_Identifier`, which fails if the kernel
  ever absorbs the ledger's step.
- `crates/substrate/nomos-ledger/src/territory.rs` — the existing record-fold tests are
  unchanged and still pass, which is what says the composition preserved behaviour rather
  than merely compiling.
- `tests/integration/src/corpus.rs` —
  `Test_A_Fact_About_A_Record_File_Should_Be_About_That_File`, asserting which of the two
  rules the fact side takes. The spelling assertions moved to the kernel with the rule,
  because repeating them beside the caller would have been the same duplication in a second
  costume.
- `tests/contract/surface/nomos-model.txt` — the two names are public surface now, so
  narrowing them is a snapshot change somebody has to bless on purpose.

## Status

Closed by P10-SUBJECT-HOME. A fourth longhand copy exists in
`tests/integration/tests/determinism.rs`, outside this item's territory and inside
`P10-ROLLUP-DETERMINISM`'s; it addresses fixture text rather than repository paths, and
converging it is that item's to weigh.
