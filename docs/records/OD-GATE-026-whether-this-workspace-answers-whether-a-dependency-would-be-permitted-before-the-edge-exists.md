---
id: OD-GATE-026
type: decision
title: Whether this workspace answers whether a dependency would be permitted before the edge exists
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - architecture
  - layering
relations:
  - target: OD-RULES-020
    type: relates-to
  - target: OD-RULES-029
    type: relates-to
  - target: OD-RULES-003
    type: relates-to
---

# Whether this workspace answers whether a dependency would be permitted before the edge exists

## Question

Two independent brainstorming loops put the same step before implementation: have Nomos
validate the proposed dependency before the work starts. Nomos answers only afterwards.
`Check_Dependency_Direction` reads `nomos.cap.dependency.edges`, which `nomos-lang-rust-cargo`
derives by running `cargo metadata` over manifests on disk — so an edge is judged once it has
been written, and never before.

The counter-argument has to be stated first, because it is good: `nomos check` over this
workspace already answers the question in one run. What is missing may be a convenience rather
than a capability.

## What Was Measured

Measured 2026-09-13, against this workspace.

**The primitives are already public, total and free of input.** `Zone_Of(name) -> Option<Zone>`
maps a crate name to its zone, `Permits(from, to) -> bool` answers a zone pair, and
`SAME_ZONE_EDGES` names the 13 peer exceptions the lattice cannot express. All are exported
from `nomos-rules` and all appear in its blessed surface snapshot. The whole answer is
`Permits(Zone_Of(a)?, Zone_Of(b)?)` plus one lookup — **no manifest, no `cargo metadata`, no
walk, no store, no reader.**

**Nothing asks.** `Permits` has exactly two non-test callers in the workspace:
`checks/dependency/violations.rs`, judging an edge that already exists, and
`tests/contract/tests/boundaries/graph.rs`, asserting the real graph. No host verb reaches it.
`nomos-lsp` names `Zone_Of`, in `walk_outward/architectural_component.rs`, only to label which
architectural component a finding falls in.

**The two answers are not in the same cost class, and the gap is not small.** Timed on this
machine over this workspace:

| | |
|---|---|
| `nomos gate run --root .` — the retrospective answer | **13,127 ms** |
| `nomos gate explain --root .` — the existing answer-about-the-rules verb | **40 ms** |

A prospective admissibility answer is in `explain`'s class, not `run`'s: it is a pure function
of two strings. And the 13 seconds is not the real cost of the retrospective route — the real
cost is that getting it at all means **writing the edge first**, running, reading, and
reverting.

**The cost of learning late is recorded rather than imagined.** `OD-RULES-020` replaced the
band number line because crates had to be renumbered for dependencies nobody disputed, and
names the worst case: `nomos-workflow-orchestration` renumbered three times across two items in
one session as each new dependency landed.

## The Decision

**Yes. This workspace answers the prospective question, as a fourth gate verb over a crate
pair, with three outcomes rather than two.**

### Whose question it is: the gate, with the seam below the verb

`gate` already owns "answer about the rules without running them" — that is what `Explain`
does, and it is the third verb for exactly this reason. A prospective admissibility answer is
the same shape and belongs beside it.

The answer lives in `nomos-gate-orchestration` beside `Explain_Gate`, and the CLI verb is a
transport onto it. That ordering is the load-bearing half: an LSP code action is the obvious
second surface, `nomos-lsp` already names `Zone_Of`, and a verb that computed the answer inside
`nomos-cli` would force that code action to compute it a second time. `OD-HOST-002` already
decided which side of that line a host sits on.

### What the subject is: a crate pair

Not a proposed manifest edit, and not a planned change-set.

The measurement decides it: the answer is a pure function of two crate names, so a manifest
edit adds a parse that yields exactly the pair, and a change-set is *n* pair questions with a
roll-up. Neither carries information the pair does not, and both add a surface that can be
wrong about something the pair cannot be wrong about. A caller wanting either builds it on the
pair.

### What it says when a zone is unknown: it does not judge, and says so

This is the case that decides whether the verb is safe, and it has already been decided
elsewhere. `Zone_Of` returns `None` for a crate with no declared zone, and that is the normal
state of every crate in every other repository.

Three outcomes, never two:

- **permitted** — both endpoints have a declared zone, and `Permits` or `SAME_ZONE_EDGES`
  admits the edge;
- **refused** — both have a declared zone and neither admits it;
- **not judged** — either endpoint has no declared zone.

The third is not an error and not a default. `OD-RULES-003` decided that a repository with no
declared architecture gets `Applicability::NotApplicable`, "the only variant that is a
positive statement about the absence of a judgment", and this session's work on
`Check_Dependency_Completeness` already separates "declared, and this member is missing from
it" from "declared nothing at all". The verb inherits that distinction rather than inventing a
second vocabulary for it.

**What it must never do is default to permitted.** A prospective check that answers "fine" for
a crate it has never heard of is worse than no check, because it is consulted precisely by
someone who does not yet know the answer.

### Why the counter-argument does not carry

`nomos check` answering retrospectively is a different answer to a different question. It
requires the edge to exist, which means the developer has already done the thing they wanted
checked — and `OD-RULES-020` records what that costs when the answer turns out to be no.

"Convenience rather than capability" is settled by the measurement: a total, public, pure
function with no host surface is not a convenience being withheld, it is a seam nobody
finished. The verb is roughly the arity of `explain`, which this workspace already ships.

### What `OD-RULES-029` changes about this, and what it does not

`OD-RULES-029` decided the layering declaration becomes data read from the repository under
check, and that two of `OD-RULES-003`'s three prerequisites are already built. A verb built now
reads `ZONES` and `Permits` as they are.

That is deliberate and it is not a trap, because **the verb asks the same question either
way**. `Zone_Of` and `Permits` are where the answer comes from today and where it will come
from after the migration; what changes is where those get their data. The verb must therefore
ask through those two functions and must not reach around them to `ZONES` directly — which is
the one constraint this record puts on whoever builds it.

## What This Record Does Not Do

It does not build the verb, the seam, or an LSP code action. It does not change `nomos-rules`,
`nomos-gate-orchestration`, `nomos-cli` or `nomos-lsp`.

It does not decide the verb's spelling, its exit codes, or its output format. Those are the
building item's, against `README.md`'s existing table.

It does not reopen `OD-RULES-029` or schedule its migration. It states the one constraint that
keeps the two compatible.

It does not extend the prospective answer beyond dependency admissibility. Whether any other
rule has a prospective form is a separate question and no measurement here bears on it.

## Status

Accepted. The prospective question is answered, by a fourth gate verb over a crate pair,
seamed in `nomos-gate-orchestration` beside `Explain_Gate` so a second surface reuses it, with
three outcomes of which the third — not judged, when an endpoint has no declared zone — is what
keeps it from answering "fine" about a crate it has never heard of.
