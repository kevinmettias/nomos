---
id: OD-CAPABILITY-015
type: decision
title: Whether a bundled contract crate doing real I/O may be classified Capability Contract zone
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - rules
  - zones
relations:
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-RULES-020
    type: relates-to
  - target: ARC-CONNECTOR-001
    type: relates-to
---

# Whether a bundled contract crate doing real I/O may be classified Capability Contract zone

## Question

`nomos-connector-coderabbit` runs `gh api` as a subprocess and `nomos-cap-requirement-trace`
reads arbitrary files across the tree, and both are classified `Zone::CapabilityContract`
beside nine siblings that are pure declarations. The zone table says why, in its own
comments: `Provider` zone "would leave this fact family structurally unreachable by
`nomos-rules`, not merely misfiled." `README.md` says the same for both rows.

So the label was chosen by what the dependency graph permits rather than by what the crate
does. `OD-CAPABILITY-002` licenses the *bundling* and its one-provider criterion is untouched
by this — but it decides nothing about the zone consequence. Two instances exist, so this is
not a population of one.

## What Was Measured

**The two crates violate no rule.** `Permits` grants
`Specification | CapabilityContract => matches!(to, Protocol | Substrate)`, and
`nomos-platform` — which declares `FileSystem` and `ProcessLauncher` — is `Zone::Substrate`.
A Capability Contract crate reaching a platform port is therefore already permitted,
zone-wide, and was before either of these crates existed. Neither is smuggling anything past
the zone model.

**The zone's other members do not use that permission.** Sampled `nomos-cap-syntax`,
`nomos-cap-naming-policy`, `nomos-cap-limits-policy` and `nomos-cap-dependency-policy`: zero
files in each name `nomos_platform::FileSystem` or `nomos_platform::ProcessLauncher`.

**The split this would otherwise owe already exists, five times, in the same zone.** The five
`nomos-cap-*-policy` contracts are pure payload declarations; `nomos-repo-policy` is
`Zone::Provider` and is where the `FileSystem` read of `standards.json` actually lives
(`crates/repository/nomos-repo-policy/src/goals/fact_context.rs`). Contract in Capability
Contract, provider doing the I/O in Provider, is not a shape this workspace would have to
invent — it is the dominant shape of this very zone.

**The table's reachability claim is true.** `Provider | Rules => matches!(to, Protocol |
Substrate | CapabilityContract)`: `Rules` may not name `Provider` at all. Classifying either
crate `Provider` would make its fact family structurally unreachable from `nomos-rules`,
exactly as the comment says.

**No test asserts anything about this.**
`Test_Dependencies_Should_Run_Strictly_Downward` (`tests/contract/tests/boundaries/graph.rs`)
walks every real dependency and asserts `Permits(zone, dependency_zone)`. That is the whole
of what the boundary tests say about the Capability Contract zone, and it is the wrong
direction to catch a misclassification: moving a row from `Provider` to `CapabilityContract`
makes that assertion *easier* to satisfy, never harder. `Test_A_Capability_Id_Should_Be_
Written_In_One_Crate` — `OD-CAPABILITY-002`'s own guard — constrains where an id is written,
not which zone its crate sits in. So the zone table's rows are load-bearing prose that
nothing checks, which is the precise reason this question is worth a record rather than a
comment.

## The Decision

**Both classifications stand, and the reason the table gives for them is retired.**

A crate is Capability Contract because it **declares a capability contract** — and, under
`OD-CAPABILITY-002`, may bundle that contract's single provider in the same crate while
there is exactly one. It is never Capability Contract *because `nomos-rules` needs to reach
it*.

The distinction is not wordplay, and it is the whole content of this record. Reasoning from
the edge to the label makes `Rules ↛ Provider` vacuous: anything a rule turns out to need is
relabelled to whatever lets it be reached, and the one edge the zone model draws to keep
judgment away from acquisition stops constraining anything. Reasoning from the criterion to
the label leaves that edge meaning what it says, and happens to classify both of these
crates the same way today — which is why this costs nothing to adopt and is worth writing
down before a third case arrives with less obvious merits.

**What would owe a split is unchanged, and it is `OD-CAPABILITY-002`'s trigger, not a new
one:** a second provider for the same contract. At that moment the bundling license lapses on
its own terms — "the first can change the ceiling, the version or the schema its peer is
bound by, and the peer cannot see the file" — and the crate splits into a pure contract that
stays Capability Contract and a provider that becomes Provider, which is what the five policy
contracts already look like. Nothing about doing I/O triggers it, because doing I/O through a
port is a permission this zone already has.

**What the boundary tests assert under this answer, stated so nobody mistakes silence for
coverage:** that every real dependency runs downward through `Permits`, and nothing else.
Whether a member's zone matches what the crate *is* remains unchecked by any test, and is
held by this record and by the zone table's own comments. A future increment that wanted it
mechanical would need a property the graph does not currently carry — which port a crate
reaches is visible, but "declares a contract" is not — and this record does not propose one
ahead of a case that needs it.

## What This Record Does Not Do

It does not split either crate. Neither has a second provider, so neither has reached
`OD-CAPABILITY-002`'s trigger, and splitting now would buy two crates and no property — the
same trade that record already refused.

It does not touch `Permits`, the zone table, `README.md`, or any code. The two rows' comments
still state the edge as their reason and should state the criterion instead; that is a
correction against files this record does not hold, and is filed separately rather than done
here.

It does not revisit `OD-CAPABILITY-002`'s one-provider criterion, which this record depends
on rather than amends.

## Status

Accepted. Both crates stay Capability Contract, on the criterion rather than on the
dependency edge. Doing real I/O through a platform port does not disqualify a member of this
zone, because `Permits` has always granted it; being reachable from `nomos-rules` does not
qualify one, because that reasoning would empty the `Rules ↛ Provider` edge of meaning. The
split stays owed at `OD-CAPABILITY-002`'s existing trigger, a second provider, and the
boundary tests are recorded as asserting downward dependency only.
