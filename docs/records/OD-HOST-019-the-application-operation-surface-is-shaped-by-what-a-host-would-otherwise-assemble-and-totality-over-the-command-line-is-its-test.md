---
id: OD-HOST-019
type: decision
title: The application operation surface is shaped by what a host would otherwise assemble, and totality over the command line is its test
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - architecture
  - layering
  - api
relations:
  - target: OD-ROADMAP-005
    type: relates-to
  - target: OD-HOST-018
    type: relates-to
  - target: OD-HOST-011
    type: relates-to
  - target: OD-HOST-012
    type: relates-to
  - target: OD-HOST-014
    type: relates-to
  - target: OD-HOST-001
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
---

# The application operation surface is shaped by what a host would otherwise assemble, and totality over the command line is its test

## Question

`OD-ROADMAP-005` decision 3 authorizes "one operation surface the hosts depend on in place of
assembling the product themselves", superseding `OD-HOST-011` v1's refusal of a contracts crate
and `OD-HOST-012` v1's decision that repo-tooling handlers keep their present home. It
authorizes the thing and says nothing about its shape: what one operation is, which of today's
verbs are operations, where the surface sits against the declared zones, and what would show
that it had been built rather than merely named.

`OD-HOST-018` is the other half of the same authorization, and it drew the line between them
itself: that record says what a host may **know**, this one decides what a host **depends on**.
It is explicit that it "does not design it, name its crate, or pre-empt its shape", and it
leaves this record one unhomed edge, settled in section 5.

## What Was Measured

Measured 2026-09-22 against `792a7a03`, by reading the dispatch table and the manifests rather
than by citing `ARC-ROADMAP-001`, which is where the figures in the item that triggered this
record came from.

**The command line dispatches nine groups.** `vacuity::Group` closes the set and `main.rs`
matches it with no wildcard arm: `work`, `spec`, `check`, `request`, `gate`, `agent`,
`correct`, `workflow`, `profile`.

**The group is the wrong unit, and counting groups gets the answer wrong twice.** The `request`
group dispatches exactly one command, `submit`, and the service answers it as
`Handle_Spec_Submit` — a match across group names. Counting groups reports `request` and
`profile` as unserved. Counting operations reports `profile` and `work widen`. The first census
is wrong in both directions at once, which is the argument for the unit chosen in section 1.

**The operation census: 32 against 30.** `nomos-api` exports 30 `Handle_` functions. The
command line offers 32 operations — twelve `work`, nine `spec`, one `request submit`, four
`gate`, two `agent`, one each for `check`, `correct` and `workflow`, and one `profile`. The
difference is exactly the two unserved operations, so the two counts corroborate each other
rather than merely coexisting.

**`work widen` has no handler.** The service carries eleven of the twelve ledger verbs. Nothing
declares the twelfth, and nothing failed when it was added.

**`profile` has no service crate at all.** It is thirteen modules inside `nomos-cli` —
capability standing, provider standing, offer standing, availability, program search, starter
policy — computing answers about a workspace entirely within the host.

**The nine groups' service crates span three zones**, which is the fact the shape turns on.
`check`, `correct`, `gate`, `workflow` and `agent` reach Application Service crates; `spec` and
`request` reach `nomos-spec-orchestration`, which is Specification; `work` reaches
`nomos-work-orchestration`, which is Repo Tooling. `Application Service` permits Protocol,
Substrate, Capability Contract, Provider, Rules and Agent — **neither Specification nor Repo
Tooling.**

**The crate counts, as a dated measurement and not as a requirement:** `nomos-cli` names 26
`nomos-*` crates and `nomos-api` names 23. `ARC-ROADMAP-001` records 22 for `nomos-api`; it has
moved by one since that record was written, which is the ordinary fate of a count in prose and
part of the reason the falsifier below is not one.

## The Decision

### 1. An operation is one rendered answer, and the membership rule is what the host would otherwise assemble

**An operation is one answer a host renders for one thing a person asked for**, taking the
arguments that host parsed and producing one outcome value it renders.

**An operation belongs to the surface when a host would otherwise have to name a service crate
to offer it.** The test is stated so that it answers for an operation nobody has written yet:
take the operation away from the host, and ask what the host would have to name to put it back.
If the answer includes any crate in Application Service, Specification or Repo Tooling, the
operation belongs to the surface. If the answer is only the host's own argument parsing, its
transport, its exit codes and the outcome type it prints, it is host-local.

