---
id: OD-HOST-020
type: decision
title: A provider set is selected once by a composer, and the service and the crates that run it receive it rather than choose it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - architecture
  - layering
relations:
  - target: OD-ROADMAP-005
    type: relates-to
  - target: OD-HOST-001
    type: relates-to
  - target: OD-HOST-019
    type: relates-to
  - target: OD-CAPABILITY-008
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
  - target: OD-PACKAGE-014
    type: relates-to
---

# A provider set is selected once by a composer, and the service and the crates that run it receive it rather than choose it

## Question

`OD-ROADMAP-005` decision item 1 authorizes `nomos-check-orchestration` to stop naming the
capability, language and repository provider crates directly and to receive them from a
composition root. Two authorings of the item that would build it were declined, each for
asserting a design that did not cover its own callers. Where is the provider set selected, how
does every crate that runs the check obtain it, and what does that cost?

## What Was Measured

At `d2467842`, on 2026-09-26.

**The set is built inside the service, twice over.** `nomos-check-orchestration` builds its
provider table inside `Run_Reassessing` at `src/run_context.rs` line 150 and its registry inside
`Run` at line 228. The table is `src/composition/provider_table.rs`, 317 lines of code beside
157 of documentation; the registry is `Registered` in `src/composition.rs`, whose `Offer` calls
name provider crates. `ComposedProviders` has sixteen fields across six distinct port types.
`nomos-cli` also calls `Registered` directly, at `src/profile.rs` line 107.

**It is not only the hosts that run the check.** `RunContext` is constructed at 34 sites in 10
crates. Four are hosts. Three are Application Service crates that call the check themselves:
`nomos-gate-orchestration` in `Judged_Sources` at `src/gate_environment.rs` line 59,
`nomos-correction-orchestration` in `Judged_Findings` at `src/run.rs` line 157, and
`nomos-workflow-orchestration` in `Dispatched_Check` at `src/run.rs` line 393. The rest are test
fixtures, in `nomos-check-orchestration`, `nomos-lsp`, `tests/contract` and `tests/integration`.

**The declared layering leaves no shared home.** `nomos-architecture.json` has no Orchestration
component; all four of those service crates are `Application Service`. `Application Service`
may reach `Provider` but may not reach `Application Service`, and `Host` may reach `Provider`
but may not reach `Host`. So no existing component holds a crate that every caller of the check
may depend on and that may itself name a provider, except `Agent`, whose members are agent
executors and which would be misdescribed by holding provider composition.

**Those three crates already receive the platform rather than choose it.** `GateEnvironment`
at `src/gate_environment.rs` line 77, `CorrectionEnvironment` at `src/run.rs` line 70 and
`nomos-workflow-orchestration`'s platform value each carry a launcher, a filesystem and an
environment their caller supplies, and each is already generic over all three.

**This problem has been solved once, one layer down.** `nomos-composer-std` exists because
hosts restated the same platform choices at 34 production sites in `nomos-api` and 22 in
`nomos-cli`. Its answer is a crate that selects the std backend set once, orchestrates nothing,
and is named by roots in place of each implementation. `OD-HOST-001` says at line 29 that naming
a concrete provider in a composition root was never the defect.

**What the moved code names**, measured over `composition.rs`, `composition/provider_table.rs`
and `composed_providers.rs`: provider crates (`Provider`), capability contracts (`Capability
Contract`), `nomos-analysis`, `nomos-capability`, `nomos-model` and `nomos-platform`
(`Substrate`), `nomos-contracts` (`Protocol`), and `nomos-check-orchestration` itself, for the
port types. `nomos-rules` appears in live code only in three `assert_eq!` lines inside a test
that checks each language constant agrees with its provider's; its other mentions are comments.

## The Decision

### 1. The set is selected once, by a composer, the table and the offers together

A new crate, `nomos-composer-providers`, under `crates/composer/` beside `nomos-composer-std`,
selects the standard provider set exactly once. It builds the `ComposedProviders` table and the
registry's provider offers together, because they are one choice made twice today: a provider
is offered against a capability in the registry and called through the table, and a set in
which the two disagree is a defect nothing currently prevents. Selecting both in one place makes
them one list.

