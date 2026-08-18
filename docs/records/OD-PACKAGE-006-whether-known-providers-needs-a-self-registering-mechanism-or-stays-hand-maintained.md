---
id: OD-PACKAGE-006
type: decision
title: Whether KNOWN_PROVIDERS needs a self-registering mechanism, or stays hand-maintained bounded to additions
status: open
version: 1
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

## Status

Open. Revisit when a third Rust language provider crate is added to this workspace, or if
the two-entry array is found to have drifted despite being pulled rather than retyped.
Recorded here so the intent is legible rather than lost between sessions, not because
either provider crate's shape has moved.
