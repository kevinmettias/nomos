---
id: OD-PACKAGE-006
type: decision
title: Whether KNOWN_PROVIDERS needs a self-registering mechanism, or stays hand-maintained bounded to additions
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - packages
  - providers
  - architecture
relations:
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
  - target: D-134
    type: relates-to
  - target: OD-PACKAGE-007
    type: relates-to
---

# Whether KNOWN_PROVIDERS needs a self-registering mechanism, or stays hand-maintained bounded to additions

## Question

`crates/packages/nomos-lang-package/src/known_providers.rs` declares `KNOWN_PROVIDERS`, a
two-element array pulling `nomos_lang_rust::PROVIDER` and `nomos_lang_rust_scan::PROVIDER`
by reference rather than retyping either string. `reader.rs` calls `Is_Known` against it
before accepting a manifest's `ProviderRegistration`, so a manifest naming a provider not
in the array is refused, not silently misregistered.

`tests/contract/tests/completeness_universes/table.rs` classifies `KNOWN_PROVIDERS` as an
unmirrored declared universe: nothing compares it against an independently-discoverable
reality the way `Table::All`, `SHIPPED` or `MIGRATIONS` are compared against theirs. Its
named risk is narrow — "a third Rust language provider crate added to the workspace is not
added to this list automatically... until somebody notices and extends it by hand" — and it
is bounded to additions rather than drift on the two entries that exist today, because both
are pulled from their own crates' `PROVIDER` constants rather than copied.

The open question is whether that bound is good enough permanently, or whether
`nomos-lang-package` should grow a generalized registration mechanism — each provider crate
declaring itself into a shared registry this crate discovers, rather than a literal array a
person extends by hand at this crate's own declaration site.

## Current Position

`OD-HOST-004` decided a directly analogous shape for `nomos_check_orchestration`: a second
hand-written `Declare_And_Offer` call is not the seam-by-accretion failure its originating
item warned about, because participation there depends on the request, and a registry
already exists underneath (`nomos_capability::Registry`) for exactly the case where
selection — not mere enumeration — is needed. `KNOWN_PROVIDERS` is not that shape: it does
no ranking or selection, only membership-checking, so `OD-HOST-004`'s decision does not
transfer directly. It is offered here as relevant precedent for the same underlying
judgment — that hand-accretion is not automatically a defect at low count — not as a
decision already made for this crate.

Two real Rust language provider crates exist in this workspace today
(`nomos-lang-rust`, `nomos-lang-rust-scan`), and the array holding them is one line each,
pulled rather than retyped. Building a self-registering mechanism now would be designed
against a population of two, with no third provider crate to check the design against.

## What Would Decide It

A third real Rust language provider crate joining this workspace is the natural trigger:
at that point, either "one more line, pulled not retyped" still holds and the array stays
hand-maintained, or the friction of a person having to notice and extend it by hand becomes
the cost this record predicted, and a registration mechanism earns its place. `OD-PACKAGE-001`
governs what such a provider crate would need to be a real `LanguagePackage` participant in
the first place; this record only covers how `nomos-lang-package` would come to know about
its `ProviderId`.

## Resolution

Accepted, not by the trigger this record originally named. No third Rust language provider
crate has joined this workspace; that population is still two, and nothing about
`KNOWN_PROVIDERS` itself has drifted. What changed is the direction, not the evidence: a
person has given explicit instruction to invest now in the infrastructure and base types a
language or rule plugin needs, specifically so that plugin can be developed on a separate
thread from nomos proper's own work, rather than waiting for a population this record's
"What Would Decide It" section already conceded might never grow before a genuinely new
language does instead.

`OD-PACKAGE-007` already anticipated a second party at this exact seam without waiting for
one to exist: it split `nomos-package` (band 24) out from `nomos-lang-package` (band 26) as
"the language-agnostic manifest core," precisely so a second language's package crate could
depend on the generic core instead of the Rust-specific wrapper — a second language's package
crate is already load-bearing in `README.md`'s own band table, not a hypothetical. The
`nomos-add-plugin` skill states the same principle explicitly for a capability contract:
"pull the contract out at the moment a second party is expected to offer against it, not only
once one actually has," citing `nomos-cap-syntax`'s own extraction from `nomos-lang-rust` as
the worked example. This record applies that already-endorsed principle to registration,
which `OD-PACKAGE-007` did not cover: a second language's package crate is exactly the
expected second party `OD-PACKAGE-007` was cut for, and it would need a `KNOWN_PROVIDERS`-
shaped array of its own the moment it existed. Building that array's shape once, generically,
at `nomos-package`'s band — rather than once per language, hand-copied from
`nomos-lang-package`'s pattern — is the same extraction this record's own "Current Position"
section already reasoned about correctly for the *wrong* population (same-language provider
count) and now resolves for the right one (language count).

**The decision:** a generic provider-registration type belongs in `nomos-package`, populated
by a package crate with its own known providers' `PROVIDER` constants pulled by reference —
the same pull-not-retype discipline `KNOWN_PROVIDERS` already follows, generalized rather than
duplicated. `nomos-lang-package` becomes this type's first real consumer, refactoring
`known_providers.rs` to build `KNOWN_PROVIDERS` through it rather than as a bespoke array,
with `Is_Known`'s refusal semantics unchanged. This is additive at `nomos-package`'s band; it
does not require a same-language provider count to change, does not touch `nomos-lang-rust`
or `nomos-lang-rust-scan`, and does not decide anything about a same-language third provider
that this record's original trigger still leaves genuinely open — should one arrive, whether
it needs anything beyond "one more line, pulled not retyped" through the now-generic type is
a question this resolution does not reach.

## Status

Accepted. `OD-PACKAGE-006`'s original trigger — a third Rust language provider crate — remains
unmet and is explicitly not what resolved this; a separate, explicit product direction to
build language- and rule-plugin infrastructure ahead of demonstrated same-population need is
what did. The follow-on capability work this resolution names is tracked on the work ledger,
not in this record.
