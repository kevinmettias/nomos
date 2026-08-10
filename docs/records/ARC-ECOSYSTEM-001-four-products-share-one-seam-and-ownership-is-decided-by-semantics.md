---
id: ARC-ECOSYSTEM-001
type: architecture
title: Four products share one seam, and ownership is decided by semantics rather than by location
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ecosystem
  - ownership
  - boundaries
relations:
  - target: OD-AGENT-001
    type: relates-to
  - target: D-130
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
  - target: OD-LEDGER-001
    type: relates-to
---

# Four products share one seam, and ownership is decided by semantics rather than by location

## Question

Four things are being built around one another, and until now this repository has recorded
nothing about where one ends and the next begins.

What it does record is a dependency rule. `D-130` decides that no crate here depends on the
sibling XVPE workspace before Phase 5 and never by path, and that only a named adapter may
mention anything beginning with `xvpe-`. That is a rule about *linking*, and it has been
doing double duty as a rule about *belonging*, which it is not: a subsystem can be owned by
XVPE while nothing here links to it, and a crate can link to XVPE while remaining entirely
this product's. KWB has not even that much — it appears in this workspace five times, all
incidental.

So the seam that decides whether a subsystem belongs to knowledge, to generic runtime, to
software-engineering reality, or to the machinery that exists only to build this repository
has been held in conversation. Every placement made without it is a placement that has to
be argued about later rather than read.

## Why This Is Recorded Now Rather Than Later

`OD-AGENT-001` refused to write this boundary into `AGENTS.md`. The reasoning was that an
instruction file is read by a machine every session and reviewed by a person approximately
never, so putting an undecided architecture into one would promote a design conversation to
normative status through the least-reviewed file in the repository.

That refusal is what makes now the right time. The harness carries an explicit statement
that the question is open and names what would close it, which is a placeholder rather than
a silence. A placeholder is cheap to fill and expensive to leave: the longer four products
accumulate work across an unstated seam, the more of that work has to be re-litigated when
the seam is finally drawn.

## What Each Owns

**XVPE owns generic runtime, application and platform infrastructure** — the primitives
that are reusable without any Nomos semantics attached. A task scheduler, a storage
primitive, a process or window or telemetry substrate: things whose definition never
mentions software engineering, because they would be equally correct under a product that
had nothing to do with code.

**Nomos owns software-specific reality and the engineering of change.** What the software
actually is and does: analysis, architecture and feature topology, engineering constraints,
change planning and the validation that execution satisfied it, findings, corrections,
gates, and the runtime evidence that any of it holds. Its subject matter is a codebase, and
every one of its concepts stops making sense without one.

**KWB owns knowledge.** Rationale, semantic intent, requirements elicitation, decision
context, long-term epistemic memory and rich authored knowledge models. Its subject matter
is what somebody meant, claimed, or decided, and why — which survives the code it was
about, and is not reducible to any state a repository can be in.

**Repository and bootstrap tooling owns the machinery that exists only to develop or
preserve this repository.** It is not automatically a product feature. Something written to
get Nomos built is not thereby part of Nomos, and the assumption that it is is how a
scaffold becomes a shipped surface nobody chose to ship.

## The Boundaries Are Projections, Not A Partition

Stating four nouns and stopping would leave the interesting cases unanswered, because the
hard ones are not *which box* but *which direction*. Two crossings are governed.

```
KWB semantic intent / rationale
        |
        |  governed projection
        v
Nomos executable software contract
```

Knowledge becomes an engineering rule only by a projection that is itself governed. A
rationale is not enforceable; a contract derived from it is. The derivation is where the
authority changes hands, and it must be a recorded step rather than an inference somebody
made once — otherwise the rule and the reason drift apart and neither can correct the
other.

```
XVPE generic execution primitive
        ^
        |  implemented / adapted by
        |
Nomos-specific service
```

Generic capability is consumed by adaptation, never by extension of the generic thing with
software-engineering meaning. When a primitive would have to learn what a crate, a rule or
a finding is in order to serve Nomos, the primitive is not the thing that should change:
the adapter is. `D-130` already fixes the mechanical form of this crossing — a single named
adapter, no path dependency — and this record supplies the reason that rule was the right
shape.

