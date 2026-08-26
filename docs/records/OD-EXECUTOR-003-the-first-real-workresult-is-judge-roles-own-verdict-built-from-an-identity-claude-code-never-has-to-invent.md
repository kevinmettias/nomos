---
id: OD-EXECUTOR-003
type: decision
title: The first real WorkResult is judge-role's own verdict, built from an identity Claude Code never has to invent
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - executor
  - contracts
  - workflow
relations:
  - target: OD-CONTRACTS-003
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-WORKFLOW-002
    type: relates-to
  - target: OD-WORKFLOW-004
    type: relates-to
---

# The first real WorkResult is judge-role's own verdict, built from an identity Claude Code never has to invent

## Question

`OD-CONTRACTS-003` made `WorkResult.plan` an `Option` so a judgment-only response could be
represented, then named the real path it declined to build: "`TaskEnvelope.expected_output_schema`,
paired with Claude Code's own `--json-schema` support, is the real shape a future increment
would use to have the agent produce a `WorkResult`-shaped response directly ... not answered
here." `nomos-agent-executor-claude-code`'s own module doc repeats the same deferral in the
same words. `OD-WORKFLOW-002`, amended by `OD-WORKFLOW-004`, names one of three conditions
that would produce the workflow tier's next real increment as: a real executor "also
constructs a real `WorkResult`, not merely when one exists." Nothing has built this yet. Two
things changed since those records were written, and this record checks both before naming
anything: `claude --help` now documents a `--json-schema <schema>` flag, and this workspace
now has a second real `TaskEnvelope` construction site — `nomos-cli`'s `judge-role` — whose
own doc already says what `execute` cannot: it "does start from a rule's own finding."

## What Was Measured

**`--json-schema` was run against the real CLI, not assumed.** From an isolated, empty
directory, with `--strict-mcp-config` and an allow-list naming no real tool — the exact
boundary `OD-EXECUTOR-001` already requires — a `--print` invocation carrying
`--output-format json --json-schema '{"type":"object","properties":{"summary":{"type":
"string"},"assumptions":{"type":"array","items":{"type":"string"}},
"unresolved_questions":{"type":"array","items":{"type":"string"}}},"required":["summary",
"assumptions","unresolved_questions"],"additionalProperties":false}'` returned:

```json
{
  "is_error": false,
  "result": "{\"summary\":\"The sky is blue.\",\"assumptions\":[\"...\"],\"unresolved_questions\":[]}",
  "structured_output": {
    "summary": "The sky is blue.",
    "assumptions": ["..."],
    "unresolved_questions": []
  },
  "total_cost_usd": 0.0798447,
  "duration_ms": 2911,
  "permission_denials": []
}
```

Every field `nomos-agent-executor-claude-code::response::Parse` already reads (`result`,
`is_error`, `total_cost_usd`, `duration_ms`, `permission_denials`) is present and unchanged in
shape. `--json-schema` is purely additive: it adds one new top-level field,
`structured_output`, already parsed as a real JSON object — not a second string a caller
would have to decode twice. A first run at the default probe budget (`$0.20`) exhausted
before answering, from a large one-time cache-creation cost this isolated directory's first
invocation paid; at the crate's own existing `$1.00` default the call completed for
$0.08. This is measured, not designed around: no claim is made here about why the first
probe's cache cost was large, only that a schema-bearing call fits inside the budget this
crate already uses for every other invocation.

**`TaskEnvelope.expected_output_schema` is a `SchemaId`, not a schema.** Verified directly:
`SchemaId::New("...")` wraps an opaque string label everywhere it is constructed in this
workspace today (`nomos-agent-contracts`'s own test, `nomos-cli`'s `Task`/`Judgment_Task`).
Nothing maps a `SchemaId` to a real JSON Schema document, and building that mapping as a
general registry — so any caller's label resolves to a body — would be exactly the
"declaration nothing enforces" shape `OD-CAPABILITY-*` and `OD-PACKAGE-006`/`OD-PACKAGE-008`
already decline to build ahead of a second real need. This record does not build one either.

**`execute` still has no identity to ground a response against.** Read in full,
`crates/host/nomos-cli/src/agent.rs`'s own doc for `execute` states this directly: "this
command has no `RuleId` or `SubjectId` to give a `Finding` either, since nothing dispatched it
as a rule's judgment; it is a person, asking a question directly." A schema could still coax
free-form structure out of the model's prose, but nothing at that call site supplies a real
`RuleId` or `SubjectId` for a `Finding` to carry — inventing one would be the identical
dishonesty `OD-EXECUTOR-001` already named for trusting a process's own account of itself,
applied to identity instead of action.

