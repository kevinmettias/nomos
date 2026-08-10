---
id: OD-RULES-002
type: decision
title: Incompleteness is a property of the claim and not of the run
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - rules
  - enforcement
  - completeness
  - capability
  - analysis
relations:
  - target: OD-RULES-001
    type: affects
  - target: D-134
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
---

# Incompleteness is a property of the claim and not of the run

## Question

`P10-PHANTOM-FLOOR` opened against a measurement taken by another item's negative control,
and the measurement is the whole of the case.

`P10-MIRROR-DISAGREEMENT` renamed a universe's claimed mirror to a name that resolves to
nothing — the one condition `D-134` decision 4 says must fail the command — and watched what
the shipped binary did:

```text
[Advisory] … resolves to no check … and the check index is incomplete,
because 1 subject(s) could not be read
```

`nomos check` exited `0`.

`D-134` decision 4 stands at version 2: **a false claim of coverage blocks; an admitted gap
is advisory.** Over this tree neither half of that asymmetry could fire for a phantom.
`tests/corpus/analysis/gamma/broken.rs` is a fixture `nomos-lang-rust` is *supposed* to
refuse — it exists so that a refusal has something to be tested against — so no fact is
materialized for it, so `CheckIndex::Incompleteness()` returned `Some` on every run over this
workspace, so every phantom in the tree was downgraded to `Advisory`. The blocking arm of the
rule was unreachable by construction, permanently, and every test was green.

`OD-GATE-001` is about checks that imply more than they hold. A rule whose severity table
publishes a **blocking** row that no input can reach is that defect in its purest form: the
table is the part later work cites, `D-134`'s own Decision section cites it, and the running
code could not honour it.

## What Is Not Wrong

The downgrade. `OD-RULES-001` introduced it and argued it correctly, and none of that
argument is disturbed here.

A claimed mirror that fails to resolve against an index missing a subject's names is not
thereby established false — the name may be in the file that was not read. Reporting it as a
phantom would be the rule manufacturing the one finding it is entitled to stop a build over
out of its own inability to look, which is the same defect as reporting clean wearing the
other face. Absence must become neither success nor failure. That reasoning is upheld without
qualification and the third state stays.

What is wrong is its **scope**. Incompleteness was one flag over the whole run, so one
unreadable subject *anywhere* silenced a phantom claimed in a file whose facts were read
perfectly well — and those two facts have nothing to do with each other. The rule was not
being careful about a claim it could not check; it was declining to judge claims it could
check, on the strength of an unrelated file.

## The Decision

**A claim is downgraded when the index is short of something that could have resolved
*it*.** Not when the index is short.

`CheckIndex::Incompleteness()` is replaced by `CheckIndex::Shortfall_For(claimed)`, which
answers about one name rather than about the run, and returns `None` — meaning *judge it* —
when nothing missing from the index could have carried that name. Two shapes of shortfall,
because they are two different sentences with two different remedies:

| Shortfall | When | What the claim gets |
|---|---|---|
| `NoIndex` | nothing admitted could answer for any subject | the reader's `MissingCapability`, advisory |
| `Withheld` | at least one unread subject's own text spells the claimed name | that subject's `Applicability`, advisory |
| *none* | every unread subject's text is silent about the name | `Supported`, **blocking** |

### How "could have resolved it" is decided

By whether the unread subject's own text spells the name. `Unread::Could_Have_Declared` is
one line and the argument for it is the whole of this record, so it is worth stating exactly:

A check name in the index is an identifier a provider read out of a token stream, and an
identifier in a token stream is spelled in the bytes the stream was lexed from. So a subject
whose text does not contain the name as a substring **cannot** have declared it, whatever a
provider would have said about the rest of that file. That direction is sound, and it is the
only direction the test is used in.

### Why an unsound text test is admissible here and inadmissible thirty lines away

This is the objection that has to be answered, because `OD-RULES-001` spent a whole section
establishing that this rule must not resolve check names out of text. `Syntax_Requirement`'s
floor exists to refuse `nomos-lang-rust-scan` precisely because a line scanner reports a
`fn Test_Renamed_Away` written inside a block comment, and resolving a claim against that
would let a universe read as covered while nothing checks it.

The two uses are opposite in direction and only one of them is dangerous.

- Text as a **source of names** produces false *positives* that resolve a claim. The rule
  then emits nothing at all, and the defect it exists to find passes through its own
  resolver. That is inadmissible and the floor refuses it.
