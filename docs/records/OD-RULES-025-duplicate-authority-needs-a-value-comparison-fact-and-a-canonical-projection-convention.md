---
id: OD-RULES-025
type: decision
title: Duplicate authority needs a value-comparison fact and a canonical/projection convention
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
relations:
  - target: OD-RULES-020
    type: relates-to
  - target: OD-RULES-023
    type: relates-to
  - target: OD-RULES-024
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
---

# Duplicate authority needs a value-comparison fact and a canonical/projection convention

## Question

A person required a rule against a class this workspace has paid for more than once: two
artifacts that independently determine the same behaviour, where one should have been a
projection of the other — not source duplication, which ordinary tools already find, but
two independent computations of one fact with no declared relationship between them. The
item's own `why` names two historical instances and asks that a real one be found today,
with a declared projection relationship suppressing a finding rather than a per-file
exemption. Whether this is buildable needed checking the same way `OD-RULES-023` checked
write authority and ownership, and `OD-RULES-024` checked drift and leakage.

## What Was Measured

**The nearest existing mechanism answers a different question.** `Check_Completeness_
Mirrors` and `universe_kind.rs`'s "Mirrored by \`Test_Name\`" convention resolve whether a
*named check exists* — it looks up a backtick-quoted identifier and confirms the function is
real. It never extracts a value from either side and never compares one. Duplicate
authority needs the opposite: pull a comparable value out of two independent artifacts and
judge whether they agree. No capability in `crates/capabilities/` does that, and no rule in
`nomos-rules` does either — every existing "the declaration matches the graph" judgment
(`Check_Dependency_Direction`, `Check_Write_Authority`, `readme.rs`'s own band-table check)
is a bespoke, hand-written comparison against one specific pair, decided and coded once, not
a general "discover two artifacts asserting the same fact" mechanism.

**Both historical instances the item cites are already fixed, and neither was fixed by a
rule.** The band table — `nomos-rules/src/dependency/bands.rs`, `tests/contract/tests/
boundaries/bands.rs`, and README's own table, three hand-maintained copies with a fourth
test checking one of them — was closed by `OD-RULES-020`'s zone migration: `zones.rs`'s
`ZONES`/`SAME_ZONE_EDGES` became the one authority, and every reader now imports it through
`nomos-rules`' own public surface. The "twelve" figure — `checks/mirror.rs`'s own module doc
once restated `UNMIRRORED_TOTAL`'s value by hand and drifted to twelve after the real number
fell to three — was closed by removing the restatement, not by a rule catching the drift; the
comment now says outright why it declines to repeat a number the table beside it already
owns. Both fixes are the general lesson ("stop maintaining two copies") applied by a person,
not a mechanically enforced property.

**A real, live instance exists, and it is instructive about the size of the problem
rather than a shortcut past it.** Measured at `edd8f1c9`: `nomos-rules/src/checks.rs`'s own
header claims "the sixty-nine rules this crate implements", and `lib.rs` narrates its rules
one at a time up to `Check_Requirement_Trace_Staleness`, "the sixty-ninth rule" — both
hand-counted prose. Three different counts of "how many rules" exist in this workspace at
that commit, none asserted equal to another by anything: the prose figure (69),
`nomos_rules::DESCRIPTORS.len()` and `nomos-check-orchestration`'s own `RULE_COUNT` (70,
proven equal to each other by `tests/contract/tests/rule_descriptors.rs`'s bidirectional set
comparison against `Composed_Rules()`, whose own array is `RULE_COUNT` long), and the raw
count of every `pub fn Check_*` this crate defines (86, counting every rule function
regardless of whether it is composed into a real run or resolved through `DESCRIPTORS`).
That three plausible "authoritative" numbers can coexist, unreconciled, in a workspace this
disciplined about mirrors and completeness is itself the strongest evidence the `why` text
offers: this property is real, current, and unchecked — and also that even a person auditing
it by hand cannot say which of the three the prose was ever supposed to match without
knowing what population each one counts. A rule cannot resolve that ambiguity either without
the same answer a person needs first.

**Those three figures are a measurement at a named commit, not a standing fact, and they
have already moved once.** This record first stated them as 67, 65 and 84. By `edd8f1c9`
every one of them had rotted, while the disagreement they were cited to prove stayed exactly
as real — the recorded property demonstrating itself on this record's own prose, which is
why the numbers above name the commit they were taken at and why nothing here asserts them
of the present tense. Re-measure before citing them; do not carry them forward. The same
discipline `tests/contract/tests/rule_composition.rs` already keeps in its own header, where
`P46-UNCOMPOSED-RULES-ARE-COUNTED`'s count is dated to the day it was taken.

**What "a declared projection relationship suppresses the finding" would require does not
exist yet, on either side of it.** It presupposes a new annotation naming which artifact is
canonical and which is a projection — not `universe_kind.rs`'s "Mirrored by", which only
names a check's existence, never a value or a source-of-truth relationship — and a decision
about whether suppression trusts the declaration on its face or the rule still verifies the
projection actually holds. Building the annotation without deciding the second question
would ship exactly the false-coverage shape `OD-COMPLETENESS-001` exists to catch: a
declared relationship nothing confirms is worse than an admitted absence of one.

## The Decision

**The rule is not built here.** It needs two facts this workspace does not materialize
today — a normalized value extracted from free prose (so "sixty-nine" reads as 69, not as
opaque text), and an element count or comparable computed value for a declared list or
table, held as an observed fact rather than read only by `cargo test` — and one convention
not yet designed: a canonical/projection annotation, and whether suppression under it is
trusted or itself verified. All three are capability-and-convention design questions of the
same weight `OD-RULES-024` named for architecture drift and representation leakage, not
details a rule's own implementation could improvise on the way past.

**The three-way rule-count disagreement measured above is recorded as the real instance a
future rule should expect to find and resolve on its first run**, the same way `OD-RULES-
023` recorded `nomos-store`'s single write door and `OD-RULES-024` recorded representation
leakage's clean baseline. Which of 69, 70, or 86 the prose is actually supposed to track is
not decided here: fixing it by hand now, without knowing what population `nomos-rules`'
module-level prose is meant to describe, risks trading one unchecked number for another —
and a future rule should expect the three values themselves to have moved again by the time
it runs, since nothing yet holds them still.

## What This Record Does Not Do

**No code changes here**, the same discipline `OD-RULES-023` and `OD-RULES-024` both held
to. It does not design the value-extraction fact, the list-length fact, or the canonical/
projection annotation — each is a real capability or convention decision, not answered by
naming that it is needed.

It does not correct `lib.rs`'s or `checks.rs`'s own rule-count prose. That edit is left
for whoever decides which of the three counts the sentence is meant to track — the fix
belongs with the design that would let a rule hold it correct going forward, not as a
one-off hand edit this record makes on its way past.

It does not withdraw the property the original item asked for. Duplicate authority remains
real, and this workspace has paid for it more than once, exactly as the item's own `why`
says. It is undecided as unready, not declined as unwanted.

## Status

Accepted. Duplicate authority needs a value-comparison fact, a list-length fact, and an
undesigned canonical/projection convention before any rule can judge it; a real, live
three-way rule-count disagreement in this crate's own prose is recorded as the instance a
future rule should resolve.
