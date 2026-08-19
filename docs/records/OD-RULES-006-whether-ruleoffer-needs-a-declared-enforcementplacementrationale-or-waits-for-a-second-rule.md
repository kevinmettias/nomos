---
id: OD-RULES-006
type: decision
title: Whether RuleOffer needs a declared EnforcementPlacementRationale, or waits for a second rule
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - rules
  - packages
  - architecture
relations:
  - target: OD-RULES-004
    type: relates-to
  - target: OD-RULES-005
    type: relates-to
  - target: D-135
    type: relates-to
  - target: D-134
    type: relates-to
---

# Whether RuleOffer needs a declared EnforcementPlacementRationale, or waits for a second rule

## Question

`docs/nomos-spec-internal-artifacts/01_authoring/markdown_volumes/
04-04-packages-providers-rules-and-applicability/` — the same corpus tree this workspace
already treats as its v14 corpus — states requirement `PKG-028`: every proposed
`RulePackage` normative contract must carry an `EnforcementPlacementRationale`, stating why
the rule is not better solved by the compiler, the type system, the build, a formatter, or a
linter.

`crates/rules/nomos-rules/src/registry.rs` (`OD-RULES-004`) carries `RuleOffer{rule,
contract_record, contract_record_version}` with no such field, and nothing in
`nomos-rules` today requires a rule to justify why it exists as a *rule* rather than as one
of those alternatives. `crates/rules/nomos-rules/src/mirror.rs`'s own module doc gives an
informal version of this justification in prose for `Check_Completeness_Mirrors` — that a
declared universe's coverage cannot be verified by the compiler, because the claim is about
what the source *fails to name*, not about what it states — but that reasoning lives in a
doc comment a reader has to find, not in a field a registration carries.

The open question is whether `RuleOffer` gains a declared
`EnforcementPlacementRationale`-shaped field now, before nomos-rules' rules and any future
rule package multiply the migration cost of adding it later, or whether it waits for a
second rule, or a rule package, to make the absence concrete.

## Current Position

`registry.rs`'s own module doc states plainly that nothing consults `RuleOffer` yet and
`Run()` stays hand-written and unconditional — the same position `OD-RULES-005` already
found for the adjacent Inputs-classification question on this identical struct. No call
site outside `registry.rs`'s own test helper constructs a `RuleOffer` today; `nomos-rules`'
one real rule, `Check_Completeness_Mirrors`, does not register one.

This is the same population-of-two shape `OD-PACKAGE-006` originally named for
`KNOWN_PROVIDERS`, and the shape `OD-RULES-005` already declined to extend the plugin-
enablement direction to. That direction is about the seam a rule package registers
through — `RuleRegistry::Offer` already accepts a `RuleOffer` today with no rationale field
present, so a rule package can register fully without one existing. An
`EnforcementPlacementRationale` is a contract-quality concern — a stronger requirement on
what a registered rule must state about itself — not a precondition for the seam existing at
all, the same distinction `OD-RULES-005` drew for Inputs. Nothing about a second rule
package's ability to register is blocked by this absence.

## What Would Decide It

A second rule is the natural trigger. `Check_Completeness_Mirrors` is the only rule in this
tree with three recorded historical instances to test a judgment against — a genuinely
different second rule, native or from a package, would be the first real case where two
rules' rationales could be compared side by side, which is what would tell whether a closed,
structured field is worth the migration or whether `mirror.rs`'s own prose-doc-comment
pattern already carries the weight PKG-028 wants.

A second, independent trigger: if a rule package's own review process — human or
mechanical — is found to need to ask "why is this a rule and not a lint" as a gate a
registration must pass, rather than as a question a reviewer asks by reading source. If that
concrete review need surfaces before a second rule ships, it argues for building the field
for a different reason than the one considered and declined here.

## Resolution

Accepted, on the trigger this record itself named. `crates/rules/nomos-rules/src/naming.rs`
(`Check_Naming_Convention`, `P13-RULE-NAMING-CONVENTION`) is now nomos-rules' second real
rule, and the side-by-side comparison this record asked for is possible on real evidence
rather than a population of one.

