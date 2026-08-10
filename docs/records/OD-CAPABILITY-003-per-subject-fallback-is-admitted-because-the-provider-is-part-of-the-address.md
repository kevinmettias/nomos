---
id: OD-CAPABILITY-003
type: decision
title: Per-subject fallback is admitted, because the provider is part of the address
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - analysis
  - provenance
relations:
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-ANALYSIS-001
    type: relates-to
  - target: OD-STORE-001
    type: relates-to
---

# Per-subject fallback is admitted, because the provider is part of the address

## Question

`OD-CAPABILITY-001` settled which of several usable offers answers, and `P8-SELECTION`
built it: the strongest usable offer is chosen, and every other usable offer comes back
beside it so the caller can see what it was chosen over. `Selection::Weaker` is what a
lowered floor buys.

Nothing spent it. A caller lowers its floor to `Approximate` because a parser refuses seven
files in a corpus of seven and a half thousand, the registry hands back the parser and the
scanner in that order, and the run asks the parser for every subject and degrades on
refusal. The coverage the floor admitted was reachable and unused, which made the floor a
requirement the caller could state and not spend.

The dispatch loop is not the hard part. This is: a `FactKey` names the provider that
answered. Falling back per subject therefore files one capability's facts under two
providers within one run, and whether that is a coherent store or two answers to one
question under one address had to be settled before the loop was written.

## The Criterion

> A store is single-valued when one question has one answer. The question is the whole key,
> and the read is what has to be single-valued — not the write.

Two things follow, and they point in opposite directions.

**Writes never collide.** `provider` and `guarantee` are components of `FactKey`, so the
parser's answer about `alpha/one.rs` and the scanner's answer about `alpha/one.rs` are two
addresses, not two values at one address. A store holding both is holding two different
facts, and each is labelled with who produced it and what they promised. That was already
true before this record and is what `OD-ANALYSIS-001` left in place when it took the
snapshot out of the key: the key says what a fact *is*, and who produced it is part of what
it is.

**Reads could become multi-valued, and that is the real exposure.** A reader asks what a
capability says about a subject. Under fallback there may be two facts that answer, and if
the reader picks by looking around the store the answer depends on the history of the run
rather than on the question. That is the failure this decision has to close.

## The Decision

**Per-subject fallback is admitted, under three conditions. Any implementation that drops
one of them is refused by this record.**

**1. The order is the selection's.** A read tries the chosen offer and then
`Selection::Weaker()` in the order the registry ranked them, and takes the first that
answers. One ordered list, one first hit, one answer — deterministic given the question and
the store's contents, which is what single-valued has to mean once more than one provider
can answer. The run does not invent an order and the reader does not invent one either;
both walk the list `nomos-capability` already produced.

**2. Every fallback is named where it happened.** A run reports which provider answered for
how many subjects, and names the subjects the chosen provider refused. A read that fell back
reports `Applicability::SupportedWithFallback`, which the registry already returns when a
named preference could not be honoured. The word means the same thing in both places — the
judgment stands and its provenance is not what was asked for — and it now also covers the
case where the provenance changed for one subject rather than for the whole run.

**3. A derived fact says how much of it was approximated.** This is the condition that does
the work. Without it, fallback is a laundering machine: the scanner covers the file the
parser refused, the directory rollup stops reporting a missing member, and a corpus that was
visibly incomplete becomes a corpus that reads as complete and sound. The rollup now counts
its approximated members and carries the count in its payload, so buying coverage cannot
also buy the appearance of precision.

## Why Not Refuse It

The alternative was to refuse per-subject fallback and tell a caller that wants coverage to
run twice — once at the parsed floor and once at the approximate floor, preferring the
scanner — and reconcile the two stores itself.

That is what the tests did before this, and it is worse in the way that matters. Two runs
produce two answers for every subject, not just for the refused ones, so the caller ends up
holding a scanner answer for `alpha/one.rs` that nothing asked for and that no rule about
precedence covers. Reconciliation would then be the caller's, written per caller, and the
precedence rule the registry already computed would be re-derived by somebody with less
information. The multi-valued store this record exists to avoid is what refusing produces.

## What Is Not Decided Here

**A rollup's guarantee is unchanged.** `GuaranteeDigest` is a key component, so weakening a
rollup's declared guarantee when a member was approximated would re-address the fact — the
same directory, two generations running, under two keys, with the second not superseding the
first. That is a bigger change than this item, and the count in the payload carries the same
information without moving the address. Recorded as open rather than settled by omission.

**Invalidation across a fallback edge is unchanged.** A read that misses the chosen
provider's key records the miss as a dependency, so the graph holds "the parser had nothing
here". Whether a parser answer *arriving later* should invalidate a rollup built on the
scanner's is a question about supersession between providers, and nothing here answers it.

## Status

Closed by `P9-FALLBACK`.
