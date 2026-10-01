---
id: OD-DETERMINISM-002
type: decision
title: The last two rows declare, and the completeness guard stops asking about facts
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - determinism
  - contracts
  - verification
  - specification-system
relations:
  - target: OD-DETERMINISM-001
    type: affects
  - target: OD-GATE-001
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
---

# The last two rows declare, and the completeness guard stops asking about facts

## Question

`OD-DETERMINISM-001` closed with two rows of the `nomos-contracts` domain table
undeclared — spec-bundle serialization and projection engine output, both in `crates/spec`,
which `P9-DETERMINISM` did not claim. It recorded the gap honestly and could not close it.

Two questions had to be answered to close it, and only one of them is about the two rows.
The other is why the gap survived at all: `Test_Every_Fact_Producing_Crate_Should_Declare_A_Strategy`
was built precisely so that an undeclared domain would fail a test rather than be noticed,
and it did not fire on either of these. Its predicate is the construction of a
`MaterializedFact`, and neither crate constructs one — they serialize and they project. The
guard against a missing declaration could not see the two declarations that were missing.

## What The Two Rows Now Declare

| Domain | Declaration | Where |
|---|---|---|
| Spec-bundle serialization | `State` / `CrossBinary` / `BitIdentical` | `nomos-spec-bundle::BundleSerialization` |
| Projection engine output | `State` / `CrossPlatform` / `BitIdentical` | `nomos-spec-project::ProjectionOutput` |

Each triple is the row's, unchanged. Neither was lowered, and the reason is worth stating
because lowering is the sanctioned repair and declining it needs an argument.

**The bundle's `CrossBinary` stands.** A bundle is the authority committed to git; the
database is the working copy. `Bundle::Parse` refuses a bundle whose manifest digest, whose
counts, or whose canonical form disagree with what this build computes — so every read of a
committed bundle is already a comparison between what one build wrote and what another
build computes. A recompile that changed the spelling of one field would not slow anything
down; it would turn every committed bundle into a file this build refuses to read. The
claim is expensive because the artefact is, and the honest options were to hold it or to
stop committing bundles.

**The projection's `CrossPlatform` stands, and `CrossBinary` was declined.** A projection is
regenerated from the store by the build that reads it, and `Check` compares a content digest
in a sidecar against the file. A recompile that moved the bytes produces a stale-output diff
somebody regenerates. A bundle has no store behind it to regenerate from. One step of scope
separates those two situations and the table already had it.

## What The Evidence Is, And Whether It Is Gated

This is the part `OD-DETERMINISM-001` insists on, and the answer is different from its own.

**Neither of these two declarations rests on a corpus-gated test.** `nomos-spec-bundle` and
`nomos-spec-project` contain no assertion that reads `NOMOS_V14_CORPUS`,
`NOMOS_SPEC_ARCHIVES` or `NOMOS_RUST_CORPUS`; every fixture either crate has is an in-memory
store built in this repository. The 68 assertions `OD-GATE-001` counts are all elsewhere.
So the sentence that record had to write about the four earlier rows — that their strongest
proofs are inside the hole on every machine the gate runs on — does not apply here. The
suite was run with the three variables set and with none of them set, and the count is the
same either way.

That is not unqualified good news, and reading it as such would be reading it wrong. The
four earlier rows have two instruments each: fixtures here with the reach, and gated tests
with the scale — 7,500 real files, a hundred ingestion orders. **These two rows have only
the first.** What holds them is a corpus of two documents, three nodes, two relations, two
statements and a lineage row, chosen to be read rather than sampled. It is enough to catch
an ordering by a surrogate and a changed encoding. It is not enough to catch a defect that
needs a thousand documents to appear, and nothing in this repository would notice one.
Stated plainly so that the next person does not mistake "not gated" for "proven at scale".

## Repeating A Production Is Not A Test Of A Serializer

`Verify` repeats a production and compares the results at the declared strength. Over a
production that rebuilds the same input every time, that comparison tests whether the code
is a pure function of its input — which, for a serializer, it visibly is. The repetition
would agree even if every ordering in the exporter were `ORDER BY uid`.

So both spec productions **alternate the insertion order of an otherwise identical corpus**
between repetitions. Insertion order is the only thing that decides a row's surrogate key,
and a surrogate is the one value in either domain that is not a function of the content.
That turns the repetition into a comparison of two stores of one corpus, which is what the
declaration actually claims: a bundle is a function of the specification, not of the order
the specification arrived in. The first call is always the forward order, so the digest the
child process and the committed golden are compared against does not depend on how many
times the closure has been called.

This mattered in practice, and the experiment is in the controls below: with the exporter
ordering nodes by `uid`, a fixed-order production passes the repetition check and fails only
the golden — which means that after somebody re-blessed the golden, the defect would have
been invisible forever. The alternating production fails the repetition itself, which is a
failure that cannot be blessed away.

## The Completeness Guard Now Has Three Derivations

The fact-producer predicate is kept and two more are added beside it. None of the three is a
list kept by hand, which is what `OD-COMPLETENESS-001` requires: a derived universe owes
nothing further on that axis, and the cost is paid by deriving.

- **What produces facts** — unchanged. A crate that constructs a `MaterializedFact` and
  declares nothing still fails.
