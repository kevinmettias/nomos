---
id: OD-LEDGER-014
type: decision
title: A refusal is phrased so the caller can name the subject, and the reading method with no caller is removed
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - cli
  - diagnostics
  - public-surface
relations:
  - target: OD-LEDGER-001
    type: affects
  - target: OD-LEDGER-005
    type: relates-to
  - target: OD-LEDGER-013
    type: relates-to
---

# A refusal is phrased so the caller can name the subject, and the reading method with no caller is removed

## Question

Two findings, both surfaced by `P10-AUDIT-STATE` in territory it did not hold.

`ClaimRefusal::HeldBy`'s description read `{item} overlaps territory held by {holder}`, where
`item` is the **blocker** rather than the item that was refused. That is a true sentence
standing alone and says the reverse of what happened as soon as a caller prints it beneath the
subject's own identifier. `work audit` does exactly that, and emitted

```
P1-MODEL      held      P9-AUTHORING overlaps territory held by agent-x until unix …
```

for forty-four items, in which a reader can tell that one of the two names was refused and
cannot tell which. `P10-AUDIT-STATE` could not reach `exclusion.rs`, so it re-phrased that one
arm locally in `work.rs` — leaving one refusal with two renderings and the library's still
wrong for the next caller.

Separately, `ExclusionLedger::Conflicts` had no caller anywhere in the workspace. `work audit`
was the last one and now goes through `Blocking_Refusal`. It was public surface and
`OD-LEDGER-001` described it, so deleting it silently was not available.

## The Decision

### 1. The description never puts a name other than the refused item in subject position

Of the two branches `P10-REFUSAL-SURFACE` offered, this is the second: the blocker's identity
is already exposed on the variant, so a caller can compose whatever sentence it needs, and
`Describe` is documented as the form for a caller that has **not** named its subject. The held
arm now reads

```
territory overlaps {blocker}, held by {holder} until unix N
```

which composes correctly under an identifier and asserts nothing false without one.

The first branch — `Describe` taking the subject — was rejected on scope rather than taste. It
changes a widely used signature and every call site, and it is dead weight for the arms whose
subject is already the refused item (`Lapsed`, `NotClaimable`, `DependencyUnmet`) and for
`LedgerUnusable`, which is not about an item at all. A parameter every caller must supply and
most arms ignore is a worse surface than a phrasing rule.

The rule generalizes, which is why it is stated on `Describe` rather than on the one arm:
**an arm carrying an identifier that is not the refused item must not open with it.** Two arms
carry one today.

### 2. The workaround is deleted in the same commit

`Blocking_Reason` in `crates/host/nomos-cli/src/work.rs` is gone and `audit` calls `Describe`
directly. `P10-REFUSAL-SURFACE` required this in the same commit and the requirement is right:
the cost of the workaround was never the duplicated `format!`, it was that two renderings of
one refusal outlive the reason for the second, and the next caller gets the wrong one.
`OD-LEDGER-005` is the standing instance of this shape.

The phrasing adopted above is the workaround's own, which is the evidence it was right — it
was written by the session that had actually read the forty-four bad lines. What was wrong was
its address, not its wording.

### 3. `Conflicts` is removed rather than kept with a reason

The trait method, its implementation, and its two lines of public surface all go.

Keeping it would mean keeping a public method with no caller, whose answer is a snapshot that
is stale the moment it returns — `Claim` re-asks the same question under the lock, which is why
its own documentation noted it took none. A reader finding it would reasonably build the very
thing `P10-AUDIT-STATE` removed: a second walk over the board that compares territory and knows
nothing about state or dependencies, which over-reported forty-four `Done` items as blocked and
was silent on every item merely waiting.

`OD-LEDGER-001`'s "What Is Actually Enforced" paragraph named it and is amended here. The
mechanism it described has not changed — the same comparison, reached only through `Claim`.
That is the amendment: a record describing a method nobody can call is a promise the crate no
longer keeps.

## What Was Considered And Rejected

**Leave `Describe` alone and document the hazard.** Refused in advance by
`P10-REFUSAL-SURFACE`'s `done_when`, and correctly: the hazard is a sentence that reads as
true while naming the wrong subject, and a note in a doc comment is not read by the person
reading the output.

**Keep `Conflicts` for future callers.** The future caller is `Claim`, which exists. An
unused public method is a design that has not been used yet, and this one has been tried:
`audit` used it and the result was the defect `P10-AUDIT-STATE` closed.

**Assert the fix with a substring search for the identifiers.** Rejected because the wrong
sentence passes it — both spellings contain both identifiers and only the order differs. The
test asserts position, which is the property that actually moved.

## What This Does Not Do

- It does not change which claims are refused, or any exit code. Both changes are to what is
  said and to what is reachable, not to what is decided.
- It does not remove the blocker's identity from the refusal. Decision 1 depends on it staying
  there: a caller composing its own sentence needs it, and the test asserts the description
  still names both the blocker and its holder.
- It does not touch the arms whose subject is already correct.

## Controls

| Weakening | What it produces |
|---|---|
| restore `{item} overlaps territory held by …` | `P1-MODEL: P9-AUTHORING overlaps …` — forty-four lines naming the blocker as the refused item |
| fix the library and leave `Blocking_Reason` | two renderings of one refusal, and the next caller inherits whichever is wrong |
| add a subject parameter to `Describe` | every call site changed, and a parameter four arms ignore |
| keep `Conflicts` unused | a stateless second walk over the board, which is the defect `P10-AUDIT-STATE` measured at forty-four wrong lines |
| assert with a substring search | green for the sentence the item was raised about |

## Status

Closed by `P10-REFUSAL-SURFACE`.
