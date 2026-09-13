---
id: OD-LEDGER-013
type: decision
title: Territory is paths, and a pattern is refused where it is authored
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - ledger
  - territory
  - concurrency
  - cli
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
  - target: OD-LEDGER-009
    type: relates-to
  - target: OD-LEDGER-016
    type: relates-to
---

# Territory is paths, and a pattern is refused where it is authored

## Question

`nomos work add` advertised `--territory-pattern <glob>` in its usage text and in `README.md`,
beside `--territory`. A value passed to it landed in `Territory::patterns`, and
`Territory::Intersect` short-circuits on a non-empty `patterns` before comparing a single
path — so every comparison involving that item answered `Intersection::Unknown`, which
`Refusal_From` maps to `ClaimRefusal::UnknownIndependence`, whose `Is_Retryable` is false.

Refusing an unanswerable overlap rather than granting it is correct and is not in question
here; `OD-LEDGER-001` is why the ledger's answer *is* the exclusion, and a claim that cannot
be shown independent must never compare as touching nothing. What is in question is that the
only documented way to reach that state was an advertised flag for a comparison nobody wrote.

`P10-PATTERN-BRICK` asked which of two things closes it: the pattern compares, or the pattern
is refused where it is authored.

## What Was Measured

No open item on the board carries a pattern — 0 of 100, checked before any change — so this
was found by reading `territory.rs` while measuring the board's serialization, not from an
incident. That is what made closing it cost a test and a refusal rather than a recovery.

**The item's own description of the failure was one clause too strong, and the correction is
the reason this record argues the way it does.** `P10-PATTERN-BRICK` states the item "can
never be claimed by anyone, its own holder included". Measured through the real ledger, that
is not what happens. `FileLedger::Conflicts` compares a candidate territory only against items
holding an **active claim**, so on a quiet board a pattern item claims like any other — the
pattern is never consulted, because there is nothing to consult it against.

The true shape is worse than an item nobody can take:

| Board state when the pattern item is reached | What happens |
|---|---|
| nothing held | it claims normally, and nothing warns the agent |
| the pattern item now held | every other claim is refused `UnknownIndependence`, non-retryable, including territory sharing no path with it |
| anything else held first | the pattern item can never be claimed, and no lease expiring resolves it |

So there is no ordering in which a board carries a pattern and keeps working, and the one
agent who acquires the power to stop every other session learns nothing about having done so.
The refusal they each receive is the code `README.md` defines as *stop and fetch a person*,
which is the one refusal an agent must not route around — so the failure does not even
present as contention. `Test_A_Pattern_Anywhere_On_The_Board_Should_Refuse_Every_Claim_As_Ledger_Unusable`
and `Test_A_Pattern_Item_Should_Refuse_Every_Claim_Wherever_It_Sits`, both in
`crates/substrate/nomos-ledger/tests/exclusion_holds/claiming.rs`, pin both directions.

## The Decision

### 1. `--territory-pattern` is a usage error, and leaves the advertised surface

`Parse_Add` refuses it before it checks `--territory`, and the flag is gone from the usage
text and from `README.md`. The refusal names the flag, quotes the pattern it refused, and says
what to write instead.

It is a **usage** error and deliberately not a ledger one. Exit 2 says the mistake is in what
was typed; 1 or 5 would say the board is broken, and the whole complaint of this item is that
a typing mistake was presenting as a broken board. The ordering is asserted separately by
`Test_A_Pattern_Alone_Should_Be_Refused_As_A_Pattern`, because before this change a pattern
passed on its own reported "an item that reserves nothing excludes nobody" — a true sentence
about the wrong problem, which sends the author to add a path rather than to drop the flag.

### 2. The narrowing costs nothing, because containment already says it

This is the load-bearing half, and without it the decision would be a real reduction in what
the ledger offers.

The justification the field originally carried was that an item may honestly say "this touches
everything under `crates/spec/`" before anyone can enumerate that. That sentence is already an
ordinary territory entry. `Contains_Or_Equals` decides containment textually, so
`--territory crates/spec` reserves every file beneath it, with no filesystem access and no
glob engine — and, unlike the pattern, it *answers*: a claim on a file inside it is refused
`HeldBy`, which is retryable, names a holder and expires. A queue rather than a wall.
`Test_A_Directory_Should_Reserve_Its_Subtree_Without_A_Pattern` asserts exactly that, together
with the other half — genuinely unrelated territory stays claimable, so a directory entry
reserves a subtree and not the board.

So the flag added no expressiveness. It added one documented route to a state the comparison
cannot decide.

### 3. The comparing branch is refused on soundness, not on cost

Implementing glob comparison is not hard for glob-against-path. It is hard where it counts:
glob-against-glob, which is what two items each carrying a pattern would require, and where
the answer that must never be wrong is `Disjoint`.

