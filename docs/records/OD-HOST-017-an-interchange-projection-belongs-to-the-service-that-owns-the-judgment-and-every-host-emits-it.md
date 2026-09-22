---
id: OD-HOST-017
type: decision
title: An interchange projection belongs to the service that owns the judgment, and every host emits it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - architecture
  - gate
  - projection
  - layering
  - orchestration
relations:
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-007
    type: relates-to
  - target: OD-HOST-011
    type: relates-to
  - target: OD-HOST-013
    type: relates-to
  - target: OD-HOST-014
    type: relates-to
  - target: OD-CAPABILITY-015
    type: relates-to
  - target: OD-RULES-020
    type: relates-to
  - target: OD-GATE-033
    type: relates-to
---

# An interchange projection belongs to the service that owns the judgment, and every host emits it

## Question

Commit `0c05967a` built the SARIF 2.1.0 projection this workspace had none of, and put it in
`nomos-api` beside the two response types it reads. Nothing outside that crate's own tests
calls it. So the one export GitHub code scanning, Azure DevOps and most CI consumers ingest
-- and the one a v14 corpus row requires -- exists and is unreachable.

The obvious repair was authored as a flag on `nomos gate run`
(`P123-A-SARIF-LOG-IS-UNREACHABLE-FROM-A-COMMAND-LINE`) and declined, because it is refused
by declared architecture rather than by an absent line: `nomos-cli` and `nomos-api` are both
zone Host, and Host does not permit Host.

So the question is not how to wire a flag. It is **which host may emit the projection when a
host may not name the host that holds it**, and there are four answers that differ mostly in
what else each one permits.

## What Was Measured

Taken at `3b79707f`. `nomos-architecture.json` is dirty in the shared tree with one peer's
C# provider rows; that change reaches nothing below and every figure here holds at `HEAD`
too.

### The zone rule, and the three tests that hold it

`nomos-architecture.json`'s top-level keys are `components`, `members`, `permits`,
`exceptions` and `authorities`.

- `members` places `nomos-cli`, `nomos-api`, `nomos-api-transport`, `nomos-mcp` and
  `nomos-lsp` all in zone `Host`.
- `permits.Host` lists nine components -- Protocol, Substrate, Specification, Capability
  Contract, Provider, Rules, Agent, Application Service, Repo Tooling -- and does **not**
  list `Host`. A Host crate may not name a Host peer.
- `exceptions` names `nomos-api-transport` -> `nomos-api` and `nomos-mcp` ->
  `nomos-api-transport`, and carries no entry at all for `nomos-cli`.

Two things enforce that from the one declaration, which is `OD-RULES-020`'s own doing:
`Test_Dependencies_Should_Run_Strictly_Downward` in
`tests/contract/tests/boundaries/graph.rs`, and `nomos_rules::Check_Dependency_Direction`
(`crates/rules/nomos-rules/src/checks/dependency/violations.rs`) over a real check run, so
this repository's own self-check reports the identical edge. A third makes an exception
non-free: `Test_Every_Same_Zone_Edge_Should_Be_A_Real_Dependency` requires every declared
exception to be a real direct dependency, in both directions, so an exception cannot be
declared ahead of the manifest edit it licenses nor kept after it lapses.

Both judgments skip a dev-dependency -- `Is_Dev_Dependency` in the rule, `Is_Not_Dev` in
`tests/contract/src/workspace.rs` for the test. A dev-only edge would therefore escape both
and produce nothing a caller can run, so it is not a fifth route.

### What holds the projection, and what it says about itself

`SarifLog` is public in `nomos-api`'s blessed surface -- three entries in
`tests/contract/surface/nomos-api.txt`, for the type and its two projection functions. No
crate outside `nomos-api` names it; the only other mentions in the workspace are that
crate's own module declaration and re-export. Only `nomos-api-transport` may name
`nomos-api`, and `nomos-mcp` reaches it through the transport rather than directly.

`crates/host/nomos-api/src/sarif.rs` places itself by `OD-HOST-002` and states the premise
in `lib.rs`: the two response types are "the canonical answers all three hosts project".
Measured, that premise does not hold.

