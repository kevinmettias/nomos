---
id: OD-COMPLETENESS-002
type: decision
title: A universe claims its mirror in one place, and the classification table is a checked copy
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - verification
  - completeness
  - enforcement
relations:
  - target: OD-COMPLETENESS-001
    type: affects
  - target: D-134
    type: affects
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
---

# A universe claims its mirror in one place, and the classification table is a checked copy

## Question

Two guards in this workspace answered the same question about the same universe and gave
different answers, and neither of them could see the other.

`nomos check` reads a universe's mirror off the doc comment at its declaration site and
resolves the name against the check index built from syntax facts. `DECLARED_RULES` in
`nomos-spec-validate/src/run.rs` named none, so the command reported it as an admitted gap —
Advisory, and one of thirteen. The `UNIVERSES` table in
`tests/contract/tests/completeness_universes.rs` classified the same constant
`Standing::Mirrored`, by `Test_A_Rule_Nobody_Declared_Should_Fail_The_Run`, and declared
`UNMIRRORED_TOTAL` at twelve.

The arithmetic isolated it exactly. Sixteen declared universes; the table called four mirrored
and twelve not; the command found thirteen with no mirror; the set difference was one element
and it was `DECLARED_RULES`. `mirror.rs`'s own module documentation said thirteen, twice, so
the rule and its prose already disagreed with the table before anybody looked.

Every one of those counts was measured again before this record was written and every one
held. What did not hold was the diagnosis.

## The Item Was Opened Against A Premise That Turned Out To Be Wrong

`P10-MIRROR-DISAGREEMENT` says, of `DECLARED_RULES`, that nothing compares it against the
rules the crate implements and that "nothing else does either". That sentence is the one the
item turns on, and it is false.

`crates/spec/nomos-spec-validate/tests/preservation_holds.rs` opens with
`Test_The_Registry_Should_Match_The_Manifest`. It calls `Validate` with the real
`Registered()` and asserts that `unregistered` and `undeclared` are both empty and that as
many rules ran as the manifest declares. That is `DECLARED_RULES` reconciled in both
directions against the identifiers the rule objects the crate actually builds return from
their own `Id()` implementations. It has been green since it was written, it fails on demand
in both directions, and no record, table or doc comment mentioned it.

The item's reading of the test the table *did* name is correct.
`Test_A_Rule_Nobody_Declared_Should_Fail_The_Run` builds a synthetic `Fake` carrying the
invented identifier `NSV-INVENTED-001` and asserts that `Validate` reports it undeclared. It
exercises the reconciliation mechanism against input the test made up; it never touches
`Registered()` and it compares `DECLARED_RULES` against nothing at all. A row citing it reads
as coverage and demonstrates a mechanism.

So the disagreement was never between a covered universe and an uncovered one. It was between
two places where a mirror could be claimed — one pointing at the wrong test, the other
pointing at nothing — with the real mirror sitting in a third file, cited by neither.

This is written down rather than quietly worked around because the remedy the item asks for in
its first branch is *writing* a check comparing `DECLARED_RULES` against the rules the crate
implements. Writing one would have put a second reconciliation beside a canonical one: two
guards for one rule, which is the shape this item exists to remove, arriving as its own fix.

## Why This Is Not A Comparison That Cannot Fail

`OD-COMPLETENESS-001`'s second corollary is what to hold this against: a comparison whose two
sides come from the same source cannot fail, and `GOVERNING_RECORD_IDS` against a store seeded
from `GOVERNING_RECORD_IDS` reported success for as long as it existed.

The two sides here are not the same source. `DECLARED_RULES` is four string literals in
`run.rs`. The other side is four `impl Rule` blocks in `preserve.rs`, each returning its own
identifier from its own `Id()`, gathered by a hand-written `Registered()`. Every identifier is
authored twice, in two files, by two separate acts, and neither list is generated from the
other. The comparison can fail, and it was made to fail in both directions before this record
was written.

