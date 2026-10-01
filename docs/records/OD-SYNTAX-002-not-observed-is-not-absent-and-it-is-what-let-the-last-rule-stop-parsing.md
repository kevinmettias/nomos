---
id: OD-SYNTAX-002
type: decision
title: Not observed is not absent, and it is what let the last rule stop parsing
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - schema
  - providers
  - rules
  - completeness
relations:
  - target: OD-SYNTAX-001
    type: relates-to
  - target: OD-RULES-001
    type: affects
  - target: OD-COMPLETENESS-001
    type: relates-to
---

# Not observed is not absent, and it is what let the last rule stop parsing

## Question

`OD-RULES-001` moved check-name resolution onto the fact layer and left one module behind.
`nomos-rules/src/universe.rs` kept `syn` and `proc-macro2`, and the reason was a measurement
rather than a preference: universe discovery needs two things `nomos.syntax.items.v1` does
not carry — that a `pub const` is of *slice* type, and the doc comment at the declaration
site. So one workspace held two independent Rust front ends, one behind
`nomos.cap.syntax.items` and one vendored inside a rule that is not supposed to know which
provider answered it.

That record named its own end condition and this item is it. The interesting part is the
sentence it added immediately afterwards, because it is the reason the version was not cut
there and then: **the two fields cannot be added as optional strings.**

## Why Neither Field Could Be A String

`nomos-lang-rust-scan` cannot read a doc comment at all. It skips comment lines and
associates nothing with the item below them, and it never looks at a declared type. A schema
in which "this item has no doc comment" and "this provider does not read doc comments" are
the same bytes would make every universe read through the scanner arrive as a universe
declaring no mirror.

That is not a loss of detail. It is a change of verdict, and it runs the wrong way. The
severity ordering the completeness-mirror rule inherits from `D-134` puts a *phantom* mirror
— a claim of coverage that resolves to nothing — above an *admitted* gap, because a false
claim is worse than an honest silence. An empty documentation field turns the first into the
second. Absence becomes success in the one field the whole ordering turns on, and it does so
silently, at exactly the moment a weaker provider is admitted.

An `Option` cannot express this, and the reason is not ergonomic. `None` is one value and
there are two empty answers: *the provider looked and there is nothing here*, and *the
provider cannot look*. Only the first is an observation about the file. The second is an
observation about the provider, and a consumer that acts on it as though it described the
file is reading blindness as a measurement — the same defect `OD-SYNTAX-001` refused for the
empty payload, which is a fact carrying no bytes rather than a file declaring nothing.

## The Decision

**`nomos.syntax.items.v2` carries, per item, what the answering provider actually observed
about the item's documentation and its declared shape — in three spellings, so that *not
observed* and *observed and absent* are different bytes.**

```
item        := "item" TAB ordinal TAB kind TAB visibility TAB qualified-name
               TAB documentation TAB shape LF
observation := "-" | "." | "+" escaped
```

`-` is not observed, `.` is observed and there is none, `+…` is observed and here it is.
`Observation` in `nomos-cap-syntax` is the type, and it has `Value()` for a caller that only
wants the text and `Was_Observed()` for a caller that must not confuse the two. A field that
is neither mark and does not begin with `+` is refused rather than read as absent, because
absent is one of the answers.

**Observation is bounded by the provider's guarantee, not by its effort.** A provider writes
`-` when the method it used cannot see the thing, which is a property of the method — the
same property its declared `Guarantee` already describes. The two providers are asymmetric by
construction and each says so at its own encoder: `nomos-lang-rust` parses, so every
observation it writes is a real one and the not-observed mark is unreachable from it;
`nomos-lang-rust-scan` writes `-` in both fields, always, and a test asserts it. That
divergence is the thing v2 exists to make statable, so it is asserted at both ends rather
than described here.

**`shape` is open like the other vocabularies and is read relative to the kind.** A typed
declaration is `slice` or `value`; a function is `fn/<arity>`; an implementation block is
`inherent` or `trait`; anything else is `.`, observed and with nothing to say. Each
distinction is one a consumer cannot recover from the rest of the record: `pub const LIMIT:
usize` and `pub const TABLES: &[&str]` are otherwise identical, `fn All()` and `fn All(&self)`
are otherwise identical, and an `All` in `impl Display for Table` is not the `All` a variant
list declares. The last of those is why source order became load-bearing rather than
cosmetic — `SyntaxPayload::Enclosing` attributes a member to the most recent record it
follows, and a consumer that sorted the items would attribute members to the wrong owner.

**`CONTRACT_VERSION` did not move.** The question a caller asks has not changed; only the
shape of the answer has, and `OD-SYNTAX-001` is the record that separated those two.

**Visibility stays a plain label, and that is a decision rather than an omission.** Making it
an `Observation` would say the scanner did not observe visibility, which is false — it reads
`pub` correctly and merely cannot see the enclosing trait. The asymmetry there is about what
a mark *means*, not about whether anyone looked, and `OD-RULES-001`'s floor is what keeps the
blind spelling away from the filter that would misread it.