- `nomos-cli` declares `nomos-check-orchestration`, `nomos-gate-orchestration` and
  `nomos-workspace-discovery`, and no `nomos-api`. `nomos-lsp` declares
  `nomos-check-orchestration` and no `nomos-api`. Neither may name it, so neither projects
  those responses; each projects an orchestration outcome directly. One host and its client
  project them.
- The responses are not the canonical answer either. `Handle_Gate_Run` calls
  `nomos_gate_orchestration::Run_Gate`, holds the `GateRunResult` it returns, and hands back
  `GateRunResponse::From(result)`. `Handle_Check_Run` does the same with
  `CheckResponse::From(outcome)` over a `nomos_check_orchestration::CheckOutcome`.
  `OD-HOST-011` already reasons about that twinning.
- Three of the four things the projection reads are declared twins of the judging services'
  own types. `FindingBucket`'s own doc says "a serializable twin of
  `nomos_gate_orchestration::FindingDisposition`"; `NoVerdictResponse`'s says the same of
  `nomos_gate_orchestration::NoVerdict`, and `CheckOutcomeResponse`'s of
  `nomos_check_orchestration::CheckOutcome`. Only the fourth, `nomos_contracts::Finding` and
  its `GateCategory`, comes from a zone every crate may reach. So the projection consumes
  nothing of `nomos-api`'s own except the twins `nomos-api` made.

`OD-HOST-002`'s own definition is what settles this. A canonical service there is "a library
crate any client can call directly -- `nomos-ledger`, `nomos-work-orchestration`,
`nomos-capability`, `nomos-analysis`, `nomos-contracts`, `nomos-spec-store`, and their kind
-- not a rendering one particular client already produced". `nomos-api` is a surface, and no
peer host may call it. Placing the projection there makes it state one surface holds and no
other client can reconstruct, which is the condition that record was written to refuse, not
the remedy it prescribes.

### What the CLI holds, and what a route costs it

- `crates/host/nomos-cli/Cargo.toml` declares neither `nomos-api` nor `serde_json`, in
  `[dependencies]` or `[dev-dependencies]`.
- `crates/host/nomos-cli/tests/request_submit.rs` records the posture in its own prose:
  `Test_Into_Should_Render_The_Accepted_Submission_With_A_Freshness_Stamp` asserts a
  projection sidecar by substring rather than parsing it, because "a dependency on
  `serde_json` in this crate's tests would be a second reason to touch `Cargo.toml`".
- Neither verb holds what the projection consumes. `crates/host/nomos-cli/src/gate.rs`
  renders what `nomos_gate_orchestration::Run_Gate` returned and
  `crates/host/nomos-cli/src/check.rs` renders a `nomos_check_orchestration::CheckOutcome`,
  while `SarifLog::Of_Gate_Run` takes a `nomos_api::GateRunResponse` and `Of_Check_Run` a
  `nomos_api::CheckResponse`.
- `sarif.rs` states that `serde_json` stays a dev-dependency of `nomos-api` and that nothing
  there serializes, so a caller supplies the serializer -- "the projection costs a host no
  dependency it did not already have". Under this placement the one host a CI job actually
  runs is the one host that cannot pay for it.

### What each candidate home already declares

- `nomos-api-transport` declares `nomos-api` and `serde_json`; `nomos-mcp` declares
  `serde_json` and `nomos-api-transport`. A wire-side emitter costs no manifest edit.
- `nomos-gate-orchestration` declares `serde` **and** `serde_json` in `[dependencies]`, not
  as dev-dependencies, and declares `nomos-check-orchestration` under the one named
  exception into it. Its `GateRunResult` carries `run`, `root`, `check_outcome`, `findings`,
  `disposition` and `no_verdict`, every field public -- the whole of what `GateRunResponse`
  carries, one conversion earlier.
- `nomos-check-orchestration` declares neither `serde` nor `serde_json`, and may not name
  `nomos-gate-orchestration`: that exception runs one way only.
- `nomos-cli` already declares both orchestration crates, and every Host may name an
  Application Service under `permits` with no exception at all.
- `nomos-workflow-orchestration` is the only other crate that reaches both judgments; its
  exceptions name all four Application Services.

### What an admitting increment on the wire costs