The unit is the operation, not the group and not the crate. Two operations the same service
answers may sit in different command groups, and one group may hold operations of both kinds.
The census above is what it costs to forget that.

### 2. The surface is its own zone, `Operation Surface`, and Application Service is refused

The surface is a new declared component in `nomos-architecture.json`, named `Operation
Surface`, holding one crate, `nomos-operations`, under `crates/operations/`.

**Its permits are exactly today's `Host` permits minus `Host`:** Protocol, Substrate,
Specification, Capability Contract, Provider, Rules, Agent, Application Service, Repo Tooling.
`Host` gains `Operation Surface`. That is the whole layering change, and it is a component with
declared permits rather than an entry in `exceptions`.

**Placing the surface in `Application Service` is refused here rather than discovered later.**
That zone permits neither Specification nor Repo Tooling, so a surface living there could not
answer the ten spec operations or the twelve work operations without widening Application
Service's permits to reach both. That widening is not local to the surface: it would also let
`nomos-check-orchestration` name `nomos-ledger` and `nomos-gate-orchestration` name
`nomos-spec-store`. Loosening six crates to place one is the wrong trade, and a zone whose
permits are the union of everything its members happen to need has stopped being a boundary.

**Any arrangement that requires a new entry in `exceptions` is refused by this record.** If an
implementer finds one necessary, the placement decided here is wrong and the question returns
here rather than being settled in the exceptions map.

**The surface is named by the hosts and names no host.** `Host` does not appear in `Host`'s own
permit list, which is the host-may-not-name-a-host rule as the declaration already carries it,
and `Host` does not appear in `Operation Surface`'s permits either. The two host-to-host edges
in the tree, `nomos-api-transport` naming `nomos-api` and `nomos-mcp` naming
`nomos-api-transport`, are declared exceptions with their own reasons; the surface joins
neither and needs no third.

### 3. `profile` joins the surface

Leaving `profile` unmentioned is the failure this record exists to prevent, so it is decided
rather than deferred.

`profile` joins. The membership rule reaches it without special pleading: remove it from
`nomos-cli` and the host would have to name workspace discovery, the capability registry and the
gate policy crates to offer it again. Thirteen modules computing capability, provider and offer
standing are service work that happens to live in a host, which is the exact condition
`OD-ROADMAP-005` decision 3 names — a host assembling the product itself.

**What that classifies next.** The rule turns on whether the host would name a service crate,
never on whether a verb is first-run, interactive, a convenience, or writes a file. A future
verb that only formats or re-prints what the host already holds is host-local; one that computes
an answer from the tree is not. `profile --write-gate-policy` writing a file does not make it
host-local, because the write is the second half of an answer whose first half is a reading of
the tree.

**`work widen` is a different finding and the membership rule does not settle it.** The service
already answers eleven of twelve ledger verbs; the twelfth is absent rather than host-local. It
is evidence for the totality test in section 4 rather than a question for section 1.

### 4. The falsifier is two properties and a check, not a crate count

A count is refused as the acceptance condition, for two recorded reasons. `OD-HOST-018` decided
that a host may name one service crate per operation it renders and capped nothing — "a host
offering twenty verbs may name twenty crates" — so a lower count is not by itself evidence of a
better boundary. `OD-HOST-011` v2 decided that a record restating a quantity a test already
asserts has minted a second authority for it, "and the second one is what goes stale, because
nothing renders a record from the code."

So the surface is real when both of these hold, each asserted by a check rather than by this
record's prose:

- **Totality.** Every operation the command line offers is answered by the surface. Both sides
  of that comparison are closed sets something already enumerates — `vacuity::Group` and its
  per-group verbs on one side, the surface's own declared operation set on the other — so the
  comparison is mechanical, and an operation added to either side with no counterpart fails it.
  `work widen` and `profile` are the two it finds today.
- **Exclusivity.** Neither large host names a service crate for an operation the surface
  carries. This is `OD-HOST-018`'s own falsifier — "an application operation surface landing
  while the hosts keep their direct edges. Then either that surface does not carry what a host
  needs, or the separation is being paid for twice" — adopted here as the acceptance condition
  rather than left as something to reopen for.

