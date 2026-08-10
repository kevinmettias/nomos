---
id: OD-SPEC-010
type: decision
title: Validation refuses what cannot be repaired later, and no later value overwrites what was submitted
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - substrate
  - validation
  - feature-lifecycle
relations:
  - target: OD-SPEC-008
    type: relates-to
  - target: OD-SPEC-009
    type: relates-to
  - target: ARC-SPECDB-002
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
---

# Validation refuses what cannot be repaired later, and no later value overwrites what was submitted

## Question

`ARC-SPECDB-002` charges every born-structured object with validation at write time that
refuses an incomplete submission rather than accepting it, and with keeping what was
originally submitted apart from what was later clarified, inferred or decided. `OD-SPEC-008`
applies that charge to feature requests, designs and results, and defers the rule set —
including the conditional rules a form contract needs.

Until the rules are written down the first writer encodes them, and a rule encoded in a
writer cannot be read, cited or disagreed with. This record states them.

It decides no physical layout, which is `OD-SPEC-011`'s. The intake surface is already
decided: `OD-SPEC-009` landed while this record was being written, and put every submission
through one accept function, with transports forbidden from validating. That makes this record
the content of the check that function performs, and the two fit without either being bent —
`OD-SPEC-009` fixed where validation happens and explicitly deferred what it checks, the
refusal vocabulary and the draft state to here.

The rules below are properties of a submission, not of a door. They are the same rules
whichever transport constructs it and whichever table it lands in. A rule set that had to be
re-argued per surface would be a rule set that lives in the surface, which is the failure
`OD-SPEC-009` refused.

## The Test For Requiredness

A list of required fields answers for the fields on the list and produces nothing for the
next field, which is the failure `ARC-SPECDB-002` refuses on the substrate axis. The same
refusal applies here, so requiredness is decided by a test rather than enumerated by
preference:

> A field is **required** when its absence cannot be repaired by a later reader without
> asking the submitter again.

That is the whole criterion, and it has a direction. `goal` is required because nobody can
recover why a thing was wanted from the thing itself. An estimate is not required because a
later reader can produce one, and a wrong estimate is visible as an estimate. The test is
answerable about a field nobody has proposed yet, which is the property it exists to have.

The field sets below are that test applied, not an independent authority. A field added later
argues from the test; a field dropped later argues against it.

## What Every Submission Carries

Applied to all three kinds `OD-SPEC-008` governs:

| Field | Why it cannot be repaired later |
|---|---|
| `id` | identity assigned elsewhere is a second identity |
| `kind` | which of the three, since the conditional rules below branch on it |
| `form_contract_version` | a submission written under an older set of questions is readable as that shape rather than silently reinterpreted |
| `title` | a sentence somebody wrote; a generated one asserts the submitter's meaning |
| `provenance` | who submitted it and through which surface — unrecoverable the moment the session ends |
| `state` | `draft` or `accepted`, per the section below |

`provenance` names the transport; it does not decide what transports exist.

There is **one** version and not two. `ARC-SPECDB-002`'s first obligation asks a
born-structured object to carry its schema version, and `OD-SPEC-009`'s second seam property
asks a submission to name the form contract version it was constructed against. Those are the
same fact, because the form contract *is* the schema of a submission — the set of questions is
what a filled-in form has to be read against. Giving them separate fields would be two names
for one version, and the first submission whose two values disagreed would have no reader able
to say which was right. So the field is `form_contract_version`, and it discharges both
obligations.

## What Each Kind Carries

A **feature request**, from `OD-SPEC-008`'s definition of one:

| Field | |
|---|---|
| `goal` | the outcome wanted, in the asker's terms |
| `behaviour` | what should happen |
| `acceptance` | how the asker will know it worked |
| `invariants` | what must not break |

A **design specification**:

| Field | |
|---|---|
| `answers` | the request or requests it answers |
| `alternatives` | what was considered, each with why it was not taken |
| `selected` | which alternative is chosen |
| `architecture_delta` | what changes in the architecture, or that nothing does |
| `acceptance` | what acceptance will require |

A **feature result**:

| Field | |
|---|---|
| `implements` | the design it was built against |
| `evidence` | what was run, and what it exited |
| `deviations` | where it departed from the design |
| `owed` | what it left owed |

## An Absence And A Claim Of Absence Are Different Facts

`invariants`, `alternatives`, `architecture_delta`, `deviations` and `owed` are each fields a
truthful submission may have nothing to put in, and that is the case the rule has to get
right.

> A required field is satisfied by content or by an explicit statement that there is none.
> It is never satisfied by emptiness.

An empty `deviations` means nobody looked. `deviations: none` means somebody looked and found
none, and is a claim the result can later be held to. Collapsing the two would make the
honest submission and the abandoned one identical rows, and the abandoned one is the common
case.

