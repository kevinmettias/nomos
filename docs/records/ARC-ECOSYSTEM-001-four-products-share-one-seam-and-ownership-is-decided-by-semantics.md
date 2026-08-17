---
id: ARC-ECOSYSTEM-001
type: architecture
title: Four products share one seam, and ownership is decided by semantics rather than by location
status: accepted
version: 4
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
  - target: D-085
    type: relates-to
  - target: D-086
    type: relates-to
  - target: D-090
    type: relates-to
  - target: D-091
    type: relates-to
  - target: D-096
    type: relates-to
  - target: D-122
    type: relates-to
  - target: ARC-HARNESS-001
    type: relates-to
  - target: P10-PACKAGE-SEAM
    type: relates-to
---

# Four products share one seam, and ownership is decided by semantics rather than by location

## Question

Four things are being built around one another, and until now this repository's own records
have said nothing about where one ends and the next begins. Version 1 of this record said
something stronger and wrong — that *nothing* had recorded it — and the correction is
`The Sibling Record Set Decided Most Of This First`, below.

What it does record is a dependency rule. `D-130` decides that no crate here depends on the
sibling XVPE workspace before Phase 5 and never by path, and that only a named adapter may
mention anything beginning with `xvpe-`. That is a rule about *linking*, and it has been
doing double duty as a rule about *belonging*, which it is not: a subsystem can be owned by
XVPE while nothing here links to it, and a crate can link to XVPE while remaining entirely
this product's. KWB has not even that much — it appears in this workspace five times, all
incidental.

So the seam that decides whether a subsystem belongs to knowledge, to generic runtime, to
software-engineering reality, or to the machinery that exists only to build this repository
is not one a reader of `docs/records/` could find. Every placement made without it is a
placement that has to be argued about later rather than read.

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
hard ones are not *which box* but *which direction*. Three crossings are governed.

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

```
Nomos run observation
        |
        |  generalization across many runs,
        |  carrying its runs as provenance
        v
KWB generalized lesson
```

A harness that records what a single run did — task class, model, attempts, verification
outcome, review findings, cost, failure cause — is producing observations about a codebase.
That is Nomos's own subject matter and crosses nothing, no matter how much of it
accumulates. What crosses is the step after it: generalizing many runs into a lesson that
outlives the runs it came from. Rationale and long-term epistemic memory are KWB's
description of itself, word for word, and a generalization is exactly that — a claim about
why, derived from what happened, meant to survive the runs that produced it.

**Observations produced by a run may become KWB knowledge.** The crossing is governed, not
forbidden, the same way the two above it are: a generalization may cross only carrying the
runs it was derived from as provenance, never as a bare assertion. A lesson without its runs
is not a governed crossing; it is a claim invented at the boundary, indistinguishable from
one nobody ever checked.

Provenance closes the first crossing and not the one after it. This record's own last table
row already says a lesson becomes an enforceable rule only by "a governed decision, projected
into Nomos" — and if that decision's evidence is the harness's own generalization about its
own runs, the review is inspecting the machine the rule will govern while reading itself as
independent of it. So the review step that promotes a lesson into a rule may not treat a
generalization's provenance as its own corroboration: it needs a source the runs did not
produce themselves — a person, or a signal the harness was not the author of — before a
lesson may become a rule. Provenance is necessary so the review has something to check;
a second source is what makes the check something other than the system grading its own
homework.

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
| May a run's own recorded observations become KWB knowledge? | KWB, only carrying the runs as provenance |
| Does a generalized lesson become an enforceable software rule? | a governed decision, projected into Nomos, and not on the provenance's own say-so |

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

## The Sibling Record Set Decided Most Of This First

Added at version 2. Version 1 claimed the seam had been held in conversation, which was true
of `docs/records/` and false of the store this build assembles from the archives
`NOMOS_SPEC_ARCHIVES` already names.

`P3-SIBLINGS` ingests three of those archives as non-root suites — `xvpe-spec-seed-v0.1`,
`kwb-spec-seed-v0.1` and `ecosystem-contracts-v0.1` — for exactly this reason, stated in
`siblings.rs`: without the distinction "XVPE's `D-085` reads as a decision this repository
made", and with it a cross-suite relation is an ordinary row. The suites had already decided
most of what is above.

