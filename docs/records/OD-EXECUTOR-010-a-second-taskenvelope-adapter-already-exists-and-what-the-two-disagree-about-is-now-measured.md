---
id: OD-EXECUTOR-010
type: decision
title: A second TaskEnvelope adapter already exists, and what the two disagree about is now measured
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - executor
  - roadmap
relations:
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-EXECUTOR-004
    type: relates-to
  - target: OD-EXECUTOR-005
    type: relates-to
  - target: OD-EXECUTOR-006
    type: relates-to
  - target: OD-EXECUTOR-007
    type: relates-to
  - target: OD-PLATFORM-003
    type: relates-to
---

# A second TaskEnvelope adapter already exists, and what the two disagree about is now measured

## Question

`P42-SECOND-HARNESS-EXECUTOR` has been authored five times and built zero times. Its premise
is that "one agent executor exists, so nothing has shown which of its choices are the
contract," and its remedy is a new `nomos-agent-executor-harness` crate over "a richer
session or harness runtime, of which Clay is the motivating example." Its stated value is
not the crate but the evidence: "with one implementation nothing distinguishes what the
canonical task contract requires from what one vendor happened to need."

An item re-authored five times without being built is either blocked on something nobody has
named, or resting on a premise that stopped being true. This record asks which, against the
tree at `86568bc6`, rather than authoring it a sixth time.

## What Was Measured

**Clay is not a thing this repository can adapt.** Grepped across `crates/`, `docs/records/`
and `README.md`: the string appears exactly once, in `OD-EXECUTOR-006`, in a sentence
declining to decide it — "It does not decide Clay's shape." There is no harness runtime, no
session integration, no dependency and no backend here. Building an adapter for it would be
inventing the case rather than adapting a real one, which is the move `OD-EXECUTOR-006`
itself refused.

**The dispatch is no longer this workspace's to add one to.** `OD-PLATFORM-003` moved the
agent dispatch down into `xvpe-agent-backend-claude-code` and `xvpe-agent-backend-ollama`
over a capability contract in `xvpe-agent-execution`, on the ground that "a general
capability sitting up here was unreachable by everything down there." Nine backends now sit
under that contract in the engine (`anthropic-cli`, `claude-code`, `deepseek`,
`deepseek-cli`, `generic-cli`, `glm`, `kimi`, `kimi-cli`, `ollama`). A *new execution
architecture* in this workspace would reverse that migration. A new *adapter* here is a
different and much smaller thing: translate a `TaskEnvelope`, call a backend, read back an
outcome.

**A second adapter of exactly that smaller kind already exists.**
`nomos-model-backend-ollama` takes the same `TaskEnvelope` and returns its own outcome, and
`README.md`'s own band table already describes it as "structurally parallel to
`nomos-agent-executor-claude-code`, no shared trait, no shared package kind."
`OD-EXECUTOR-005` classifies it a `ModelBackend` rather than a second `AgentExecutor`, which
is a claim about its *role*, not about whether it is a second consumer of the envelope. As a
consumer of the envelope it is the second one, and it always was.

**The discriminating evidence arrived without a third crate.** `OD-EXECUTOR-007` decided
`available_tools` must be refused rather than silently dropped. `P42-EXECUTOR-ENVELOPE-
IMPLEMENTATION-4` built that refusal in one adapter and not the other, and the result was
immediately visible as a caller-facing defect rather than a design opinion: both adapters are
dispatched from one `match` on one envelope in `nomos-agent-orchestration::run.rs` and
`nomos-workflow-orchestration::run.rs`, so a declared capability was refused or silently
dropped depending only on which arm a backend flag selected. `P91` closed it by making
ollama refuse too. That sequence is the experiment the item wanted: a thing was enforced in
one implementation, the second implementation showed whether it was the contract, and the
answer was that it is.

### What the two adapters agree about, and what they do not

Measured from the two crates directly rather than inferred from their docs.

**Shared, and therefore the contract:**

- The envelope going in (`TaskEnvelope`) and a refusal type going out, both with an
  `Execute_Task`/`Execute_In` pair over an injected `ProcessLauncher`.
- `AgentCapability::Isolated` with `ToolGrant::Nothing` — neither grants any capability, and
  neither has ever dispatched under anything wider.
