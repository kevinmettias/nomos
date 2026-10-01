---
id: OD-CAPABILITY-007
type: decision
title: Whether a provider needs a declared decline-with-reason, or waits for a second capability contract
status: open
version: 2
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

## A Second Capability Contract Arrives: What It Does And Does Not Settle

`nomos-cap-dependency` (`nomos.cap.dependency.edges`) was built after this record was
written and is, checked directly rather than assumed, a real second capability contract —
its own crate doc states it earned that immediately, "band 23... owned by neither its
provider nor any rule that reads it," with `nomos-rules::Check_Dependency_Direction` as its
real second party. This record's own first named trigger — "a second capability contract" —
has arrived in the literal sense the sentence names.

What it has not done is give the decline question a real case to check a design against.
Verified directly against the real code: exactly one provider, `nomos-lang-rust-cargo`,
offers against `nomos.cap.dependency.edges` (grepped for every reference to
`nomos_cap_dependency::Capability` across the workspace — the provider itself, the rule that
reads it, and the contract crate are the only three sites), and its offer
(`crates/languages/nomos-lang-rust-cargo/src/guarantee.rs::Provider_Offer`) sits at the
contract's own declared ceiling — `Sound`/`Sound` — the same as `nomos-cap-syntax`'s
strongest existing offer, not a narrower one. No provider anywhere in this workspace
structurally cannot satisfy this contract; the one provider that offers against it satisfies
it maximally. The trigger this record actually cares about — "the moment a real provider
exists that could plausibly be asked to offer against a contract it structurally cannot
satisfy" — has still not fired: a second contract now exists, but nothing has yet failed to
meet one.

## What Would Decide It

A second capability contract is the natural trigger — the moment a real provider exists
that could plausibly be asked to offer against a contract it structurally cannot satisfy,
the question becomes concrete: what a decline actually needs to say, and where a
completeness check would read it from, rather than a mechanism designed against zero real
declines, which is exactly the mistake `D-135` warns building generic machinery from a wish
produces. That contract now exists (`nomos-cap-dependency`), but the concrete case it was
meant to supply has not arrived with it: its one provider satisfies it fully, so there is
still nothing a decline mechanism's design could be checked against beyond a second instance
of "and it offers," which this record already had one of.

A second, independent trigger: if a completeness or coverage check is found to need to
distinguish "never asked" from "structurally declines" for a capability that already
exists today — for instance, if `nomos-lang-rust-scan`'s narrower `Guarantee` on
`nomos.cap.syntax.items` is ever read by a consumer as silence rather than as the offer it
actually is, and that silence is mistaken for a gap. If that need surfaces against an
existing contract rather than a hypothetical second one, it argues for building the
mechanism for a different reason than the one considered and declined here, and should be
evaluated on its own evidence. Unchanged by `nomos-cap-dependency`'s arrival.

## Status

Open. The record's own first named trigger, read literally, has fired — a second capability
contract, `nomos.cap.dependency.edges`, exists — but the deeper condition that sentence was
standing in for has not: no provider anywhere in this workspace structurally fails to
satisfy any capability contract that exists, `nomos-cap-dependency`'s one provider included,
which offers at the contract's own ceiling. Revisit when a real provider is found that
cannot satisfy a contract it could plausibly be asked about — against `nomos.cap.dependency.
edges`, a third contract, or the existing `nomos.cap.syntax.items` — or when the second,
independent trigger above fires.