- Text as a **bound on what an unread file could have held** produces false positives that
  merely *withhold* a block. A name spelled only in a comment inside `broken.rs`, or spelled
  as part of `Test_X_And_More`, counts as "could have declared it" and the claim stays
  advisory. The rule says *I could not tell*, which is the answer it is allowed to give and
  the direction `OD-RULES-001` designed the third state for.

So the unsoundness is confined to the arm that adds doubt. An unsound signal cannot
manufacture a phantom through this path; it can only fail to establish one.
`Test_A_Name_Spelled_In_A_Comment_Should_Still_Withhold_The_Block` pins that shape so a later
edit cannot quietly tighten the test into an identifier match and reverse the direction.

### The counter-argument, answered

**"Any unread file might have held the check."** That is the argument for the global flag and
it is not obviously wrong: a claimed mirror is a *name*, resolving it means finding a check
with that name **somewhere** in the index, and an unreadable subject is somewhere.

The answer is that *any* is doing all the work in that sentence. A file whose bytes do not
spell the name held nothing named that, and the rule holds those bytes — it was handed every
subject's text, and the reason it consults facts for names is soundness in the positive
direction, not lack of access. The set of files that might have held the check is not "the
unread ones"; it is "the unread ones that spell it", and over this workspace that set is
empty for every phantom anyone would plant, because `broken.rs` is seventeen bytes of
deliberate syntax error.

### Is the guarantee exact or probabilistic

**Exact with respect to what the source spells, and no better** — and that bound is not new
here, which is the point worth carrying forward.

The one way the test errs in the unsafe direction is a name no reading of the bytes spells:
an identifier a macro composed, or one behind an `include!`. That hole is not opened by this
record. It is `Assurance::Unknown` completeness, which `Syntax_Requirement` already declares
for exactly this reason — a parser cannot bound what a macro hid — and which this rule
already blocks under: a macro-generated `Test_X` in a *readable* file is equally absent from
the index and is reported as a phantom today, before and after this change. The scoping
inherits the rule's existing completeness bound rather than introducing a second one.

Stated plainly, because a declared guarantee must not exceed what is demonstrated: this
change does not make the rule sound about macro-generated checks. It makes the rule's
treatment of *unread* subjects no weaker than its treatment of *read* ones, which is the
inconsistency the defect was made of.

### Why `MissingCapability` stays whole-run

Not a carve-out, and not policy. `Reader::Require` returns a resolution applicability only
when the registry produced no offer, and `Resolution::Applicability` reads the registry and
the requirement — neither of which varies by subject. So `MissingCapability` holds for every
subject of a run or for none of them, and when it holds there is no index at all. A name
cannot be shown absent from an index that was never built, so there is nothing for the
substring test to be a bound on.

Without that arm, a composition offering this rule nothing would block on every claim in the
tree — a verdict reached by a rule that never got an answer from anybody, which is exactly
what `OD-RULES-001` forbids and what
`Test_The_Scanners_Guarantee_Should_Not_Satisfy_This_Rules_Floor` has asserted since it
landed. `Test_With_No_Provider_Admitted_No_Claim_Should_Be_A_Phantom` states it as the rule
rather than as a side effect.

### The caveat travels with the judgment

When the index is short of subjects that could *not* have resolved a claim, the claim blocks
**and the finding says so**:

```text
`Test_A_Check_That_Does_Not_Exist_Anywhere_In_This_Tree` resolves to no check, so this rule
is declared enforced and never runs — and the check index is short 1 subject(s), none of
whose text spells `Test_…`, so no reading of them could have declared it
```

A reader who wants to know what the run did not see is owed that on the finding, not instead
of it. And the advisory form names the subjects rather than counting them — "the check index
is short `b.rs`, whose text spells this name" — because this record's `done_when` asks that
the scoping state *how* it is decided, and a summary that said only "the index is incomplete"
would leave a reader unable to check that decision against the tree.

## D-134 Decision 4 Is Satisfied As Written

It needs no second amendment, and `D-134` is not edited by this item.

Decision 4 reads: *"A universe naming a check that does not exist is a `Blocking` finding and
fails the command."* Before this record no such finding could be produced over this
workspace; after it, one can. The change moves the code towards the decision rather than away
from it, which is the whole reason the item was opened as an enforcement-power defect and not
as a request to relax a rule.

The version 2 amendment's extension also survives, and reading it precisely is what shows
why: *"A run whose check index is incomplete now reports neither a false claim nor an
admitted gap **for a mirror it could not resolve**, and says so."* The load-bearing phrase is
*a mirror it could not resolve*. A claim whose name is spelled in no unread subject is one the
rule **did** resolve — negatively, having looked everywhere the name could have been. This
record sharpens what that phrase denotes and changes neither the decision nor its extension.