**`judge-role` already has exactly the identity a `Finding` needs, read from a rule that
already ran, not from the model.** `nomos_rules::Check_Declared_Role_Matches_Surface` (`nomos-
rules`'s `role_surface.rs`) unconditionally reports `Applicability::AgentRequired` and, per its
own module doc, "never reaches a verdict." Its `Finding` already carries a real
`rule: RuleId::New("declared-role-matches-surface")` and a real
`subject: Subject_Of_Path(&pair.crate_root)` — a content-addressed identity computed from the
crate's own path, the same derivation `OD-MODEL-002` already uses, not invented by an agent
that never sees it. `judge_role.rs`'s `Judgment_Task` already threads `finding.rule` into the
dispatched `TaskEnvelope` (`applicable_rules: vec![finding.rule.clone()]`); what it does not
yet do is carry that same finding's identity back out the other side.

**`Applicability` has no pass/fail slot to invent one for.** Read in full: "There is no
`is_pass`. `Applicability::Was_Evaluated` is the closest thing, and it is not a pass — it says
only that a judgment was reached, not what it was." A model's verdict — agrees, disagrees,
partially agrees — is narrative content for `summary`, never a new `Applicability` variant. The
correct post-judgment value is `Applicability::Supported`: the rule bound the subject and a
judgment was reached, at `EvidenceClass::AgentJudged` — the evidence floor `nomos_contracts`
already defines for exactly this case: "a model produced this and no tool corroborated it."

**`WorkResult`'s other three report-only fields are honestly empty here, not merely unfilled.**
`plan`, `tests`, and `requested_verification` each presuppose grounded knowledge of the real
repository — a real edit against real file content, a real command worth running, a real
predicate to accept a submission — that `OD-EXECUTOR-001`'s boundary (an empty isolated
directory, no tools, one `--print` turn) structurally withholds from every dispatch this
crate makes, `judge-role` included. This is not a gap this record leaves to a future
increment by oversight; it is the same boundary `OD-EXECUTOR-001` already decided, read
against a new type.

## The Decision

**`judge-role` is the first call site permitted to assemble a real `WorkResult`.** `execute`
is not touched; it keeps rendering `AgentExecutionOutcome`'s free text exactly as today,
because it has no identity to assemble anything against.

The mechanism, precisely:

1. `judge-role`'s dispatch requests `--json-schema` with a small, hand-authored schema —
   `{summary: string, assumptions: string[], unresolved_questions: string[]}`, all three
   required, `additionalProperties: false` — carried as literal schema text at this one call
   site, not resolved from `TaskEnvelope.expected_output_schema`'s `SchemaId` through any new
   registry.
2. The response reader gains a path that requires `structured_output` to be present (a real
   JSON object, read the way `response.rs` already reads every other field: by key, tolerant
   of everything else the document carries) and reports `AgentExecutionError::Unparseable` —
   the same category a missing `result` already uses — when a schema was requested and
   `structured_output` is absent.
3. The caller assembles one `Finding` by copying `rule`, `subject`, `subject_name`, `gate`,
   and `locations` **verbatim** from the original `AgentRequired` finding
   `Check_Declared_Role_Matches_Surface` already produced, and setting only
   `applicability: Applicability::Supported`, `evidence: EvidenceClass::AgentJudged`, and
   `summary` from the parsed `summary` field. This is the only field in `WorkResult` this
   dispatch can honestly populate with more than an empty value.
4. `WorkResult.plan`, `.tests`, and `.requested_verification` stay `None` / empty. `.assumptions`
   and `.unresolved_questions` are read directly from the parsed document's own arrays.

## What This Does Not Do

It does not touch `execute` or its rendering. It does not build a `SchemaId → JSON Schema`
registry, or move `judge-role`'s schema onto `TaskEnvelope.expected_output_schema` — that
field stays a label naming this call site, exactly as `nomos-cli`'s own existing doc already
states it does today. It does not attempt to populate `plan`, `tests`, or
`requested_verification` for any dispatch this crate makes; `OD-EXECUTOR-001`'s boundary
still withholds what any of those three would need to be honest. It does not implement any of
this in Rust — that is a separate, following item, scoped by this decision rather than
decided by it. It does not itself declare `OD-WORKFLOW-002`/`OD-WORKFLOW-004`'s condition 3
fired; that record's own re-audit, once the following item lands, is where that finding
belongs. It does not change `Applicability`, `EvidenceClass`, or `Finding`'s own shape.

## Status

Accepted. `--json-schema` was run against the real CLI before this record named a mechanism
around it; `judge-role`'s existing identity was read from its own source, not assumed from
its name. Implementation follows in a separate item.