Two derivations are named and refused, because either would produce exactly the defect the
rule exists to catch. Deriving `DECLARED_RULES` from `Registered()` makes `Validate`'s
reconciliation compare a list against itself and pass having checked nothing. Deriving
`Registered()` from `DECLARED_RULES` by looking implementations up by identifier does the same
thing facing the other way. The duplication between the two lists is not the defect; it is
what the guard is made of.

## The Decision

**A universe's mirror is claimed in one place — the doc comment at the declaration site,
where `nomos check` can read it — and any other statement of that claim is a copy that a test
holds equal to it.**

Three parts.

**`DECLARED_RULES` names the mirror it already had.** Its doc comment now says
``Mirrored by `Test_The_Registry_Should_Match_The_Manifest` ``, and says in the same breath
what that reaches and what it does not. `nomos check` resolves it, and the workspace's count
of unmirrored universes falls from thirteen to twelve, which is where the table already was.
`UNMIRRORED_TOTAL` does not move.

The alternative — raising `UNMIRRORED_TOTAL` to thirteen and calling `DECLARED_RULES` a hole
— was refused for a reason rather than for taste. That figure is a debt list, and a register
that over-reports is as useless as one that under-reports. Declaring a hole where a real
two-directional reconciliation runs on every test would make the honest-declaration machinery
say something false in the other direction, and would leave the only real mirror for that
universe still cited by nothing.

**The table's `by:` field is demoted from a claim to a copy.**
`Test_The_Scan_And_The_Table_Should_Name_The_Same_Mirror` reads the mirror each universe
claims at its site, reads the mirror each row claims, and fails when any universe's two claims
differ — naming the universe and printing both sides. `D-134` put the claim at the site
precisely so that meaning would stay with the author and only one thing would need resolving;
a second spelling in a file the command cannot read is what let these two drift apart for the
length of `P10-FIRST-CHECK`.

It compares *claims* and not verdicts. Whether a claim resolves is
`Test_Every_Named_Mirror_Should_Exist_In_The_Source`'s fact; folding the two together would
make a phantom agree with a row that called the universe unmirrored, which is the
false-coverage reading `mirror.rs`'s severity ordering exists to refuse.

It names rather than counts, and that is not a stylistic preference. Across the whole
disagreement this record is about, the table's own totals were four and twelve throughout; the
row that was wrong was wrong about *which test*, not about *how many*. A check comparing two
numbers would have been green the entire time. Two rows swapping standing has the same
property, and is one of the controls below.

**The mirror is held to being one.** `Test_The_Registry_Should_Match_The_Manifest` gains the
guard it lacked. Two empty lists reconcile perfectly, and a test that is now a declared mirror
must not be able to report clean having compared nothing.

## What Was Confirmed Red

Each control below was applied to the tree, watched failing, and reverted. Controls that could
not be exercised from this item's territory are not described.

**The table's classification restored.** Putting `Test_A_Rule_Nobody_Declared_Should_Fail_The_Run`
back in the `by:` field, with the site claim in place, reddens
`Test_The_Scan_And_The_Table_Should_Name_The_Same_Mirror` and nothing else — five of six tests
in the file stay green, because every total is identical before and after:

```
DECLARED_RULES (crates/spec/nomos-spec-validate/src/run.rs): the site claims
Some("Test_The_Registry_Should_Match_The_Manifest") and the table claims
Some("Test_A_Rule_Nobody_Declared_Should_Fail_The_Run")
```

**The original disagreement, whole.** The same edit plus deleting the site's `Mirrored by`
paragraph reproduces the state this item was opened against, and it is caught:
`the site claims None and the table claims Some("Test_A_Rule_Nobody_Declared_Should_Fail_The_Run")`.

**Two rows swapping standing.** `MIGRATIONS` made mirrored by a function that exists and
`DECLARED_RULES` made unmirrored keeps mirrored at four, unmirrored at twelve, and
`UNMIRRORED_TOTAL` at twelve. Exactly one test goes red, naming both universes and both sides
of each. This is the control for "two totals that happen to match conceal which row moved".

