---
id: OD-GATE-031
type: decision
title: A comparison attributes a difference to the repository only when the rest of the judgment was compatible
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - enforcement
  - determinism
relations:
  - target: OD-GATE-022
    type: affects
  - target: OD-GATE-030
    type: relates-to
  - target: OD-DETERMINISM-002
    type: relates-to
---

# A comparison attributes a difference to the repository only when the rest of the judgment was compatible

## Question

`Compare_Gate_Runs` reports which findings were added, removed, and moved between buckets
across two runs. A caller reads that as a fact about the repository, because that is what a
gate is for. It is only that when every judgment input other than the source was compatible
between the two sides, and today nothing checks it, nothing records it, and nothing in the
result says it was not checked.

So `before ≠ after` currently means only *these two `GateRunResult` values differ*, and it is
read as *the repository changed*. Where the actual cause was a different gate policy, a
different rule selection, or a rule that could look on one side and not the other, the report
is not merely incomplete. It is a false causal story, told confidently, by the tool whose
whole purpose is to stop exactly that.

## What Was Measured

Checked 2026-09-14 against the tree, and the measurement narrowed this record's scope rather
than confirming a plan. What matters here is not the list of things that *could* vary in
principle; it is which of them can vary **between the two sides of a comparison today**.

**Two can vary through this workspace's own hosts.**

*The effective gate policy.* `Run_Gate` resolves `nomos-gate.json` from `command.root`, and
`compare` judges two roots. `nomos-cli`'s own usage text promises that every flag other than
`--against` narrows both sides alike, and that promise does not reach the policy, because a
policy file is not a flag — it is a property of each tree. Comparing two checkouts compares
two policies, and a finding that moved because a suppression was added reports as movement.

*Which capabilities could be materialized at each root.* Measured on `P106`'s probe tree: a
root with no `deny.toml` leaves `dependency-policy` unmaterialized, which changes the findings
and the claim together. Two roots differing only in that produce different findings for a
reason that is not the code, and a rule that could not look on one side is not a rule that
found nothing there.

**Two more can vary through the published library surface, though not through these hosts.**

*The whole `GateCommand`.* `nomos_api::Handle_Gate_Compare` takes two independent ones, and
its own module documentation names "one tree under two policies" as an intended case. A
narrower `rules.include` on one side means that side never looked, and the findings it
therefore lacks are reported as `added` on the other — the clearest available form of the
failure this record is about.

*`BuildVariant` and the run's moment.* Both are `GateEnvironment` fields a caller supplies per
`Run_Gate` call, and `Compare_Gate_Runs` is public and takes two finished results. This
workspace's hosts pass one variant and one clock reading to both sides; the crate publishes no
constraint saying a caller must, so the guarantee is a habit of two call sites rather than a
property of the type.

**One cannot vary yet, and that is why this record is smaller than the question looks.** The
nomos build itself, and the provider binaries it runs, are identical across both sides by
construction: `OD-GATE-022-A` decided a compare caller re-derives both runs inside one process,
builds no run-history store, and serializes no `GateRunResult`. A different tool version on one
side becomes reachable when a run outlives the process that made it, which that record defers.

**`GateRunResult` records none of it.** It carries `run`, `root`, `check_outcome`, `findings`,
`disposition`, `unmatched_policy` and `no_verdict`. Not the command, not the variant, not the
resolved policy, not the moment judged against, and not which rules were able to look.

**The vocabulary for all of this is already published, and it already says the hard part.**
`nomos-contracts`' domain table — the one `OD-DETERMINISM-002` made a derived universe — puts
"Analysis kernel — facts, checks, findings" at `StateTemporal`, `CrossPlatform`,
`BitIdentical`. Two things follow that were not obvious before reading it. `StateTemporal`
makes **time a declared judgment input**, and it genuinely is one, because a temporary waiver
stops applying against the run's own moment. And `CrossPlatform` is strictly weaker than
`CrossBinary` in that scale, so the table has always said that findings are **not** claimed to
reproduce across builds of this tool — which is precisely the claim a comparison across two
tool versions would have to rely on.

## Decision

**A comparison attributes a difference to repository state only when the non-source judgment
inputs of its two sides were compatible, or their differences are explicitly represented in
what it reports.** Never by the absence of evidence that they differed.