Because decision 4 is not amended, the question it would have forced does not arise. It is
worth recording the answer anyway, since the item asks and since the answer is what makes
this a defect rather than a preference: **a phantom that can never fail a build is worth an
advisory line, which is worth nothing a gate consults.** `Can_Fail_A_Build` is the only
question the exit code asks. A severity ordering whose top row is unreachable is a table
about intentions, and `enforcement.rs`'s own sentence about phantoms — *nothing runs, nothing
can fail, and the declaration says the rule is covered so no reader looks twice* — applies to
this rule's own blocking row exactly as it applies to the universes it judges.

## What Was Measured

Both runs are the shipped `nomos check` over this workspace, at the same commit, with only
the rule changed.

| | before | under a planted phantom |
|---|---|---|
| files examined | 192 | 192 |
| files with a syntax fact | 191 | 191 |
| findings | 14 | 15 |
| findings that can fail a build | **0** | **1** |
| exit code | 0 | 1 |

The item recorded 191 / 190 / 14 / 0 when it was written. Re-measured at this record's HEAD
the first two denominators are 192 / 191, and the difference is fully accounted for:
`tests/contract/tests/agent_harness.rs` landed with `P10-AGENT-HARNESS` between the two
readings. The two numbers this record is about did not move on their own — 14 findings, 0
blocking, before and after the change, over a tree holding no phantom.

That last row is the acceptance test and it is worth being explicit about what it shows. The
"before" column is this record's own code over an unplanted tree: **the fourteen findings are
byte-identical to those the previous binary printed**, the same twelve admitted gaps and the
same two lines for `broken.rs`. Nothing was reclassified by this change except the thing that
was reclassified deliberately. The phantom was planted by adding one `pub const` with a
mirror claim naming a check that exists nowhere, and it was reverted before the commit; it
is the fifteenth finding and the first that can fail a build in this workspace's history.

## What This Costs

`Unread` gains a borrow of its subject's text, so `CheckIndex` gains a lifetime parameter and
`Check_Index_Of` returns one tied to `sources`. No allocation and no second copy of any file;
the sources were already in the caller's hand for the whole of the rule's run.

The scoping is *O*(unread × unresolved claims) substring searches. Over this workspace that is
one file against sixteen names once per run, and the unread set is the set of files no
provider could answer for — a set whose being small is the thing the vacuity guard in
`nomos-cli` already refuses to let grow silently.

`nomos-rules` now reads a subject's text in two places for two purposes, and the second one
must never grow into the first. The field carries the restriction in its own doc comment and
`Could_Have_Declared` is its only reader, which is as much as a comment can do; what actually
holds the line is that a change turning it into a source of names is caught by
`Test_A_Check_Named_Only_Inside_A_Fixture_Should_Not_Resolve` and
`Test_A_Mirror_Should_Resolve_Only_Through_A_Fact`, both of which predate this record.

**What this does not do**, stated plainly because `OD-GATE-001` is about claims wider than
what holds:

- It does not make the rule sound about checks a macro generated. See the guarantee section;
  that bound is `Syntax_Requirement`'s and is unchanged.
- It does not exclude `broken.rs` from the walk. That was the tempting close and it would
  have bought a blocking phantom by deleting the case the third state exists for. The fixture
  is still walked, still reported twice, and still downgrades exactly those claims it could
  have resolved.
- It does not make every unresolved claim block. Five tests turn red under that weakening;
  they are listed in Controls.
- It does not close the twelve unmirrored universes, and it does not widen the rule to
  private lists. Both remain somebody's next item, as `D-134` and `OD-RULES-001` left them.
- It does not wire `nomos check` into the gate. `P10-CHECK-GATE` still holds that, and this
  record sharpens it rather than answering it: the command now has a verdict worth gating on,
  which it did not before.
- **It did not amend `OD-RULES-001` in place; `P10-RULES-001-STALE` did, at that record's
  version 2.** Its Decision section had said *"the check index is marked incomplete for the
  whole run: while it is incomplete, a claimed mirror that fails to resolve is not reported as
  a phantom and cannot fail a build."* The first clause stopped describing the code; the last
  clause was outright false, which is the stronger fact and the reason the amendment could not
  wait. The file was outside `P10-PHANTOM-FLOOR`'s territory — the item lists
  `docs/records/OD-RULES-002` and nothing else under `docs/records` — so the sentence was left
  standing rather than widened into mid-claim, and the one-file amendment followed as its own
  item. `OD-RULES-001` now scopes the downgrade to the claim, quotes the superseded sentence at
  the site, and carries an amendment note pointing here; the `affects` edge is what carries a
  later reader between them.

