---
id: OD-GATE-022
type: decision
title: A compare caller re-derives both runs inside one process; no persisted store or serialized GateRunResult is needed yet
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - compare
  - host
  - architecture
relations:
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-007
    type: relates-to
  - target: OD-WORKFLOW-001
    type: relates-to
  - target: OD-ANALYSIS-009
    type: relates-to
---

# A compare caller re-derives both runs inside one process; no persisted store or serialized GateRunResult is needed yet

## Question

`P40-GATE-COMPARE-VERB` built `nomos_gate_orchestration::Compare_Gate_Runs`, a real
function that diffs two `GateRunResult`s into `added`, `removed` and `changed` findings.
Nothing calls it outside that crate's own tests: `gate/parsing.rs` still refuses `compare`
as unrecognized CLI usage, `nomos-api-transport`'s `ServedMethod` and `nomos-mcp`'s
`ServedTool` both stop at three Gate verbs, and `nomos-gate-orchestration`'s own module doc
says so directly -- "`GatePlan` still does not vary by... it reports the registry, not a
walk" is the neighboring sentence to the one naming `compare` as unbuilt. Wiring any of
those three surfaces to a function requires deciding how its caller obtains the two values
it takes, and that question was open before this record: `Compare_Gate_Runs`'s own doc says
it takes "two already-produced `GateRunResult`s directly rather than looking either up by
`RunId` from a store this crate does not own," and calls persisting and retrieving a past
run by its own identity "a composition-root concern" -- naming that somebody else must
decide it, not deciding it.

Checked directly rather than assumed: `GateRunResult`
(`crates/orchestration/nomos-gate-orchestration/src/gate_plan/gate_run_result.rs:16`)
derives `Clone, Debug, PartialEq, Eq` and nothing else. It cannot be written to a file or a
wire message and read back as itself. `nomos-api`'s `response.rs` already answers the
adjacent question for a single run -- projecting `GateRunResult` into a one-way, serializable
`GateRunResponse` for `gate.run` -- and its own module doc gives the reason neither
`GateRunResult` nor any of its siblings gained `Serialize` directly: doing so "would grow
`nomos-gate-orchestration`'s or `nomos-rules`' own public surface on behalf of one caller's
shape, before a second transport exists to check that shape against." That projection is
one-way by construction: nothing converts a `GateRunResponse` back into a `GateRunResult`,
so it does not by itself answer what `compare` would consume.

## What was measured

**`Compare_Gate_Runs`'s own signature is the actual constraint.**
`pub fn Compare_Gate_Runs(baseline: &GateRunResult, candidate: &GateRunResult) ->
GateCompareResult` (`gate_compare.rs:77`) takes two borrowed, already-constructed values. It
does not take two `RunId`s, two paths, or two file handles. Anything a composition root
builds to call it must produce two real `GateRunResult`s from *something* -- and the crate's
own doc is explicit that inventing a lookup-by-`RunId` store is out of scope for it.

**`RunId` already exists and is already threaded through, which answers less than it looks
like it does.** `GateRunResult.run: RunId` (`OD-WORKFLOW-001`'s first real consumer,
`P13-GATE-REPORT-RUNID`) gives every run an identity, and the CLI report already prints it.
But an identity is not a store: nothing persists a `GateRunResult` keyed by the `RunId` it
carries, so naming a past run's id today gives a caller nothing to retrieve. `RunId` would
be exactly the right key for a persistence layer if one existed; its existence does not
imply one does.

**Both motivating cases `gate_compare.rs`'s own doc names are same-process cases.** That
doc states the question `compare` answers as "what changed since the run I already
trusted," and gives two concrete forms: a code change ("what changed since the run I
already trusted") and a policy change ("did tightening or loosening a policy change what
can fail this build"). Neither inherently requires a value to outlive the process that
produced it:

- A policy comparison holds the tree fixed and varies `GateCommand`'s `suppressions`,
  `baseline`, `rules` or `adoption` between two calls to `Run_Gate` over the *same* already-
  walked sources -- both `GateRunResult`s exist in the same process, at the same moment, and
  neither needs to survive past the call that produces it.
