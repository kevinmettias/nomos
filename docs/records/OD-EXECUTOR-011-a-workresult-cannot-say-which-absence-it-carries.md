---
id: OD-EXECUTOR-011
type: decision
title: A WorkResult cannot say which absence it carries
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - executor
  - contracts
relations:
  - target: OD-EXECUTOR-008
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-EXECUTOR-005
    type: relates-to
  - target: OD-CONTRACTS-003
    type: relates-to
---

# A WorkResult cannot say which absence it carries

## Question

`OD-EXECUTOR-008` decided what content a canonical `WorkResult` may honestly carry, and
that decision holds: four of its six fields stay structurally absent for a bare-prompt
executor rather than model-filled. What that record did not decide is how a consumer tells
one absence from another. `plan: None` and `claims: []` are byte-identical whether the task
genuinely produced no plan and no findings, or the producing executor structurally cannot
substantiate either field. The only carriers of that distinction today are doc comments —
one in the executor crate, two at the API boundary — so a consumer holding only the value
cannot recover it at all.

A person has stated the invariant this record exists to satisfy: **an empty value must mean
the task produced none, never that the adapter never implemented extraction.**

This is a decision rather than a correction because neither of the two records that reach
this ground decides it. `OD-EXECUTOR-001` governs the invocation-site capability boundary
and `OD-EXECUTOR-008` governs content honesty; the representation of unavailability is a
third question, and it is open.

## What Was Measured

Measured 2026-09-17 at `f8b9310b`, against the working tree and the crate sources rather
than against this record's own summary of them.

**The type carries no presence marker of any kind.**
`crates/agent/nomos-agent-contracts/src/work_result.rs` declares six public fields —
`plan: Option<CorrectionPlan>`, `claims: Vec<Finding>`, `tests: Vec<VerificationPredicate>`,
`requested_verification: Option<VerificationPredicate>`, `assumptions: Vec<String>`,
`unresolved_questions: Vec<String>` — and nothing beside them. Its own doc comment reasons
carefully about which of them an agent may legitimately have nothing to say about, which is
exactly the reasoning a reader of the *value* cannot repeat.

**There is exactly one producer of a `WorkResult` in this workspace.**
A sweep for `WorkResult` across `crates/` finds one non-test construction site:
`Work_Result` in `crates/agent/nomos-agent-executor-claude-code/src/response.rs`. Every
other occurrence is a consumer, a re-export, a test fixture, or prose. In particular
`nomos-model-backend-ollama` builds none: `OD-EXECUTOR-005` established it is a
`ModelBackend` and not a second `AgentExecutor`, so the phrase "the producing executor" names
one crate today and not an abstraction with two instances.

**Four of the six portions are set unconditionally, and the reason is documented rather than
typed.** `response.rs` constructs `plan: None`, `claims: Vec::new()`, `tests: Vec::new()`,
`requested_verification: None` on every invocation, above a doc comment stating that this
dispatch "has no honest grounding for any of them, only for the judgment `assumptions` and
`unresolved_questions` name." `lib.rs`'s `JSON_SCHEMA` declares the other two and refuses
additional properties, so the model is structurally unable to return a plan.

**The two portions that are read have a real guarantee, and it too is prose.**
`String_Array` refuses an absent, non-array, or non-string field as
`AgentExecutionError::Unparseable`, so a conforming answer always carries both as arrays and
an empty `assumptions` genuinely means the model produced none. That is a stronger guarantee
than the four others have, it is the guarantee the invariant asks for, and a reader can
recover it only by reading `response.rs`.

**The distinction has already been forced into prose twice, at the API boundary.**
`crates/host/nomos-api/src/agent/agent_dispatch_response.rs` and
`crates/host/nomos-api/src/workflow/agent_execution_outcome_response.rs` both drop the four
portions from their wire shape and explain the omission identically: "not projected here
because they are always structurally absent for this executor, never because a wire caller
could not use them if they existed." That sentence *is* the distinction this record decides,
written out by hand in two places because the type cannot carry it. A wire caller holding
either response cannot recover it.

**The assessment registry cites a doc comment as the gap.**
`tests/contract/requirements/AGT-002.assessment` carries `verdict: Partial` with a `gap:`
line naming `crates/agent/nomos-agent-executor-claude-code/src/lib.rs#stay structurally
absent rather` — a phrase from a comment. A requirement whose remaining gap is a sentence is
a requirement whose gap no checker can see.

