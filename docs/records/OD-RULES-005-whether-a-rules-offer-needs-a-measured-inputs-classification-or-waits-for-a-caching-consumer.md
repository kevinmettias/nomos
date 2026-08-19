---
id: OD-RULES-005
type: decision
title: Whether a rule's offer needs a measured Inputs classification, or waits for a caching consumer
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
  - target: D-135
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
---

# Whether a rule's offer needs a measured Inputs classification, or waits for a caching consumer

## Question

`code-standards`/`nomos-proto` (`github.com/kevinmettias/nomos-proto`), an earlier Go
implementation of the same tool this workspace rebuilds in Rust, declares a closed, optional
`checkspec.Inputs` enum on every check — `INPUT_SCOPED_FILES`, `INPUT_WHOLE_CORPUS`,
`INPUT_REPOSITORY_CONFIG`, `INPUT_EXTERNAL_TOOL` — stated by the check itself, defaulting to
undeclared (which disables caching rather than wrongly enabling it), and explicitly
*measured*, not typed from reading the source: "a plausible list would be indistinguishable
from a measured one while carrying none of the evidence."

`crates/rules/nomos-rules/src/registry.rs` (`OD-RULES-004`) carries `RuleOffer{ rule,
contract_record, contract_record_version }` with no equivalent. A rule states which
capability contract it reads through `Syntax_Requirement` — but nothing states what *kind*
of thing its answer depends on: a single file, the whole corpus, repository configuration,
an external tool. `crates/substrate/nomos-analysis/src/invalidation.rs` already treats
invalidation as a first-class fact-layer concern, so a rule-level classification would
extend an existing discipline rather than introduce one from nothing.

The open question is whether `RuleOffer` gains that classification now, before nomos-rules'
two rules and any future rule package multiply the migration cost of adding it later, or
whether it waits for a real consumer to demonstrate the need.

## Current Position

`registry.rs`'s own module doc states plainly: "Nothing here is consulted by `Run()`, and
nothing here changes what runs on any given `nomos check`." There is no caching or
invalidation consumer for a rule-level Inputs classification anywhere in this workspace
today — `nomos-analysis`'s invalidation module operates on facts, not on rules, and nothing
downstream of `RuleRegistry` reads `RuleOffer` for any purpose beyond registration itself.

This is the same population-of-two shape `OD-PACKAGE-006` originally named for
`KNOWN_PROVIDERS`: a plausible generalization with no second real consumer to check the
design against. `OD-PACKAGE-006` was resolved anyway, but on a different basis than "a
consumer exists" — an explicit, standing product direction to invest in language- and
rule-*plugin* infrastructure ahead of demonstrated need, so a package could be developed on
a separate thread from this workspace's own core work. That direction is about the seam a
rule package registers through, which `OD-RULES-004`/`RuleRegistry` already provides in
full: a rule package can register a `RuleOffer` today without an Inputs classification
existing at all. An Inputs classification is a caching/invalidation quality concern, not a
plugin-enablement one — it does not gate whether a rule package can be built independently,
the way `KNOWN_PROVIDERS`' shape did gate a second language's package crate. The extract-
early precedent that resolved `OD-PACKAGE-006` does not automatically transfer here for that
reason, even though the surface shape of the question looks similar.

## What Would Decide It

A real consumer is the natural trigger: `nomos_check_orchestration::run::Run`, or some
successor, wanting to skip or cache a rule's re-run based on what changed since its last
run. At that point the question becomes concrete — what Inputs vocabulary an actual caching
mechanism needs — rather than a classification designed against zero real uses, which is
exactly the mistake `D-135` warns building generic machinery from a wish produces.

A second, independent trigger: if a rule package (native or contributed) is found to need
`RuleOffer` to state something about its own dependency shape for a reason *other* than
caching — for instance, so a caller can refuse to run a rule against a scope it structurally
cannot answer for, the way `nomos-proto`'s own `INPUT_WHOLE_CORPUS` distinguishes a
project-wide check from a per-file one before the check runs, not only for cache
invalidation after. If that need surfaces, it argues for building the classification for a
different reason than the one considered and declined here, and should be evaluated on its
own evidence rather than folded into this record's answer.

## Status

Open. This question is deliberately left open rather than resolved either way: unlike
`OD-PACKAGE-006`, no direction has been given to build ahead of need here, and no consumer
exists to build against. Revisit when either trigger above arrives.
