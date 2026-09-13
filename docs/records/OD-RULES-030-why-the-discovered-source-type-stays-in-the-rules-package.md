---
id: OD-RULES-030
type: decision
title: Why the discovered-source type stays in the rules package
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
  - layering
relations:
  - target: OD-RULES-014
    type: relates-to
  - target: OD-RULES-020
    type: relates-to
  - target: OD-CONTRACTS-001
    type: relates-to
---

# Why the discovered-source type stays in the rules package

## Question

`SourceFile` is declared in `nomos-rules`, and it is what every walker returns and what the
whole orchestration passes around. `P41-SOURCE-ARTIFACT-OWNERSHIP` complained that this
inverts ownership — a discovered artifact exists before any rule does, so the discovery layer
should not depend on the judging layer to describe its own input — and asked for the type to
be owned by `nomos-workspace`, `nomos-model` or `nomos-analysis` instead.

That item has now been raised, stranded on a dependency edge that had gone stale, and declined.
It will be raised again, because the complaint is correct and nothing records the answer.

## What Was Measured

Measured 2026-09-13 over the seventeen crates naming `nomos-rules` and the zone lattice they sit in.

**The inversion is real, and it is two crates wide.** Of the seventeen crates naming
`nomos-rules`, exactly two use nothing from it but `SourceFile`: `nomos-lang-rust-cargo` and
`nomos-workspace-discovery`. Every other dependent also names rule identifiers, checks,
descriptors or the registry, so it would depend on `nomos-rules` whatever happened to this
type. And the second of the two is the discovery service itself, which is precisely the
inversion the complaint names rather than an incidental case of it.

**The move is refused by this workspace's own layering.** `SourceFile` carries
`language: Option<Language>`, and `Language` is declared in `nomos-cap-syntax`, a Capability
Contract crate. Asked directly:

| | |
|---|---|
| `nomos-model` → `nomos-cap-syntax` | **refused** |
| `nomos-workspace` → `nomos-cap-syntax` | **refused** |
| `nomos-analysis` → `nomos-cap-syntax` | **refused** |
| `nomos-rules` → `nomos-cap-syntax` | permitted |

All three candidate owners are Substrate, and `Permits` does not admit Substrate → Capability
Contract. `nomos-rules` is Rules, and does. **That is why the type sits where it sits**, and
nothing about it was accidental.

**Both halves of that constraint are accepted decisions, not accidents.**

- `OD-RULES-014` decided a rule's language restriction is *carried* rather than derived, having
  measured the alternative: eleven functions across nine modules each answering it privately
  from the file extension, "the same three lines" eleven times.
- That record's own amendment, *The Carried Type Belongs At Band 23, Not Band 0*, moved the
  language identity from `nomos-contracts` to `nomos-cap-syntax` **deliberately**, because a
  language identity fails `OD-CONTRACTS-001`'s deciding question — whether a peer that never
  compiles the crate would be unable to agree with us without the type. Nothing exchanges a
  language name with a peer; the parties that must agree are five crates inside this workspace.

## The Decision

**The discovered-source type stays in `nomos-rules`. The ownership inversion is a consequence
of two accepted decisions acting together, not an oversight, and this record exists so the
next reader who notices it finds that out instead of re-deriving it.**

### The chain, stated once so it can be checked rather than retold

1. A rule's language restriction is carried on the source, not derived by the rule —
   `OD-RULES-014`, against eleven measured duplicates.
2. The carried language identity lives at Capability Contract, not Protocol — `OD-RULES-014`'s
   amendment, against `OD-CONTRACTS-001`'s peer-visibility test.
3. Substrate may not name a Capability Contract — `Permits`, and `OD-RULES-020` is where that
   lattice was decided.
4. Therefore a type carrying a language identity cannot live in Substrate, and the three
   owners the complaint proposes are all Substrate.

Each link is a decision somebody made for a stated reason. The inversion is what those reasons
cost, and it is a smaller cost than any of them: two crates naming one type they do not judge
with.

### What would have to change first

Not "move `SourceFile`". One of the three:

- **`Language` stops being a Capability Contract type**, which means arguing against
  `OD-RULES-014`'s amendment on `OD-CONTRACTS-001`'s test — and the amendment's own measurement
  stands until something is exchanged with a peer that names a language. `PackageManifest`
  carrying `language_versions` but no language name is where that would first show.
- **`SourceFile` stops carrying a language**, which reopens `OD-RULES-014`'s main decision and
  owes a better answer than the eleven duplicates it removed.
- **Substrate is permitted to name a Capability Contract**, which is a change to the zone
  lattice and belongs in a record amending `OD-RULES-020`, argued over every edge it would
  admit rather than the one that prompted it.

None of the three is proposed here. What this record refuses is the fourth option that has been
tried twice: moving the type without touching any of them, which the lattice declines.

### What a reader should conclude on noticing the inversion again

That it was noticed before, measured, and left deliberately. The complaint is not wrong and the
answer is not "it is fine" — it is that the cost was weighed against three named alternatives
and is the smallest of the four. Raising it again is worthwhile only with an argument against
one of the three above, and an item that proposes the move alone is refused by `Permits` before
any reviewer reads it.

## What This Record Does Not Do

It does not move any type, change any crate, edit any manifest, or touch the zone table.

It does not reopen `OD-RULES-014`, its amendment, `OD-CONTRACTS-001` or `OD-RULES-020`. It
reads all four and states what they imply together, which none of them says on its own.

It does not decide that two crates depending on `nomos-rules` for one type is good. It decides
that it is the cheapest of four options, and names the other three so a future argument has
somewhere to start.

## Status

Accepted. `SourceFile` stays in `nomos-rules`, because it carries a language identity that
`OD-RULES-014` decided it must carry and that the same record's amendment placed at Capability
Contract, and `Permits` does not let Substrate name a Capability Contract — so all three
proposed owners are refused the edge the type needs. The inversion costs two crates naming one
type they never judge with, and the three changes that would make the move available are named
above.
