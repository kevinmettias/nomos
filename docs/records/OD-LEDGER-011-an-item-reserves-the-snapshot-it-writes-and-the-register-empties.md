---
id: OD-LEDGER-011
type: decision
title: An item reserves the snapshot file it writes, and the debt register empties
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-007
    type: affects
  - target: OD-LEDGER-001
    type: affects
  - target: OD-SPEC-007
    type: relates-to
  - target: OD-LEDGER-004
    type: relates-to
---

# An item reserves the snapshot file it writes, and the debt register empties

## Question

`OD-LEDGER-007` opened a debt register with two entries, said in terms that it "is expected
to empty", and declined to empty it. `OD-SPEC-007` removed the code coupling behind the
first entry: a canonical record is now registered by its own
`crates/spec/nomos-spec-store/records/<ID>.record` file, `governing.rs` is not edited, and
two items writing two different records write two different files. It could not remove the
entry, because the register reads *declared territory* rather than edited files, and twelve
items still reserved the whole crate — authored under `OD-LEDGER-001`'s third rule as it
stood before that record.

The second entry is the same shape one level up. `P9-PUBLIC-API` checks each crate's public
API into `tests/contract/surface/<crate>.txt`, and an item that widens one crate's API
reserved `tests/contract` entire — twenty-one crates' surfaces plus the whole harness — in
order to write one file inside it. That is `OD-LEDGER-004`'s finding about `docs/records`,
arriving independently and later, and nothing in the mechanism ever prevented the finer
grain: territory is file resolution and `Territory::Intersect` has always compared by
containment.

So this item is a re-authoring pass, not a code change. What made it an item rather than an
edit is that the territories belong to other work, and `P10-SEEDING-SERIALIZES` refused to
narrow somebody else's territory to manufacture a pair — rightly, for that item.

## What Was Measured, And Under Which Projection

Every figure below is under the **record-writer projection**, which is what
`Record_Writers` in `crates/substrate/nomos-ledger/tests/records_do_not_serialize.rs`
defines: an item that is open, reserves a path under `docs/records`, and has an empty
`depends_on`. Thirteen of the board's items are in it. A parallelism number without its
projection is an unqualified claim, and `OD-SPEC-007` states its own floor the same way.

Two further projections apply and are named because they change the answer:

* **Claims cleared.** `Unclaimed_Copy` sets every open item back to `Ready` before
  measuring, because the question is whether two items *could* be held at once, not whether
  they happen to be free this second.
* **Constant membership.** `P10-TEST-MASK` finished during this pass. It is excluded from
  all three points below, so that a change in the board's membership is not read as a change
  bought by the re-authoring.

| | pairs | blocked | blocked only by a declared serializer | most holdable at once |
|---|---|---|---|---|
| A — territories as authored, at `ed8c167` | 78 | **78** | 42 | **1** of 13 |
| B — after the narrowings | 78 | **36** | **0** | **6** of 13 |
| C — after the widen below, what this commit leaves | 78 | **42** | **0** | **5** of 13 |

Point A is the census the item was opened against and is the line
`Test_A_Run_Should_Report_Whether_The_Board_Is_Parallel` printed on arrival: *13 items, 78
pair(s), 78 blocked, 42 of those only by a declared serializer*. The **records-only
projection** — `Only_Records`, the one `OD-LEDGER-007` restated its acceptance property over
— is 0 blocked pairs at all three points and is not what moved.

## The Decision

**An item reserves the artefact it will write. A snapshot file is an artefact; the snapshot
directory is not. A record's registration file is an artefact; the crate holding it is
not.**

Twelve unclaimed items were re-authored. Each verdict was re-derived from that item's own
`done_when` against the code at `HEAD` rather than carried over from an earlier pass, which
mattered: the board moved three times while the table was being computed.