## Current Placement Does Not Prove Permanent Ownership

This is the clause that keeps the record honest about the repository it was written in.

**Where a subsystem lives today is evidence of what was needed to bootstrap, not of what
owns it.** Several things here are in that position and are named so that nobody later
mistakes their address for a decision:

- The specification system. `ARC-SPECDB-001` makes the specification a database behind a
  preservation ledger, and it lives here because the corpus had to stop losing content
  before anything could be built on it. Its subject matter — authored knowledge, rationale,
  normative statements and their preservation — is KWB's description almost word for word.
  Its mature home may be KWB, reached from here through a knowledge capability, which is
  already how the product is permitted to see it.
- The work ledger. Territory-based mutual exclusion over a committed JSON document is
  either repository bootstrap machinery or a generic coordination primitive; `OD-LEDGER-001`
  records that its territory is declared rather than enforced, which is a bootstrap-shaped
  compromise. Nothing about it requires the subject to be software.
- The contract tests, the surface snapshots and the gate derivation. These preserve this
  repository. They are the clearest case of tooling that a reader could mistake for product
  simply because it is here and it is good.

Naming them is not a plan to move them, and this record moves nothing. It removes the
argument from precedent: nobody may later cite a subsystem's location as proof of its
ownership, because this record says in advance that the location was not the decision.

## Which Product Answers Which Question

| Question | Authority |
|---|---|
| What does the software actually do? | Nomos |
| Does an implementation satisfy an executable engineering rule? | Nomos |
| Why was a design chosen? | KWB |
| What did a source or a decision claim? | KWB |
| What generic task, runtime or storage primitive exists? | XVPE |
| Is this repository's local coordination board valid? | repository tooling, as Nomos bootstrap |
| Does a generalized lesson become an enforceable software rule? | a governed decision, projected into Nomos |

The last row is the one that does work. A lesson does not become a rule by being true and
widely applicable. It becomes a rule by a decision that says so, and the decision is the
projection boundary described above.

## The Anti-Drift Clause

> A subsystem should move toward the lowest layer or product whose semantics fully explain
> it.

And three arguments that do not settle ownership, each of which is the plausible mistake
for one of the products:

**Reuse alone does not make something XVPE.** Plenty of software-specific machinery is
reusable across projects. Generality of *audience* is not generality of *meaning*, and only
the second is the criterion.

**Structure alone does not make something KWB.** A well-modelled graph of typed nodes is not
knowledge because it is well-modelled. If its nodes are facts about code, it is Nomos with
a good schema.

**Current repository location alone does not make something Nomos.** This is the corollary
of the section above, stated as a rule so that it can be cited.

The clause is directional on purpose. It says *move toward*, because the cost of a
premature move is real and this record does not require anybody to pay it today. What it
requires is that the direction be known, so that each future placement is made with it
rather than against it.

## Conflicts With Existing Decisions

Checked deliberately rather than assumed, because a record that quietly reinterprets an
earlier one is worse than no record.

`D-130` is untouched and unweakened. It governs linking; this governs belonging. Where they
meet — the single adapter crate — they agree, and this record supplies the reason `D-130`
recorded only as a rule.

`ARC-SPECDB-001` is untouched. Naming the specification system as a candidate for a
different mature home says nothing about whether the specification is a database, which it
is and remains.

`OD-AGENT-001`'s deferral is answered by this record existing. Its instruction is unchanged:
`AGENTS.md` may gain a routing line to this record when there is a concrete routing question
that needs one, and it gains no conclusions from it at any point. The harness stays thin.

## What This Record Does Not Do

No crate moves. No package is renamed. No dependency changes. No KWB or XVPE integration is
implemented, and none is scheduled here. `AGENTS.md` and `CLAUDE.md` are not touched.

Nothing mechanical enforces this boundary, and that is the honest state rather than an
oversight. There is no test that can decide whether a subsystem's semantics are fully
explained by a lower layer — that is meaning, and this workspace has no type for it. What
this record buys is that the next placement argument is settled by reading rather than by
whoever is most recently convinced, and that a future enforcement, when something concrete
enough to enforce exists, has a criterion to be derived from.

## Status

Closed by `P10-ECOSYSTEM-BOUNDARY`.