This is the invariant, and the rest of this record exists to make it checkable rather than
aspirational.

**A run carries the identity of what judged it, and that identity is sized to what can differ
today.** Enough to answer the comparability question for the four varying inputs measured
above, and no more:

- **the source it judged**, so that "the repository changed" is a fact the comparison can
  establish rather than the residue left when nothing else explains the difference;
- **the effective policy it judged under**, after `nomos-gate.json` was resolved over the
  command, because that is the input most likely to differ and least likely to be noticed;
- **the selection it judged with** — which rules were allowed to count, and which paths were in
  scope — because a side that did not look has no findings to contribute and must not read as a
  side that looked and found nothing;
- **the instrument** — the `BuildVariant` and the registered rule set with its contract record
  versions — because two different instruments measuring the same tree is the case the domain
  table already declines to claim reproducibility for;
- **the moment**, because `StateTemporal` says so and waivers expire.

**Which rules could actually look is not a sixth component, because it is already derivable.**
A rule whose provider could not run reports a finding carrying `MissingCapability` or
`ProviderUnavailable`, and `Claim_Of` already reads exactly those. A comparison holds both
sides' findings, so it can establish that difference from what it already has. Adding a field
for it would be a second encoding of a fact the findings carry, which is the defect class
`OD-GATE-011` names.

**A comparison reports its own comparability, in three states.** Compatible, where every
non-source input agreed and a difference is therefore a difference in the repository.
Compatible-with-stated-differences, where something else did differ and the report names what,
so a reader attributes the difference themselves rather than being told a story. And
incomparable, where the two sides cannot be meaningfully diffed at all.

**Unknown provenance is incomparable, not compatible.** A run that carries no identity, or one
whose identity a comparison cannot interpret, resolves to incomparable rather than being
assumed to match. This is the same discipline `Applicability` holds one layer down, where
`MissingCapability` is kept apart from a clean result so that "nothing could look" never reads
as "nothing was wrong", and the same one `OD-GATE-030` holds for a baseline entry whose
continuity cannot be shown. A comparability claim made from the absence of evidence is the
failure this record exists to prevent, and it would be perverse to introduce it in the
mechanism meant to stop it.

## What This Does Not Decide

**It does not schedule a run history.** `OD-GATE-022-A` owns that deferral and keeps it. This
record adds a third consumer to the case for one — after `OD-GATE-030`'s second — and does not
schedule it. Everything decided here is answerable inside one process, which is where a
comparison lives today.

**It does not add provenance for what cannot differ.** A provider binary's version, the host
operating system, and the identity of the nomos build itself are all constant across a
same-process comparison. Recording them now would be shape ahead of a body: a field nothing can
make differ is a field no test can prove is read. They belong to the same increment as the run
history, and their absence is stated here so that a later reader knows it was a decision.

**It does not reopen `OD-GATE-030`.** A baseline entry's snapshot identity, and the occurrence
continuity that would make `PersistentDebt` and `Reintroduced` separable, are that record's
question. The identity decided here is of a *run*, not of a baseline entry, and the two are not
the same thing even though both are provenance.

**It does not decide what a caller should do about an incomparable pair.** Whether a
continuous-enforcement loop treats stated differences as tolerable, or refuses to proceed, is a
policy question of the kind `nomos-gate.json` already carries, and it belongs with the others
rather than in the meaning of a comparison.

**It builds no mechanism.** The follow-up work is a value on `GateRunResult` filled by
`Run_Gate`, a comparability verdict on `Compare_Gate_Runs`' own result, and the rendering of
both at the two surfaces — named precisely enough here for an item's territory, and not
written here.

## Status

Accepted. A comparison may attribute a difference to the repository only when the non-source
judgment inputs were compatible or their differences are represented, and never from the
absence of evidence. A run carries the identity of what judged it — source, effective policy,
selection, instrument, moment — sized to the four inputs that can differ between two sides
today rather than to the eventual shape. Which rules could look stays derivable from the
findings rather than duplicated. A comparison reports compatible, compatible with stated
differences, or incomparable, and unknown provenance is incomparable. `OD-GATE-022-A`'s
run-history deferral and `OD-GATE-030`'s baseline continuity question are both untouched. No
mechanism is built here.
