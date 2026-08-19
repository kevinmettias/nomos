---
id: OD-RULES-006
type: decision
title: Whether RuleOffer needs a declared EnforcementPlacementRationale, or waits for a second rule
status: open
version: 1
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

## Status

Open. This question is deliberately left open rather than resolved either way: unlike
`OD-PACKAGE-006`, the plugin-enablement direction does not reach it — a rule package can
already register fully without this field existing — and no second rule or rule package
exists to check the design against. Revisit when either trigger above arrives.