## The Guarantee That Changed

This is the part that is not a schema addition, and it is stated plainly because six tests
had to be rewritten to say it.

**A universe declared in a file whose syntax fact was never read is no longer discovered.**

Discovery reads facts now. Where there is no fact — the file did not parse, no provider met
the floor, the store held nothing, the payload was stamped with a schema this build does not
read — there is no universe either, and the rule cannot report the claim that file made. What
it reports instead is the *subject*, once, carrying the `Applicability` the reader returned.
Nothing renders clean: a file that could not be read is still a finding, still non-blocking,
and still says which unavailability it was. What is gone is the per-claim detail inside those
files.

The old behaviour looked more informative and was the rule reaching around its own fact
layer. It found the universe by parsing the text it had been handed, then reported the claim
as withheld because the check index was short — one reading of the file for the claim and a
different one for the names. That is the bypass `OD-RULES-001` closed for check names, still
open in the other half of the same rule, and closing it is what removes the second front end
rather than merely relocating it.

The exchange is worth naming as an exchange. A count moved: a scenario that once produced
three findings now produces two. That is a real reduction in what the run says about
unreadable files, taken deliberately, in return for one reading of one file behind one
capability. Each of the six rewritten tests carries the reasoning at its own site so the
change is not discoverable only from here.

The new failure mode this creates has its own test rather than a paragraph.
`Test_A_Universe_Read_Through_A_Blind_Provider_Should_Not_Be_An_Admitted_Gap` files a
scanner-shaped payload under an admitted guarantee and asserts that the rule reports a file
it could not judge, naming the field it could not judge it by — and does not report a list
declaring no mirror. Today's floor keeps that provider out; the test is what happens when
some future provider is admitted and still cannot read documentation, which is the case this
whole record exists for.

## Every Fact Keyed Under The v1 Bytes Has Been Re-Addressed

`PARSED_GOLDEN` and `SCANNED_GOLDEN` in `tests/integration/tests/determinism.rs` moved. Two
fields per item changed the bytes, and a payload digest is a content address, so every fact
ever filed under a v1 payload now addresses differently.

That file's own comment asks for this to be a sentence somebody has to read before merging,
and this is the sentence. No store in this repository is invalidated by it, because no fact
in this repository outlives a run; a store that persisted facts across this change would find
every syntax fact in it unreachable by digest and would have to be rebuilt rather than
migrated. The two goldens that did not move — the snapshot and the bundle — did not, and the
diff is the evidence for which encodings this reached.

## A Provider In The Rule's Tests, In Dev Only

`nomos-rules` gained `nomos-lang-rust` as a **dev-dependency** at the same commit that took
`syn` out of its dependencies, and the two are not in tension.

A rule names a capability and lets the registry choose who answers it. A *test* of that rule
needs bytes some real provider would actually have written. Hand-written fixture payloads
were doing that job and drifted the moment the schema gained a field — a payload nobody could
have produced is a subject the rule is not really being tested against, which is the same
objection in miniature that `OD-SYNTAX-001` raised against three hand-written readers.

So the fixtures are encoded by the real parser and the check names a test wants resolved are
stated on top, separately. Both halves are deliberate. Deriving the items means a fixture's
universes reach the rule the way they do in the product; stating the checks separately keeps
the tests honest, because a `Test_X` written inside a fixture string is *not* in the parser's
output, and a test that wants one resolved has to say so rather than smuggle it through the
text.

`tests/contract` took the same edge for the same reason: it quantifies over the universes
this workspace declares, and the walk that derives them now goes through the provider and its
encoding, because that is the subject the rule is handed at run time.

The band direction is the safe one either way — 30 observing 25, and 100 observing 23 and 25.
What would be wrong is a rule that reaches a named provider in the product, and the dev table
is where that distinction is kept. A rule that grows a parser again has stopped taking its
subject as an argument, and the remedy then is a capability, not a dependency line here.

## What This Does Not Do

- **It does not amend `OD-RULES-001`.** That record named this outcome as its own end
  condition and attached it to this item; the exception expiring on schedule is the record
  working, not being corrected. What is no longer true of the tree is the residual front end
  it describes, and the edge from here is what says so.
- **It does not close the `kind` or `visibility` vocabularies**, and it does not enumerate
  `shape` either. A provider that can distinguish fewer forms must stay able to answer
  honestly, and a closed vocabulary is how that stops being possible.
- **It does not make the scanner useful to this rule.** The floor still excludes it. What
  changed is that the day some provider below the parser is admitted, the rule refuses to
  guess instead of quietly agreeing with it.
- **It does not settle whether a `Function` record is a definition or a signature.** That is
  still not a property of every conforming answer, still obtainable only from the guarantee a
  consumer required, and `OD-SYNTAX-001` is still where it is written down.
- **It does not remove `Render_Payload`'s hazard.** Nothing stops a provider from calling the
  canonical encoder and making interchangeability true by construction; that is still a note
  rather than a mechanism, and v2 adds two more fields it would make agree for free.

## Status

Accepted, landed by `P10-SYNTAX-V2`.
