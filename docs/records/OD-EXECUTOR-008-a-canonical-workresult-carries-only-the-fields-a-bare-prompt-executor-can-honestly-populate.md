---
id: OD-EXECUTOR-008
type: decision
title: A canonical WorkResult carries only the fields a bare-prompt executor can honestly populate
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - executor
  - contracts
relations:
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-EXECUTOR-007
    type: relates-to
  - target: OD-CONTRACTS-003
    type: relates-to
---

# A canonical WorkResult carries only the fields a bare-prompt executor can honestly populate

## Question

`TaskEnvelope.expected_output_schema` names a schema and nothing builds a `WorkResult`
against it. What comes back from Claude Code is read structurally rather than trusted as
prose — the right instinct, per `OD-EXECUTOR-001` — but the type callers hold is still the
vendor's own JSON shape: `AgentExecutionOutcome.response` is free text, and
`crates/host/nomos-cli/src/agent/dispatch.rs` and `workflow.rs` both print it directly. A
validated correction path, a workflow body and any second harness executor all have to
agree on what an agent produced, and today there is no one artifact to agree on.

`OD-EXECUTOR-007`, decided immediately before this record, named this gap directly as one
of `applicable_rules`' own blockers and declined to build it speculatively. A person has now
required it decided.

## What Was Measured

**`--json-schema` gives a real, separately validated `structured_output` field, verified
adversarially.** A narrow schema — `assumptions` and `unresolved_questions`, both string
arrays, `additionalProperties: false` — produced a clean `structured_output` object holding
exactly those two fields on a first, unforced prompt. A second, adversarial prompt
explicitly instructed the model to also emit a top-level `plan` field bypassing the schema;
`structured_output` still carried only the two declared fields, and the model's own text
named the reason: `additionalProperties: false` refused the extra key structurally. This is
the same real-denial shape `OD-EXECUTOR-001`'s own `permission_denials` finding already
established for tool access, now confirmed for output shape.

**`WorkResult`'s richer fields have nothing honest to ground them in this executor.**
`plan: Option<CorrectionPlan>` embeds a `ChangeSet`/`Edit` — a concrete file diff.
`claims: Vec<Finding>` embeds a `SubjectId` — a content digest of a real subject. Both
presuppose the agent read or produced something real to describe. This executor's own
structural boundary (`OD-EXECUTOR-001`: isolated empty directory, no tool granted by
default, one `--print` turn) means the model has seen no real file and computed no real
digest by the time it answers. A schema cannot make an invented `SubjectId` or an invented
diff honest; it can only make the *shape* conform, and a conforming lie is not this record's
goal. `assumptions: Vec<String>` and `unresolved_questions: Vec<String>` are different in
kind: free-text judgment a model forms about the goal it was given, with no claim to have
touched anything real.

**`AgentExecutionOutcome.response` is read by real callers today, all as display text, none
structurally.** `crates/host/nomos-cli/src/agent/dispatch.rs` (two sites) and
`crates/host/nomos-cli/src/workflow.rs` (two sites) each `writeln!` it directly to a human.
No caller anywhere parses `.response` as data. Replacing what the field carries changes four
call sites, all renderers, none consumers with a structural dependency on the current shape.

## The Decision

**The schema this executor requests is the narrow, honest subset:**
`{"type":"object","properties":{"assumptions":{"type":"array","items":{"type":"string"}},
"unresolved_questions":{"type":"array","items":{"type":"string"}}},"required":
["assumptions","unresolved_questions"],"additionalProperties":false}`, passed via
`--json-schema` on every invocation. `TaskEnvelope.expected_output_schema` names this schema
by `SchemaId`; a future caller wanting a richer schema is a different `SchemaId` and a
different question, not decided here.

**`AgentExecutionOutcome` carries a real `nomos_agent_contracts::WorkResult`, built only
from `structured_output`, never from `result`.** `assumptions` and `unresolved_questions`
are read off `structured_output`'s own two fields; `plan`, `claims`, `tests` and
`requested_verification` are always `None`/empty, constructed by this crate, never read from
the model — structural absence, the identical principle `OD-EXECUTOR-001` already applies to
tool grants, applied here to output content. `denied_tool_uses`, `is_error`, `cost_usd` and
`duration_ms` stay exactly as they are: facts about the invocation, not about its content,
unaffected by this decision.

**A response missing `structured_output`, or one that fails to decode into the two required
fields, is refused rather than coerced.** A new `AgentExecutionError` variant —
`Unparseable`'s existing shape already fits, reused rather than duplicated — reports it. The
existing free-text `result` field is not read as a fallback: a caller that gets a `WorkResult`
back got one Claude Code's own schema validation produced, or got nothing.

## What This Record Does Not Do

**No code changes here.** `response.rs` gains a `structured_output` reader; `Command_For`
gains `--json-schema`; `AgentExecutionOutcome`'s own field changes from `response: String` to
`result: WorkResult`; `dispatch.rs` and `workflow.rs`'s four rendering sites move from
printing `outcome.response` to rendering `assumptions`/`unresolved_questions` for a human —
each is a follow-up item's own territory, named here so its own territory can be declared
completely rather than discovered mid-claim the way this record's own first framing was.

It does not decide a schema for `plan`, `claims`, `tests` or `requested_verification`. Those
stay unbuilt until a real executor exists that can ground them — real tool access, a real
file read, a real fact — which `OD-EXECUTOR-007`'s own `scope` mechanism is the first step
toward, not a claim this record makes about when that arrives.

It does not change `nomos-model-backend-ollama`'s own response handling, or decide whether a
second executor shares this schema. `OD-EXECUTOR-002` already measured that this workspace
has exactly one real agent-given-tool-access executor today; a second one earns its own
measurement against its own real mechanism the same way this one did.

It does not build a richer schema for a future executor that does have real grounding. The
schema named above is this executor's own honest ceiling today, not a permanent limit on
what `WorkResult` could ever carry from an agent.

## Status

Accepted. A narrow, adversarially-verified JSON schema is the mechanism; `WorkResult`'s
ungroundable fields stay structurally absent rather than model-filled; no code changes here.