**Eight items gave up `tests/contract`** — every open item that reserved the bare directory.
Seven narrowed to the snapshot files they will write — `surface/nomos-ledger.txt`,
`surface/nomos-contracts.txt`, `surface/nomos-rules.txt`, and so on. One,
`P10-ADD-PROMISE`, gave it up outright: the only
crate it changes is `nomos-cli`, which is the sole entry in `WITHOUT_A_LIBRARY` and has no
snapshot at all, and it adds no test to a corpus-gated file. Reserving the directory bought
it nothing and cost everybody else.

**Twelve items gave up `crates/spec/nomos-spec-store`**, narrowing to
`records/<ID>.record`. This is the larger half and it did not exist as an option before
`OD-SPEC-007` landed: until then, seeding a record was an edit to two shared files in that
crate and the reservation was honest. It is the dividend of that record, collected here.

**Two items kept a directory, and one that is not a snapshot directory.**
`P10-SPEC-DETERMINISM` genuinely changes the harness — its `done_when` extends the
completeness guard to reach domains that do not construct facts, and that scanner is
`Fact_Domains` in `tests/contract/src/strategies.rs` — so it reserves that file, `src/lib.rs`
where a new export is declared, `tests/determinism_declarations.rs`, and four snapshots. It
does not reserve `tests/contract/src` whole: reading a helper is not writing it, and
`gates.rs` was under a live claim while this ran. The point is accuracy and not a smaller
number, and it is also not a larger one.

**Three under-declarations were repaired**, which is the same defect wearing the other face
and costs concurrency rather than buying it:

* `P10-PATTERN-BRICK` gains `tests/contract/surface/nomos-ledger.txt`. One of the two
  outcomes its `done_when` offers removes `Territory::With_Pattern`, which is exported.
* `P10-REFUSAL-SURFACE` gains `docs/records/OD-LEDGER-001-…md`. Its own `done_when` says
  that if `Conflicts` is removed "the surface snapshot and OD-LEDGER-001 move with it". The
  snapshot was declared; the record was not.
* `P10-RECORD-STEM` gains `work/ledger.json`. Its `done_when` re-authors the open items that
  amend an existing record, which is an edit to that file — the same defect at the file
  every item on the board is edited in that this item's own `why` names. Nothing had ever
  declared it while re-authoring it; this item is the second to do so.

**Only unclaimed items were re-authored.** Two were held while this ran — `P10-TEST-MASK`,
by another session, and this item — and neither was touched. The applying script refuses any
item whose `claim` is non-null and was tested against a board with one held; it also refuses
if an item's territory is neither the recorded before nor the after, which fired repeatedly
and correctly as the board moved underneath it.

This item did not narrow its own territory while holding it either. It reserved
`crates/spec/nomos-spec-store` entire, which stopped being true the minute `OD-SPEC-007`
landed and was blocking `P10-TEST-MASK` over a file this item never touched. It **released
itself unstarted**, was re-authored on the board like everything else, and was claimed again
— the abandonment and its reason are on the item. Narrowing belongs before a claim rather
than during one, and an item whose subject is that rule cannot be the exception to it.

## One Widen, Reported Rather Than Netted Out

`P10-FACT-BYPASS` was **widened** in the same pass, on the lead's instruction, to add
`crates/host/nomos-cli`. Its `done_when` settles whether a rule consumes facts, and the one
rule signature and its only caller — `crates/host/nomos-cli/src/check.rs` — have to move in
the same commit; no ordering of two items avoids it. Its predicate also omitted
`-p nomos-spec-store` while the item seeds a record, and omitted `--no-fail-fast`, which
undercounts failures. Widening while an item is `Ready` and unclaimed is the sanctioned
order — `4365af6` is the precedent — and once anybody claims it, it is not.

It costs six blocked pairs and one concurrent slot: that is the whole of the difference
between rows B and C above. It is reported as its own row rather than folded into the
narrowings, because a pass that reported only its improvements would be measuring itself.

## The Defect Found While Closing

`Test_Every_Declared_Serializer_Should_Still_Serialize` counted with `Paths_Collide`, which
is **symmetric** containment. `tests/contract/surface/nomos-ledger.txt` therefore counted as
reserving `tests/contract`. Measured on this board, under the record-writer projection:

| entry | symmetric, before | symmetric, after | ancestor-or-equal, before | ancestor-or-equal, after |
|---|---|---|---|---|
| `crates/spec/nomos-spec-store` | 13 | **13** | 12 | **0** |
| `tests/contract` | 9 | **9** | 8 | **0** |

The symmetric column does not move. That is worse than a wrong number: under symmetric
counting **the register can never empty**, because narrowing a reservation never reduces the
count and only deleting the item does — and `OD-LEDGER-007` promises it will empty. The
one entry this item exists to remove was, as written, unremovable by the work that earns it.

The repair is one line and it is a direction, not a new comparison: count a writer only when
it reserves the declared path **or an ancestor of it**. `Covers` takes that direction from
the collision rather than deciding containment a second time beside `Territory::Intersect` —
if two paths collide under this ledger's rule then one contains the other, and the shorter
normalized spelling is the container, which is the tie-break `Shared_Paths` already used to
name the broader of two paths.

This is recorded as a defect found while closing rather than as a silent adjustment, because
a guard edited in the same commit as the thing it is supposed to judge is exactly the shape
that wants writing down. Two controls hold it:

* With both entries restored and the direction fixed, the test is **red** and names both.
  The removals are earned.
* With both entries restored and symmetric counting restored, the test is **green** on a
  fully re-authored board where nothing reserves either path. That is the defect, observed.

A related finding, smaller: `tests/contract` was never a *universal* reservation even under
the old authoring. `P10-LOCK-BYPASS` reserved nothing beneath it, so
`Test_Every_Universal_Reservation_Should_Be_Declared` would not have required the entry. It
was held in the register by the staleness test's symmetric counting alone.

The same symmetric comparison appears in `Undeclared_Serializers` and is deliberately left
there. Its failure mode is the opposite one — it over-reports, and a guard whose instruction
is "declare this or remove the coupling" errs safely by over-reporting. Changing two
directions in one commit with a control for only one of them is not a thing this file should
do.

## What The Guard Proves, Rather Than The Author Asserting It

`Test_Two_Items_Widening_Different_Crates_Should_Be_Held_At_Once` derives a pair of open
record writers that reserve *different* crates' snapshot files and are otherwise
territorially independent, and claims both through the real ledger against a copy of the
real board. Derived and not named: two identifiers written into a test would be right until
one of them finished, which is precisely how `OD-LEDGER-007`'s predecessor died.

Stated over the whole territory rather than over a projection, unlike
`Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer`. That test has to project because
two record writers genuinely do share code; here the pair is required to be independent
outright, because a snapshot grain that works only once the rest is ignored buys nobody a
concurrent claim.

Its control, `Test_Restoring_The_Snapshot_Directory_Should_Refuse_The_Pair`, puts one of the
pair back on the bare `tests/contract` and asserts the other is refused **by name**. Both
were also confirmed red against the pre-re-authoring board: with the empty register and the
old territories, the acceptance test fails reporting that no such pair exists and
`Test_Every_Universal_Reservation_Should_Be_Declared` fails naming
`crates/spec/nomos-spec-store`. Removing a register entry early reddens the universal test;
removing it late is what the staleness test is for; and the only commit in which both are
green is the one that does the re-authoring and the removal together.

## The Pair, Through The Real Command

Both boards are the repository's own `work/ledger.json` with claims cleared, driven through
a freshly built `nomos.exe` under `NOMOS_WORK_DIR` — the real command against a copy,
because a suite with side effects on the board it measures is not a suite.

Territories as authored at `ed8c167`:

```
$ nomos work claim --item P10-SPEC-DETERMINISM   --holder agent-alpha
P10-SPEC-DETERMINISM held by agent-alpha until unix 1786344053     (exit 0)
$ nomos work claim --item P10-MIRROR-DISAGREEMENT --holder agent-beta
refused: P10-SPEC-DETERMINISM overlaps territory held by agent-alpha until unix 1786344053
                                                                   (exit 3)
```