The comparison does not show what the "Current Position" section above predicted. It is not
that both rules carry the same informal-prose pattern; the two carry placement rationale in
genuinely different shapes. `naming.rs` is explicit and structurally marked: its module doc
opens by naming exactly what PKG-028 asks for — "the one rustc lint that would have caught a
deviation from Rust's own convention... is turned off... and nothing was turned on to check
the convention that replaced it. This rule is that check" — and carries a dedicated `# Why
this has no CONTRACT_RECORD` section besides. `mirror.rs`, re-read for this resolution, does
not: nowhere in `mirror.rs` or its `mirror/` submodules does the text name the compiler, the
type system, a formatter or a linter, or otherwise frame itself against PKG-028's alternative
list. Its module doc argues *what* the rule is for — "a false claim of coverage is worse than
an admitted gap" — not why that judgment could not have been placed somewhere other than a
rule. The "informal version of this justification" this record's own "Question" section
credited to `mirror.rs` is this resolution's own paraphrase of that reasoning, not a
quotation, and does not hold up as one on a second, closer read. `D-134`, the contract
`mirror.rs` cites by `CONTRACT_RECORD`, does not carry a `PKG-028`-shaped rationale either.

That is a real inconsistency, not a confirmation that prose already carries the weight
uniformly — the two rules genuinely disagree about whether this gets written down at all,
which is closer to the risk a structured field would guard against than to evidence a field
is unnecessary. It does not change the answer, for a narrower reason than "prose is
consistent": a field on `RuleOffer` would not fix this particular inconsistency, because
`RuleOffer` is a runtime registration struct — `rule`, `contract_record`,
`contract_record_version` — populated at the call site that offers a rule into a
`RuleRegistry`, not the rule's normative contract itself. PKG-028 asks for the rationale on
"every proposed `RulePackage` normative contract," the record a rule cites (`D-134` for
`Check_Completeness_Mirrors`; nothing versioned for `Check_Naming_Convention`, which cites
`README.md` prose instead and says why in its own `# Why this has no CONTRACT_RECORD`
section). Adding a `String` field to `RuleOffer` would let a registration carry a rationale
that nothing checks for content, the same "a plausible list would be indistinguishable from
a measured one while carrying none of the evidence" concern `OD-RULES-005` already raised
against a parallel field on the same struct — it would prove a caller typed something at
`Offer()` time, not that the reasoning is real, is current, or was ever the rule author's
actual justification, and it would not touch `mirror.rs`'s own already-written, already-
inconsistent module doc at all. The gap this comparison surfaced is a documentation-
consistency gap in the two rules' own module docs and (for `mirror.rs`) in `D-134` itself,
not a registration-struct gap `RuleOffer` is positioned to close.

`RuleOffer` gains no new field. The registration seam `OD-RULES-004` built is unchanged, and
a rule package can register fully today exactly as it could before this resolution, which is
the distinction `OD-RULES-005` already drew and this resolution applies again: the standing
direction to build language- and rule-plugin infrastructure ahead of a second real instance
is about that seam existing, not about mechanically enforcing every corpus-named quality
field the moment a second data point makes it nameable. Whether `mirror.rs`'s own module doc
or `D-134` should be brought into line with `naming.rs`'s more explicit shape is a documentation
question about one existing rule, left to whoever next touches `mirror.rs`'s contract, and is
not this record's own question to reach.

## Status

Accepted. The trigger this record named — a second, genuinely different rule — has arrived
and was compared against the first on real evidence, not speculation. The comparison argues
against a structured `EnforcementPlacementRationale` field on `RuleOffer` for a narrower
reason than originally anticipated: not because the two rules already agree, but because the
inconsistency between them is a documentation gap in each rule's own module doc and
governing contract, which a field on a runtime registration struct would not close. The
second, independent trigger this record also named — a rule package's own review process
needing this as a mechanical gate — has still not arrived, and would still argue for
building the field on its own evidence rather than this comparison's, should it arrive.
