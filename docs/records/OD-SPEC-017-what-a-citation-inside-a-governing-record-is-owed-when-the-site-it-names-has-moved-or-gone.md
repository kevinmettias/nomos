---
id: OD-SPEC-017
type: decision
title: What a citation inside a governing record is owed when the site it names has moved or gone
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - spec
  - records
  - completeness
relations:
  - target: D-134
    type: relates-to
  - target: OD-COMPLETENESS-002
    type: relates-to
  - target: OD-AGENT-001
    type: relates-to
---

# What a citation inside a governing record is owed when the site it names has moved or gone

## Question

`AGENTS.md` sends every agent to `docs/records/` for why something was decided, and those
records cite the tree constantly — a test by name, a file by path, a symbol. Nothing resolves
any of it. A record may name a test that no longer exists and a file that was deleted, and
the only way to find out is to go looking.

The same claim in a code doc comment is already resolved. `checks/mirror.rs` reads a declared
mirror out of a doc comment and resolves it against real parsed test functions, and `D-134`
ranks the failure deliberately: a name that resolves to nothing is `Blocking` and fails the
command, while a universe admitting it has no mirror is only `Advisory`. The asymmetry is
stated there — a phantom is worse than declaring no enforcer at all, because a false claim of
coverage is worse than a hole somebody wrote down.

Records make claims of exactly that kind and are held to none of it.

## What Was Measured

Measured 2026-09-13 over all 229 files in `docs/records/`, against the whole workspace.

### Test names

4,079 real `fn Test_*` exist in the tree. 83 records name at least one test in prose, 178
distinct names, and **28 resolve to no `fn Test_` anywhere**.

Two of those 28 are artefacts of reading rather than defects, and both have to be named
because a guard written without them reports 14 false phantoms on its first run:

- **Nine are truncations.** A name broken across a line wrap leaves a fragment ending in `_`
  — `Test_Registered_`, `Test_A_Capability_Id_Should_Be_`, `Test_The_Registry_Should_Match_The_`.
  Every one of the nine ends in an underscore, and every one has a real test whose name it
  is a prefix of.
- **Five are deliberate non-existence.** `Test_A_Check_That_Does_Not_Exist_Anywhere_In_This_Tree`,
  `Test_This_Check_Does_Not_Exist_Anywhere`, `Test_Anything`, `Test_Name`, `Test_X_And_More` —
  each in prose that says outright it is a placeholder or a fixture.

That leaves **14 real dead citations**, which fall into three groups:

| what it is | count | example |
|---|---|---|
| a rename with an obvious surviving successor | 6 | `..._Read_Off_The_Doc_Comment` against the real `..._Read_Off_The_Documentation_Comment`, which is `OD-RULES-017`'s own abbreviation rename; `..._Should_Match_The_Manifesto` against `..._Should_Match_The_Manifest`; `..._Offer_All_Eight_Shipped_Rules` against `..._Offer_Every_Composed_Rule` |
| a rename whose successor changed subject too | 1 | `Test_Two_Items_Writing_Different_Records_Should_Be_Claimable_At_Once` |
| **retired, and the code says so at the site** | 2 | `graph.rs` carries "`Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` was here, and is retired as of 2026-09-10 by the owner's decision"; `declarations.rs` carries a "# Why this test was renamed" section naming the old name and the record that cites it |
| **resolves to nothing, and nothing anywhere notes it** | 5 | below |

The five:

| citation | cited by |
|---|---|
| `Test_A_Lapsed_Item_Should_Refuse_A_New_Holder_And_Keep_The_Old_One_Visible` | `OD-LEDGER-012` |
| `Test_A_Held_Pattern_Should_Refuse_Every_Other_Claim_On_The_Board` | `OD-LEDGER-013` |
| `Test_A_Pattern_Item_Should_Be_Unclaimable_Once_Anything_Is_Held` | `OD-LEDGER-013` |
| `Test_A_Phantom_Mirror_Should_Fail_The_Command` | `OD-GATE-004` |
| `Test_No_Strength_Should_Refuse_A_Trace_Claim` | `OD-ANALYSIS-006` |

Each appears in exactly two files: its own record, and `spec/domain-specification.md`. The
committed projection republishes every one of them.

### Paths

**253 repository-rooted cited paths carry a file extension, and 80 do not exist** — measured
by taking paths whose first segment is a real top-level entry of this repository, discarding
ellipsis-truncated fragments and relative ones. The looser reading that admits those gives
357 and 184; the method is recorded here because the two numbers differ by a factor the
conclusion does not, and a later reader deserves to know which was counted.