**The comparison cannot shrink.** Deleting one `UNIVERSES` row leaves fifteen of the sixteen
scanned universes with a row, and the count assertion fires:
`1 of 16 scanned universes have no row here, so this test compared less than the whole set`.
Without it, deferring membership to `Test_The_Declared_Table_Should_Match_What_Is_Derived`
would let this test pass over a shrinking subset.

**The mirror in both directions.** A fifth rule added to `Registered()` and left out of
`DECLARED_RULES` reddens `Test_The_Registry_Should_Match_The_Manifest` with
`built but not declared: ["NSV-CONTROL-999"]`; the same identifier added to `DECLARED_RULES`
alone reddens it with `declared but not built: ["NSV-CONTROL-999"]`. Both drag the tests that
consult `Passed()` red with them, which is correct and not a mis-cut control.

**The vacuity hole was real.** With `DECLARED_RULES` emptied to `&[]` and `Registered()` to
`vec![]`, the test as it stood **passed** — three assertions over two empty lists. With the
guard added it fails with `an empty manifest reconciles against anything`. Both states were
observed, in that order.

**The phantom, and what it actually does over this tree.** Renaming the site's mirror by one
letter — `…Manifesto` — reddens `Test_The_Scan_And_The_Table_Should_Name_The_Same_Mirror`
naming `DECLARED_RULES`, while `Test_Every_Named_Mirror_Should_Exist_In_The_Source` stays
green, because that test checks the table's `by:` and the table was untouched. That asymmetry
is the argument for checking the site claim separately.

It does **not** make `nomos check` block, and the draft of this record predicted that it
would. Measured: the command reports

```
[Advisory] completeness-mirror: DECLARED_RULES (`Test_The_Registry_Should_Match_The_Manifesto`
resolves to no check, so this rule is declared enforced and never runs — and the check index
is incomplete, because 1 subject(s) could not be read, so this rule cannot tell a false claim
of coverage from a name it did not get to look for)
```

and exits 0. The unread subject is `tests/corpus/analysis/gamma/broken.rs`, a fixture that is
deliberately unparseable. `OD-RULES-001` decided that a claim which fails to resolve against a
short index is not established as false, so while that file is in the tree **no
completeness-mirror phantom anywhere in this workspace can fail a build** — the severity
ordering `D-134` set is exercised by `nomos-cli`'s own tests over a whole index, and is
unreachable over the real tree. That is a true statement about the tree today, recorded rather
than papered over; changing it is not this item's subject.

## The Arithmetic, Measured

Taken on 2026-08-09 against this workspace at the commit this record lands on, by running the
command and by re-deriving the same sets independently.

| | before | after |
|---|---|---|
| declared universes | 16 | 16 |
| claim a mirror that resolves | 3 | 4 |
| claim a mirror that does not resolve | 0 | 0 |
| claim nothing | 13 | 12 |
| `UNMIRRORED_TOTAL` | 12 | 12 |
| `nomos check` findings / blocking | 15 / 0 | 14 / 0 |

`nomos check` examines 191 files and has a syntax fact for 190 of them, before and after. The
two findings that are not completeness advisories are both about
`tests/corpus/analysis/gamma/broken.rs`: one from the text side that could not parse it, one
from the fact side that had no fact for it.

Two prose figures were wrong and are corrected here rather than carried forward. `mirror.rs`
said thirteen unmirrored universes in two places and `nomos-cli` said thirteen in two more;
all four now say twelve. `universe.rs` said that widening to private lists takes the workspace
"from thirteen unmirrored universes to thirty-four". Measured with the `Is_Public` narrowing
removed: 44 universes of which 41 had no resolving mirror before this change and 40 have none
after. Whether thirty-four was wrong when written or has drifted is not established; the
sentence now carries the measurement and its date.

