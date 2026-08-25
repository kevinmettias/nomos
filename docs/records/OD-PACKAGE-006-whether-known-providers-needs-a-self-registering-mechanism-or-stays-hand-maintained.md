---
id: OD-PACKAGE-006
type: decision
title: Whether KNOWN_PROVIDERS needs a self-registering mechanism, or stays hand-maintained bounded to additions
status: accepted
version: 4
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
  - target: OD-ROADMAP-001
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

Superseded by `OD-ROADMAP-001` — see the amendment below. This section is left as written
for the record of what the original trigger was, not because it still governs.

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

## Amendment: The Remaining Third-Provider Wait Is Retired

Added at version 3. Version 2's resolution built the generic registration type ahead of a
second language, but left one narrower question open on its own original trigger: whether a
same-language *third* Rust provider crate would need anything beyond "one more line, pulled
not retyped." `OD-ROADMAP-001` retires waiting for that trigger too, as part of the same
cluster of repeatedly-overridden population cautions it names. If a self-registering
mechanism for same-language providers turns out to be worth building, it does not need a
third provider crate to exist first — the same standing instruction that applies to
`RulePackage`, model routing, and corrections applies here.

## Amendment: The Second Real Consumer Landed, And Needed The Type Unchanged

Added at version 4. Version 2's resolution built `nomos_package::KnownProviders` ahead of a
second language's package crate, anticipating "a second language's package crate is exactly
the expected second party ... and it would need a `KNOWN_PROVIDERS`-shaped array of its own the
moment it existed." `nomos-lang-go-package` (`P14-PACKAGE-GO-LANGUAGE-MANIFEST-3`) is that
party, real rather than anticipated, and this amendment checks the prediction against it —
`OD-PACKAGE-008`'s own amendment discipline, applied here rather than merely cited.

**`KnownProviders` needed no change.** `crates/packages/nomos-lang-go-package/src/
known_providers.rs` builds `KNOWN_PROVIDERS` as `KnownProviders::New(&[nomos_lang_go::
PROVIDER]).As_Slice()` — the identical construction `nomos-lang-package`'s own
`known_providers.rs` uses, for a wholly unrelated language's provider constant. Not one line
of `nomos_package::known_providers` is in `P14-PACKAGE-GO-LANGUAGE-MANIFEST-3`'s own territory,
because the type served a second real consumer exactly as built.

**The remaining unmirrored-universe cost repeated once, not accumulated.** `nomos-lang-go-
package::KNOWN_PROVIDERS` is a second hand-maintained, unmirrored universe, classified in
`tests/contract/tests/completeness_universes/table.rs` beside `nomos-lang-package`'s own row,
with the identical bounded risk — an addition to the list nothing forces, drift on what exists
today structurally prevented by pulling each entry from its own crate's `PROVIDER` constant.
`UNMIRRORED_TOTAL` rose by exactly one for exactly one new hand-maintained list, not by more:
the generic type did not multiply the cost this record already accepted, it repeated it once
per real consumer, which is the shape a population-of-N-crates cost was always going to take
regardless of whether the underlying mechanism were generic or bespoke.

## Status

Accepted. `OD-PACKAGE-006`'s original trigger — a third Rust language provider crate —
remains unmet and was explicitly not what resolved version 2; a separate, explicit product
direction to build language- and rule-plugin infrastructure ahead of demonstrated
same-population need is what did. Amended to version 3 by
`P13-ROADMAP-001-POPULATION-CAUTION-RETIRED`, retiring the remaining third-provider wait via
`OD-ROADMAP-001`. Amended to version 4 by `P14-LANG-GO-RECORD-2`: `nomos-lang-go-package`
confirms the generic `KnownProviders` type serves a second real consumer, unchanged. The
follow-on capability work this resolution names is tracked on the work ledger, not in this
record.