`OD-HOST-014`'s decision 5 names three places: `ServedMethod::REGISTRY` in
`nomos-api-transport` (five entries today), `ServedTool::REGISTRY` in `nomos-mcp`, which
projects it, and the `ADMITTED` array in
`tests/contract/tests/boundaries/transport_registry.rs`. A served method is also a
projection of `nomos_contracts::OperationName`, which by its own doc is the only
authoritative name an operation has.

## The Decision

### 1. What decides is what the projection is, not which host wants it first

`OD-HOST-013` answered this shape of question for a protocol -- "what decides is what the
thing *is*, not who currently calls it" -- and `OD-CAPABILITY-015` answered it for a zone
label, refusing to reason from the edge to the label because that reasoning empties the edge
of meaning. Neither is about a projection. Both state the criterion this record applies to
one.

A SARIF log is the interchange rendering of a judgment, and every property it carries is
derived from what a run found and what a policy did about it: `level` from a finding's own
`GateCategory` and applicability, `suppressions` from the bucket a gate reduced the finding
into, `invocation.executionSuccessful` and its notifications from whether the run reached a
verdict and why not. The last two vocabularies are the judging services' own, as the
twinning above measures. So the log is the judging service's projection. It belongs where
the judgment is, not beside one surface's twin of it.

### 2. The projection moves to `nomos-gate-orchestration`, and every host emits it from there

Application Service is a zone every Host may name under `permits` with no exception, so the
move makes the projection reachable by `nomos-cli`, `nomos-api`, `nomos-api-transport`,
`nomos-mcp` through the transport, `nomos-lsp`, and any later host, without an architectural
decision being taken for each one.

`nomos-gate-orchestration` rather than `nomos-check-orchestration`, and that is forced
rather than preferred: the projection needs both judgments, `GateFindings` is
gate-orchestration's own type, and the exception into check-orchestration runs one way, so
the check crate cannot name its own reducer. A check run's log stays the degenerate case the
existing `Of_Check_Run` already describes -- no policy applied, so no bucket on a result --
and it is projected by the crate that owns the bucket vocabulary the gate case needs.

Not `nomos-workflow-orchestration`, the only other crate that reaches both: a step sequencer
is not what an interchange projection is, and every host would then need a workflow
dependency to emit a gate log.

Not a new crate in a lower zone. Its zone would have to be argued under
`OD-CAPABILITY-015`'s criterion, and a SARIF document model is not Protocol vocabulary, a
substrate, or a capability contract; and the one property a new crate would buy -- a home two
Application Services can share -- is a property a named exception already supplies.

### 3. `nomos-api` keeps its reach and stops being the owner

Exactly one projection exists after the move. `nomos-api` projects from the `GateRunResult`
and `CheckOutcome` its own handlers already hold before they convert, rather than from its
twins, and `nomos_api::SarifLog` stays namable by whatever spelling the building item
chooses -- a re-export or nothing at all. What is refused is a second copy: two projections
of one judgment is the state decision 7 refuses one host lower down, and it is no better
between two hosts.

### 4. The first emitter is `nomos-cli`, and the route costs it no manifest edit

The consumer the corpus row and every CI system stand for is a file a command wrote. So the
first emitter is the command line, and that is the whole reason the placement question had to
be answered rather than worked around.

The measured cost is part of the decision. `nomos-cli` declares neither `nomos-api` nor
`serde_json` today, and this route needs neither: it already declares
`nomos-gate-orchestration`, and that crate declares `serde` and `serde_json` in production,
so the projection can hand a host serialized bytes and the CLI writes them through
`nomos_platform::FileSystem`.

That does reverse the "a caller chooses the serializer" clause in `nomos-api`'s `sarif.rs`.
The clause is not an independent commitment; it is a consequence of a placement in a crate
whose `serde_json` is a dev-dependency, and its effect where it stands is that the host
which most needs the export cannot pay for it.

### 5. A named Host exception letting `nomos-cli` reach `nomos-api` is refused

An exception is a standing permission at crate grain, not a licence for one call, and six
things follow from granting this one:

1. It admits `nomos-cli` to all thirty `Handle_*` functions, including the twenty-one
   repo-tooling handlers `OD-HOST-007` excluded from the wire structurally rather than
   advisorily, and the two `OD-HOST-014` refused for spending money outside the tree.
