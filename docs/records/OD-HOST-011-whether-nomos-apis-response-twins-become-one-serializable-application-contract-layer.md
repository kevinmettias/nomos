---
id: OD-HOST-011
type: decision
title: A response twin is the accepted cost of keeping serialization out of orchestration, and one shared contract layer waits for a second transport that needs it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - api
  - architecture
  - serialization
relations:
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-007
    type: relates-to
  - target: OD-GATE-011
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
  - target: OD-PACKAGE-015
    type: relates-to
---

# A response twin is the accepted cost of keeping serialization out of orchestration, and one shared contract layer waits for a second transport that needs it

## Question

An eighth-round external architecture review named `nomos-api`'s response types as schema
proliferation, describing a trend toward "domain type, orchestration outcome, API twin,
JSON-RPC twin, MCP twin" and recommending one canonical serializable application-contract
layer -- either its own crate, or serialization moved onto the orchestration outcome types
themselves.

The crate's own module documentation supplies the review's evidence: it describes response
families as deliberate twins of orchestration outcomes, built because the underlying domain
objects do not serialize. Whether the twin is an accepted cost or a defect was never decided,
and `OD-HOST-002` governs what a surface may *hold* rather than what shape a canonical service
*returns*.

## What Was Measured

**The population, counted rather than characterized.** `nomos-api` is 7,826 lines across
thirteen modules and declares sixty-nine `Response` types, with 182 mentions of `Serialize`.
Twenty-nine `pub fn Handle_*` entry points. Ten response types live in a dedicated
`response/` module and the rest sit beside the handler family they serve.

**The review's five-layer chain does not exist.** It predicts `domain type → orchestration
outcome → API twin → JSON-RPC twin → MCP twin`. The actual chain is three:
`nomos-api-transport::dispatch` serializes `nomos_api` handler results directly with no
envelope type of its own beyond `WireResponse`, and `nomos-mcp` projects the same values
rather than re-typing them. Two of the five predicted layers are not there. The observed
duplication is one twin per orchestration outcome, not a stack.

**The twin buys a real property, and the alternative spends it.** The orchestration crates
carry no `serde` dependency. Moving `Serialize` onto the orchestration outcome types would put
a wire-format concern into the crates that decide judgments -- and a derived `Serialize` makes
every public field's name and shape a compatibility surface, so a field rename inside an
orchestration crate would become a wire break. The twin is what currently keeps those two
questions separable.

**One transport consumes the surface today, and it consumes a seventh of it.**
`nomos-api-transport::ServedMethod` is a closed enum of four variants -- `GatePlan`,
`GateRun`, `GateExplain`, `Correction` -- against `nomos-api`'s twenty-nine handlers. A shared
contract layer's whole value is agreement among several consumers; there is one, and it does
not use most of what exists.

## The Decision

**The response twin stays, and is the accepted cost of keeping `serde` out of the
orchestration band.** No `nomos-application-contracts` crate is created. No `Serialize` is
added to an orchestration outcome type.

This is `OD-GATE-011`'s tracked-duplication shape rather than an unaccountable one: the twin's
reason is stated at the site in each module's own documentation, and the twin is mechanical
rather than interpretive -- it renames nothing and restructures nothing, so a reader comparing
the pair can see the correspondence.

**`OD-PACKAGE-015`'s test governs the crate question and answers it in the negative today.**
That record asks whether a crate is versioned independently, enforces an isolation boundary a
module could not, or has a consumer that does not also depend on its current siblings. A
contracts crate extracted now would have exactly one consumer, `nomos-api-transport`, which
already depends on `nomos-api` -- failing all three clauses, and the review's own point 5
argues for exactly this test in the same document.

## What Would Decide It Differently

- **A second transport that needs the same response shapes and does not want the handlers.**
  An MCP or LSP projection depending on a contracts crate without depending on `nomos-api`
  fires `OD-PACKAGE-015`'s third clause directly.
- **A twin that stops being mechanical.** A response type that renames a field, flattens a
  structure, or drops information the orchestration outcome carried is no longer a
  transcription and no longer qualifies for `OD-GATE-011`'s legitimate-exception treatment.
  This is the failure mode worth watching, and it is not detected by counting types.
- **A wire break traced to an orchestration rename.** Would prove the separation the twin is
  bought for was not actually held.
- **`nomos-api` growing a second consumer of its own response types inside this workspace.**

## What This Does Not Decide

Whether `nomos-api` should hold twenty-nine handlers at all -- `OD-HOST-012` takes that up.
Whether the transport's four-method surface should grow. Neither depends on this answer.

## Status

Accepted. Measured against a one-transport, four-method reality; the review's five-layer chain
was checked and two of its layers do not exist. Revisit on a second transport that would depend
on a contracts crate without depending on `nomos-api`, or on the first twin that stops being a
mechanical transcription.
