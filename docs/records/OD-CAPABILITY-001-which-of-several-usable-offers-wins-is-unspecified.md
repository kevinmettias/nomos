---
id: OD-CAPABILITY-001
type: decision
title: The guarantee decides which usable offer answers, and the caller decides how far down to spend
status: closed
version: 2
authority: canonical-normative-record
tags:
  - capability
  - resolution
relations:
  - target: OD-ANALYSIS-001
    type: relates-to
---

# The guarantee decides which usable offer answers, and the caller decides how far down to spend

## Question

`Registry::Resolve` takes a requirement with a floor and an optional preferred provider. The
floor decides which offers are *usable*. Nothing decided which of several usable offers is
*chosen*, and until P8-SECOND-PROVIDER there was never more than one.

## What It Did

`offers.iter().find(usable)` over a list sorted by provider name.

`nomos.lang.rust.scan` sorts before `nomos.lang.rust.syn`. So a caller that lowered its floor
to `Approximate` — in order to get an answer for the seven files in the scale corpus a parser
refuses — was served the scanner for all 7,580, including the 7,573 the parser would have read
exactly. The registry reported `Applicability::Supported`, which was true and said nothing
about the better answer that was available.

## Why It Was Not Obviously A Bug

"Strongest usable offer wins" is the reflex answer and it is not free. A guarantee has no cost
axis, so the strongest offer is also frequently the most expensive one — a compiler-backed
provider is orders of magnitude slower than a parser, and a parser is slower than a
line-reader. A registry that always reaches for the strongest turns "I will accept an
approximation" into "give me the best you have", which is the opposite of what the caller
said.

Three candidates were stated, none free: **strongest usable wins**, predictable and silently
expensive; **weakest usable wins**, which matches "I asked for approximate, give me
approximate" but means the parser never answers unless somebody names it; **declared order,
authored by the composition root**, which puts the choice where the cost knowledge is at the
price of a root that must rank every provider it registers.

## What Decided It

None of the three. The situation that raised the record defeats all of them, and seeing that
is what settled the rule.

The caller in the motivating case lowers its floor because a parser refuses seven files. Under
**name order** it is served the scanner for every file: it buys the coverage and pays for it
everywhere, silently. Under **strongest-wins** it is served the parser for every file — which
is what it already had, so the same seven files go unanswered and *lowering the floor bought
nothing at all*, equally silently. Under **weakest-wins** it is name order's outcome by
construction. Under a **declared order** it is whichever of those two the composition root
wrote down, and the root has no way to write down "the parser, except where it refuses".

Every candidate is a single provider for the whole run, and the caller's requirement was never
about the whole run. A floor says what an answer must be worth *for a subject*. The registry
resolves once per requirement and a requirement names a capability, not a file, so a registry
that returns one provider has decided something it cannot know: which subjects that provider
will refuse.

So the rule is in two parts, and the second is what makes the first safe:

**The offer that no usable offer is strictly stronger than answers.** Strictly stronger is
`Guarantee::Satisfies` holding one way and not the other — the same comparison that decides
usability, used to decide precedence, so nothing new is invented and no axis is weighted
against another. A named preference outranks the rule, because a caller saying what it wants
is not the registry's to overrule.

**Every offer it was chosen over comes back with it.** `Resolution::Satisfied` carries a
`Selection`, and `chosen` together with `alternatives` is exactly the set that cleared the
floor. `Selection::Weaker` is what the lowered floor bought. The registry ranks; the caller
spends.

That is what makes a floor mean something. Widening what you will accept now widens what is
*reachable* without changing what you are *served*, which is the distinction all three
candidates collapsed.

## Why There Is No Cost Axis On `Guarantee`

The record said a missing cost model might be the actual gap. It is not, and the reason is the
split above.

Cost does not rank offers; it decides how far down a ranking to spend. Ranking is the
guarantee's, and it is complete for the purpose — "is this answer good enough, and is that one
better" needs no prices. Spending is the caller's, and it already had two ways to say so: a
floor, and now the alternatives the floor admitted.

Putting a cost on `Guarantee` would also put a scheduling concern into `nomos-contracts` — the
crate whose every type is reimplemented by systems that will never compile it, and which
`Test_Contracts_Should_Depend_On_The_Allowlist_And_Nothing_Else` exists to keep neutral. A
peer would have to agree with us about what a provider costs in order to agree with us about
what a fact *is*. `nomos-contracts` is unchanged by this decision.

## Why There Is No Total Order, And What Is Reported Instead

`Guarantee::Satisfies` is deliberately not a score — "every axis, not a score", because a
weighted average would let a very sound syntactic provider answer a question that needs name
resolution. What it gives is a preorder, and a preorder has maximal elements rather than a
maximum. Two offers can be **equivalent** — each reaching everything the other does — or
**incomparable**, each better on some axis. A compiler-backed provider that resolves names but
can only refresh a whole project is incomparable to a parser, and which of them is preferable
is not a question a guarantee can answer.

`Standing` names all four cases rather than collapsing the last two into "not stronger".
Collapsing them is how a registry comes to report a decision it did not make: an equivalent
offer would read as one that was passed over for being worse.

Where the rule does not decide, the answer is the first by provider name among the maximal
offers — deterministic, and explicitly not a reason. `Selection::Unranked` is non-empty
exactly then, so a composition root that wants its provider choice to be a decision can assert
it is empty, and this workspace's does. Name order survives as a tiebreak that is no longer
load-bearing: whatever it picks, every alternative came back with it.

## What Was Considered And Rejected

**Refusing incomparable offers at `Registry::Offer`.** Attractive, because it would make the
rule total at composition time, where a human could act on it. Rejected: two providers that
are each better on some axis are a legitimate and useful composition, and a registry that
refuses to hold both prevents the arrangement rather than ranking it. It would also refuse
equivalent offers, and two providers making the same promise is the ordinary case of a second
implementation.

**A new `Applicability` for "chosen, but something better existed".** There is no such state
under this rule without a preference, and with one the caller asked for what it got. Adding a
value to the vocabulary a run reports in, to describe a case that only arises when the caller
caused it, would grow the enum for a distinction the caller already holds.

**`Resolution::Undecided` when several offers are maximal.** It would make the registry's
indecision impossible to ignore, at the cost of a bare no invented by the registry itself —
the one thing `nomos-capability` exists not to produce. Reporting the choice *and* that it was
not decided says the same thing without withholding an answer.

## What This Does Not Do

The registry ranks and does not dispatch. `Selection::Weaker` is reachable and nothing yet
walks it: `Slice::Dispatch` asks the chosen provider and, when a parser refuses a file, the
run degrades rather than falling to the offer the floor admitted for exactly that case.

That is a phase of work rather than part of this decision — per-subject fallback changes which
provider answers for which file, and therefore what every fact is keyed under and what a run
reports. It is stated here so that the gap is a plan and not an oversight.
`Test_A_Lowered_Floor_Should_Make_The_Weaker_Offer_Reachable_Without_Serving_It` asserts the
half that exists, and its name is where the other half is written down.

## Status

Closed by P8-SELECTION. Four controls were confirmed red before the change was kept: selection
by provider name (four tests, including the composition's own — and it caught a test of the
new rule that passed under the old one because `parse` happens to sort before `scan`), a
selection that returns the winner alone (six tests), alternatives drawn from every registered
offer rather than the usable ones (five tests), and a `Standing` that reads "reaches
everything the other does" as stronger without checking whether the other reaches back (two
tests).
