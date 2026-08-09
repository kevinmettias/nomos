---
id: OD-GATE-001
type: decision
title: A skipped test reports ok, so the size of the hole is declared rather than the hole being closed
status: closed
version: 2
authority: canonical-normative-record
tags:
  - verification
  - corpus
  - continuous-integration
relations:
  - target: ARC-SPECDB-001
    type: affects
---

# A skipped test reports ok, so the size of the hole is declared rather than the hole being closed

## Question

Several of this workspace's strongest assertions are made against corpora that live outside
the repository: the v14.36 authoring tree, the versioned archives, and a large real Rust
tree. Each is reached through an environment variable, and a test that cannot find its
corpus returns early rather than failing.

A test that returns early prints `ok`. What distinguishes it from one that read seven
thousand files and agreed with all of them?

## What Was Found

Nothing did, and the scale was not small.

Measured rather than estimated. With all three variables set to nonexistent paths and
`--no-fail-fast`, **68 tests fail**. With none set, **697 pass** — and the same 68 are
inside that number, having read nothing. `.github/workflows/gate.yml` sets none of the
three, so on every pull request the normalizer's reproduction of the real canonical hash,
the byte order mark sweep, the 282/258/234 and 30/28 domain counts, the whole-corpus
ingestion, restoration, the family counts, revision fingerprinting, the regression report,
the sibling suites and the analysis slice's scale claims all report success without
touching their subject.

That is the defect `Test_The_Workspace_Should_Not_Appear_Empty` exists to prevent — *a check
that cannot find its subject must fail loudly, not quietly verify nothing* — applied one
level down from where it was written and not applied one level up from it.

## What Was Reported And Was Not True

The finding arrived stated as "only 3 tests fail with the variable pointed at a nonexistent
directory, so a misconfigured corpus is mostly silent too". Both halves are wrong, and the
remedy differs accordingly.

**Misconfiguration is loud.** Every helper behind these variables asserts the path is a
directory and fails when it is not, with the reasoning written at the assertion. A corpus
pointed somewhere wrong stops the suite.

**Three variables, not one.** The three-test figure was one variable set. Setting one does
not wake the tests gated on the others, which is three independent gates rather than
silence.

So the defect is *absence*, and absence is exactly the case CI is in. Recorded because a
remedy aimed at the reported problem — validating configured paths harder — would have
changed nothing at all.

## The Decision

**Declare the size of the hole. Do not close it, and do not fail for it.**

The corpora are not in this repository and most machines will never have them. A test that
failed for their absence would make the suite red everywhere by default, which is the
mutually-contradictory-gate defect `rustfmt.toml` already records: a gate that can never be
green is a gate everybody learns to ignore.

So `tests/contract/tests/corpus_gates.rs` holds a table of every test file that reaches a
corpus, what it reads, how many of its tests are gated, and how many tests it has in total.
Six assertions hold it to the source:

- Every file that reaches a corpus is in the table, so a gated file added tomorrow cannot
  join a green suite unnoticed.
- Every file in the table still reaches one, so the table cannot over-report and inflate the
  hole.
- Every file holds the number of tests it declares and reads the variables it declares —
  which is what makes a *new test inside an already-listed file* visible. Without it the
  table would be satisfied forever by listing fourteen files once.
- Every file's `gated` count is the number the source holds. This is the column the headline
  sums, this record cites and the gate step prints, and it was for a while the only one that
  was declared and never derived — held to nothing but being no larger than its file's test
  count, which `gated: 1` satisfies in every row. The derivation is
  `nomos_contract_tests::Corpus_Gates`, in `tests/contract/src/gates.rs`.
- The scanner and the table name the same three variables, so the two lists cannot drift into
  counting different holes.
- The headline, 68, is the table's own sum.

The derivation is not a text search, and could not be one. Almost no gated test names a
variable itself: the gate is in a `Corpus()` helper the test calls, sometimes through a
second helper, so counting the tests in a file that mention a variable finds nearly none of
them. `Corpus_Gates` resolves a test to the corpora it reaches by propagating along calls
until nothing changes, over a file scanned with comments and string literals masked out —
brace matching without that mask desynchronises on the first `panic!("{} ...")`, of which
this workspace has many, because a failing assertion here is required to name what it saw.

It was written against the table without consulting it and reproduced all fourteen rows and
the total of 68. It also independently reached the corrected attribution below, for
`corpus.rs` and `portable.rs`. Two derivations built separately, agreeing on every row, is
what the number rests on now.

And one that reports rather than asserts: a run prints how many corpus-backed assertions did
not run, in both directions, so a configured run prints zero and the line's absence is
itself visible. CI runs it under its own step with `--nocapture`, because cargo swallows a
passing test's output — which is precisely how sixty-eight assertions came to report success
in silence.

## The Two Things The Check Found Immediately

**Its own file.** `corpus_gates.rs` names all three variables, to report on them, and the
scanner found itself. The variable names are now assembled with `concat!` so the file does
not contain the spellings it searches for, and comment lines are excluded so that a variable
named in prose is a reference rather than a use. Both rather than an exemption list: an
exemption is the easiest place in a check to hide something, which is what P8-COMPOSE
recorded when `Test_Nothing_In_The_Slice_Should_Invent_An_Identity` had the same problem.

**Two wrong rows in the table it was checking.** The table was generated by a script that
searched raw text, and it recorded `nomos-lang-rust/tests/corpus.rs` and
`nomos-workspace/tests/portable.rs` as reading `NOMOS_V14_CORPUS`. They mention it in a
comment. The check disagreed with the table on its first run and the table was wrong — which
is the whole argument for checking a declaration against the source rather than trusting the
measurement that produced it.

## What This Does Not Do

It does not make the assertions run. 68 corpus-backed assertions are still unverified on
every machine but the one holding the corpora, and that remains true until either the
corpora are reachable from CI or the properties are restated over fixtures that are not.
Both are real options and neither is decided here.

What changed is that the number is now a value somebody chose, checked against the source,
and printed beside the green tick — rather than a silence nobody had measured.

## Status

Closed by P9-SKIP, and amended at version 2 when the `gated` column stopped being declared
only and started being checked against the source.

Six controls confirmed red: a gated test added to an already-declared file, a new gated file
in nobody's table, a walk that finds no test files at all (the vacuity guard, which would
otherwise let every assertion here pass over an empty set), a headline drifting from the
table it sums, a `gated` count disagreeing with what the source holds, and the scanner's
variable list drifting from the table's.
