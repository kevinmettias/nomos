---
id: OD-CAPABILITY-004
type: decision
title: An optional knowledge capability that is absent is reported by the packet that carries its resolution, not by the registry and not through a rule's applicability
status: closed
version: 1
authority: canonical-normative-record
tags:
  - capability
  - knowledge
  - reporting
relations:
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# An optional knowledge capability that is absent is reported by the packet that carries its resolution, not by the registry and not through a rule's applicability

## Question

A context builder is proposed that asks a knowledge capability for a bounded,
provenance-backed packet — prior decisions, known failure patterns, previously rejected
approaches — and that stays fully functional when no such capability is installed.
`ARC-ECOSYSTEM-001` already permits the seam. The open question is what the seam returns
when nobody answers.

*No relevant prior knowledge exists* and *no capability was installed to look* are different
facts with different remedies. An empty packet reports them as the same one, and graceful
degradation is exactly the mechanism that makes the conflation silent: the run continues and
looks complete.

## What Was Measured

Read against the registry as it stands, rather than against the description of it in the
item.

**The registry already tells absence from an answer.** `Registry::Resolve` returns
`Resolution::Unsatisfied { reason: Unmet::NoProvider }` when a contract is declared and
nothing offers it, `Unmet::Undeclared` when nothing declares it, and
`Resolution::Satisfied { .. }` when a provider answers. Those are distinct values of a
two-variant enum, and the two absences are distinct from each other because their remedies
differ — authoring a contract is not installing a provider.

Nothing asserted any of it. Three tests in `registry.rs` now do, and the item's first branch
is closed by them rather than by new machinery: the seam could always say this, and no test
said so.

**The registry cannot tell an empty answer from a useful one, and must not learn to.**
Whether a provider that answered found anything *relevant* is a statement about content.
`Resolve` never sees a payload — it is a function of declarations and offers alone — and a
registry that reported on content would be grading a provider's work, which is the defect
`OD-CAPABILITY-002` put a ceiling in place to prevent one level up. So the missing half of
the distinction is not missing from the registry. It was never the registry's to hold.

**Every absence reports as `Applicability::MissingCapability` today.** All four `Unmet`
reasons map to it, asserted now so this record reasons from a measurement. That variant says
no installed provider offers a capability the rule *requires*.

## The Decision

### 1. The distinction lives in the packet, which carries its resolution

A context builder returns a packet that carries the `Resolution` that produced it, not a
bare collection of entries. Then:

| | resolution | entries |
|---|---|---|
| nothing installed | `Unsatisfied { reason: NoProvider }` | none |
| no contract at all | `Unsatisfied { reason: Undeclared }` | none |
| answered, nothing relevant | `Satisfied { .. }` | none |
| answered, something relevant | `Satisfied { .. }` | some |

Both values already exist and are already computed at the seam. The only way to lose the
distinction is to discard the resolution and hand back the entries — which is precisely what
a builder returning `Vec<Entry>` would do, and why the obligation is stated as a *type* one.
A packet whose absence and whose emptiness are the same value is refused by this record.

### 2. `Applicability` does not grow a variant, and this is where the shape stops matching `OD-CONTRACTS-002`

The two look identical and they are not, so the difference is stated rather than left to be
rediscovered.

`OD-CONTRACTS-002` added `Applicability::AgentRequired` because a rule *binds* a subject and
cannot reach a judgment mechanically. The thing being reported is a rule's verdict on a
subject, which is exactly what `Applicability` is the vocabulary for; the only question was
which value.

An optional knowledge packet is not a rule's verdict on a subject. It informs how work is
planned before any rule runs, and no subject's judgment is weaker for its absence. Routing it
through `Applicability` would be a category error, and paying for that error in
`nomos-contracts` — the crate every non-Rust peer reimplements, where a variant is a
published protocol commitment — would export it to systems that have never seen this
repository.

`MissingCapability` is therefore also the wrong report for an absent optional capability, and
for the same reason `OD-CONTRACTS-002` found it wrong for an agent-required subject: it
points a reader at installing something, and produces coverage debt no installation is
obliged to pay. The remedy differs because the diagnosis differs. There, the run had a
verdict to give and no value to give it in. Here, the run has no verdict to give at all — the
capability is optional, and the harness is complete without it.

### 3. What a run reports, and that it is never silence

Silence is refused on the grounds `OD-CONTRACTS-002` established. What replaces it is the
packet's own provenance, reported wherever the packet is reported: *this context was built
without knowledge, because none was installed* is a different sentence from *this context was
built with knowledge, which had nothing to add*, and a reader acts differently on each.

Today no run makes the request. Nothing in this build resolves a knowledge capability, so
there is currently nothing being reported silently — the exposure is entirely prospective,
and saying otherwise would be inventing a defect to make this record sound urgent. What this
record does is bind the first builder, so the packet is right before anything depends on it.

### 4. The registry is unchanged

No new `Unmet` reason, no optionality flag, no third kind beside providers and executors.
`WF-006` makes executors a kind distinct from providers and the item asks whether a knowledge
provider needs a third. It does not, because nothing here needs the registry to know that a
requirement was optional: optionality is a property of the *caller's* reaction to
`Unsatisfied`, not of the resolution. A caller that must have an answer treats `Unsatisfied`
as fatal; a caller that is complete without one records it and continues. The registry gives
both the same true answer, which is the one thing it is qualified to say.

## What Was Considered And Rejected

**An optionality flag on `Requirement`.** It reads naturally — the caller does know whether
it can proceed — and it puts a caller's policy inside the value that describes what is
wanted. The registry would then have to report differently for the same installed state
depending on who asked, which makes `Resolve` a function of the asker rather than of the
tree. Rejected: the resolution is already correct, and only the caller's reaction differs.

**A `Knowledge_Unavailable` variant on `Applicability`.** Rejected in decision 2, on grounds
of category rather than cost.

**An empty packet with a flag beside it.** Rejected by the item explicitly, and rightly: a
flag nothing is required to read is exactly as silent as no flag, and the first consumer to
ignore it reintroduces the defect with the record still on file saying it was fixed.

**Making the knowledge capability mandatory.** This removes the distinction by removing the
case, and it removes the property the seam exists for — the harness is complete without it.
An absence that cannot occur is not the same as an absence that is reported.

## What Holds It

Three tests in `crates/substrate/nomos-capability/src/registry.rs`:

- **absent and answering are different resolutions** — the item's first branch, and the one
  a builder discards by returning a bare collection;
- **undeclared and unoffered are different absences** — documented on `Unmet` since it was
  written, asserted now, because the remedies differ;
- **every absence reports `MissingCapability` today** — the measurement decision 2 argues
  from, which fails if that mapping moves underneath this record.

What does *not* hold it is anything in the registry about relevance, deliberately: there is
no mechanism there and this record says why there must not be one.

## Status

Closed by P11-KNOWLEDGE-ABSENT. No successor item is opened, because there is no context
builder to build and an item reserving work nobody has scheduled would be a placeholder on a
board that refuses them. The obligation in decision 1 binds whichever item builds the first
one.
