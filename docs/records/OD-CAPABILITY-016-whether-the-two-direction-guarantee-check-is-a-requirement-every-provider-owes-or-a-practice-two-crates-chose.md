---
id: OD-CAPABILITY-016
type: decision
title: The two-direction guarantee check is a requirement, and a declaration names what exercises it or says it cannot be exercised
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - guarantee
  - providers
  - verification
relations:
  - target: OD-RULES-001
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-RULES-031
    type: relates-to
  - target: OD-CAPABILITY-014
    type: relates-to
---

# The two-direction guarantee check is a requirement, and a declaration names what exercises it or says it cannot be exercised

## Question

`nomos-lang-rust/src/guarantee.rs` and `nomos-lang-go/src/guarantee.rs` open with the same
sentence — *what this provider promises, stated as a value and checked in two directions* —
and name both. Downward, `nomos_capability::Registry` refuses an offer claiming more than the
capability ceiling permits, so a provider does not grade its own work. Upward, a test asserts
one property per axis against what the provider actually emits, so the declaration is not
merely permitted but true. The Rust one states the principle outright: a declaration nobody
exercises is a comment rather than a fact.

That convention lives in those two module docs and nowhere else — no record carries it.
Whether it is a requirement every provider owes, or a practice two crates chose, is the
question this record answers.

## The census

Read per crate rather than grepped, because the distinction that matters is invisible to a
pattern. Three things are separated:

- **downward** — the ceiling admits the claim (`Ceiling().Satisfies(&Declared_Guarantee())`)
  or the registry accepts the offer (`registry.Offer(Provider_Offer()) == Ok(())`);
- **plumbing** — an emitted fact carries the declared value
  (`fact.guarantee == Declared_Guarantee()`);
- **upward** — a claimed property is asserted to hold of what the provider actually emits.

A fourth appears in the reading and is listed separately, because calling it upward would
overstate it: a **value-algebra** assertion, which checks what the declared `Guarantee`
satisfies or fails to satisfy as a value (`Declared_Guarantee().Satisfies(&requirement)`).
That is a fact about the type's ordering, not about anything the provider emitted.

Eleven crates declare a provider guarantee; between them they define **18**
`Declared_Guarantee()` functions, because five live in `nomos-repo-policy` and three in
`nomos-lang-rust`.

| crate | downward | plumbing | upward, on an axis of its own guarantee |
|---|---|---|---|
| `nomos-lang-rust` | yes | yes | **yes** — `tests/guarantee.rs`, one property per axis, including the axis it declares it fails |
| `nomos-lang-rust-scan` | yes | yes | **yes** — `Test_The_Declared_Unsoundness_Should_Be_Demonstrable` exhibits four real inputs the scanner mis-reports |
| `nomos-lang-go` | yes | yes | no |
| `nomos-lang-go-modules` | yes, plus value-algebra | yes | no |
| `nomos-lang-rust-cargo` | yes | yes | no |
| `nomos-lang-rust-clippy` | yes | yes | no |
| `nomos-lang-rust-deny` | yes | yes | no on any axis; see below |
| `nomos-lang-rust-compiler` | yes | yes | no — no `tests/` directory |
| `nomos-cap-requirement-trace` | yes | yes | no |
| `nomos-connector-coderabbit` | yes | yes | no |
| `nomos-repo-policy` (five) | yes | yes | no — no `tests/` directory |

**Two of eleven hold the convention, and one of the two stating it is not among them.**
`nomos-lang-go`'s module doc says "Upward, this crate's own tests assert what it actually
emits against the claim" — it names no test, and its only test file asserts the declared value
is carried and the registry accepts the offer. Its nearest candidate,
`Test_Declared_Guarantee_Should_State_A_Sound_Syntactic_File_Granular_Claim`, asserts the
value equals itself; a provider that emitted nothing at all would pass it. The crate that does
hold the convention without stating it, `nomos-lang-rust-scan`, has no test file named for a
guarantee — which is why file naming was not the measure taken here.

