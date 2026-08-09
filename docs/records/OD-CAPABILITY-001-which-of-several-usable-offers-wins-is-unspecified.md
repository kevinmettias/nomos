---
id: OD-CAPABILITY-001
type: decision
title: Which of several usable offers wins is unspecified, and is currently alphabetical
status: open
version: 1
authority: canonical-normative-record
tags:
  - capability
  - resolution
relations:
  - target: OD-ANALYSIS-001
    type: relates-to
---

# Which of several usable offers wins is unspecified, and is currently alphabetical

## Question

`Registry::Resolve` takes a requirement with a floor and an optional preferred provider. The
floor decides which offers are *usable*. Nothing decides which of several usable offers is
*chosen*, and until P8-SECOND-PROVIDER there was never more than one.

## What It Does Today

`offers.iter().find(usable)` over a list sorted by provider name.

`nomos.lang.rust.scan` sorts before `nomos.lang.rust.syn`. So a caller that lowers its floor
to `Approximate` — in order to get an answer for the seven files in the scale corpus a
parser refuses — is served the scanner for all 7,580, including the 7,573 the parser would
have read exactly. The registry reports `Applicability::Supported`, which is true and says
nothing about the better answer that was available.

`Test_Two_Usable_Offers_Should_Resolve_By_Name_Order_Until_Something_Says_Otherwise` states
it, and its name is the record of the fact that this is observed behaviour rather than
anybody's decision.

## Why This Is Not Obviously A Bug

"Strongest usable offer wins" is the reflex answer and it is not free. A guarantee has no
cost axis, so the strongest offer is also frequently the most expensive one — a
compiler-backed provider is orders of magnitude slower than a parser, and a parser is slower
than a line-reader. A registry that always reaches for the strongest turns "I will accept an
approximation" into "give me the best you have", which is the opposite of what the caller
said.

"First by name" is worse than both, because it is not an answer at all. It produces a
defensible outcome only by coincidence and changes if somebody renames a provider.

Three candidates, none free:

**Strongest usable wins.** Predictable, and silently expensive. A caller lowering its floor
for coverage gets no speed back.

**Weakest usable wins.** Matches "I asked for approximate, give me approximate", and means
registering a strong provider changes nothing for callers who did not ask for it. It also
means the parser never answers unless somebody names it, which makes the floor the only
control and preference mandatory in practice.

**Declared order, authored by the composition root.** Puts the choice where the cost
knowledge is, at the price of a composition root that must rank every provider it registers.

## Why It Is Not Decided Here

`Registry::Resolve` is `nomos-capability`, which P8-SECOND-PROVIDER's territory does not
name, and the choice above is a design decision rather than a defect to patch. It also wants
a cost model to be decidable well, and there is not one — a `Guarantee` says how good an
answer is and nothing about what it costs to produce, which may be the actual gap.

P8-SELECTION carries it.

## What Holds The Line Until Then

A named preference is honoured when it can be, and `SupportedWithFallback` when it cannot,
so a caller that knows what it wants can say so and can tell whether it got it. That is
enough for a composition root and not enough for a rule author, who will not know a second
provider exists.

## Status

Open. The behaviour is asserted rather than left to be discovered, so that the day it
changes, a test fails and says what changed.
