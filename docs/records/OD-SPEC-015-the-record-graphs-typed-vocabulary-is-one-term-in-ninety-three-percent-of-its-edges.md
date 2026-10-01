---
id: OD-SPEC-015
type: decision
title: The record graph's typed vocabulary is one term in ninety-three percent of its edges
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - spec
  - records
relations:
  - target: OD-SPEC-011
    type: relates-to
  - target: OD-SPEC-012
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
---

# The record graph's typed vocabulary is one term in ninety-three percent of its edges

## Question

The relation vocabulary is enforced by name — `OD-SPEC-011` made an unknown term a diagnosable
refusal rather than a foreign-key failure — and `OD-SPEC-012` gave every type a declared
domain, range and cardinality. The mechanism works. What nothing has measured is whether the
authoring feeding it is honest, and the answer is that one term carries almost all of it.

## What Was Measured

Measured 2026-09-13 over all 231 files in `docs/records/`.

**831 authored edges, across four terms:**

| term | edges | share |
|---|---|---|
| `relates-to` | 774 | 93.1% |
| `affects` | 51 | 6.1% |
| `affected_by` | 5 | 0.6% |
| `supersedes` | 1 | 0.1% |

**Every record declares at least one edge.** There are no orphans, so the concentration is
not an artefact of most records declaring nothing.

**The committed diagram is what it costs a reader.** `diagrams/relations.mmd` draws 1,634
arrows, of which **1,522 are labelled `relates-to`**, against 55 `affects`, 55 `affected_by`,
1 `supersedes` and 1 `superseded_by`. `relates-to` is declared as its own inverse, so each
authored pair appears once in each direction carrying the same word. Somebody opening the one
relation projection this repository commits, to learn how these decisions stand to one
another, is handed fifteen hundred arrows that say only that two records are connected.

**A record can only reach five of the nine seeded terms.** `supersedes`/`superseded_by`,
`affects`/`affected_by` and `relates-to` declare `RECORD_KINDS` — `architecture` and
`decision` — as both domain and range. The other four declare corpus node kinds:
`answers`/`answered_by` over `design-spec` and `feature-request`, `implements`/`implemented_by`
over `feature-result` and `design-spec`. **No record could carry one of those four**, and
`OD-SPEC-012`'s constraint would refuse it. Their absence is not neglect and must stop being
read as any.

**This is current practice, not historical drift.** Five records were authored in the session
this one was written in. They added 11 edges. All 11 are `relates-to`.

## The Decision

**`relates-to` is the honest term for what it says, and a 93 per cent share is not by itself a
defect. But the share is inflated by one measurable cause, and naming it is what this record
is for: the vocabulary has no term for a record that reasons from another one.**

### Why the share is not a defect on its own

The three terms available to a record are not three shades of the same thing. `supersedes`
means one record replaces another wholesale. `affects` means one record changes what another
one says. `relates-to` is declared symmetric — its own inverse — and means two records bear on
each other.

Most pairs of decisions about one subject genuinely are symmetric, and this repository has
already decided, in writing, that a true general edge beats a false specific one. The seed
table's own comment records it at the moment `affected_by` was added rather than forcing
`affects`: "`OD-CAPABILITY-002` borrows `OD-STORE-001`'s criterion; it does not affect it, and
a wrong edge in the graph this system exists to keep honest is worse than a vocabulary one
term short."

That authors reach for `affects` 56 times when it fits is the evidence the choice is being
made rather than defaulted.

### The cause the measurement exposes

The most common relationship between two governing records in this repository is neither
replacement nor effect. It is **citation**: this record reasons from that one — borrows its
criterion, applies its ranking, declines to reopen it, inherits its measurement.

The vocabulary cannot say that. `supersedes` is too strong, `affects` is false (a record that
cites another changes nothing about it), and what is left is `relates-to`. So a directional,
asymmetric, extremely common relationship is recorded with a symmetric term, and the direction
— which record is reasoning from which — is lost on the way in.

Every one of the 11 edges added while this record was written is that shape. They cite
precedent, and none of them affects what it cites.

### Why no term is added here

The obvious remedy is a sixth seed term, and it is refused for a stated reason rather than an
aesthetic one. The seed table declares its own tier as `seed` and says why: "the real
vocabulary is `ADR-ARTIFACT-GRAPH-002`'s, and it arrives with the corpus and supersedes this
whole table." Adding a term to a table already declared temporary means authoring the same
vocabulary twice and then reconciling two versions of it, which is a worse trade than the
information currently lost.

What this record does instead is make the gap legible, so the corpus vocabulary arrives to a
question already framed rather than to a 93 per cent figure nobody has explained.

### The criterion an author uses today

Stated because its absence is what lets the share be read as laziness:

- **`supersedes`** — this record replaces that one. One edge in the whole corpus, correctly.
- **`affects`** — this record changes what that one says. Use it when the cited record would
  have to be amended to stay true. Its inverse `affected_by` is written only when the
  direction genuinely runs that way and writing `affects` from the other end is not available.
- **`relates-to`** — everything else, including citation. It is correct, and it is not
  precise, and until the corpus vocabulary lands those are the same answer.

An author who is unsure between `affects` and `relates-to` should write `relates-to`. A wrong
specific edge is worse than a general true one — that is not new here, it is the seed table's
own already-recorded reasoning, restated where an author will look for it.

## What This Record Does Not Do

It does not re-author any relation. The 774 `relates-to` edges stay as they are; the record
that judged them a defect would owe a per-edge measurement this one deliberately does not
make.

It does not change the seed table, add a term, or alter any declared domain, range or
cardinality.

It does not change any projection, and `diagrams/relations.mmd` is not regenerated for
content reasons by the item that landed this — only, as always, because a record body moved.

It does not decide what `ADR-ARTIFACT-GRAPH-002`'s vocabulary should contain. It states one
thing that vocabulary will have to answer for, with the measurement attached.

## Status

Accepted. `relates-to` is honest for a symmetric bearing between two decisions and its 93 per
cent share is not a defect in itself; the share is inflated because citation — the commonest
relationship between two records here — has no term and is directional, which a symmetric term
cannot carry. No term is added, because the table it would join is already declared superseded
by the corpus vocabulary. The four corpus-scoped terms are structurally unavailable to a
record and their absence is not neglect.
