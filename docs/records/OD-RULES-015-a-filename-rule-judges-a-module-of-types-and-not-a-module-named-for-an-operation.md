---
id: OD-RULES-015
type: decision
title: A filename rule judges a module of types and not a module named for an operation
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - naming
  - capability
relations:
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-014
    type: relates-to
  - target: OD-CAPABILITY-004
    type: relates-to
---

# A filename rule judges a module of types and not a module named for an operation

## Question

`file-name-matches-declared-type` says a public type should live in the file its name spells.
Every `OD-RULES-011` capability family lands in violation of it twice over, and does so by
design rather than by drift: `crates/capabilities/nomos-cap-<X>-policy/src/payload.rs`
declares `<X>PolicyPayload`, and `crates/repository/nomos-repo-<X>/src/reading.rs` declares
`<X>PolicyError`. Five families exist, each new one adds two more findings, and the rule and
the layout have therefore been disagreeing at a fixed rate with no decision behind it.

The question is not whether either is convenient. It is which of the two is stating something
true. A rule that a settled layout violates on every instance is either finding a real defect
five times or is being asked a question it cannot answer, and the two look identical from the
finding list.

## What Was Measured

**The workspace was censused whole rather than at the families.** At `5b0dcc06` the rule
raised 46 blocking findings across 36 files. Classifying every one of those files by whether
its public surface includes a free function — not a method, a free function — splits them
exactly:

- **13 files declare at least one free public function, and account for 17 findings.** They
  are the five `payload.rs`, which export `Encode_Payload` and `Parse_Payload` beside their
  types; the five `reading.rs`, which export `Discover_Workspace` and the error it returns;
  and three `crates/packages/**/reader.rs`, which have the same shape in a different
  directory.
- **23 files declare none, and account for the other 29.** Every one is ordinary naming
  drift — `AuthorizedCandidateFitAdjustment` in `candidate_fit_adjustment.rs`,
  `CandidateOutcome` in `validation_diagnostic.rs` — and the rule is right about all of them.

No file sits on the wrong side of that line. The split was not chosen to fit the families: the
three `reader.rs` files fall on the layout side while sitting in the directory the neighbouring
drift item was scoped to, which is what makes the boundary module shape rather than location.

**Renaming would name a module after the least important thing in it.**
`nomos-repo-goals/src/reading.rs` exists for `Discover_Workspace`. `GoalsPolicyError` is what
that function returns when it cannot. Renaming the file `goals_policy_error.rs` would move the
module's name off its purpose and onto its failure mode, which is a worse name than the one
the rule objected to.

**Nothing in the public API moves either way.** Every one of the 13 modules is private, with
its types re-exported at the crate root: `nomos_cap_goals_policy::GoalsPolicyPayload`, not
`::payload::GoalsPolicyPayload`. So neither the stutter that would argue against renaming nor
the snapshot churn that would argue against splitting is real. The choice had to be made on
what the names mean, because nothing mechanical was at stake.

**The rule already makes this judgment one level up.** `Comparable_Stem` returns `None` for
`lib`, `main` and `mod`, and `file_names.rs`'s own module doc calls that "a name that carries
no claim about a type". That is the same category, recognized by enumeration rather than by
the property the enumeration is standing in for.

## The Decision

**A module whose public surface is types alone is named for a type, and this rule says which
one. A module that also exports a free public function is named for what it does, and this
rule has no claim about its name.** `Violations_In` returns nothing for the second kind.

The discriminator is a *free* function deliberately. A method is named inside the type it
belongs to and says nothing about what the module is for — `OrderBook::New` in `orders.rs`
leaves `orders.rs` a module of types, and it stays judged. Only a function the module exports
in its own right is evidence that the module was named for an operation.

**This holds for the next capability family without a further decision**, which is what the
item that produced this record asked for. A sixth `nomos-cap-<X>-policy` needs no exemption
entry, no waiver and no rename: its `payload.rs` exports `Parse_Payload`, and that is already
the answer.

**The families are not thereby blessed as a layout.** This record says the rule was asking a
question it could not answer, not that `payload.rs` is the right name. If a family's payload
module ever loses its codec and becomes types alone, the rule starts judging it again and will
be right to.

## What This Record Does Not Do

It does not touch `one-public-type-per-file`, the sibling rule in the same file. That rule is
not composed into `Run` and finds nothing today, so giving it the same exemption would be
writing a decision about a population of zero — and `OD-CONTRACTS-001`'s honesty vocabularies
refuse exactly that. When it is composed, this record's reasoning is available to it and the
argument is the same one; making it is that increment's work.

It does not decide the 29 drift findings. Those are ordinary renames and the rule is right
about them; they are `P27-SELFCHECK-FILENAME-DRIFT-PACKAGES`'s subject, not this record's.

It does not claim the free-function discriminator is the only one that would have worked. It
claims it is the one this workspace's own population supports without a single misclassified
file, which is a stronger warrant than an argument from taste and a weaker one than a proof.

## Status

Accepted. The rule reads the discriminator, a unit test holds each side of it, and the 17
findings the families raised are gone while all 29 drift findings still report.

Revisit if a module that is genuinely a type module acquires one free function and stops being
judged when it should be. That would be evidence the discriminator is a proxy that has come
apart from what it stands for, and the answer then is a better discriminator rather than a
list of exceptions to this one.
