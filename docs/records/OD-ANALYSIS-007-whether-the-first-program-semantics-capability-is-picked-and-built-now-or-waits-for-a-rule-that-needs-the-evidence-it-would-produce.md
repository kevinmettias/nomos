---
id: OD-ANALYSIS-007
type: decision
title: Whether the first program-semantics capability is picked and built now, or waits for a rule that needs the evidence it would produce
status: open
version: 1
authority: canonical-normative-record
tags:
  - analysis
  - capability
  - architecture
relations:
  - target: OD-ANALYSIS-004
    type: relates-to
  - target: OD-CAPABILITY-008
    type: relates-to
  - target: D-135
    type: relates-to
---

# Whether the first program-semantics capability is picked and built now, or waits for a rule that needs the evidence it would produce

## Question

`OD-ANALYSIS-004` settled what a program-semantics capability must say — the five existing
epistemic types (`FactVariant`, `Guarantee`, `EvidenceClass`, `Applicability`, `Observation`),
never a domain-local substitute — and what `FactVariant::SemanticallyResolved` obliges a
producer to have actually done. It deliberately does not pick which capability gets built
first: its five worked shapes (ownership crossing a closed boundary, lifetime/allocation
against a performance policy, escape past a module boundary, a synchronization ordering a
determinism policy forbids, an effect a package or authority policy restricts) are named as
"illustration of the boundary test... not a closed list," and its "What This Record Does Not
Do" section states plainly: "The first program-semantics producer is separate work, reserving
its own territory."

`crates/capabilities` holds exactly one crate, `nomos-cap-syntax`, filing every fact it
produces at `FactVariant::Syntactic`. `FactVariant`'s ordering — `Predicted`, `Approximate`,
`Syntactic`, `SemanticallyResolved`, `RuntimeObserved` — exists so a rule needing resolved
names can refuse a weaker answer, but today it orders one value against nothing: no rule in
this workspace has ever asked for `SemanticallyResolved` or `RuntimeObserved` evidence, and
neither of `nomos-rules`' two real rules (`Check_Naming_Convention`,
`Check_Completeness_Mirrors`) reads past `Syntactic`/`Approximate`. Picking one of the five
illustrative shapes and building a crate under `crates/capabilities` for it now — the first
question a session reading `OD-ANALYSIS-004` reaches for — would be exactly the shape `D-135`
already named a mistake elsewhere: inferring genericity, or in this case a whole new capability
domain, from a wish rather than a demonstrated concrete need. The open question is whether that
choice gets made now, on the strength of `OD-ANALYSIS-004`'s worked examples alone, or waits
for a real consumer to force it.

## Current Position

`OD-ANALYSIS-004` names no trigger for this choice — neither it nor `OD-ANALYSIS-002`,
`OD-ANALYSIS-003`, `ARC-CONFORMANCE-001` nor `OD-CAPABILITY-002` states a concrete condition
under which the first program-semantics capability should be selected. Nothing in this
workspace today is blocked on the absence: no rule, native or from a package, has a subject
that needs an ownership, lifetime, escape, concurrency-structure or effects claim to reach a
verdict it cannot reach otherwise. `nomos-cap-syntax` remains the only capability contract;
`OD-CAPABILITY-007`'s decline-with-reason question and `OD-CAPABILITY-008`'s provider-trait
question were both left open for the identical reason — a population of one (or, for
`OD-CAPABILITY-008`, two) provider instances is not enough to check a design against, and the
same is true here for a population of zero real consumers of the two unused `FactVariant`
levels.

Building any one of the five illustrative shapes now would fix a choice — which fact, in
which payload shape, against which of this repository's own policies — before a real rule
exists to hold that choice to account. `OD-ANALYSIS-004`'s own worked examples are explicit
that each is a *shape* of claim, not a specific one this workspace has committed to; treating
one of them as pre-selected would be reading a decided answer into a record that named the
opposite.

## What Would Decide It

A rule — native, or from a future rule package — that needs a program-semantics fact to reach
a verdict it cannot reach at `Syntactic` or `Approximate` today. That rule's own subject would
name which of `OD-ANALYSIS-004`'s five shapes (or a sixth this record does not anticipate) is
the real one to build, the same way `Check_Naming_Convention`'s arrival was what let
`OD-RULES-006` compare two rules' placement rationale on real evidence instead of one. Until
such a rule exists, any of the five shapes is equally unmotivated, and picking among them would
be a design choice with no case to check it against.

A second, independent trigger: this repository's own architecture, requirement or policy
records naming a specific claim in this family as something a conformance check must make —
for example a future band or authority-class rule that can only be enforced by knowing whether
a value's ownership crosses a closed boundary once resolved. A named requirement of that shape
would pick the first capability by naming the fact it needs, rather than by a session choosing
among `OD-ANALYSIS-004`'s illustrations for its own reasons.

## Status

Open. No rule in this workspace needs a `SemanticallyResolved` or `RuntimeObserved` fact
today, so nothing picks among `OD-ANALYSIS-004`'s five illustrative shapes yet. Revisit when a
rule's own subject names the fact it needs, or when an architecture, requirement or policy
record names a specific program-semantics claim a conformance check must make.