2. Its natural consequence is that the CLI becomes a client of `nomos-api`. That is a
   decision about the whole product surface, and taking it as a side effect of one export is
   deciding it by not asking it.
3. Nothing would then say which of two composition roots a new verb must use. `OD-HOST-002`
   built `nomos-cli` and `nomos-api` as deliberate peers over one set of orchestration
   crates; an edge between them leaves two legal paths to every answer and no rule choosing.
4. A CLI that called `Handle_Gate_Run` for the log and its own `Run_Gate` for the human
   report would walk and judge the tree twice in one process. The alternative is abandoning
   its own composition root, which is not a flag.
5. It reasons from the edge to the label, which `OD-CAPABILITY-015` refuses and
   `OD-HOST-013` refuses in its own domain.
6. It cannot be a paper exception.
   `Test_Every_Same_Zone_Edge_Should_Be_A_Real_Dependency` requires the real edge, so the
   declaration, `nomos-api` and `serde_json` all land in the CLI's manifest in one commit.

`nomos-api-transport` -> `nomos-api` and `nomos-mcp` -> `nomos-api-transport` are not
precedent for it. Those two are a layering of surfaces, each strictly above the next, which
is what a same-zone exception is for. A composition root reaching a sibling surface is a
different shape.

### 6. Admitting a SARIF-emitting operation to the transport and MCP is refused as this question's answer, and stays available as its own item

`OD-HOST-014`'s criterion admits it. A projection of a judgment already made causes nothing
on the host that the served `nomos.gate.run` does not already cause, and it costs no manifest
edit anywhere. It is refused here, and not because it is wrong:

- It does not answer the question. `nomos-cli` still could not emit, and no JSON-RPC method
  produces a file in a CI job without a client this workspace does not ship.
- It would settle the placement question by not asking it. The projection would stay owned
  by a surface, and the next host to want it -- `nomos-lsp`, a connector -- would meet the
  identical refusal.
- It spends a public commitment, a new `OperationName` and three registry edits, on an
  export whose first real consumer is a file on disk.

Once the projection lives where decision 2 puts it, admitting it is a registry edit and no
architectural decision, and `OD-HOST-014`'s criterion is the whole of what decides it.

### 7. A CLI that rendered the log from its own outcome types is refused

It is a second rendering of a finding, which is what `OD-HOST-002` exists to refuse. Three
further consequences make it worse than the one that already exists: the corpus requirement
would be satisfied by whichever emitter a test happened to read; the level and suppression
derivation would be duplicated over a vocabulary that is the gate's, so the compile-time
arity guard `Bucketed_Findings` keeps against a sixth bucket would protect one copy and not
the other; and the host that most needs the export would be the one holding the copy nothing
governs.

### 8. The capability item that follows, and what it reserves

`P123-THE-SARIF-PROJECTION-MOVES-TO-THE-SERVICE-THAT-OWNS-THE-JUDGMENT`, a Capability item.
Its territory by path:

- `crates/orchestration/nomos-gate-orchestration/src/lib.rs`
- `crates/orchestration/nomos-gate-orchestration/src/sarif.rs`
- `crates/orchestration/nomos-gate-orchestration/src/sarif`
- `crates/orchestration/nomos-gate-orchestration/Cargo.toml`
- `crates/host/nomos-api/src/lib.rs`
- `crates/host/nomos-api/src/sarif.rs`
- `crates/host/nomos-api/src/sarif`
- `crates/host/nomos-cli/src/gate.rs`
- `crates/host/nomos-cli/src/gate`
- `crates/host/nomos-cli/src/check.rs`
- `crates/host/nomos-cli/src/check`
- `crates/host/nomos-cli/Cargo.toml`
- `tests/contract/surface/nomos-api.txt`
- `tests/contract/surface/nomos-gate-orchestration.txt`
- `README.md`

Each `lib.rs` is reserved because a `pub` item in a private module is reachable by nobody,
and the snapshot beside it is the oracle for that rather than the modifier. Both snapshots
are reserved because three entries leave one crate and arrive in the other, so a blessing is
owed on both sides.

`README.md` is reached for its crate-table rows for `nomos-api` and
`nomos-gate-orchestration`, which state what each crate holds, and **not** for verb usage:
that file documents no verb's flags, and its `Running the gate` section deliberately
reproduces no commands.