- **What the table says exists.** `Domain_Table` parses the six-row table out of
  `nomos-contracts`'s own module documentation, which is the authority: every peer that
  reimplements those types reads that table, so a row there is a published claim whether or
  not anything here occupies it. A row claiming reproducibility that no declaration matches
  on all three axes fails `Test_Every_Occupied_Row_Of_The_Domain_Table_Should_Be_Declared`.
  The match is on all three axes and not on strength alone, because three rows of the table
  share a strength and differ only in scope — matching on less would report the weaker row
  as occupied by the stronger row's declaration, which is a completeness guard passing
  because it compared too little.
- **What the harness measures.** `Harnessed_Strategies` reads which strategies
  `tests/integration/tests/determinism.rs` registers. A declaration the harness does not
  measure is a promise with no test behind it; a strategy the harness measures that no crate
  declares is a check whose subject was deleted. Both fail
  `Test_Every_Declaration_Should_Be_Held_To_It_By_The_Harness`, in the two directions.

That third one replaced a check asking whether a declaring crate produced facts, with two
crates named beside it as "serving" them instead. That list was about to grow to four. A
list that grows every time the check is right is a list that will one day be wrong, and the
question worth asking was never what a declaring crate does — it is whether anything would
notice if the declaration were false.

`Test_The_Declared_Domains_Should_Be_The_Ones_This_Item_Covered`, which
`OD-DETERMINISM-001` cites as the place the gap was written down, is now
`Test_Every_Domain_In_The_Tree_Should_Declare_And_Be_Registered`. The old name asserts a
sentence that is no longer true, and a citation resolving to a test that asserts the
opposite of what the citing record says is worse than one resolving to nothing.

## The Two Rows With No Domain

Deriving the universe from the table surfaced something the earlier accounting missed.
`OD-DETERMINISM-001` says two of six rows are undeclared and treats the sixth — progress UI,
logs, telemetry — as having no domain in the tree. It is silent about the fifth,
**correction planning and staging**, which also has no domain in the tree: `nomos-rules`
judges and reports, and the step that would apply a fix does not exist.

Both are recorded in `UNOCCUPIED` in `tests/contract/tests/determinism_declarations.rs`,
each with the reason a reader would need to decide whether to close it — the `OD-GATE-001`
remedy rather than a gate that can never be green. The list is mirrored in both directions:
a row named there that the table no longer has fails, and a row named there that something
has since declared fails, because an exception that is no longer needed is an exception that
will excuse the next gap.

So the table's accounting is now: four rows with a domain, all declared; one row whose
domain does not exist; one row that promises nothing by design.

## Controls Confirmed Red

Five, each run and each restored.

- **The bundle's declaration removed.** `Test_Every_Declaration_Should_Be_Held_To_It_By_The_Harness`
  fails: *the harness measures these strategies and no crate declares one:
  `["BundleSerialization"]`*. The row-coverage check does **not** fail, because
  `SnapshotSerialization` still occupies that row — which is exactly why the harness binding
  had to exist as well as the table binding.
- **The projection's declaration removed.** Two tests fail: the harness binding as above,
  and `Test_Every_Occupied_Row_Of_The_Domain_Table_Should_Be_Declared` with *these rows of
  the contracts domain table claim reproducibility and nothing in this workspace declares
  them: `["Projection engine output"]`*.
- **The exporter ordered by a surrogate.** `ORDER BY n.node_id` changed to `ORDER BY n.uid`:
  *bundle-serialization declares State and repetition 1 produced a different set*.
- **The same defect, with a fixed-order production.** The repetition check passes and only
  the golden fails, which is the result that justifies the alternating production and is
  recorded above.
- **A golden that does not match the bytes.** Both new domains were first committed with
  invented digests and both failed by name, at their own declared scope — `CrossBinary` for
  the bundle, `CrossPlatform` for the projection.

Two controls are committed rather than reconstructed, because a harness that only ever runs
over things that behave proves the harness and not the property.
`Test_A_Domain_That_Does_Not_Repeat_Itself_Should_Fail_The_Harness` runs `Verify` over a
production that is not a function of its input and asserts the failure and its message.
`Test_An_Altered_Byte_Should_Move_The_Digest_The_Golden_Pins` alters one byte of a real
bundle and asserts the `State` digest moves — because a `State` digest is taken over the
sorted line set, and a golden insensitive to a changed byte would pin nothing.

## What This Does Not Cover

The `CrossBinary` and `CrossPlatform` claims rest on digests captured on one platform by one
build and committed here. `OD-DETERMINISM-001` already recorded what that is worth for
`CrossPlatform`, and the same bound applies one step further out: the bundle's golden is
compared by every later build of this workspace, which is a real comparison across
recompilation and is not yet a comparison across compiler versions or optimization levels.
It becomes one the first time this gate runs on a second toolchain. Until then it is a claim
checked against one build's answer, and saying so is worth more than implying more.

The bundle's golden also pins the store's schema version, which travels in the header. A
migration moves the constant. That is right rather than unfortunate — a bundle written under
one schema and read under another is the interchange case the claim is about — but it means
the constant will be edited by items that have nothing to do with determinism, and the
message it fails with has to be read rather than skimmed.

## Status

Accepted, closing `P10-SPEC-DETERMINISM`. Six domains declare, each is checked against its
own behaviour by obligations derived from its own triple, the completeness guard reaches
domains that produce no facts, and the two rows with no domain in this tree are counted
rather than assumed.