## Conditional Rules

The rules that cannot be expressed as a required-field list, which is why `OD-SPEC-008` named
them separately.

1. **`selected` names one of `alternatives`.** A chosen option absent from the options
   considered means one of the two fields is false, and which one is not decidable by a
   reader.
2. **`alternatives` carries at least two entries, one of which may be `do nothing`.** A
   design with one alternative did not choose; it recorded. `do nothing` is admitted
   explicitly so the rule is satisfiable honestly rather than by inventing a straw option.
3. **Each entry in `deviations` names the design clause it departs from.** A deviation from
   nothing in particular cannot be reviewed, and cannot be closed.
4. **`implements` resolves to an `accepted` design, and `answers` to an `accepted` request.**
   A result built against a draft is a result whose target may still change under it.
5. **`evidence` is required for an `accepted` result and not for a draft one**, because
   evidence is the thing acceptance is acceptance *of*. This is the one field whose
   requiredness is conditional on `state` rather than on `kind`.
6. **A submission with an open blocking decision gap cannot be `accepted`**, per the gaps
   section below.

Rule 4 is the only one that reads a second row, and it is stated as a rule on the submission
rather than as a constraint on the edge. The store's `relation_types` table carries `name`,
`tier` and `inverse_of` and has no domain, range or cardinality, so nothing today stops
`implements` from joining a suite to a table row. That gap is real and is not closed here;
it is named in *What This Does Not Decide*.

## What A Refusal Names

A refusal that says `invalid submission` moves the work of finding out to the submitter, who
has less information than the validator did.

> A refusal names the submission, every field that failed, the rule each one failed, and what
> would satisfy it. It names **all** failures, not the first.

Reporting the first failure only makes a form with six holes take six refusals, and the
submitter learns the rule set by exhaustion. This repository has paid for the same shape
before: `OD-LEDGER-007`'s subject is the cost of a refusal that stops more than it needed to.

And the refusal is total:

> A refused submission is stored nowhere. There is no partially written row, no `invalid`
> state, and no quarantine table.

An `invalid` state would mean every reader of the store needs the rule set in order to know
which rows are real, which is validation moved back into the readers — the thing write-time
validation exists to prevent. `OD-SPEC-008` already forbids the storage half of this from the
other direction: only tables something writes to exist, and a quarantine table is a table
whose writer is a bug.

## Draft And Accepted

Two states, and the boundary between them is where this rule set could most easily become
vacuous.

A **draft** carries every universal field, every field of its kind, and satisfies every
conditional rule except 5. It is a complete submission that has not been accepted.

An **accepted** submission is a draft that additionally satisfies rule 5, has no open
blocking gap, and may be cited: only an accepted request may be the target of `answers`, and
only an accepted design the target of `implements`.

What `draft` explicitly is **not** is a home for incomplete submissions.

> Incompleteness is refused in both states. `draft` weakens which rules apply; it never
> weakens whether they are checked.

The alternative — drafts stored under a weaker rule set — makes write-time validation
vacuous, because every refusal becomes a draft and the store fills with rows that satisfy a
schema and assert nothing. `P10-VACUITY-HOME` is an open item about a command that can report
success without checking anything; the same shape installed in the substrate would be worse,
because a command can be fixed and a corpus of half-written rows cannot.

Promotion from `draft` to `accepted` is a transition, and it is recorded by the mechanism that
already exists rather than by a second one: `node_history` is append-only, hash-chained, and
its `reason` column is `NOT NULL` with a non-empty check, because a history entry that does
not say why is a row that satisfies a schema. Acceptance with no reason is refused by the
table.

## Submitted, Clarified, Inferred, Decided

`ARC-SPECDB-002`'s third obligation is separation, and separation is a shape rather than a
convention. The shape:

> Every value carries an **origin**: `submitted`, `clarified`, `inferred` or `decided`. A
> later value supersedes an earlier one for reading and never replaces it in storage.

So a field is a sequence of attributed values, not a cell. The current reading is the latest;
what was originally asked is always recoverable, which is the one thing a request exists to
preserve. This is `normative_statements.supersedes_hash` applied to a second kind of row
rather than a new mechanism, and reusing it is deliberate: two supersession models in one
store would be two answers to what a superseded value is.

`OD-SPEC-009` requires a transport to hand over what was submitted verbatim, separately from
anything the transport itself supplied — a CLI default, a form's pre-populated field, a value
an agent filled in. It requires the separation and does not say where the transport's own
values land, because that is an origin and origins are here. They land in `inferred`:

> A value a transport supplied rather than a submitter typed has origin `inferred`, whatever
> made it up.

