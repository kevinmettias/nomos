---
id: OD-HOST-007
type: decision
title: An MCP projection fires OD-LEDGER-036 and OD-HOST-006's shared trigger, and the answer bounds the transport rather than nomos-api
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - host
  - api
  - mcp
  - architecture
relations:
  - target: OD-LEDGER-036
    type: relates-to
  - target: OD-HOST-006
    type: relates-to
  - target: OD-HOST-001
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# An MCP projection fires OD-LEDGER-036 and OD-HOST-006's shared trigger, and the answer bounds the transport rather than nomos-api

## Question

`AGT-006.assessment` is the only entry in the requirement registry that calls its own gap
total: "`MCP tools` is a clean, total gap: `nomos-api`'s own module doc states outright it
does not read argv, listen on a socket, or speak MCP's JSON-RPC framing, and no MCP transport
or tool surface exists anywhere in the workspace." `ARC-ROADMAP-001` places `CLI / API / MCP
projections` inside the near-term boundary and says of the requirement behind it that `IF-001`
is "the CLI/API/MCP-parity item named below, not a proposal." `OD-ROADMAP-001` already
licenses building ahead of a caller. So *whether* is settled and *what it may carry* is not.

What it may carry is currently answered by two records, on a ground a transport removes.
`OD-LEDGER-036` settled that the work ledger is repository bootstrap machinery, and that
`nomos-api`'s exposure of the full ledger verb set is honest because `nomos-api` is a seam
exercise. `OD-HOST-006` extended the identical reasoning to `nomos-spec-orchestration`'s
`Preview`/`Render`/`Commit`, and both name one shared trigger, stated once so the two halves
could not drift apart: *if `nomos-api` becomes a real, externally-consumed product surface,
its exposure stops being an internal coordination detail and starts being a public
commitment.*

An earlier item, `OD-HOST-007-REPO-TOOLING-THROUGH-THE-API-HOST`, asked this and was declined
as already governed by exactly those two records. That decline is correct today and stops
being correct the moment a transport exists. This record answers it before one does, which is
the difference between a boundary and an apology for one.

## What Was Measured

**Twenty-four handlers, and twenty-one of them are `[repo tooling]`.** Counted directly from
`tests/contract/surface/nomos-api.txt`, which is blessed and therefore not a guess:
`Handle_Gate_Plan`, `Handle_Gate_Run` and `Handle_Gate_Explain` are three; the other
twenty-one are eleven `Handle_Work_*` and ten `Handle_Spec_*`. `README.md` marks
`nomos-ledger`, `nomos-work-orchestration`, `nomos-spec-orchestration` and
`nomos-surface-provenance` `[repo tooling]` — crates that "exist to develop or preserve this
repository, not to answer a question an end-user repository would ask Nomos." A transport that
projected `nomos-api` wholesale would publish seven repo-tooling verbs for every product verb.

**Two of the twenty-one carry authority, not merely information.** `Handle_Spec_Commit` writes
this repository's own governing records through `nomos-spec-orchestration`'s commit door.
`Handle_Work_Finish` records completion evidence on `work/ledger.json`, which `AGENTS.md`
calls "global coordination state, shared with live sessions." An MCP client is an agent by
construction; these are not read verbs an external caller would merely find uninteresting.

**`README.md` already draws the line this record needs, at the band table rather than in
prose.** `nomos-api`'s own row ends: "apart from choosing a platform or wiring an actual
transport over it." Wiring a transport is named as the thing `nomos-api` does not do — not as
an omission awaiting this record, but as the crate's own stated boundary.

**Band 90 cannot hold the transport.** `nomos-cli` and `nomos-api` are both band 90, and a
crate may depend only on a crate in a strictly lower band, which `tests/contract` asserts in
both directions. A transport must depend on `nomos-api`, so it cannot be its sibling. Band 91
is occupied by `nomos-surface-provenance` and band 100 by the two test crates.

## The Decision

**The trigger fires, and it fires on the day a transport lands rather than on some later
judgment that the surface has become popular enough.** MCP exists to be spoken by a client
that is not this repository's own tooling; a server with no client yet is not a smaller
version of the commitment, it is a server nobody has connected to. `OD-LEDGER-036` and
`OD-HOST-006` asked for a concrete trigger rather than a standing suspicion, and this is the
concrete event. Reading it any other way would make the trigger unfireable, because there is
no measurement of "really externally consumed" that a repository can take of itself.

**But the trigger fires against the transport, not against `nomos-api`.** This is the whole of
the answer, and it is why neither amended record needs its reasoning revised. External
consumption is a property of the crate that wires a transport, and `nomos-api` wires none — by
its own module doc and by `README.md`'s own row for it. `OD-LEDGER-036`'s and `OD-HOST-006`'s
claims are about `nomos-api`, and they stay true of `nomos-api` after a transport crate exists
above it. What moves is not their answer but the location of the question: the public
commitment is made by whatever selects which handlers a client may call, and that is a crate
they do not govern.

**An MCP surface projects the Gate verbs and does not project the repo-tooling verbs.**
`Handle_Gate_Plan`, `Handle_Gate_Run` and `Handle_Gate_Explain` are what an end-user
repository asks Nomos; the twenty-one others are how this repository is developed. The
exclusion is structural rather than advisory: the transport crate declares its tool registry
explicitly, and `tests/contract` asserts that the registry names no handler belonging to a
`[repo tooling]` crate — the same shape
`Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` already uses to keep a
boundary from being crossed by accident instead of by decision. A registry that is merely
short today, with nothing stopping a later increment from lengthening it, would be the
absence-as-boundary `OD-CONNECTOR-001` refuses.

**The transport lives in its own crate above band 90, not inside `nomos-api` and not inside
`nomos-cli`.** `OD-HOST-001` already decided that choosing a platform, running a verb and
rendering its outcome are three crates rather than one; speaking a wire protocol is a fourth
concern by the same argument, and the band rule forbids the sibling arrangement in any case.
It does not carry `[repo tooling]`: unlike the four crates bearing that mark, it exists
precisely to answer a question an end-user repository would ask.

## What This Record Does Not Do

It does not build the transport, choose a JSON-RPC implementation, decide stdio versus socket,
or name the crate. Those are the transport increment's own work, judged against this record
the way a second executor was judged against `OD-EXECUTOR-001`.

It does not remove a verb from `nomos-api` or forbid it from growing more. `OD-LEDGER-036`
and `OD-HOST-006` each state that non-prohibition for their own half, and this record's whole
point is that the seam exercise is unaffected: the selection happens one crate up.

It does not decide whether a repo-tooling verb could ever be projected to an external client
under some later, different framing — an end-user repository running a ledger of its own, say.
It decides that the first transport does not, and that a later one making the opposite case
must make it explicitly rather than by widening a registry nobody is watching.

It does not revise `OD-LEDGER-036`'s or `OD-HOST-006`'s reasoning. Both are amended only to
record what became of the trigger they named, which is what a named trigger is for.

## Status

Accepted, and it discharges the trigger `OD-LEDGER-036` and `OD-HOST-006` share by answering
it in advance rather than waiting to be surprised by it: the trigger fires on a transport, the
transport is bounded to the three Gate verbs, and both records keep their reasoning intact
because the commitment they worried about is made one crate above the one they govern.

Closed by `P40-API-TRANSPORT-2`, which built the first transport: `nomos-api-transport`, band
92, speaking JSON-RPC 2.0 over a TCP socket. This record was open for one stated reason -- its
exclusion was a rule with no artifact enforcing it, and a contract test asserting a registry
that did not exist would have been the vacuous-truth trap `OD-CONTRACTS-001`'s honesty
vocabularies refuse. The artifact exists now.
`tests/contract/tests/boundaries/transport_registry.rs` reads `nomos-api`'s own blessed surface
snapshot and refuses the transport for calling any handler outside `Handle_Gate_Plan`,
`Handle_Gate_Run` and `Handle_Gate_Explain`. Two companion assertions are what make that one
worth having: one refuses any import from `nomos-api`, so no call can hide under an
unqualified name the first never searches for, and one requires both sides of the comparison
to have real subjects -- this record's own vacuity objection, turned on the check it asked
for.

The revisit condition did not fire, and it was measured rather than assumed. It asked whether
a transport increment would find a Gate verb it could not serve without a repo-tooling handler
beneath it, which would have been evidence the three-verb boundary was drawn in the wrong
place. None of the three needed one: `Handle_Gate_Plan` reaches `nomos_gate_orchestration::Run`,
`Handle_Gate_Run` and `Handle_Gate_Explain` reach `Run_Gate` and `Explain_Gate` with that
crate's own walk and build-variant composition, and the transport crate's manifest names
`nomos-api`, `nomos-contracts` and `nomos-gate-orchestration` and no `[repo tooling]` crate at
all. The boundary was drawn in the right place, on exactly the evidence this record asked to
be shown.

The three questions this record left to the increment were answered there rather than here: a
socket rather than stdio, line-delimited framing rather than `Content-Length`, and no JSON-RPC
library, each argued at the site that makes it. None of them reopens what this record decided.
What stays governing is the exclusion: `P40-MCP-SURFACE-4` projects this same registry over MCP
framing and is bound by this record rather than by a fresh decision of its own, and a later
surface making the opposite case about a repo-tooling verb must still make it explicitly.