It selects and orchestrates nothing, in the sense `nomos-composer-std`'s own documentation gives
that phrase: no verb, no sequence, no type that exists to be rendered. It is a library, not a
composition root. Roots name it.

### 2. The service receives the set and names no provider

`RunContext` gains the field the table arrives in, and the registry arrives beside it.
`nomos-check-orchestration` names no language provider, repository policy provider or connector
crate in its library dependency graph. `Run` and `Run_Reassessing` keep their shape.

### 3. The crates that run the check receive the set; they never select it

`nomos-gate-orchestration`, `nomos-correction-orchestration` and `nomos-workflow-orchestration`
obtain the set from their caller, through the context structs they already carry, as one more
field beside the platform they already receive. None of them names the composer.

This is the line between the defect and its cure, and it is drawn at who chooses rather than at
who holds a value. A service that holds a provider set its caller handed it has chosen nothing.
A service that calls the composer has chosen the standard set on its caller's behalf, which is
the composition root disguised as an application service that `OD-ROADMAP-005` removes, moved
one crate over. So the three may hold the set and may not select it.

### 4. `Composer` is its own component, and Application Service is refused

`Composer` is a new declared component in `nomos-architecture.json`, holding
`nomos-composer-providers`.

**Its permits are exactly what its library code names, measured above:** Protocol, Substrate,
Capability Contract, Provider and Application Service. **`Host` gains `Composer`.** No other
component gains it; in particular `Application Service` does not, which is what makes section
3 a boundary the declaration enforces rather than a convention a reader must remember.

**Rules is deliberately not among its permits.** Its only live use of `nomos-rules` is a test
assertion, which is a dev-dependency concern and not a reason to widen the component a library
lives in.

**Placing the composer in `Application Service` is refused here rather than discovered later.**
It would work under today's permits, because that component may reach `Provider`. But it would
put a crate that orchestrates nothing in the component named for orchestration, and, since
`Application Service` may not reach `Application Service`, it would still leave the three crates
of section 3 unable to reach it, so the placement would buy nothing section 3 needs and misstate
what it holds.

**Any arrangement that requires a new entry in `exceptions` is refused by this record.** If an
implementer finds one necessary, the placement decided here is wrong and the question returns
here rather than being settled in the exceptions map. `OD-HOST-019` sets that rule for the
component it declared and this record adopts it for the same reason.

### 5. The port types stay in the service, and the service's own unit tests pay for it

`ComposedProviders` and its port types stay in `nomos-check-orchestration`, so the composer
depends on the service. Moving them below both would remove that dependency, and it has no home
today: a crate holding them needs `nomos-cap-syntax` for `SyntaxProvider`'s language, and
`Capability Contract` may not reach `Capability Contract`.

The cost is specific and bounded, and it is stated here so it is not rediscovered. A crate's own
unit tests cannot use a dev-dependency that depends back on that crate: the dev-dependency links
the crate's ordinary build, so the `ComposedProviders` it returns is a different type from the
one the unit tests see. `nomos-check-orchestration`'s **unit** tests therefore keep one local,
test-only provider table, with the providers it names as dev-dependencies. Its integration tests,
and every other crate's tests, are unaffected by this and use the composer.

### 6. `nomos-composer-std` stays in `Substrate`

By this record's own definition `nomos-composer-std` is a composer, and it is not moved.
`nomos-surface-provenance`, a `Repo Tooling` crate, depends on it, and `Repo Tooling` reaches
only Protocol and Substrate. Moving it into `Composer` would break that dependency or widen
`Repo Tooling`, and neither is worth a consistent name. It stays where its permits already fit,
and the asymmetry is recorded rather than hidden.

### 7. Tests obtain the set from the composer

Every crate whose tests run the check obtains the set as a dev-dependency on
`nomos-composer-providers` rather than keeping a copy, with the single exception section 5
names and explains. The test asserting that each language constant agrees with its provider's
moves with the table.

## What The Design Gives Up

Naming the provider at the materialization site did two jobs: it declared the set, and it
priced additions to it, because adding a provider was an edit to the service crate that somebody
reviewed there. The composer keeps the first and drops the second: a provider becomes one row in
a crate whose only job is to hold rows, and the friction that made anyone ask what a provider
costs goes with the edit that caused it.

