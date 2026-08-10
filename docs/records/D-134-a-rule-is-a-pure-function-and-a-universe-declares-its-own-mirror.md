---
id: D-134
type: decision
title: A rule is a pure function over source, and a universe declares its own mirror at the site
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - rules
  - enforcement
  - completeness
  - command-line
relations:
  - target: OD-COMPLETENESS-001
    type: affects
  - target: OD-GATE-001
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
  - target: OD-RULES-001
    type: affected_by
---

# A rule is a pure function over source, and a universe declares its own mirror at the site

## Context

`P10-FIRST-CHECK` opened against a measured fact: ten public types in `nomos-contracts` had
no consumer anywhere, and four of them — `GateCategory`, `EnforcerRef`, `EnforcementBreach`,
`EnforcementReach` — were the whole of `enforcement.rs`. It was the only module in the
protocol crate that was wholly unimplemented, in the one crate whose every type is
reimplemented by peers that will never compile it. A declaration there with no
implementation is a published protocol commitment accruing cost.

The wider fact the item named is the one that mattered more: nothing in this tree had ever
judged code. Phase 8 was six follow-ups to Phase 7, Phase 9 was nine findings against Phase
8 and the tree itself, and two consecutive batches had produced correction and audit rather
than capability.

The rule to build first was not a free choice. `OD-COMPLETENESS-001` had just been closed by
`P9-ONE-DIRECTION` with three historical instances analysed and a discovery scanner landed
in `tests/contract`, which made the completeness-mirror rule the only rule in this tree with
recorded cases to test a judgment against.

Two questions had no answer in the tree, and both had to be settled before a line was
written.

**Where does a rule get its subject?** The obvious shape is a rule that walks the tree it is
standing in. That shape cannot be tested against the three instances `OD-COMPLETENESS-001`
analyses, because none of the three can be replayed from git — each was repaired at the site
where it was found. A rule that reads the filesystem can only ever be judged against the
tree it happens to be in.

**Who decides whether a universe is mirrored?** `tests/contract` answered this with a
hand-written classification table and said in its own doc comment why: the question is about
meaning, and that crate deliberately has no types to answer it with. That answer is correct
for a test crate and does not survive contact with a binary. The table lives in a crate at
band 100 that nothing may depend on, so `nomos check` cannot read it, and duplicating it
into the product would create a second source of truth for a rule that exists because a
comparison could not fail.

## Decision

**A rule takes its subject as an argument.** `nomos-rules` opens no file, walks no
directory, and does not know where the workspace is. The caller supplies the subject: the
binary composes it from the real tree, and a test composes it from three files it wrote by
hand. This is what makes `Test_All_Three_Historical_Instances_Should_Fail_This_Rule`
possible at all — the rule judges code that no longer exists anywhere, in the shape it had
when it was wrong.

*(Amended at version 2. This paragraph originally read "a pure function from source text to
findings" and concluded that the subject is source text. Replayability proves that the
subject is an argument and no more; a `FactReader` passed as a parameter satisfies it
identically. See the amendment note below.)*

**A universe declares its mirror at the site, in a doc comment.**

```rust
/// Mirrored by `Test_Every_Table_In_The_Schema_Should_Be_Declared`.
pub const fn All() -> &'static [Self]
```

This does not answer the meaning question. It moves it to where meaning already lives — with
the author, next to the thing being declared — and leaves the rule with the question a
machine can answer: *does the named check exist?* That is exactly the split `enforcement.rs`
was built on. Naming an enforcer is a claim about existence; whether it runs is a claim about
execution; only the second can be checked against reality, and it is the one that matters.

**The judgment is `EnforcementReach`, not a second vocabulary next to it.** A universe that
names a check declares `EnforcerRef::Check` and expects `GateCategory::Blocking`; one that
names nothing declares `EnforcerRef::Review` and expects `GateCategory::Review`. `computed`
comes from resolving the name against the real source, and a name that resolves to nothing
produces `EnforcementBreach::Phantom`. `Is_Enforced` decides whether there is a finding and
`Is_Truthful` decides how bad it is. Both methods already existed and were already right;
writing a second judgment beside them would have been a second place for one rule to be
spelled.