**`nomos-lang-rust-deny` is listed as no and deserves its sentence.**
`Test_A_Real_Run_That_Resolved_No_Workspace_Should_Be_Refused_Rather_Than_Reported_Clean` is a
real property assertion against a real subprocess, and a valuable one — it holds that a run
which could not look is refused rather than reported clean. But that is `OD-RULES-001`'s
absence rule, not an axis of this provider's guarantee, and counting it as upward would make
the census report a property nobody declared.

### The raw `Assurance::Sound` count answers neither question

255 occurrences across the workspace. **17 are in a `Ceiling()` and 18 in a
`Declared_Guarantee()`** — the only two places the value means *a contract's permission* or *a
provider's claim*. The other 220 are fixtures, doc examples and `Guarantee` values constructed
in `nomos-analysis`, `nomos-capability` and `nomos-rules` to exercise something else entirely;
65 are in a test file or `tests/` directory outright. So the single number is dominated by
occurrences that declare nothing, and no reading of it distinguishes a ceiling from a claim.
That is why it is split here rather than quoted.

The counts are per-file and per-function textual, and nested `#[cfg(test)]` modules are not
subtracted from the enclosing function's scope, so the 17 and 18 are exact (they count
definitions) while the 220 remainder is a difference and carries that method's slack.

## Decision

**A requirement, not a practice.** `Assurance::Sound` is the single value for which
`Satisfies_Requirement` returns true, so it is the one value that clears a rule's requirement
floor — and the downward direction establishes only that a claim is *permitted*, never that it
is *true*. The registry would accept a provider declaring `Syntactic` that secretly resolved
names, and one declaring `Sound` that invented items; `nomos-lang-rust`'s own test module doc
says exactly that. Nine of eleven crates clearing that floor on an unexercised claim is the
defect this workspace exists to remove, stated in its own words in two of its own files.

**What is required is a named exerciser at the declaring site, not a test per axis.** A
`Declared_Guarantee()` names, in its own doc, what exercises it — or states that it cannot be
exercised, and why. Both halves are the requirement; the second is not an escape from it.

The reason it is shaped that way rather than as "every axis owes a test" is that some axes are
not demonstrable and forcing a test for them produces exactly the tautology this census found:
`nomos-lang-go`'s value-equals-itself assertion, which a provider emitting nothing would pass.
A policy provider that reads a declared file and reports its contents may have no way to
exhibit unsoundness, and the honest answer there is a stated one, not a manufactured test.
`OD-RULES-001` is the same principle one level down: an absence must be declared rather than
silent.

**It is enforced mechanically, by the `completeness-mirror` precedent.** `checks/mirror.rs`
already reads a name declared at a site and resolves it against real parsed test functions,
reporting a phantom when it resolves to nothing. A guarantee declaring an exerciser is the
identical shape, and `OD-RULES-031` chose the same mechanism for a benchmark naming its oracle
one week of work earlier. What makes it worth the mechanism rather than a review habit is that
this census is the second time the convention has been found stated and unheld, and prose is
what failed both times.

## What this record does not do

It does not write any test, weaken any guarantee, or change any provider, contract, ceiling or
offer. Nine crates owe an exerciser or a stated reason under this decision and none of them
gains one here.

It does not amend the two module docs, which is outside this item's territory and is named as
the first thing the enforcing increment does. One of them needs it on accuracy grounds and not
only on convention: `nomos-lang-go/src/guarantee.rs` presently asserts that its own tests check
what it emits against its claim, and they do not.

It does not decide the rule's identifier, its gate category, or whether a missing exerciser is
blocking or advisory. Those belong with the increment that builds it, against the constraints
`OD-RULES-031` already fixed for the sibling mechanism.

It does not re-open what a `Guarantee` is, what its axes are, or what `Satisfies_Requirement`
returns. Every one of those is load-bearing here exactly as it already stands.

## Status

Accepted. The two-direction check is a requirement; a declared guarantee names what exercises
it or states that it cannot be exercised; and the naming is enforced the way a declared mirror
already is. No code moves here.