**The residue a host keeps, named by kind so that it needs no number.** Its own argument parsing
and exit codes, its transport, the platform composer `nomos-composer-std` per `OD-HOST-001` and
`OD-RULES-028`, and the outcome types it renders. A host naming those has not failed
exclusivity.

Whatever the crate counts become is a consequence of those two properties and not a target. The
figures in the measurement above are dated evidence about the tree on one day, and this record
does not ask anybody to move them.

### 5. `nomos-lsp`'s correction edge belongs to the surface

`OD-HOST-018` records one live violation — `nomos-lsp` naming `nomos-correction-orchestration`
for `CorrectionFamily::Of`, an edge justified by what a different service's pipeline contains —
and says that where it goes instead "belongs to the item that decides what a host depends on".
That is this record.

It belongs to the surface. Which rules a correction family covers is an answer about what the
product can do, so it is an operation under section 1, and the LSP obtains it from the surface
rather than from the correction service. This records the destination only; it does not repair
the edge, and the repair is one of the items in section 7 rather than something an implementer
may fold into another.

## What This Record Does Not Authorize

**It is not a second authority beside `OD-HOST-018`.** That rule governs what a host may know
and is untouched here. A host depending on the surface still has to satisfy it, and, as that
record says, the same question is then asked of the surface itself and gets the same answer.

**It does not re-decide `OD-HOST-014`.** Surface membership is not transport admission. An
operation may belong to the surface and still be refused the transport on that record's
criterion of what the operation causes on the host; `agent execute` and `workflow run` are
refused there today, and joining the surface does not admit them.

**It does not touch `OD-HOST-011`'s response twins.** Whether serialization ever collapses into
one contract layer still waits on that record's own trigger, a second transport that needs it.
The surface is a dependency boundary, not a serialization format.

**It does not move logic between service crates, and it adds no orchestration-to-orchestration
edge.** The six declared edges between service crates stand exactly as `OD-HOST-018` left them.

**It does not authorize a ninth piece.** `OD-ROADMAP-005` is explicit that a piece needing a
further change is a new item, and, where it reaches another record, a new question for the
owner.

## The Items This Triggers

Sized so that no single item holds both hosts and the registration files at once, because that
territory blocks most of the board and this record is the reason several sessions would
otherwise reach for it.

1. **The zone lands empty.** `Operation Surface` is added to the components list with the permits
   in section 2, and `Host` permits it. Establishes: the boundary checks pass with the zone
   declared and no member in it. Reserves `nomos-architecture.json` and the boundary tests, and
   nothing else.
2. **The crate lands**, declaring its operation set and answering the operations one service
   already covers. Reserves the new crate and the registration files. Establishes: it compiles,
   it is zoned `Operation Surface`, and its declared operation set is readable by a test.
3. **The totality and exclusivity checks land.** Reserves the contract tests. Establishes both
   properties in section 4 as checks. They fail while the hosts still hold direct edges — which
   they will — so this item lands them as a measurement of the remaining distance rather than as
   a green.
4. **Each host projects the surface, one item per host**, and neither reserves the registration
   files. Establishes exclusivity for that host.
5. **`profile`'s computation leaves `nomos-cli` for the surface.** Reserves `nomos-cli`'s profile
   modules and the surface crate. Establishes that the last operation with no service crate has
   one.
6. **`work widen` gains its handler.** The smallest of these, and independent of the rest.
7. **`nomos-lsp` obtains its correction family from the surface**, closing the violation
   `OD-HOST-018` recorded.

## What Would Decide It Differently

- **An operation the membership rule cannot classify.** The rule is a question about what a host
  would name, and it assumes that is always answerable. One that is genuinely ambiguous is
  evidence the unit in section 1 is wrong.
- **The zone needing an exception.** Section 2 refuses that in advance; an implementer who finds
  one necessary has found that the placement is wrong, and the question returns here.
- **A second transport arriving before the surface.** That is `OD-HOST-011`'s own trigger, and it
  would merge two questions this record keeps separate.
- **Exclusivity holding while totality does not.** A surface the hosts depend on that does not
  answer every operation means the hosts kept something, and what they kept is the real shape.

## Status

Accepted. Checked 2026-09-22 against `792a7a03`: the nine dispatched groups, the 32 operations
they offer, the 30 handlers that answer them, the three zones their service crates occupy, and
the `Application Service` permit list that refuses the obvious placement.