- A code-change comparison holds policy fixed and varies the tree: walking `root` as it
  stands now, and walking a second root -- a second working-tree path, or the same path
  checked out at a named git revision into a scratch directory -- produces the second
  `GateRunResult` the same way, inside the same invocation.

Both are answerable today, with the exact function `Compare_Gate_Runs` already has, by a
composition root that walks (or re-walks) twice and calls it once. Neither needs
`GateRunResult` to gain `Serialize`, and neither needs a persisted run history keyed by
`RunId`.

**The case that is not answered this way is a real, different one, and it is not today's
case.** "Diff this run against the one CI produced an hour ago, from a different process or
a different machine" needs a value that outlives its producing process -- either a
serialized `GateRunResult` twin (`response.rs`'s premature-surface caution then applies to
`compare`'s own shape, not only to `run`'s) or a real store keyed by `RunId`. Nothing in
this workspace needs that today: no CLI flag, API request or MCP tool exists yet for
`compare` at all, so there is no caller asking for cross-process comparison to leave
unserved. Building either the serialized twin or the store now, before that caller exists,
is the same "no invented shape ahead of a real body" `gate_compare.rs`'s own module doc
already states as the reason it took `GateRunResult`s directly rather than a store lookup --
applied here to the CLI/API/MCP wiring question rather than to the diff function itself.

## The decision

**A compare surface's first real increment re-derives both runs inside one process; it
does not serialize `GateRunResult` and does not persist a run history.** Whichever host
wires `compare` first -- CLI, `nomos-api`, or `nomos-mcp` -- takes two same-process
producible things (two roots, one root at two git revisions, or one root under two
policies) and two calls to `Run_Gate`, and hands both real `GateRunResult`s directly to
`Compare_Gate_Runs`, the same way `gate run` already composes a single walk today. No new
serializable type is invented, `GateRunResult` gains no derive, and no run-history store is
built. This is a narrower, cheaper first increment than a `RunId`-keyed store would be, and
it answers both cases `gate_compare.rs`'s own doc names.

**Cross-process comparison is real, and it is later work, not declined work.** "Compare
today's run against one a prior invocation produced" needs either a serialized
`GateRunResult` twin (mirroring `response.rs`'s existing one-way projection, but built for
round-tripping rather than only rendering) or a persisted run history keyed by `RunId`,
whichever the composition root that first needs it prefers -- and that choice is deferred to
whoever builds it, once a real caller needs it, the same way `P40-FACT-STORE-PERSISTENCE`
defers incremental-analysis persistence until a real long-lived consumer needs it. Naming
`RunId` as the eventual key, if a store is built, is not a commitment to build the store now.

## What this record does not do

**It does not wire `compare` into the CLI, `nomos-api`, `nomos-api-transport` or
`nomos-mcp`.** Each is separate implementation territory, and each can now proceed without
inventing its own answer to the question this record closes.

**It does not add `Serialize`/`Deserialize` to `GateRunResult`, `GateCompareResult`,
`FindingDisposition` or `DispositionChange`.** `response.rs`'s existing caution about
growing `nomos-gate-orchestration`'s public surface on behalf of one caller's shape stays in
force; a first increment needs none of them serialized.

**It does not decide the CLI's or API's argument shape for naming a second root** (a second
path, a `--against <git-ref>` flag, or something else) -- that is ordinary implementation
judgment for whichever item builds the CLI verb, not a question this record needed to settle
to unblock it.

**It does not build or schedule a persisted run-history store.** That remains real, future
work, unblocked by this record rather than started by it.

## Status

Accepted. `Compare_Gate_Runs` needs no new store and no new serializable type to gain a
first real caller: two same-process walks of `Run_Gate`, over two roots, two revisions, or
two policies, produce the two `GateRunResult`s it already takes. Cross-process comparison
against a run a prior invocation produced remains a real, separate, deferred question, with
`RunId` already in place as the key a future store would use if one is built.
