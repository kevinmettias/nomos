---
id: OD-TRACE-004
type: decision
title: A User Story is narrative evidence for a requirement's own assessment, not a second assessable statement
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - traceability
  - requirements
  - corpus
  - completeness
  - verification
relations:
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-TRACE-002
    type: relates-to
  - target: OD-TRACE-003
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
---

# A User Story is narrative evidence for a requirement's own assessment, not a second assessable statement

## Question

`OD-TRACE-001` declared the registry's population in prose — "the v14 corpus carries 363
requirements in 61 families at `authority: canonical-normative-record`" — and never named
the corpus's other statement kind. The v14 corpus also carries User Story statements: the
ingest catalog's own vocabulary is `Requirement` and `User Story`
(`crates/spec/nomos-spec-ingest/src/phases/statements.rs`), and the corpus's own naming
convention writes a User Story's id as `US-` in front of the requirement id it narrates —
`US-AGT-001` beside `AGT-001`, `US-CHK-001` beside `CHK-003`
(`crates/spec/nomos-spec-ingest/tests/corpus/statements-slice.yaml`,
`crates/spec/nomos-spec-model/tests/corpus/statements.json`). `Is_Requirement_Id` checks
shape only — uppercase family segments, then three digits — and that shape admits `US-CHK-001`
by accident: nothing wrote the check with User Story in mind, and nothing decided whether one
belongs in the population `FEWEST_ASSESSMENTS` counts.

Whichever answer is right, it must not stay implicit. A User Story admitted without a
decision reports coverage of a population nobody chose — the overstatement `OD-TRACE-002`
already refused when it kept the corpus comparison out of the guard entirely, one level up.
A User Story that genuinely binds and is excluded is unmeasured. This record picks a side and
states the reason.

## What A User Story Already Is, In This Repository

Not hypothetical. `OD-CONTRACTS-002` already read one. Deciding whether `CHK-003` binds this
build, it cited `US-CHK-001`'s acceptance text directly: "applicable, not-applicable and
unsupported counts are shown; agent-required and correctable combinations are identified."
That citation did one job — it told the record what "reported" has to mean for `CHK-003`'s
seventh category, agent-required, because the requirement's own text does not say and the
story's acceptance criteria do. `OD-CONTRACTS-002` never produced a second assessment for
`US-CHK-001` itself, never gave it a verdict, and never treated it as owing one. It read as a
lens on `CHK-003`, not as a second subject.

That is the shape a User Story has by construction. Its text is "As a `<persona>`, I can
`<capability>`. Acceptance: `<criteria>`" — a scenario told from a persona's side, existing to
make a requirement's own acceptance concrete for a human reader. It is not a second
specification of behavior independent of the requirement it narrates; the corpus's own id
convention says as much structurally, by spelling a User Story's id as the requirement's id
with a prefix rather than as an identifier of its own.

## The Decision

**This registry's assessable population is Requirement statements, at `authority:
canonical-normative-record`. A User Story statement is not a second member of that
population, and `Is_Requirement_Id` refuses its shape rather than admitting it by accident.**

A verdict here is a claim about whether a *site in this workspace* satisfies a *norm the
corpus states*. `Met`, `Diverges`, `NotBinding` and `Partial` are all decisions (or the
absence of one) about that relationship, and `Diverges`/`NotBinding` each owe a governing
record because they assert somebody chose the departure. None of that is coherent read
against a User Story:

- **`Met` would duplicate, not add.** A User Story's acceptance criteria restate the
  requirement it narrates in a persona's voice. An entry for `US-CHK-001` reading `Met` beside
  an entry for `CHK-003` reading `Met` is one fact, filed twice under two identifiers,
  inflating `FEWEST_ASSESSMENTS` without adding anything a reader could learn from `CHK-003`'s
  own entry.
- **`Diverges` and `NotBinding` have no decision to name.** Both verdicts assert somebody
  chose that this build will not do what the statement says. Nobody decides against a
  persona's story independently of deciding against the requirement it narrates; the decision,
  if there is one, is `CHK-003`'s to record, not `US-CHK-001`'s to record a second time.
- **`Partial` fares no better.** `OD-TRACE-003` gave `Partial` a gap instead of a record
  because unfinished is not a decision. A User Story half-satisfied is `CHK-003` half-satisfied,
  read from the persona's side — the same fact through the same shape, not a distinct one.

So a User Story is not a statement this registry holds to `Met`/`Diverges`/`NotBinding`/
`Partial` on its own. It remains exactly what `OD-CONTRACTS-002` already used it as:
authoring evidence a Requirement's own entry, or the governing record behind a `Diverges` or
`Partial` gap, may cite in prose to say what the requirement's acceptance concretely means.
Nothing here forbids that citation; the registry's `record:` and `gap:` fields already accept
free-form paths and reasoning, and a User Story id inside that prose is not a stem the reader
parses.

## The Shape, Enforced Rather Than Only Decided

`Is_Requirement_Id` refuses a stem whose leading `-`-delimited segment is `US`. The corpus's
own convention makes that check exact rather than a guess: a User Story's id is always the
requirement's own id with `US-` prepended, so the leading segment is the tell, and refusing it
there is refusing the kind rather than pattern-matching one family's spelling. A file named
`US-CHK-001.assessment` is refused by `Parse` at read time, the same way every other
non-entry shape in this registry already is — `OD-TRACE-002`'s rule, that a lenient reader
here would turn a mismatch into an assessment nobody chose to count.

`Test_A_Requirement_Identifier_Should_Be_A_Family_And_A_Number` moves `US-CHK-001` from its
accepted list to its refused list, so the test that names the population and this decision
cannot silently drift apart again.

## Why Not Widen `FEWEST_ASSESSMENTS`'s Population Instead

The alternative — admit `US-` ids and give a User Story its own, weaker obligation ("Met"
without a site requirement, say) — was considered and rejected. It would need a fifth
distinction inside `Verdict` or a parallel enum, doubling the shape this registry already
has, to hold a fact that the Requirement-side entry already states. `OD-SPEC-007` and
`OD-TRACE-002` both chose the narrower shape when a wider one was available for the identical
reason: a second place to say the same thing is a second place for it to disagree with the
first, and nothing here needs a User Story to disagree with the requirement it narrates in
order to be useful.

## What This Record Does Not Do

- **It does not assess a User Story.** No entry under `tests/contract/requirements` is
  authored or changed by this record; `FEWEST_ASSESSMENTS` stays where the committed set
  leaves it.
- **It does not forbid citing a User Story.** `OD-CONTRACTS-002`'s citation of `US-CHK-001`
  stands as the model for how one belongs in this registry's reasoning — inside a Requirement
  entry's `record:` prose, not as a stem of its own.
- **It does not reach the other 60 families' worth of User Stories.** The refusal is
  structural, by id shape, so it holds for all of them without naming each.
- **It does not reopen `Verdict`.** Four writable verdicts stand as `OD-TRACE-001` and
  `OD-TRACE-003` left them; this record narrows the population they apply to, not the shapes
  themselves.

## Status

Closed by `P12-TRACE-POPULATION`.