| The clause here | The sibling record that reached it first | Suite |
|---|---|---|
| XVPE owns generic runtime, application and platform infrastructure | `D-085`, XVPE is the shared application platform | `xvpe-spec-seed` |
| Generic capability is consumed by adaptation, never by extension of the generic thing | `D-086`, product semantics do not flow downward into XVPE Platform | `xvpe-spec-seed` |
| Reuse alone does not make something XVPE | `D-090`, cross-product reuse by itself is insufficient reason to move product semantics into the platform | `xvpe-spec-seed` |
| Nomos owns software reality and KWB owns knowledge | `D-096`, Nomos and KWB are sibling domain analyzers over XVPE | `ecosystem-contracts` |

That two record sets reached the anti-drift clause independently is evidence for it rather
than against it. What it is not is a licence to keep restating it: the next clause written
here without checking the seeds is the one that will disagree with them and nobody will
know.

### The edges are declared, not described

Naming those four in prose and stopping would be the mistake this system exists to refuse,
because a summary of a checked thing is an unchecked copy of it. The front matter declares a
relation to each instead, so *which suite decided this first* is a query rather than a
reading. Before this record there were none: 197 relation rows across this repository's
records, three of them naming a target no local record holds — `ADR-DOC-001`, `D-120` and
`D-128`, all v14 corpus records — and not one naming a sibling suite.

The edges resolve rather than dangle, and both halves of that already exist.
`Seed_Governing_Records` mints an external placeholder for a target nothing has ingested and
reports it by name rather than counting it, and `Write_Node` updates a node in place only
where its authority is external — which is what lets `Ingest_Sibling_Suite` claim the
identifier when the seed is read. So over a store holding only this repository's records
these five are placeholders, and over one that has ingested the seeds they are the seeds'
own nodes, carrying their suite. A test asserts that rather than leaving it as a reading of
two functions.

### `D-122` is adopted here, because a sibling's decision does not govern by itself

A sibling suite is `authority_root = 0`. Its records are not this repository's decisions, and
citing one does not make it one — that is the whole point of ingesting them as non-root. So
where the two statements of this seam differ, this record governs here, and the difference is
worth naming rather than absorbing silently.

`D-122` is the case. It admits a shared analysis mechanism into XVPE only after Nomos and KWB
slices demonstrate materially identical domain-neutral semantics. That is narrower than the
anti-drift clause above: this record says reuse does not settle ownership, and `D-122` says
what does, imposing a burden of proof this record never stated.

**It is adopted, as a clause of this record:**

> A shared analysis mechanism moves to XVPE only after two products have demonstrated
> materially identical domain-neutral semantics over it. An argument that both products
> *would* use it is the reuse argument, and it is refused above.

Adopting it rather than citing it is this record's own projection rule applied to itself — a
statement from another authority becomes enforceable here by a recorded step, not by being
true and nearby. The edge to `D-122` says where it came from; the clause is why it binds.

### `D-091` is adopted here, on the same footing as `D-122`

The sibling suite states a second claim this record had left uncited: `D-091`, XVPE owns the
generic package-management platform. Nothing above adopted it, so until now it sat exactly
where `D-122` sat before version 3 — a sibling's unilateral position, true of XVPE's own
product and non-binding here by the rule stated two sections up. This record's own projection
rule applies to it the same way: a statement from another authority becomes enforceable here
by a recorded step, not by standing nearby in an ingested suite.

**It is adopted, as a clause of this record:**

> XVPE owns the generic package-management platform: package envelope and manifest shape,
> dependency and version resolution mechanics, registries, archive ingestion, artifact
> ownership, transactional installation, rollback and recovery, profiles, integrity and
> signature primitives, `TargetAdapter` mechanics, and disk accounting.

This is the mechanism half of the crossing already drawn above —

```
XVPE generic execution primitive
        ^
        |  implemented / adapted by
        |
Nomos-specific service
```