**This workspace already has the shape this needs, one crate over.**
`crates/kernel/nomos-model/src/subject/unknown_reason.rs` solves the same problem — a caller
cannot act on an absence that does not say why — with `UnknownReason`: named variants rather
than a bare absence, carried as a payload (`Intersection::Unknown(UnknownReason)`), each
variant holding the data that makes its reason specific, a `Describe()` rendering one line a
person can read, and `Every_Way_Independence_Can_Go_Unresolved` enumerating *every* variant so
a new one is a compile error in that test rather than a variant nothing covers.

**The surface is blessed, so this change cannot be silent.**
`WorkResult`'s six fields are blessed in `tests/contract/surface/nomos-agent-contracts.txt`,
and `AgentExecutionOutcome::result` in
`tests/contract/surface/nomos-agent-executor-claude-code.txt`. A change to either is visible
to the contract tests rather than to whoever next reads the diff.

## The Decision

**A `WorkResult` carries what its producer could substantiate, as a value rather than as
convention.**

**1. Each of the six portions is declared.** A new type in `nomos-agent-contracts` states,
per portion, either that the producing executor substantiated it or that it could not. The
shape is two variants, not a boolean and not a presence flag.

**2. The reason is named.** Where a portion cannot be substantiated, the value carries *why*,
following `UnknownReason` rather than a bare absence: today exactly one reason is justified —
the producer's own mechanism cannot ground the portion, which is the bare-prompt dispatch
`OD-EXECUTOR-008` measured (an isolated empty directory, no tool granted, one turn, so the
model has seen no real file and computed no real digest by the time it answers). A test
enumerates every reason, so a second reason is a compile error in that test rather than a
variant nothing covers, the discipline `Every_Way_Independence_Can_Go_Unresolved` already
holds one crate over. A boolean was refused precisely here: "this executor cannot ground it"
and "the answer's schema did not carry it" call for different replies, and a boolean cannot
tell them apart.

**3. The declaration is one field on `WorkResult`, with one named entry per portion.** A
reader asks the value directly — `result.substantiation.plan` — so no lookup by a separate
portion identifier is needed and no portion can be silently omitted from the declaration:
the six entries are the six fields, and a seventh field added to `WorkResult` is a compile
error until its substantiation is declared.

**4. The producing executor publishes the declaration it sets, beside the declaration it
already publishes.** `crates/agent/nomos-agent-executor-claude-code/src/lib.rs` already
publishes `JSON_SCHEMA` — a `pub const` stating what its answers carry. The substantiation
declaration is published the same way and in the same place: what this executor can
substantiate about its own answers is a property of the executor, stated once, and the value
it returns carries it. `JSON_SCHEMA` answers "what shape may the model return"; the
declaration answers "what did this crate ground" — two facts, both about the producer, both
published rather than implied.

**5. The consumer's rule, stated so it is derived rather than inferred.** A portion declared
substantiated whose value is empty means the task produced none: this is the invariant, and
it now holds by construction rather than by reading a comment. A portion declared
unsubstantiated means the value says nothing about the task, whatever it holds. No consumer
infers, and no consumer has to know which executor it is holding.

**6. `OD-EXECUTOR-008` is unchanged, and this record does not weaken it to make the
distinction easier.** The four portions still come back empty; they are still constructed by
the crate and never read from the model; they are still empty for exactly the reason that
record measured. The narrow schema stays narrow, `additionalProperties: false` stays, and a
response missing `structured_output` is still refused rather than coerced. What changes is
that the emptiness is accompanied by a value stating it, which is the opposite of weakening:
the content decision is now enforced by a type that a consumer can read instead of by a
comment a consumer must trust.

## What This Record Does Not Do

**No code changes here.** The types, the field on `WorkResult`, the constant, the six test
fixtures that construct a `WorkResult` literal, the blessed surface snapshot, and the two API
twins are each a follow-up item's own territory. Named so those territories can be declared
completely rather than discovered mid-claim — the way `OD-EXECUTOR-008` named its own four
rendering sites — they are: `nomos-agent-contracts`'s new types and new field (three new
files, one per public type, plus `work_result.rs`, `lib.rs` and the blessed snapshot);
`nomos-agent-executor-claude-code`'s published constant, its `response.rs` construction and
its snapshot; and `nomos-api`'s two response twins if the wire question below is answered
yes.