Every other answer this mechanism gives is safe when wrong in the conservative direction — an
unnecessary `Overlaps` costs throughput, and `Unknown` costs a refusal. A wrong `Disjoint`
costs an edit, and an edit does not come back. That is the same trade `Normalize_Path` already
makes when it folds case, decided the same way. Buying a feature nobody is using, at the price
of a new opportunity to answer `Disjoint` incorrectly from subtle glob semantics, is the wrong
side of it.

### 4. The field and its `Unknown` are kept, and this is not half a decision

`Territory::patterns` stays, and `Territory::Intersect` still short-circuits to `Unknown` on
it. Two reasons, and neither is reluctance:

- `#[serde(deny_unknown_fields)]` governs this document, so removing the key would refuse
  every ledger ever written, including the hundred items carrying `"patterns": []` today.
  `OD-LEDGER-008` is the standing decision that a writer which does not understand a document
  must not write it; deleting a field every existing document carries is that failure chosen
  deliberately.
- A hand-edited document, or some future authoring surface, can still put one there. Such a
  document must keep failing closed, and `Unknown` is what does that.

**Withdrawing the flag removes the way in. It deliberately does not remove the guard.** The
two are different jobs and this record does both of them on purpose.

`Territory::With_Pattern` stays public for the same reason: a guard against a state nothing
can construct is a guard nothing can test, and it is how the fail-closed behaviour above is
exercised.

## What Was Considered And Rejected

**Leave the flag and document the hazard.** `P10-PATTERN-BRICK`'s `done_when` refuses this in
advance and is right to. The warning arrives after the write, and the board a documented flag
can brick is every session's board at once.

**Refuse patterns in `Validate`, so the document rejects one too.** This is the more complete
answer and it is the right shape — it would close the hand-edit route as well as the flag.
It is not done here because `Validate` lives in `store.rs`, which is outside
`P10-PATTERN-BRICK`'s territory, and widening a territory mid-claim is the failure this
repository has already had twice. The state remains safe without it — it fails closed, which
is decision 4 — so this is a completeness gap and not a hole. It is worth an item.

**Expand the pattern against the filesystem at authoring time.** Rejected on
`OD-LEDGER-009`: the ledger's validity must not depend on when it is read, and an expansion is
a photograph of a working tree that three sessions are writing. Two agents expanding one
pattern seconds apart would reserve different sets, and the document would not say which.

## What This Does Not Do

- It does not change `Territory::Intersect`, `Refusal_From`, or what `Unknown` means. The
  refusal semantics are untouched; only the way of reaching them by accident is.
- It does not remove `patterns` from the schema, and decision 4 names the identifier that
  would break if it did.
- It does not make the state unreachable, and does not pretend to. It makes it unreachable
  *through a documented flag*, which is what `done_when` asked for, and leaves the guard
  standing behind it.

## Controls

| Weakening | What it produces |
|---|---|
| keep the flag, document the hazard | one agent silently acquires the power to refuse every other session, and finds out after the write |
| relax `Unknown` to `Disjoint` for patterns | two agents editing one file, told the territory was independent — the failure the ledger exists to prevent |
| implement glob-against-glob comparison | a new way to answer `Disjoint` wrongly, bought for a feature 0 of 100 open items use |
| remove the `patterns` field outright | every ledger ever written stops loading, `OD-LEDGER-008`'s refusal chosen deliberately |
| check `--territory` before the pattern | "an item that reserves nothing excludes nobody" — true, about the wrong problem, and it sends the author to add a path rather than drop the flag |

## Amendment: The Two Tests Named Above Were Renamed

Version 1 cited `Test_A_Held_Pattern_Should_Refuse_Every_Other_Claim_On_The_Board` and
`Test_A_Pattern_Item_Should_Be_Unclaimable_Once_Anything_Is_Held` and said they pin both
directions. Neither name exists, and neither has since some point before 2026-09-13, when
`OD-SPEC-017`'s census over every record found them.

**Both assertions still hold and both tests still exist**; only the names moved, to
`Test_A_Pattern_Anywhere_On_The_Board_Should_Refuse_Every_Claim_As_Ledger_Unusable` and
`Test_A_Pattern_Item_Should_Refuse_Every_Claim_Wherever_It_Sits`. Nothing about the decision
changes and no assertion was reversed; the sentence above now names the tests that are
actually there.

`OD-SPEC-017` decided why this was worth repairing rather than leaving: a test name in a
record is a claim about coverage, `D-134` already ranks a false coverage claim above an
admitted gap, and this repository's committed `spec/domain-specification.md` republished the
claim for as long as it stood.

## Status

Closed by `P10-PATTERN-BRICK`. Version 2, amended once to name the two tests the renamed ones
became.