The same two items, this commit's territories:

```
$ nomos work claim --item P10-SPEC-DETERMINISM   --holder agent-alpha
P10-SPEC-DETERMINISM held by agent-alpha until unix 1786344053     (exit 0)
$ nomos work claim --item P10-MIRROR-DISAGREEMENT --holder agent-beta
P10-MIRROR-DISAGREEMENT held by agent-beta until unix 1786344053   (exit 0)
$ nomos work validate
ledger is valid                                                    (exit 0)
```

Two items writing two different records, widening two different crates' public APIs, held at
once. It is not a pair only. The whole independent set claims:

```
$ nomos work claim --item P10-PACKAGE-SEAM --holder agent-gamma    (exit 0)
$ nomos work claim --item P10-LOCK-BYPASS  --holder agent-delta    (exit 0)
$ nomos work claim --item P10-RECORD-STEM  --holder agent-epsilon  (exit 0)
$ nomos work validate
ledger is valid                                                    (exit 0)
$ nomos work list --state claimed
P10-SPEC-DETERMINISM    claimed  [agent-alpha]
P10-MIRROR-DISAGREEMENT claimed  [agent-beta]
P10-PACKAGE-SEAM        claimed  [agent-gamma]
P10-LOCK-BYPASS         claimed  [agent-delta]
P10-RECORD-STEM         claimed  [agent-epsilon]
```

Five holders at once where the same board took one. On the old territories the same sequence
leaves one item claimed and thirteen reported `held`.

## What The Census Cannot See

`Record_Writers` excludes any item with a non-empty `depends_on`, and it is right to: an
unmet dependency refuses a claim for reasons that have nothing to do with territory, and
every assertion in that file would fail for the wrong reason. The consequence is that **a
dependent record writer's serialization is invisible to the guard that exists to measure
it.** A dependent item can reserve `tests/contract` entire, serialize every claim on the
board against it, and appear in none of the numbers above — the pair count, the register's
staleness, and the search for an undeclared universal reservation all look past it.

`P9-AUTHORING` was exactly that while it was open, and no figure in `OD-LEDGER-007` or in
this record ever counted it.

This is stated and not fixed here. Fixing it means deciding what the census should say about
an item that cannot be claimed today for a reason other than territory, and that is a
different question from the one this item settles. It is also not repairable by widening this
item's territory, since the exclusion is a line in a test file this item already holds and
the difficulty is the meaning rather than the reach.

## What This Costs

The debt register is empty. `KNOWN_SERIALIZERS` is `&[]`, which makes
`Test_Every_Declared_Serializer_Should_Still_Serialize` vacuous — it iterates nothing — and
that is the intended terminal state rather than a hole. The register is the *declared* half
of the pair `OD-LEDGER-007` set up; the derived half,
`Test_Every_Universal_Reservation_Should_Be_Declared`, is unchanged, still runs against the
real board, still has its own control, and is what fails when a third structural serializer
arrives. A declaration that makes growth a decision has to be able to start from nothing.

Forty-two of the seventy-eight pairs are still blocked, and every one of them is now ordinary
contention: two items that want the same crate, which is what territory is for and which
resolves when one of them finishes. Nought are blocked by a rule that will not resolve. That
distinction is the whole of what `OD-LEDGER-007` built the register to make, and this is the
first run on which the second number is zero.

Those seventy-eight pairs count this item, which is a record writer while it is open. It
finishes in this commit, so the line the next run prints is *12 items, 66 pair(s), 36
blocked, 0 of those only by a declared serializer* — the same board, one writer fewer. Both
figures are under the record-writer projection; neither is comparable to a run on a board
with different membership, which is why the table above holds membership constant.

What is not bought: the finer grain is a property of what the items *say*, and nothing stops
the next item from being authored at `tests/contract` again. `OD-LEDGER-001`'s third rule is
what would have to change to prevent it, and the two guards here are what would notice.

## Status

Closed by `P10-SURFACE-GRAIN`.