That is not answered with a mechanism here, and `OD-ROADMAP-005` authorizes the composition move
and nothing more. The observable condition under which it would need answering is the one
`OD-RULES-009`'s 2026-09-14 amendment and `OD-ROADMAP-003`'s *What Would Decide It Differently*
already name: a measured cost that makes materializing an unneeded family expensive enough that
skipping it is worth deciding rather than deriving. That amendment found none of its three
conditions present, and this record does not claim otherwise, because what has since been
measured is adjacent to that condition rather than the condition itself. The two families backed
by `ra_ap_hir` are now measured to be expensive: they are why `nomos-gate-orchestration`'s suite
did not finish at default parallelism before `0f63ee51`, and why `nomos-check-orchestration`'s
suite reaches about 14,300 MB of working set. But a family is already materialized only when a
selected rule declares it, so that is the cost of a family a run needs, not of an unneeded one
worth skipping, and the condition stays unmet. The measurement belongs here for a narrower
reason: it shows the cost one provider can carry is no longer small, so the question an added
provider ought to prompt is a real one. A provider added without asking it is the cost this
design accepts.

## What This Record Does Not Authorize

It does not move the port types out of `nomos-check-orchestration`; section 5 records why not.
It does not move `nomos-composer-std`; section 6 records why not. It does not change what any
provider materializes, what any rule judges, or which families a run demands: demand is still
derived by `Demanded_Families` from the selected rules' declarations. It does not add a provider,
remove one, or change the order of rows. It grants no permission beyond the `Composer` component
and the one edge `Host` gains to it.

## The Items This Triggers

One implementation item, written against this record and not re-deriving it. It must: declare
`Composer` and its permits in `nomos-architecture.json` and give `Host` the edge to it; create
`nomos-composer-providers` and register it wherever this workspace registers a member; move the
table and the registry's offers into it; give `RunContext` the field of section 2; add the field
of section 3 to `GateEnvironment`, `CorrectionEnvironment` and `nomos-workflow-orchestration`'s
platform value; supply the set from each of the four hosts, which are `nomos-cli`, `nomos-api`,
`nomos-lsp` and `nomos-daemon`; remove every provider crate from
`nomos-check-orchestration`'s library dependencies; and move every test fixture to the composer
except the one section 5 keeps.

It must amend `OD-CAPABILITY-008`, whose declared-trait trigger this record answers by settling
where the ports live and why they do not move; `OD-HOST-004`, whose composed-by-hand clause the
composer supersedes; and `OD-PACKAGE-014`, whose activation semantics now meet a composed set
rather than a service's own. Each amendment names `OD-ROADMAP-005` as the authorization and this
record as the design.

Its predicate's bound is derived from measured cost and not chosen. On this machine on
2026-09-26, `nomos-gate-orchestration` at the library target ran between 3,025 and about 3,400
seconds after `0f63ee51`, and about 4,000 at all targets on a loaded machine;
`nomos-check-orchestration` at the library target ran 1,798 seconds at `0e9d7918` and is heavier
since; and a six-package predicate covering both took 5,558 seconds.

## What Would Decide It Differently

A home for the port types below both the composer and the service would remove section 5's cost
and the composer's dependency on the service, and should be taken if one appears: a component
that may reach `Capability Contract` and that both `Application Service` and `Composer` may
reach.

A host that needs a provider set other than the standard one would make the composer select
among sets rather than hold one. At that point it is a selection with a parameter, and this
record should be amended to say who chooses the parameter, because that choice is exactly the
one section 3 forbids the service crates to make.

Evidence that a service crate needs to select providers itself, as distinct from holding what
its caller supplied, would reopen section 3 and the edge section 4 withholds.

## Status

Accepted. `OD-ROADMAP-005` decision item 1 is the authorization; this record is the design it
did not carry. The two declined authorings of the implementation item are
`P127-THE-CHECK-SERVICE-RECEIVES-ITS-PROVIDERS-FROM-A-COMPOSITION-ROOT` and
`P127-THE-CHECK-SERVICE-RECEIVES-ITS-PROVIDERS-FROM-A-COMPOSITION-ROOT-2`, whose decline reasons
carry the measurements this record cites.