- `AgentExecutionError::UnsupportedTools` for a declared `available_tools` neither can grant,
  refused before any process starts. Shared *because* `P91` measured that a caller is owed
  the same answer either way, not because the two crates were written together.
- `AgentExecutionError::Unavailable` for "no answer was produced," including the engine's
  own `#[non_exhaustive]` errors folded in by a deliberate wildcard.
- `knowledge_context` and `scope` accepted and structurally unenforced in both, each for a
  reason already recorded — `OD-EXECUTOR-009` and `OD-EXECUTOR-007` version 2 respectively.

**Divergent, and therefore one vendor's own, each with its measured reason:**

| Choice | `nomos-agent-executor-claude-code` | `nomos-model-backend-ollama` |
|---|---|---|
| Answer shape | `JSON_SCHEMA`, `additionalProperties:false`, read from `structured_output` | none; stdout is free text, so there is nothing to conform |
| Failure taxonomy | `Unparseable` kept apart from `Unavailable` | no `Unparseable`: a clean exit with some stdout is a valid answer by construction |
| Spend | `MAXIMUM_SPEND`, a `MicroDollars` ceiling | none; local inference has no metered charge, a wall clock stands in its place |
| Effort | six `EffortLevel` values mapped onto five engine values, `Minimal` approximated to `Low` | unmapped; no control in the command line that `EffortLevel` honestly maps onto |
| `prohibited_changes` | compared before and after, refused by path | unenforced; no tool-use loop, so nothing can write |
| `applicable_rules` | folded into the goal text as advisory context | unenforced |
| Outcome | `WorkResult` plus `denied_tool_uses`, `is_error`, `cost`, `duration_ms` | one `response: String` |

Every divergent row traces to a real difference in what the backend can do, not to a
preference. That is the distinction the item existed to draw, and it is drawn.

## The Decision

**A third adapter is not the next increment, and `P42-SECOND-HARNESS-EXECUTOR-5` is declined
citing this record.** Its premise — that only one implementation exists, so nothing
discriminates contract from vendor choice — is false against this tree, and its named
motivating example is a system this repository has no integration with. Building it would
mean adapting an absent vendor to learn something two present adapters have already shown.

This is not a decision that a third adapter is forever unwarranted, and it moves nothing out
of scope permanently. It records that the condition which made the item worth doing has been
met by another route, so the item is complete in substance while its `done_when` — a new
crate, registered in four places — is now a cost with no remaining evidentiary return.

## What Would License A Third Adapter

Named as conditions rather than dates, so a future session can check them rather than
re-derive this argument:

- **A real backend this workspace must reach that the engine does not already carry.** The
  nine `xvpe-agent-backend-*` crates are the supply; an adapter here is warranted when
  something needs one of them, or something outside them, through a `TaskEnvelope`.
- **A third row in the divergence table that no existing adapter can produce.** The table
  above has two columns because there are two consumers. A candidate that would only add a
  third column agreeing with one of them everywhere teaches nothing; one that would disagree
  in a new place is evidence.
- **A `TaskEnvelope` field gaining a mechanism that cannot be built once.** Every shared row
  above is currently duplicated per adapter rather than factored into `nomos-agent-contracts`.
  Two copies are two copies; a third would be the point at which duplicating the envelope's
  own mechanisms is the defect rather than the cheaper option.

## What This Record Does Not Do

It does not retire `OD-EXECUTOR-005`'s classification: ollama is still a `ModelBackend` and
not a second `AgentExecutor`. This record's claim is narrower and about a different axis —
that it is the second *consumer of the envelope*, which is what the evidence needed.

It does not factor the shared mechanisms into `nomos-agent-contracts`. Two copies of
`Refuse_Ungrantable_Tools` exist and are named above as the thing a third adapter would make
intolerable; deciding to unify them now, with two consumers, would be the same
ahead-of-the-evidence move this record declines elsewhere.

It does not change any code. `P91` and `P92` already landed the behaviour this record
measures.

## Status

Accepted. The second implementation that would discriminate contract from vendor choice
already exists and has now been used for exactly that, so `P42-SECOND-HARNESS-EXECUTOR-5` is
declined rather than authored a sixth time. What the two adapters share and where they
diverge is recorded above, measured against the crates, with each divergence traced to a real
difference in backend capability.