That is the rule that makes the two records meet. A default is a guess by machinery, which is
what `inferred` is for, and it therefore cannot satisfy acceptance under the rule below. The
alternative — admitting transport defaults as `submitted` — would let a CLI's convenience
become the asker's stated intent at the exact seam `OD-SPEC-009` built to prevent that.

The origins are not interchangeable, and one rule keeps them from becoming so:

> A submission is `accepted` only if every required field's current value has origin
> `submitted`, `clarified` or `decided`. An `inferred` value is readable and is never
> sufficient.

`inferred` is where a tool's or a reader's guess goes. Admitting it is useful — a guess
written down as a guess is better than a guess written down as the asker's intent — and
letting it satisfy acceptance would make the system accept its own inferences as what
somebody wanted. That failure is silent by construction, which is why the rule is stated
rather than left to care.

`decided` is the origin of a value a governing record or a recorded decision closed. It
satisfies acceptance because a decision is answerable to something; an inference is not.

## Decision Gaps

`OD-SPEC-008` says a feature request names the decisions it needs that nobody has taken yet.
A gap is therefore a first-class row and not a note in prose.

A gap carries the question, the fields it blocks, and a severity of `blocking` or
`non-blocking`.

> A gap is closed only by a citation — to a governing record, or to a recorded decision. It
> is **never** closed by supplying the value it blocks.

Supplying the value is exactly the silent resolution `ARC-SPECDB-002`'s third obligation
exists to prevent: the field fills in, the gap disappears, and nothing anywhere says the
question was answered by whoever happened to be typing. Under this rule the value arrives
with origin `decided` and the citation is what makes it so.

A `blocking` gap prevents acceptance. A `non-blocking` gap does not, and survives into the
design and the result rather than being dropped at each hand-off — an open question that
disappears at acceptance is an open question nobody will ask again.

## What This Binds

It binds what a writer must check before persisting, and what it must say when it refuses. A
writer that persists something failing these rules is a defect against this record, not a
lenient implementation.

It binds the shape of attribution and supersession for these three kinds, and it binds them to
the mechanisms the store already has rather than to new ones.

It binds `OD-SPEC-011` to a layout that can express an attributed sequence per field and a
gap as a row. That is a real constraint on the layout and is stated here because the layout
cannot be judged without it.

## What This Does Not Decide

It does not decide the physical layout. Which of these are columns, rows or a payload is
`OD-SPEC-011`'s, and that item lands the layout with its first writer so no table arrives
empty.

It does not decide the intake surface. `OD-SPEC-009` decided it, and nothing above reads a
transport: the accept function is where these rules run, and which transports exist is not a
question this record can reopen.

It does not add domain, range or cardinality to `relation_types`. The lifecycle edges
`OD-SPEC-008` implies — a design answering a request, a result implementing a design — are
expressible today as rows in that table, and are unconstrained: the schema has no column
saying which node kinds an edge may join or how many of an edge a node may have. Rule 4 above
constrains the *submission* and cannot constrain the *graph*. Closing that is separate work
reaching `crates/spec/nomos-spec-store/src`, which is `P10-REQUEST-LAYOUT`'s territory today,
and it is named here so the next reader finds it stated rather than absent.

It does not govern document-first objects. Governing records keep the preservation machinery
`ARC-SPECDB-001` exists for, and none of the rules above applies to them.

## Consequences

The first writer of the typed layer implements a rule set it can be measured against, instead
of establishing one by being first.

A submission that is refused is refused for a named reason, and the refusal is a fact about
the submission rather than about the writer's tolerance.

The store can be read without the rule set, because every row in it passed. That is the
property that makes write-time validation worth more than read-time checking, and it is lost
the moment one incomplete row is admitted for convenience.

## Controls

| Weakening | What it produces |
|---|---|
| store a failing submission in an `invalid` state | every reader needs the rule set to know which rows are real; validation moves back to the readers |
| let `draft` mean "incomplete" | validation is vacuous, because every refusal becomes a draft |
| overwrite the submitted value on clarification | the record can no longer show what was asked, which is the one thing a request preserves |
| close a gap by filling the field it blocks | the question is answered by whoever was typing, and nothing records that it was answered |
| accept a submission whose required values are all `inferred` | the system accepts its own guesses as the asker's intent, silently |
| admit a transport's defaults as `submitted` | a CLI's convenience becomes the asker's stated intent, at the seam `OD-SPEC-009` built to prevent it |
| report the first failing field only | a form with six holes takes six refusals, and the rule set is learned by exhaustion |
| treat an empty field as "none" | the honest submission and the abandoned one become the same row |
| validate in the reader rather than at write time | two readers of one rule set, which is how the two disagree |

## Status

Accepted. It states the rule set, the conditional rules, what a refusal names, the draft and
accepted boundary, the attribution shape and where an open decision is recorded. It decides
no table, no surface, and no relation constraint.
