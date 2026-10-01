---
id: OD-TRACE-002
type: decision
title: A requirement assessment is one file per requirement, and the corpus comparison does not live in the guard crate
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - traceability
  - requirements
  - corpus
  - completeness
  - verification
relations:
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-SPEC-005
    type: relates-to
---

# A requirement assessment is one file per requirement, and the corpus comparison does not live in the guard crate

## Question

`OD-TRACE-001` decided the shape of a requirement assessment and deliberately built nothing:
it named four verdicts and the three things a guard asserts, and left the registry to
whichever of `P10-AGENT-REQUIRED` and `P10-LEDGER-CORPUS` landed first. `P10-AGENT-REQUIRED`
landed first and could not carry it — its territory was two source files, two surface
snapshots and one record, and widening a territory mid-claim is a failure this repository
has already had twice. So `CHK-003`'s verdict has been **Met in the prose of
`OD-CONTRACTS-002`** and in no machine-readable place, which is the state `OD-TRACE-001`
exists to end.

Two things were left open by that record, and neither is answered by restating it.

The first is grain: what one entry *is* as a file. The second is the one the item's
`done_when` asked as a conditional — "a corpus comparison, if one is added, is corpus-gated
and declared in `corpus_gates.rs`". That conditional turns out to have no true branch, and
the reason is structural rather than a matter of scheduling.

## 1. An assessment is one file, named for the requirement

**`tests/contract/requirements/<REQUIREMENT-ID>.assessment`, one file per requirement, with
the identifier as the stem and nowhere inside the file.**

This is `OD-SPEC-007`'s registration shape, transplanted, and the reason transplants with it.
`OD-SPEC-007` broke `GOVERNING_RECORD_IDS` into one file per record because a single shared
list made every record writer edit one file, which serialized the board. The registry has the
same pressure and more of it: 349 requirements are unassessed, assessing them is inherently
many-hands work spread over many items, and a single `assessments.toml` would make any two
assessors collide by construction on the one file neither of them is really writing.

An entry carries a verdict, the sites the verdict is about, and — when there is one — the
governing record holding the reasoning. The identifier is not a key inside the file, for the
reason `OD-SPEC-007` gives about `id:`: a second place to spell the identity is a second
place for it to be wrong.

Three verdicts are writable. `Unassessed` is **refused as a written word**, because it is
held by the absence of an entry; a file saying `Unassessed` would record that somebody looked
and did not look. That refusal is a unit test rather than a convention.

## 2. A site is `path#symbol`, not a path

`OD-TRACE-001` says the guard "can see that a named site has vanished". A path alone barely
delivers that. Files in this workspace are renamed rarely and the symbols inside them are
renamed constantly, so a path-only site would decay silently in the common case and fire in
the rare one.

So a site names a file and text that must occur in it, and both are checked.
`Test_An_Entry_Naming_A_Vanished_Site_Should_Be_Reported` exercises both halves, and the
second half is the point: a renamed symbol in a file that still exists is what a path-only
check misses.

This does not reach semantic drift, and `OD-TRACE-001` already took that trade explicitly.
It is not re-argued here.

## 3. The assessed set is held by a floor

`FEWEST_ASSESSMENTS`, asserted with `>=`. Adding an assessment costs no edit to the guard,
which is the whole of decision 1; removing one costs lowering the floor in the same commit.

The guarantee and its limit are both `OD-SPEC-007`'s, exactly: the floor is exact only while
the count sits on it, and a deletion inside the slack, once the registry has grown above, is
caught by nothing here. The alternative — an upper bound as well — was refused there for a
reason that applies unchanged: it reintroduces the shared edit on an unpredictable schedule,
so instead of every assessor colliding, one unforeseeable assessor in every N collides with
all the others.

Observed rather than assumed. With `tests/contract/requirements/CAP-002.assessment` moved
aside, `cargo test -p nomos-contract-tests --test requirement_trace --no-fail-fast` reported
`3 requirements are assessed and FEWEST_ASSESSMENTS says at least 4`. Two tests went red
rather than one, because `Test_The_Reader_Should_Enumerate_The_Directory_It_Is_Given`
asserted the floor as well; that duplicate assertion was removed, so one deletion now
reddens one test. The file was restored and the suite returned to green.

## 4. No corpus comparison is added, and one cannot live in this crate

This is the part that is a finding rather than a design, and it was measured in the tree
rather than reasoned about.

A corpus comparison would be corpus-gated by construction — it reads `NOMOS_V14_CORPUS`, and
`OD-GATE-001` requires that such a test be declared in `corpus_gates.rs` so the size of the
hole stays a number somebody chose. Putting one in `tests/contract/tests/requirement_trace.rs`
makes the two derivations behind that table contradict each other.

**Measured on 2026-08-10.** A test reading `NOMOS_V14_CORPUS` was added to
`requirement_trace.rs` and `cargo test -p nomos-contract-tests --test corpus_gates
--no-fail-fast` run twice:

| tree | failing test | what it said |
|---|---|---|
| probe added, `GATES` untouched | `Test_Every_File_That_Reaches_A_Corpus_Should_Be_Declared` | `these test files reach a corpus and are not in GATES: [tests/contract/tests/requirement_trace.rs]` |
| probe added, `GATES` row added (`gated: 1, tests: 12`), `GATED_TOTAL` 68 → 69, `TESTS_IN_GATED_FILES` 130 → 142 | `Test_Every_Declared_Gate_Count_Should_Be_The_One_In_The_Source` | `tests/contract/tests/requirement_trace.rs: declares 1 gated, source has 0` |

The first test demands the row. The second refuses every value the row can take. There is no
consistent declaration, and the cause is that the two derivations do not scan the same set:

- `Files_Naming_A_Corpus` in `corpus_gates.rs` walks **every** workspace member's `tests/`
  directory, `nomos-contract-tests` included.
- `Corpus_Gates` in `tests/contract/src/gates.rs` **skips the member named
  `nomos-contract-tests`**, because the crate doing the counting names the three variables in
  order to count them and would otherwise report its own inventory as a gate.

That skip is right and stays. What it means is narrower than it looks and worth stating
plainly: **a corpus-gated test cannot be declared in the crate that declares the corpus
gates.** The probe and both `GATES` edits were reverted; nothing of this measurement is
committed except this table.

### The workaround that must not be taken

There is an arrangement that makes both tests green, and it is strictly worse than the red.
Put the corpus lookup in a helper under `tests/contract/src/` and have the test call it.
`Files_Naming_A_Corpus` then does not find the test, because it scans only `tests/`
directories for a literal variable name; `Corpus_Gates` does not find it either, because it
skips the crate. The gate becomes invisible to **both** derivations, prints `ok` on every
machine, and is counted by nobody.

That is `OD-GATE-001`'s defect reconstructed inside the file written to prevent it, and it is
exactly what `corpus_gates.rs` warns about in its own words — "an exemption is the easiest
place in a check to hide something". The skip in `gates.rs` is such an exemption, and this is
the first thing it has been asked to hide.

### So the comparison is not owed here

`OD-TRACE-001` already draws the line this lands on: "the corpus is required to *author* an
entry and to *re-verify* one. It must not be required to *run the guard*." Re-verification is
an authoring act performed by somebody holding the corpus, and its output is a committed
entry — not a test result. The guard checks what is committed, which is what it does today.

If a mechanical corpus comparison is ever wanted, it lives **outside `nomos-contract-tests`**
— `tests/integration` is the obvious home — and it is declared in `corpus_gates.rs` from
there, where both derivations can see it. That is a separate item and is not widened into
this one.

## What Is Assessed Today

Four entries, and every one of them is `Met`. Three are `OD-TRACE-001`'s own hand audit
against `NOMOS_V14_CORPUS` — `EVID-001`, `CAP-002` and `CAP-003`, each found satisfied
exactly. The fourth is `CHK-003`, which that audit found diverging by one element and
`OD-CONTRACTS-002` then closed.

They are authored from those two records rather than re-read from a corpus, because no corpus
is configured in the tree this item was worked in. That is the arrangement working as
designed rather than a shortcut: a record is canonical, and an assessment committed here is a
declared entry whose authority is the audit that produced it.

**The two known divergences are deliberately absent.** `WORK-LEDGER-005`'s `Blocker` drops
`StaleProbeArtifact` and `WORK-LEDGER-001`'s `LedgerItem` drops `priority`, and neither has a
record saying why. A `Diverges` entry with no record is refused by the reader, which is
`OD-TRACE-001`'s rule enforced rather than restated — so those two cannot be entered until
`P10-LEDGER-CORPUS` writes the reasons. Entering them without records would have been the
one thing this registry must not be able to express.

The consequence is stated rather than hidden:
`Test_Every_Divergence_Should_Name_A_Governing_Record` is **vacuous over the committed set
today**, because every entry is `Met`. It is kept honest by
`Test_A_Divergence_With_No_Record_Should_Be_Refused`, which runs the same
`Divergences_With_No_Record` function — not a copy of it — over a synthetic entry, in the
manner `OD-SPEC-005` and `OD-SPEC-007` recorded their controls.

## What This Does Not Do

- **It assesses 4 of 363 requirements.** `OD-TRACE-001` chose a floor over a count precisely
  so that this is a readable state rather than a failure, and any scheme requiring all 363
  answered before any of them is answered will not be adopted.
- **It does not close the two known divergences.** They are `P10-LEDGER-CORPUS`'s subject.
- **It adds no surface.** The reader and every predicate live in the test binary, so
  `tests/contract/surface/nomos-contract-tests.txt` is unchanged and this item reserved no
  snapshot.
- **It does not make an entry self-verifying.** A `Met` entry whose site still exists and
  whose code stopped satisfying the requirement reads exactly like one that did not — the
  semantic-drift limit `OD-TRACE-001` states, unchanged.

## Status

Closed by `P10-TRACE-REGISTRY`.