**A false claim of coverage blocks; an admitted gap does not.** A universe naming a check
that does not exist is a `Blocking` finding and fails the command. A universe naming nothing
is `Advisory` and does not. The asymmetry is `enforcement.rs`'s own — a phantom is "worse
than declaring no enforcer at all: nothing runs, nothing can fail, and the declaration says
the rule is covered so no reader looks twice" — and it is also what keeps the check usable:
this workspace has thirteen unmirrored universes today, and a gate that can never be green
is a gate everybody learns to bypass.

**`Finding` joins `nomos-contracts`.** It carries `applicability`, `evidence` and `gate`
alongside the summary, because all three are destroyed by printing: prose cannot be asked
whether the rule reached its subject, how the claim was come by, or whether anything would
actually have failed a build over it. `Can_Fail_A_Build` consults the gate *and* the
applicability, so a finding from a rule that never reached its subject cannot fail anything.

**A finding is identified by the universe's name, never by its path.** `identity.rs` states
the rule for the whole system, and a finding is the type most likely to break it because a
location is the field a human wants first. A universe that moves file is the same universe;
keyed on its path it would read as one finding closed and another opened.

**Discovery parses; it does not scan lines.** This was decided by being wrong three times.
The first implementation matched trimmed lines, which is what `tests/contract` already did,
and each of its first three runs against this workspace reported the rule's *own test
fixtures* as real universes — one of them as a phantom mirror, because a fixture named a
check that does not exist. A trimmed continuation line of a multi-line string literal is
indistinguishable from a declaration.

Three successive textual fixes each closed the case in front of them and left the next one
open: stopping at `#[cfg(test)]` missed integration tests under `tests/`, which carry no
such marker; tracking quote parity was defeated by a raw string containing an ordinary
string; tracking raw strings on top of that left the state machine drifting on a case
neither rule covered. `syn` is already in this workspace, "is this a declaration" has an
exact answer, and a rule that reports its own test data is not one anybody reads the output
of. Check names are parsed for the same reason — a `fn Test_X` written inside a fixture
would otherwise resolve a claim that nothing checks, which is this rule's defect arriving
through its own resolver.

## Amendment, Version 2

`P10-FACT-BYPASS` found that this record's argument does not reach its conclusion, and the
finding is upheld.

The argument is that none of the three `OD-COMPLETENESS-001` instances can be replayed from
git, so a rule that reads the filesystem can only ever be judged against the tree it is
standing in. That is sound and it defeats a filesystem-walking rule. It does not defeat a
fact-consuming one: a `FactReader` handed to the rule as a parameter is an argument in
exactly the sense that a `&[SourceFile]` is, a test can build one over three files that no
longer exist anywhere, and the replay requirement is met identically. The premise proves
*the subject is an argument*. It was written down as proving *the subject is text*.

The gap mattered because of what was on the other side of it. `nomos-analysis`,
`nomos-capability` and `nomos-cap-syntax` existed when this record was written, and
`boundaries.rs` had already placed `nomos-rules` at band 30 so that a rule needing a parsed
tree "must be able to reach a provider rather than vendor a second parser". This record
vendored the second parser and recorded a reason that does not carry it. There were real
reasons available — scope, and what the payload can carry — and neither is what was written.

`OD-RULES-001` settles the relationship. Its short form: the rule reads facts for the half
the agreed payload can serve, states its own floor so a composition root cannot feed it an
approximation, and keeps `syn` in `universe.rs` alone because `nomos.syntax.items.v1`
carries no doc comment and no declared type, which is what discovery needs. The reason for
the residual front end is now a measurement with a named end condition rather than an
inference that does not hold.

