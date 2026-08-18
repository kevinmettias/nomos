---
id: OD-RULES-004
type: decision
title: A rule package registers into a declared registry, extracted before a second rule needs it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - capability
  - architecture
  - packages
relations:
  - target: OD-HOST-004
    type: relates-to
  - target: D-135
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
---

# A rule package registers into a declared registry, extracted before a second rule needs it

## Question

`crates/rules/nomos-rules` holds exactly one rule, `Check_Completeness_Mirrors`, and has no
registration contract of its own — no `RuleId`-carrying offer a second, independently
authored rule could hand to something, the way `nomos_capability::Registry` already lets a
provider hand a `ProviderOffer` to `Registry::Declare_And_Offer` without the registry
knowing the provider's crate in advance. A person gave explicit direction to invest now in
the base types a rule plugin needs, so a rule package can be designed and built on a thread
separate from Nomos proper's own work, rather than waiting for `nomos-rules`' second rule
to force the question by accretion.

`OD-HOST-004` already answers an adjacent question and its answer could be read as refusing
this one: it decided `Run()` stays hand-written "for as long as every rule it calls runs
unconditionally," and named the criterion for building a mechanism as conditionality, not
cardinality — explicitly declining to build a selection mechanism before a rule's
participation actually needs to vary by request. Whether a declared *registration* contract
is the same kind of premature machinery `OD-HOST-004` warned against, or a different thing
entirely, is what this record has to settle before naming a shape.

## The Decision

**Registration and selection are different questions, and `OD-HOST-004` answered only the
second one.** `Run()` calling every rule the crate knows about, unconditionally, is a
statement about which rules execute on a given check — that is selection, and `OD-HOST-004`
correctly leaves it hand-written until a rule's participation is meant to vary by request.
Nothing about *how a rule becomes a thing `Run()` could know about in the first place* is
answered by that record; today, that answer is "somebody hand-writes a function inside
`crates/rules/nomos-rules` and adds a hand-written call," the same shape `nomos-add-plugin`
section 1 documents for providers before `nomos-cap-syntax` gave capability contracts a home
independent of any one provider's crate.

`nomos-rules` gains a declared rule-registration contract now: a `RuleId`-carrying offer
type — mirroring `nomos_capability::ProviderOffer`'s shape (`provider: ProviderId,
capability: CapabilityId, version: ContractVersion, guarantee: Guarantee`) closely enough
that a rule package can hand its own offer to a registry without either party naming the
other's crate — is the shape a follow-on capability item builds. `RuleId` already exists in
`nomos-contracts` (band 0) as a `Named_Identity` newtype, used today by `Finding::rule`; the
registry and the offer type it accepts are new, and belong beside `nomos-rules` for the same
reason `Registry` belongs beside `nomos-capability` rather than beside its first provider.

**This is built now, ahead of a second rule, on the same principle `nomos-add-plugin`
section 2 already states for capability contracts**: pull a contract out at the moment a
second party is expected to offer against it, not only once one actually has. A rule
package is that expected second party. Unlike `nomos-cap-syntax`'s extraction — which
answered a need `nomos-lang-rust-scan` already demonstrated by existing — this extraction
answers a need stated directly rather than demonstrated by a second rule shipping first.
That is a real departure from the evidence-over-inference posture this repository otherwise
insists on, the same caution `D-135` states for where new domain-neutral code is authored:
proposing shared machinery because it *would* be convenient, without anything having built
it, produces the appearance of the evidence a proof gate wants by paying the cost the gate
exists to avoid. What changes the answer here is that the second party is not inferred from
a wish; it is an explicit, recorded decision to build rule packages as independent work —
the same distinction `D-135` draws between code "designed for shared use from the outset"
and genericity inferred from a hope that two products would happen to converge. The
extraction is justified by a decision already made, not by anticipation of one that might
be.

## Why This Does Not Contradict `OD-HOST-004`

`OD-HOST-004`'s finding for the rule that exists today is untouched: `Run()` remains
correct, hand-written, and unconditional for `Check_Completeness_Mirrors`, and stays that
way for any rule whose participation does not vary by request. The registry this record
authorizes is additive infrastructure a rule package registers *into* — it does not require
`Run()` to consult it, select from it, or change shape at all. A rule can exist in the
registry and still be wired into `Run()` by the same hand-written call `OD-HOST-004` already
found correct; registration answers "how does a second rule become a nameable thing,"
not "which rules run on this invocation." The moment a registered rule's participation is
meant to vary by request — the criterion `OD-HOST-004` already names — `Run()` still needs
the selection mechanism that record describes, and this record does not build it, does not
schedule it, and does not weaken the reasoning that says not to build it yet.

## What This Record Does Not Do

It does not build the registry, the offer type, or any change to `crates/rules/nomos-rules`
or `crates/orchestration/nomos-check-orchestration`. It names the shape a follow-on
capability item constructs.

It does not give `nomos-rules` a second rule, and does not require one to exist before the
registry is built — the registry's first real consumer may be `Check_Completeness_Mirrors`
itself, registering into a structure built to hold more than one, the same way
`Registered()` already holds more capability/provider pairs than exist today.

It does not build a rule-selection mechanism for `Run()`, and does not weaken `OD-HOST-004`'s
finding that one is premature until a rule's participation actually needs to vary by
request. Registration and selection stay two different questions with two different
answers.

It does not decide whether a `RulePackage` manifest crate (mirroring `nomos-lang-package`,
built on `nomos-package`'s generic core) is built next, only that the registry the manifest
would target now has a stated shape to build against.

## Status

Accepted, landed by `P13-RULE-PACKAGE-DECISION`.
