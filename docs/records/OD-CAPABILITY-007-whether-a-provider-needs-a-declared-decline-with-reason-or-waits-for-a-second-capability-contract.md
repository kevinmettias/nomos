---
id: OD-CAPABILITY-007
type: decision
title: Whether a provider needs a declared decline-with-reason, or waits for a second capability contract
status: open
version: 1
authority: canonical-normative-record
tags:
  - capability
  - packages
  - architecture
relations:
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
  - target: OD-RULES-005
    type: relates-to
  - target: D-135
    type: relates-to
---

# Whether a provider needs a declared decline-with-reason, or waits for a second capability contract

## Question

`code-standards`/`nomos-proto` (`github.com/kevinmettias/nomos-proto`), an earlier Go
implementation of the same tool this workspace rebuilds in Rust, gives every language
kernel a `<lang>_declines.go` file: `DeclinedCapabilities.Declined_Capability(capability) ->
(reason, declined)`, letting a provider state, ahead of time and in one discoverable place,
that it will never offer against a given capability, and why. Rust's own decline map in that
codebase runs to roughly seventy reasoned entries in production — refusing capabilities like
"untypedaccess," "disposal" and "exhaustive" because the type system and borrow checker
already make the defect the capability would report unrepresentable.

`crates/substrate/nomos-capability/src/provider_offer.rs` carries `ProviderOffer{provider,
capability, version, guarantee}` with no equivalent. `offer_refusal.rs`'s `OfferRefusal` is a
different concept entirely: `ForUndeclared`, `ExceedsCeiling` and `Duplicate` are why the
*registry* rejects a submitted offer, not a statement a *provider* makes in advance that it
will never submit one. Today, a provider that structurally cannot satisfy a capability and a
provider that was simply never asked about it look identical: both are silent absence from
`Registry::Offers`.

The open question is whether `ProviderOffer`/`Registry` gains a decline-with-reason
mechanism now, before a second capability contract multiplies the migration cost of adding
it later, or whether it waits for one to exist so the design has a real case to check
against.

## Current Position

Exactly one capability contract exists in this workspace today:
`nomos.cap.syntax.items`, declared by `nomos-cap-syntax` (`OD-CAPABILITY-002`) and offered
by both `nomos-lang-rust` and `nomos-lang-rust-scan` — and both of those providers actually
offer against it, at different `Guarantee` levels. Neither is a decline; a decline is a
provider that structurally cannot ever satisfy a contract it could plausibly be asked about,
and this workspace has no such case yet, because it has no second capability contract for a
provider to structurally fail to satisfy.

This is the same population-of-two shape `OD-PACKAGE-006` originally named for
`KNOWN_PROVIDERS`, and the same shape `OD-RULES-005` left open for `RuleOffer`'s Inputs
classification. `OD-PACKAGE-006` was resolved anyway, on an explicit, standing product
direction to invest in language- and rule-*plugin* infrastructure ahead of demonstrated
need, so a package could be developed on a thread separate from this workspace's own core
work. That direction is about the seam a second party registers through. A decline-with-
reason mechanism is not that seam: `Registry::Declare_And_Offer` already lets a second
provider register fully today, against any contract that exists, with no decline mechanism
present at all. Nothing about a second language *package*'s ability to register is blocked
by this absence — a decline is a refinement of what an existing provider states about a
contract it could offer against, not a precondition for offering at all. The extract-early
precedent that resolved `OD-PACKAGE-006` does not automatically transfer here for that
reason, the same conclusion `OD-RULES-005` already reached for the adjacent Inputs
question.

## What Would Decide It

A second capability contract is the natural trigger — the moment a real provider exists
that could plausibly be asked to offer against a contract it structurally cannot satisfy,
the question becomes concrete: what a decline actually needs to say, and where a
completeness check would read it from, rather than a mechanism designed against zero real
declines, which is exactly the mistake `D-135` warns building generic machinery from a wish
produces.

A second, independent trigger: if a completeness or coverage check is found to need to
distinguish "never asked" from "structurally declines" for a capability that already
exists today — for instance, if `nomos-lang-rust-scan`'s narrower `Guarantee` on
`nomos.cap.syntax.items` is ever read by a consumer as silence rather than as the offer it
actually is, and that silence is mistaken for a gap. If that need surfaces against an
existing contract rather than a hypothetical second one, it argues for building the
mechanism for a different reason than the one considered and declined here, and should be
evaluated on its own evidence.

## Status

Open. This question is deliberately left open rather than resolved either way: unlike
`OD-PACKAGE-006`, the plugin-enablement direction does not reach it — a provider can
already register fully without a decline mechanism existing — and no second capability
contract exists to check a design against. Revisit when either trigger above arrives.
