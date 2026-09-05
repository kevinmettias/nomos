---
id: OD-HOST-004
type: decision
title: A second rule or provider is composed by hand until its participation depends on the request, not merely on its existence
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - host
  - orchestration
  - rules
  - capability
  - architecture
relations:
  - target: OD-HOST-001
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
  - target: D-135
    type: relates-to
  - target: ARC-HARNESS-001
    type: relates-to
---

# A second rule or provider is composed by hand until its participation depends on the request, not merely on its existence

`nomos_check_orchestration::run::Run` hand-calls `nomos_rules::Check_Completeness_Mirrors`
by name. `nomos_check_orchestration::composition::Registered` hand-declares exactly one
capability contract and hands exactly one provider offer against it. Both are correct at
the crate's current size — its first and only shipped rule, against its first and only
capability — but nothing on the board or in a governing record decided what happens to
either function when a second rule or a second capability/provider pair arrives. Two
shapes are both plausible: a declared selection mechanism driven by what a request asks
for, or a second hand-written call beside the first, then a third, deciding the seam by
accretion rather than by a recorded choice.

## The decision

**Registered() never needs a selection mechanism, at any count.** It does not select
anything today, and adding a second `Declare`/`Offer` pair would not make it start.
`nomos_capability::Registry` (`crates/substrate/nomos-capability/src/registry.rs`) already
holds an arbitrary number of declared capabilities and an arbitrary number of offers per
capability, and it is already the declared, generic mechanism that picks among them:
`Registry::Resolve` ranks the offers standing against whatever `Requirement` a caller
states and returns a `Selection`, per `OD-CAPABILITY-001`'s "the registry ranks; the caller
spends." `Registered` only feeds that mechanism data. A second `Declare_And_Offer` call —
for a second provider against the same capability, or for an unrelated second capability
entirely — is one more line writing into a structure engineered to hold more than one of
each, not a new hand-written selection. This is the same shape `OD-HOST-001` already found
correct: "naming a concrete provider in a composition root was never the defect." Nothing
about a second pair changes that.

**Run() stays hand-written for as long as every rule it calls runs unconditionally, on
every invocation.** Unlike `Registered`, `Run` has no registry standing behind its one
call: `Check_Completeness_Mirrors` is invoked directly, and there is no `Rule` type, trait,
or collection a second rule would be added *to*. But a second hand-written call beside the
first is not the seam-by-accretion failure this record's originating item warned about as
long as both calls are unconditional — every rule the crate knows about runs on every
`sources`/`variant` pair `Run` is given, the same way the first rule does today, and a list
of unconditional calls does not become a selection problem by growing longer. `Run` takes
no field today that could vary that set per request, and while that stays true, a third and
a fourth hand-written call are the same kind of statement as the first two: composition,
not choice.

## What flips the answer

The criterion is not a count of rules or of capability/provider pairs. It is whether the
next entry's participation is unconditional or is meant to vary by request.

`Registered` never flips: `Registry::Resolve` already resolves per `Requirement`, and every
rule that asks stays free to state its own floor (`nomos_rules::Syntax_Requirement` is
`Check_Completeness_Mirrors`'s own, not `Registered`'s or `Run`'s) — no future capability
count changes what `Registered` itself has to do, because it was never the layer that chose.

`Run` flips the first time a rule (or a group of rules) is meant to run for *some* check
invocations and not others — selected by something the request states, rather than by
whether the crate happens to know about it. At that point, adding the choice as a
hand-written `if`/`match` naming rules by branch is exactly the accretion this record
exists to head off: a second arm, then a third, each decided in the function body instead
of against a declaration. The fix at that point is a declared selection mechanism — a
request-carried name or property, resolved against something `Run`'s caller states rather
than something `Run`'s body names — the same division `ARC-HARNESS-001` draws between what
should run (a question with an answer that depends on the codebase and the request) and how
a step is performed once chosen.

Concretely: when `nomos-rules` ships its second rule, ask whether it is meant to run on
every `nomos check` the way `Check_Completeness_Mirrors` does today. If yes, `Run` gains one
more hand-written call and this record still holds. If the answer is "only when the request
asks for it" — a subset of checks, a per-language rule, an opt-in — `Run` needs the
mechanism before that rule ships, not after a second and third conditional accrete beside
it.

## Why not build the mechanism now

`D-135` records the same caution in the adjacent case of deciding a subsystem's home too
early: inferring a need for generic machinery from a wish that it might be needed, rather
than from a demonstrated one, produces the evidence a proof gate wants by paying the cost
the gate exists to avoid. A rule-selection mechanism built before any rule needs to be
selected — as opposed to merely added — would be exactly that: machinery justified by "a
second rule might need it" rather than by a second rule that does. `nomos-rules` itself
records the matching restraint on its own side of this seam: it shipped with exactly one
rule "because a single check that is honest end to end is worth more than three that are
nearly wired," not because a selection mechanism was waiting for a second entry to arrive.

## Alternatives considered

**A flat count threshold** — for example, "hand-written composition is correct through two
entries, and a third requires a mechanism." Rejected: nothing about a third unconditional
rule differs in kind from a second one, and a threshold picked by count rather than by
property would be re-argued the moment a real third entry showed up unconditional, which is
the exact re-litigation this item exists to prevent.

**Deciding it only when `nomos-rules` ships its second rule.** Rejected on `ARC-HARNESS-
001`'s reasoning applied one layer down: the longer a seam like this is left unstated, the
more likely it is decided by whichever shape the second rule happens to need, rather than by
a recorded choice checkable in advance. This record is written before that rule exists so
the criterion, and not the accident of what the second rule turns out to want, decides the
shape.

## Status

Accepted, and the trigger this record names has since fired.

The criterion was stated here as: `Run` flips the first time a rule is meant to run for
*some* check invocations and not others, selected by something the request states.
`nomos_check_orchestration::Run` now takes `selected: &[RuleId]`; `OD-GATE-017` decided that a
non-empty selection narrows what `Run` computes at all rather than merely filtering its
output; and `nomos gate run --rule <id>` reaches it from a command line. That is the criterion
met, not an approximation of it.

What answered it was not the declared mechanism this record asked for. `Run` gained a
`RuleId` filter over a fixed `[ComposedRule; RULE_COUNT]` array, and its own doc calls the
structure beside it "a fixed, hand-written mapping from `RuleId` to the fact(s) it needs". A
declaration doing registry work inside a function body is what the reasoning above refuses; it
arrived as a table rather than as the `if`/`match` this record predicted, which is why the
flip passed unremarked at the time.

`OD-RULES-022` picks the question up from here and decides what composition resolves against:
a declared rule package matched to a linked implementation, refused in both directions. It does
not revise any reasoning on this page. The criterion above was right, and finding it already
satisfied is what that record is a response to.

Nothing here is withdrawn. `Registered()` still never flips, for the reason stated above — the
capability registry was always the layer that chose. This record's own restraint about
building a mechanism ahead of a rule that needs one also stands: what changed is that a rule
that needs one now exists, which is exactly the condition it said to wait for.