**It does not populate the four portions.** `OD-EXECUTOR-008`'s deferral stands unchanged:
`plan`, `claims`, `tests` and `requested_verification` stay unbuilt until a real executor
with real grounding exists. This record makes their absence *sayable*. It does not make them
fillable, and a reader must not read "declared unsubstantiated" as "about to be implemented".

**It does not decide the wire.** It binds any surface that projects a `WorkResult`: a
projection that omits a portion must be able to say why the portion is omitted, or it is back
to the sentence this record found written twice by hand. Whether the two existing twins carry
the declaration, or are justified in not carrying it because every portion they omit is
declared unsubstantiated, is that follow-up's question — and it is a real question rather
than a formality, because the twins are `nomos-api`'s own serializable shapes and not the
contract type (`OD-HOST-002`).

**It does not move the distinction to the outcome, and it does not add an error variant.**
`AgentExecutionOutcome` carries facts about the invocation — `denied_tool_uses`, `is_error`,
`cost`, `duration_ms` — and `OD-EXECUTOR-008` drew that line deliberately. Substantiation is a
fact about the content's completeness, so it belongs to the content. An
`AgentExecutionError` variant was refused as well: an error is a dispatch that produced no
result, and this is a result that was produced and is honest about its own limits. Refusing
rather than answering would delete the judgment that `assumptions` and
`unresolved_questions` legitimately carry.

**It does not generalize to `TaskEnvelope`, and it does not touch model-input assembly.**
`TaskEnvelope.expected_output_schema` names a schema by `SchemaId`; whether a task's declared
expectation should also declare what a conforming answer may substantiate is a question about
the envelope, not about the result, and no measurement here reaches it.

## Alternatives Considered

**A declaration of six booleans.** Refused. A boolean says *that* a producer cannot
substantiate a portion and never *why*, so a consumer still cannot choose a reply, and the
six booleans are individually unverifiable — nothing distinguishes a producer that honestly
declares a limit from one that sets a field to `false` and moves on. `UnknownReason` already
rejected the bare absence for this reason, and this workspace would then hold two answers to
one question.

**Changing the four field types so the distinction is inside each field** — replacing
`Option<CorrectionPlan>` with a three-state value, and the `Vec`s likewise. Refused on two
counts. It makes the ordinary case worse for every existing reader, who would unwrap a
presence marker to reach a plan that is there; and it expresses one fact about the producer
six times over, so a second executor's declaration becomes six edits in six places instead of
one value, which is how a declaration drifts out of agreement with itself.

**Carrying the declaration on `AgentExecutionOutcome` rather than on `WorkResult`.**
Refused, and this is the alternative closest to sound. `OD-EXECUTOR-008` separated invocation
facts from content, and putting substantiation beside `cost` would follow that line. It fails
on the measurement that opened this record: a consumer holding only the value cannot recover
the distinction, and a declaration on the outcome leaves exactly that consumer unable to.
`WorkResult` is the artifact the workspace agrees on — the reason `OD-EXECUTOR-008` built one
at all — so the fact that makes it readable belongs on it.

**An `AgentExecutionError` variant, or reuse of the existing `Unavailable`.** Refused. It
converts an honest, useful result into a failure. A judgment-only dispatch that returns its
assumptions and its open questions, and truthfully reports that it could ground nothing else,
is not an errored dispatch, and modelling it as one would make the common case an error path.

**Deriving the distinction from the schema the task requested** —
`TaskEnvelope.expected_output_schema`, which names `JSON_SCHEMA` by id. Refused as the
*carrier*, though it is true that the schema describes which portions an answer can carry.
The linkage is enforced in `response.rs`'s code and not by the schema, so a reader would
still be inferring; and a consumer holding a result has no envelope to consult. It is
recorded here as a related fact and not as a second authority: the schema says what the model
may return, and this record's declaration says what the crate grounded.

**A presence flag per field.** Refused for the reason the boolean was: `Some`/`None` and a
boolean are the same absence with a different spelling, and the question this record answers
is not whether a field is present but why it is not.

## Status

Accepted. A `WorkResult` carries its producer's own substantiation declaration as a value,
with a named reason where a portion cannot be grounded; `OD-EXECUTOR-008`'s content decision
is unchanged and unweakened; the producing executor publishes the declaration beside
`JSON_SCHEMA` and sets it on every result it builds. No code changes here — the follow-up
territories are named above.