**What stands from this record is everything except that one inference.** A rule takes its
subject as an argument and opens no file; a universe declares its mirror at the site; the
judgment is `EnforcementReach` and not a second vocabulary beside it; a false claim of
coverage outranks an admitted gap; a `Finding` carries its applicability, evidence and gate;
a finding is identified by name and never by path; and discovery parses rather than scanning
lines. Six of the seven decisions are untouched, which is why this record is amended in
place rather than superseded — a `superseded_by` edge would tell a later reader that all
seven are dead.

Two of the six are extended rather than merely retained, and both extensions are additive.
A run whose check index is incomplete now reports *neither* a false claim nor an admitted
gap for a mirror it could not resolve, and says so; that is the asymmetry with a third state
above it rather than a change to the ordering between the two. And `Finding::applicability`
becomes load-bearing in a second way, carrying the difference between "no provider offers
what this rule needs" and "the store had nothing for this subject".

## Consequences

`nomos check` runs over this workspace: 174 files examined, 14 findings, 0 of which can fail
a build. Thirteen are unmirrored universes. The fourteenth is
`tests/corpus/analysis/gamma/broken.rs`, which is deliberately unparseable and is now
*reported* as unread rather than silently skipped — `Applicability::Unparseable`, and
`Can_Fail_A_Build` refuses it on the applicability alone.

*(Amended at version 2.)* Those three numbers are what was measured on the day and are left
as written. The tree has grown since; `OD-RULES-001` re-measured it with the old binary and
the new one over the same 190 files and recorded both, which is the comparison that matters
rather than either number on its own.

The negative control was confirmed on the real tree rather than only in a fixture — a mirror
annotation naming a check that does not exist was added to `MIGRATIONS` in
`nomos-spec-store/src/schema.rs`, the command reported `Blocking` and exited 1, and the
annotation was removed. Three more controls were confirmed red by weakening the rule and
watching the tests fail: a claim carrying to the next item, a phantom mirror downgraded to
advisory, and a fixture read as a declaration.

The single scanner is shared rather than duplicated. `tests/contract` keeps the walk, which
is a question about the workspace and needs `cargo metadata`; recognising a universe in a
file moved to `nomos-rules`, and `tests/contract` depends on it. Band 100 observing band 3
is the safe direction.

*(Amended at version 2.)* That claim is about the universe *recogniser* and not about the
parser. One recogniser with two callers and two independent `syn` front ends over one
language are different objects, and this record stated the first as though it discharged the
second. `OD-RULES-001` separates them: recognition stays shared, and the second front end is
reduced to `universe.rs` and given a stated cause and a condition for its removal.

Three of the four mirrored universes now declare their mirror. The fourth, `DECLARED_RULES`
in `nomos-spec-validate`, is mirrored in fact and does not declare it, so the command reports
it as an admitted gap. That is a true statement about the source and it is left standing
rather than papered over: `nomos-spec-validate` is outside this item's declared territory,
and widening mid-flight is what the ledger's own rule says not to do.

**What this does not do**, stated plainly because `OD-GATE-001` is about checks that imply
more than they hold:

- It does not close the thirteen unmirrored universes. It makes each one a line of output
  with a named risk instead of a silence.
- It judges public lists only. A private list is still a list, and widening to them takes
  this workspace from thirteen unmirrored universes to thirty-four — every one of which
  needs a person to say what would go wrong. That is somebody's next item, not a side
  effect of this one. The narrowing is defensible on its own terms: all three instances
  `OD-COMPLETENESS-001` analyses were public lists consumed from somewhere else, which is
  what let each go wrong unnoticed for months.
- It recognises a universe as a `pub const` of slice type or an inherent `All()`. A list
  built any other way is not seen. This is the same recognition `tests/contract` already
  used, now shared rather than duplicated, and it is a shape convention rather than a
  proof.
- Two universes sharing a name in different crates would share a subject identity. Nothing
  in this workspace does today. The fix is a qualified name, not a path, and it is not
  written yet.
- `nomos check` is not wired into the gate. The command exists and is honest; making the
  gate run it is a separate decision about what the workspace is prepared to block on, and
  claiming it here would be the overclaim this record is partly about.