`nomos-architecture.json` is **not** territory, and that is a property of the choice rather
than an omission. This is the only one of the four routes that needs no edit to `permits` or
`exceptions`, because every edge it draws is a Host naming an Application Service, which
`permits` has always allowed.

Both manifests are reserved although the measured cost of this route is zero for each. A diff
that touches either is evidence against the reachability argument in decision 4, and owes the
report `AGENTS.md` asks for rather than a widening.

## What This Record Does Not Do

It builds nothing and moves nothing. Every path in decision 8 is a reservation for an item
that does not exist yet.

It does not spell the flag, write its usage text, or choose between `--format sarif` and a
dedicated `--sarif <path>`. That is the building item's, decided with the parser conventions
a previous claimant measured and the declined item carried forward: a valued flag through
`crate::arguments::Named_Value_From_String_Arguments` with `Required_Value(value, Name,
Usage)` for the refusal, `gate/parsing.rs`'s `KNOWN_ARGUMENTS` and each verb group's own
`USAGE` constant, `check/parsing.rs`'s prefix guard that refuses any unrecognized argument,
`main.rs` naming no command type so a CLI-owned invocation can wrap an orchestration command
without touching it, and production code in both verbs reaching the disk only through
`nomos_platform::FileSystem`, whose `Replace_Atomically` and `FileSystemError::Path` give an
unwritable destination a refusal that names the path.

It does not admit any operation to the transport or to MCP. Decision 6 refuses that as this
question's answer and leaves it to an item of its own, judged against `OD-HOST-014`.

It does not decide whether `nomos-api`'s response twins become one serializable contract
layer. `OD-HOST-011` holds that question. This record narrows it by one case instead of
answering it: a thing that was placed beside the twins turns out to belong beneath them.

It does not amend `OD-HOST-002` and does not revisit its decision. What it corrects is a
reading of that record in `nomos-api`'s own module documentation, and the correction is taken
from the decision `OD-HOST-002` made rather than against it.

It does not decide whether any other interchange export exists or where -- GraphML, CSV and
the JSONL the spec bundle already emits. Decision 1's criterion decides the next one when
somebody writes it.

## What Would Decide It Differently

**A measured external client that can only be served over the wire.** Decision 6 is refused
for being a partial answer rather than a wrong one. A real consumer speaking JSON-RPC or MCP
makes the transport admission the answer to its own question, and it was always available on
`OD-HOST-014`'s criterion.

**A second judging service whose outcome a SARIF log must carry and which
`nomos-gate-orchestration` may not name.** The home in decision 2 is forced by the one-way
exception into `nomos-check-orchestration`. A second judgment with no legal path to it
reopens the shared-home-in-a-lower-zone alternative, and `OD-CAPABILITY-015`'s criterion is
then what decides that crate's zone.

**`OD-HOST-011` resolving the twins into one serializable contract layer below zone Host.**
The projection would then have a home that is neither a surface nor a reducer, and decision
2's crate is the best answer available today rather than the only conceivable one.

**A deliberate decision that `nomos-cli` becomes a client of `nomos-api` for every verb.**
That is the decision decision 5 refuses to take as a side effect of an export. Taken
deliberately, on its own evidence, the exception becomes ordinary and decision 5 is moot.

**An implementer finding that the move costs `nomos-cli` a manifest edit after all.** The
reachability claim in decision 4 is measured from two manifests and a zone table, and a
manifest edit contradicts it. It does not reinstate any refused route on its own, but it
does mean this record's cheapest-route argument was wrong about the thing it claimed to have
measured.

## Status

Accepted. The SARIF projection is the judging service's and belongs in
`nomos-gate-orchestration`, where every host reaches it through a permitted zone crossing and
none needs an exception. `nomos-cli` is the first emitter, at no manifest cost. A Host
exception for `nomos-cli` is refused as a standing permission whose real content is a decision
about the whole product surface; a transport admission is refused as this question's answer
and left available as its own item; a CLI-local rendering is refused as a second rendering of
a finding. The building item is named in decision 8 with its territory, and
`nomos-architecture.json` is deliberately absent from it.