The 80 come from three causes, not one, and all three are honest history:

- **A module was reorganized.** `nomos-contracts/src/finding/applicability.rs` is now
  `src/reporting/finding/applicability.rs`.
- **A crate was renamed.** `crates/agent/nomos-agent-executor/src/lib.rs` and
  `nomos-agent-executor-ollama` name crates that no longer exist under those names.
- **The architecture deleted the file.** `crates/host/nomos-lsp/src/server.rs` went when
  `OD-HOST-013` moved the protocol to the xvpe backend.

## The Decision

**A citation is a live reference, and the one kind in scope is the test name. A path citation
is a historical statement that may age, and this record says so plainly so that no later
reader treats a dead path as a defect.**

Both halves follow from what was measured, and they differ because the two kinds of citation
make different claims.

### Why a test name is live

A test name is a claim about **coverage** — this is proven, and here is the proof. `D-134`
already decided what a false coverage claim is worth relative to an admitted gap, and decided
it against the same failure in a different file: a phantom is `Blocking`, a declared absence
is `Advisory`, because "a false claim of coverage is worse than a hole somebody wrote down."
A record asserting a test that does not exist is that phantom, one indirection further from
the reader and republished by the committed projection.

The population supports holding them to it. 14 of 178 is 8 per cent, six of those are
mechanical renames, and two are already annotated at the site. The cost of making the claim
live is small and bounded, and it is smallest now.

### Why a path is not

A path citation is a claim about **where something was when the record was written**, and a
record is dated. 80 of 253 is nearly a third, and every cause measured is a legitimate change
to the tree rather than an error in the record. Holding paths live would make every module
reorganization and every crate rename a record-editing exercise across accepted records whose
reasoning did not change — which trades a small reading cost for a large authoring one, and
invites the worse failure of editing a record's history to keep a footnote resolving.

So: **a dead path in a record is not a defect and is not to be filed as one.** A reader who
finds one reads the record's date, takes the path as where the thing was then, and takes the
tree as the authority on where it is now. If the reader wants the move recorded, the place
for that is the record that moved it, not an edit to the record that cited it.

### Which mechanism resolves a test name

The shape `tests/contract/tests/rule_contract_citation.rs` already uses: a contract test that
reads records' own content and checks it against the workspace. Not a new rule, not a new
capability, and not a third mechanism — `mirror.rs` supplies the resolution idea (a declared
name resolved against real parsed `fn Test_*`) and that file supplies the shape (a test that
reads records).

Whatever builds it inherits the two exclusions above as requirements, not as conveniences:

1. **A name ending in `_` is a line-wrap truncation, not a citation.** Nine of the 28 measured
   are exactly this. A guard that reports them is wrong nine times before it is right once.
2. **A record may name a test that deliberately does not exist**, and five do. The guard needs
   a way for prose to say so which is not "the guard happens not to look there".

### The five are disposed of, and the tombstone is the convention

The five in the table above are the case no reading of this question leaves alone: they assert
coverage that does not exist, in a projection this repository commits. They are filed as their
own item, because this record edits no record.

The remedy is not invention. Two of the 14 are already handled correctly and by hand, and
they show the convention: **when a cited test is renamed or retired, the code says so at the
site it left** — `graph.rs`'s "was here, and is retired as of 2026-09-10", `declarations.rs`'s
"# Why this test was renamed" naming both the old name and the record that cites it. That
costs a comment, keeps the record's history intact, and resolves the reader's question at the
place they arrive. It is the cheap half of the answer and it is already working.

## What This Record Does Not Do

It does not edit any record, repair any citation, or rename any test.

It does not write the guard, a rule, or a test. It decides what is owed and which shape the
guard takes; building it is the item that follows.

It does not change `D-134`, `mirror.rs`, or how a phantom mirror is ranked. It applies that
decision's reasoning to a second population rather than reopening it.

It does not make path citations checkable later by a side door. A record that wants a path
held live has to argue against the measurement above.

## Status

Accepted. A test name cited in a governing record is a live reference and is owed resolution,
by a contract test of `rule_contract_citation.rs`'s shape using `mirror.rs`'s resolution,
excluding line-wrap truncations and prose-declared non-existence. A path citation is dated
history and a dead one is not a defect. The five fully-unresolved test citations —
`OD-LEDGER-012`, `OD-LEDGER-013` twice, `OD-GATE-004` and `OD-ANALYSIS-006` — are filed for
repair, and the tombstone comment two sites already carry is the convention the rest follow.