## Controls

Two weakenings, applied one at a time, failures observed, code restored. They are chosen to
be the two ways this decision can be got wrong, one in each direction, because a test that is
not red under a plausible weakening is not evidence of anything.

**The global downgrade is restored** — `Shortfall_For`'s filter accepts every unread subject
regardless of what its text spells, which is `OD-RULES-001`'s behaviour exactly. Three tests
fail across two crates:

- `mirror::…::Test_A_Phantom_In_A_Read_Subject_Should_Block_Though_Another_Subject_Was_Unread`
  — `assertion left == right failed / left: Advisory / right: Blocking`.
- `mirror::…::Test_Two_Claims_Under_One_Shortfall_Should_Be_Judged_Separately` —
  `left: [("PHANTOM", Advisory), ("WITHHELD", Advisory)]` against
  `right: [("PHANTOM", Blocking), ("WITHHELD", Advisory)]`.
- `check::…::Test_A_Phantom_Should_Block_Though_The_Tree_Holds_A_File_The_Parser_Refuses`
  — the composed command exits `Ok` and prints, over a two-file tree:

```text
[Advisory] completeness-mirror: T (`Test_Renamed_Away` resolves to no check, so this rule is
declared enforced and never runs — and the check index is short broken.rs, whose text spells
this name, so this rule cannot tell a false claim of coverage from a name it did not get to
look for) — a.rs
```

  `broken.rs` is `pub const ??? = ;` and does not spell that name. The weakened summary
  asserts it does, which is the shape of the defect wearing this record's own vocabulary.

**Every unresolved claim blocks** — `Shortfall_For` returns `None` unconditionally, which is
the over-correction the item names. Five tests fail:

- `Test_A_Claim_In_An_Unread_Subject_Should_Not_Be_A_Phantom` — `left: Blocking / right: Advisory`.
- `Test_An_Incomplete_Index_Should_Not_Manufacture_A_Phantom` — `left: Blocking / right: Advisory`.
- `Test_A_Name_Spelled_In_A_Comment_Should_Still_Withhold_The_Block` — `left: Blocking / right: Advisory`.
- `Test_Two_Claims_Under_One_Shortfall_Should_Be_Judged_Separately` —
  `left: [("PHANTOM", Blocking), ("WITHHELD", Blocking)]`.
- `Test_An_Empty_Store_Should_Not_Report_A_Clean_Tree` — `left: 2 / right: 3`, the third
  `DependencyUnavailable` finding being the claim that must inherit the doubt.

The second control is the one that stops the correction going too far, and it is asserted at
two granularities rather than one: the claimed name sitting in *another* unread file
(`Test_An_Incomplete_Index_Should_Not_Manufacture_A_Phantom`, which predates this record and
still passes unchanged in substance) and the claimed name sitting in the unread file that
*declares the universe itself* (`Test_A_Claim_In_An_Unread_Subject_Should_Not_Be_A_Phantom`).
The second is not a corner: a universe and the test that mirrors it living in one file, with
the test under `#[cfg(test)] mod tests`, is the ordinary shape in this workspace.

## Amendment, Version 2

**No decision moved.** One bullet of *What This Does Not Do* recorded that amending
`OD-RULES-001` was owed. `P10-RULES-001-STALE` did it, at that record's version 2, so the
bullet described work that no longer existed. It now records what happened instead of what was
outstanding.

The same bullet said only that the sentence's first clause had stopped describing the code.
That understated it: the final clause — *"cannot fail a build"* — was outright false once a
phantom in a read subject could block, and that is the stronger fact and the reason the
amendment could not wait. The correction says so.

The reciprocal `affected_by` edge is deliberately **not** hand-authored here. `Put_Relation`
completes inverses on the way into the store, so writing one into the front matter would
duplicate a derivation the store already performs — which is the move this repository argues
against everywhere else. `D-134`'s version 2 did hand-author its reciprocal; that inconsistency
is noted rather than propagated, and settling which form is canonical belongs to whichever item
next touches the relation vocabulary.

## Status

Accepted, landed by `P10-PHANTOM-FLOOR`. Amended at version 2 while closing
`P10-RULES-001-STALE`, in the record's own account of what it left owed.