— applied to packages rather than to execution. What that crossing already says still
governs the boundary: generic capability is consumed by adaptation, and a primitive does not
learn what a `RulePackage` or a `LanguagePackage` is in order to serve Nomos.

**What the adoption does not reach.** `D-091` names package mechanism, not package meaning,
and this record keeps the second on this side of the seam: `PackageKind`'s variant semantics
— what a `RulePackage`, a `LanguagePackage`, a `ProviderPackage` and their kin actually mean —
Nomos-specific compatibility rules, capability declarations, permission meanings, and package
activation semantics remain Nomos's, the same way `D-122`'s adoption left rule and finding
semantics on this side of the analysis-mechanism crossing. Nomos owns what a package means;
XVPE, once this clause is implemented, owns the machinery that moves, verifies, and installs
one.

**What this settles and what it does not.** `nomos-contracts::PackageKind` is declared today
and consumed by nothing, a gap `P10-PACKAGE-SEAM` already found, contradicted by the
workspace's own `publish = false`. This clause does not close that gap — no crate moves by
this record, the same as everywhere else in it — but it removes the ambiguity that gap was
sitting in: the direction any implementation takes is now decided, not argued fresh the next
time somebody reaches for a package registry. `D-091`'s own list — envelope, resolution,
registries, ingestion, ownership, transactions, rollback, profiles, signatures, disk
accounting — is the list of mechanism `PackageKind`'s eventual consumer must not reimplement.

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

`ARC-HARNESS-001` is untouched and its table is unweakened. It decides who owns each harness
component and names "run history" as Nomos content in an XVPE store; it explicitly declines to
draw the arrow from a run's observations to KWB, naming this item instead. This record draws
that arrow. Nothing in `ARC-HARNESS-001`'s ownership table changes: the run history it assigns
to Nomos content is exactly the observation half of the crossing above, and what happens to it
after a harness exists to produce it is this record's question, not that one's.

`P10-PACKAGE-SEAM` is untouched. It found `PackageKind` declared and consumed by nothing,
contradicted by `publish = false`, and left the remedy's direction open. This record does not
implement a remedy — it decides the direction one would take, the mechanism half to XVPE, the
meaning half staying here — and leaves the item itself exactly where it was, done and
unrevised.

## What This Record Does Not Do

No crate moves. No package is renamed. No dependency changes. No KWB or XVPE integration is
implemented, and none is scheduled here. `PackageKind` is not touched, and neither is
`nomos-contracts`: the adoption above states a direction, not a migration, the same as `D-122`
stated a direction for shared analysis mechanisms without moving `nomos-analysis`. No harness
that records a run's observations exists either — `ARC-HARNESS-001` proposes one and has not
been built — so the third crossing above governs a step nothing yet takes. `AGENTS.md` and
`CLAUDE.md` are not touched.

Nothing mechanical enforces this boundary, and that is the honest state rather than an
oversight. There is no test that can decide whether a subsystem's semantics are fully
explained by a lower layer — that is meaning, and this workspace has no type for it. What
this record buys is that the next placement argument is settled by reading rather than by
whoever is most recently convinced, and that a future enforcement, when something concrete
enough to enforce exists, has a criterion to be derived from.

## Status

Closed by `P10-ECOSYSTEM-BOUNDARY`. Amended to version 2 by `P10-SEAM-CITATION`, which found
the seam already recorded in the sibling suites this build ingests, declared the edges to it,
and adopted `D-122`. Amended to version 3 by `P11-ECOSYSTEM-UPWARD`, which drew the third
crossing — a run's own observations becoming KWB knowledge — answering the arrow
`ARC-HARNESS-001` named and left undrawn, with a provenance requirement on the crossing and a
second-source requirement on the review that promotes a lesson into a rule. Amended to
version 4 by `P13-XVPE-PACKAGE-ADOPT`, which adopted `D-091` on the same footing as `D-122`:
the generic package-management platform moves toward XVPE, `PackageKind`'s semantics stay
here, and `P10-PACKAGE-SEAM`'s open remedy now has a direction without being implemented.
