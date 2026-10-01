---
id: ARC-SPECDB-002
type: architecture
title: A canonical substrate is decided by whether the schema precedes the content
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - substrate
  - preservation
relations:
  - target: ARC-SPECDB-001
    type: relates-to
  - target: OD-SPEC-006
    type: relates-to
  - target: OD-SPEC-005
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
---

# A canonical substrate is decided by whether the schema precedes the content

## Question

This repository holds two kinds of thing and has one rule for both.

`ARC-SPECDB-001` says the specification is a database. `OD-SPEC-006` says the markdown files
under `docs/records` remain the substrate this repository authors in, and gives evidential
grounds: nothing persists a specification database, the store is assembled on every
invocation, so what survives an invocation is the file.

Neither says which objects belong on which side, because until now every object was on the
same side. That is no longer true. A typed feature-request layer is being designed whose
content is a set of answers to questions that exist before anybody answers them, and the
question of where such an object lives has been argued twice with no rule to argue from.

`P10-CANONICITY-DECISION` recorded the strongest form of the objection in its abandonment:
deciding this per artifact would put two substrates in one build and encode the split in the
schema of every typed row. That objection is correct and it is fatal to a split drawn on
**artifact kind**. It says nothing about a split drawn on a property the objects themselves
have. The difference between those two axes is what has never been written down, and this
record writes it down rather than deciding any particular object.

## The Rule

> An object is **born structured** when its schema can be stated before its content exists.
> Its canonical form is the structured record, and every document is a projection of that
> record.
>
> An object is **document-first** when its content exists before any schema can describe it.
> Its canonical form is the document, and the store holds a derived index of it.

The test is not who wrote the object, nor when, nor which directory it currently sits in. It
is whether a schema written in advance can be filled in without deciding, at fill-in time,
what the fields should have been.

That test is answerable about an object nobody has proposed yet, which is the property this
rule exists to have. A reader with a new object asks one question: can I write the form
before I know the answer? If yes, the record is canonical and the markdown is generated. If
no, the document is canonical and the rows index it.

## Why The Artifact-Kind Axis Is Refused

A rule of the form *feature requests live in the store, governing records live in files* is
refused, and refused rather than merely not chosen.

It is a list, not a rule. It answers for the objects on the list and produces nothing for the
next object, so every addition reopens the question and is settled by whoever is holding the
pen. That is the second-authority failure this repository is organised against, and it would
be installed in the schema of every typed row rather than in a document somebody could read
and disagree with.

It also cannot be checked. A rule about a property of the object can be applied by a reader
and got wrong visibly. A list can only be consulted, and a list that has fallen behind looks
exactly like a list that is complete.

The provenance framing — *born here versus imported* — is closer, and is still wrong. A
governing record is born here and is not born structured. Origin is a fact about who typed
it; what decides the substrate is whether the thing typed had a shape before it was typed.

## What Each Side Owes

The two substrates are not two standards of care. Each side pays for what the other side's
machinery gives it for free, and the exchange is stated here so that neither is later read as
the cheap option.

A **born-structured** object skips the preservation and round-trip machinery, because there
is no arbitrary document to prove recoverable — the fields were fields when they were
written. In exchange it owes four things:

- a declared schema carrying its own version, so a record written under an older shape is
  readable as that shape rather than silently reinterpreted;
- validation at write time, refusing an incomplete submission rather than accepting it and
  relying on a later reader to notice what is missing;
- separation of what was originally submitted from what was later clarified, inferred or
  decided, so that the two are never merged into one field that no longer says which it is;
- a freshness obligation on every committed projection, because the moment a generated
  document is committed it can disagree with the record it came from.

That last item is the exchange stated plainly: a born-structured object trades a
**round-trip proof** for a **freshness proof**. It is not exempt from proof. `OD-SPEC-006`
already names the same obligation for the other direction, and the gate already runs one
freshness step.

A **document-first** object keeps the machinery `ARC-SPECDB-001` exists for: every block
accounted for by a disposition, every statement traced to its source, and any content that
leaves recorded as an omission with a justification pointing at a decision. It owes that
because nothing else can establish what it contained.

## What This Rule Does Not Change

It does not amend `OD-SPEC-006`, and reading it as an amendment would be wrong.

`OD-SPEC-006` reaches the document-first outcome for governing records on evidential grounds
— that nothing persists a specification database today. This rule reaches the same outcome
for those records on structural grounds: a governing record is an argument, and the argument
is the artifact. There are no fields to fill in, because the shape of the case is not known
until the case is made. Rendering such a record from fields would lose the thing it was
written for.

The two grounds are independent and agree, which is worth saying because they have different
lifetimes. If the evidential ground ever changes — if the store becomes durable and the seed
stops compiling the files into the binary — the structural ground still holds, and governing
records stay document-first for a reason that no infrastructure change can retire.

It does not decide anything about feature requests, designs or results. Those are objects
this rule can be applied to, and applying it is a separate decision with its own record.

It does not license a second substrate on demand. Two substrates exist here because two
kinds of object exist, and a proposal for a third substrate is a claim that a third kind of
object exists, which has to be argued on this axis or not at all.

## Consequences

The next substrate question is answered by reading this record rather than by re-arguing
`OD-SPEC-006` from its evidence, which is what the last two attempts did.

An object that fails the test in one direction and is placed in the other is now a defect
somebody can name, rather than a preference somebody exercised.

The preservation machinery stops being the universal tax it currently looks like. It is what
document-first objects owe, and the work of proving that arbitrary markdown round-trips is
not owed by an object that was never arbitrary markdown.

## Status

Accepted. It states an axis and the obligations on both sides of it, and decides no
particular object.