## Coordination — One Method, Not Two

`OD-LEDGER-007` §"Why The Structural Remedy Is Not Applied Here" declined to derive
`GOVERNING_RECORD_IDS` from `docs/records` and asked, by name, that that question and this
one be settled together: *"Replacing a universe with a derivation is `OD-COMPLETENESS-001`'s
subject and overlaps `P10-MIRROR-DISAGREEMENT`, which is open and holds the same question
about `DECLARED_RULES`. Two items deriving two universes by two methods is how they come to
disagree, so it wants coordinating rather than doing twice."*

`P10-SEED-GRAIN` settled the other one, and `OD-SPEC-007` records it: a record becomes
governing by having its own registration file, one per record, and the guard's two sides stay
independently authored because the generator reads that directory and never `docs/records`.

**The answer here is the same method.** Neither universe is replaced by a derivation. Each
keeps its hand-authored declaration at its own site, and each is mirrored against a set of
per-item artifacts authored independently of that declaration — one registration file per
record for `GOVERNING_RECORD_IDS`, one `impl Rule` block per rule for `DECLARED_RULES`.
Neither guard can go vacuous, because in neither case is one side generated from the other,
and both now say so at the site rather than in a table somewhere else.

The difference is scope, and it is worth naming. `OD-SPEC-007` had to *build* the independent
side, because it did not exist as separate artifacts. This item had to build nothing: the
independent side was already there and already compared, and the whole of the work was moving
the claim to where the rule that judges claims can read it. Same rule, cheaper instance.

No second scanner was introduced. This record adds no recognition mechanism at all.
`tests/contract/src/universes.rs` records what happens otherwise — the recognition existed in
two crates for about an hour, and that hour is why it is shared now.

## What This Does Not Do

Stated plainly, because `OD-GATE-001` is about checks that imply more than they hold.

- **It does not close the twelve remaining holes.** The number is unchanged. What changed is
  that both guards now report the same one.
- **The mirror reaches the registry, not the implementations.** This is the boundary of the
  guarantee and it is a boundary, not a deferral. A fifth `impl Rule` written in `preserve.rs`
  and never added to `Registered()` is in neither list, never runs, and is reported by
  nothing. That is a completeness defect of `Registered()`, a different universe — and one the
  scanner cannot see at all, because `Read_Universes` recognises a `pub const` of slice type
  and an inherent `All()`, not a function returning `Vec<Box<dyn Rule>>`. The list that
  enumerates the real thing is not judged by the rule that judges lists. Closing it means
  widening what the whole workspace counts, which is a change to every figure in this record.
  The claim at the declaration site is worded so that it promises the registry and nothing
  more.
- **It does not make the table derivable.** The `risk` text on an unmirrored row is a human
  judgment about what would go wrong and there is nothing to derive it from. What is now
  checked rather than trusted is the `by:` field.
- **The two scans are still two walks.** `Declared_Universes()` visits workspace members'
  `src` and `tests` — 182 files; `nomos check` visits every `.rs` under the root but `target`
  and `.git` — 191. They see the same sixteen universes today, measured in both directions
  with an empty difference each way, because the nine files only the second visits declare
  none. A universe declared outside a member's `src` or `tests` would be counted by the
  command and invisible to the table, and no test would be red. The agreement test is built on
  the same walk the table is checked against, so it compares like with like; the exposure is
  named here rather than guarded, because closing it is the same scanner question as the
  paragraph above.
- **A phantom cannot fail this build today.** See the last control: one unparseable fixture
  keeps the check index short, and a short index downgrades every phantom to advisory by
  design.
- **`nomos check` is still not wired into the gate.** Unchanged by this record.

## Status

Accepted, closed by `P10-MIRROR-DISAGREEMENT`. Sixteen declared universes; four mirrored, each
claiming its mirror at its own site; twelve declared holes with the number checked; and the
two guards that disagreed now hold a per-universe equality that names what moved.
