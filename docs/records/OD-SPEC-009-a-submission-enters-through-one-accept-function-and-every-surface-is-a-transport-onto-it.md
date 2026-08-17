---
id: OD-SPEC-009
type: decision
title: A submission enters through one accept function, and every surface is a transport onto it
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - specification-system
  - feature-lifecycle
  - intake
relations:
  - target: OD-SPEC-008
    type: relates-to
  - target: ARC-SPECDB-002
    type: relates-to
  - target: OD-STORE-001
    type: relates-to
  - target: OD-PLATFORM-001
    type: relates-to
---

# A submission enters through one accept function, and every surface is a transport onto it

## Question

`OD-SPEC-008` made `FeatureRequest`, `DesignSpec` and `FeatureResult` born structured and left
one thing open in the same sentence: the intake surface, listed as form, CLI, API or MCP. That
list is the question as it was first asked, and answering it as asked is the mistake this record
exists to avoid.

Nothing writes a submission today. The layout question inherits the constraint that only tables
something writes to exist, so the first writer is also the first table, and whichever surface
gets built first will have decided where validation lives, what a refusal is, and what a
submission is — by existing, in code, rather than in something a reader can disagree with.

## Decision

**A submission enters the store through exactly one accept function.** It takes a typed
submission value, decides acceptance or refusal, and is the only path by which a
`FeatureRequest`, `DesignSpec` or `FeatureResult` becomes durable.

**No surface is that door.** A form, a CLI verb, an HTTP endpoint and an MCP tool are
*transports*: each constructs the typed submission from whatever it collects, hands it to the
accept function, and renders the verdict it gets back. A transport that validates, defaults or
persists on its own behalf is a second write door and is a defect, not a variant.

**The primary surface is the CLI**, in the sense that it is the first and currently the only
transport this repository will build — `nomos-cli` is the only host that exists, and it is the
only surface the gate can run. It is primary as an adapter and not as an authority. Nothing here
promises a form, an HTTP API or an MCP tool, and none of them is blocked either.

**What surfaces render is the form contract**, not each other. The questions a submission
answers are writable before any submission exists — that is exactly the test `ARC-SPECDB-002`
applies, and it is what made these objects born structured in the first place. So the questions
live in one versioned contract, and a transport is a rendering of it: a CLI verb, a set of form
fields, a tool schema. Two transports disagreeing about which questions exist is then a defect
in one rendering rather than a difference of opinion between products.

## Why Answering The List Would Be Wrong

Picking one of *form, CLI, API, MCP* as the canonical surface answers a question about
transport with a decision about authority, and the two failures it produces are already
recorded elsewhere in this repository.

**It puts the rule set in a transport.** `ARC-SPECDB-002` charges every born-structured object
with validation at write time that refuses an incomplete submission. If the CLI is the door,
that refusal is CLI code. The second transport then either re-derives the rules or calls into
the first one, and re-derived rules drift with nothing able to notice. `P10-SERVICE-SEAM` names
precisely this shape for the work verbs — a second adapter having to re-derive the claim, lease
and finish sequence rather than call it — and observes that it is cheapest to settle while there
is one adapter. Intake has zero adapters, which is cheaper still.

**It contradicts the shape the store band already uses.** `nomos-store` is described in one line
as content-addressed documents *with one write door per authority*. Intake is a write door onto
an authority. Giving it one door is applying an existing shape, and giving it one door per
surface would be the first place in this workspace where the number of write doors is decided by
the number of user interfaces.

**It fixes a list.** The four named surfaces are the ones somebody could think of while writing
`OD-SPEC-008`. A rule that names them answers for those four and produces nothing for the fifth,
which is the failure `ARC-SPECDB-002` refused when it declined an artifact-kind axis. A rule
about the relationship between a transport and the door answers for surfaces nobody has proposed.

## What Crosses The Seam

A **submission** is what a transport hands to the accept function. Three properties are fixed
here because they are properties of the seam rather than of the rule set:

- It carries what was submitted **verbatim**, separately from anything the transport supplied.
  A default a CLI filled in, a field a form pre-populated and a value an agent inferred are not
  submitted content, and `ARC-SPECDB-002` requires the two never be merged into one field that
  no longer says which it is. A transport that folds its defaults into the submitted text
  destroys that distinction before the door can see it, so the door cannot be the place it is
  first enforced.
- It names the form contract version it was constructed against, so a submission written under
  an older set of questions is readable as that shape rather than silently reinterpreted.
- It is **complete or refused, never partially stored**. A draft that has not passed validation
  is not a stored submission with missing fields; what a draft is, and where it lives, is
  `OD-SPEC-010`.

## What This Binds

Every transport, present and future, constructs a submission and calls the accept function.
There is no second path to durability for these three objects.

The form contract is a versioned artifact that transports render. Its content — which questions
are required, which are conditional and on what — is `OD-SPEC-010` and is not decided here.

The CLI transport is the one that gets built, and it is built as a transport: a verb that
collects, calls and renders. If it grows validation of its own, that is a regression against
this record and not a local convenience.

## What This Does Not Bind

**It does not decide where the accept function lives.** Which crate holds application
orchestration, and whether choosing a platform, running a verb and rendering its outcome are
separated at all, is `P10-SERVICE-SEAM`'s subject and belongs to the record that item writes.
This record says there is one door and that surfaces are transports onto it; it deliberately
says nothing about the module it sits in, because that answer must serve the work verbs too and
this is not the item that can see them.

It does not decide the validation rule set, the refusal vocabulary, or the draft state. That is
`OD-SPEC-010`.

It does not decide the physical layout, the table set, or what the accept function writes. That
is `OD-SPEC-013`, which lands the layout together with the first writer because a table nothing
writes to may not exist.

(This citation named `OD-SPEC-011` in version 1. That identifier was reserved for the layout
decision by `P10-REQUEST-LAYOUT`, and was then published for an unrelated decision — an unknown
relation type refused by name — by a second item that had reserved it independently; nothing
checked that two items had reserved one identifier. `OD-SPEC-013` is the record the layout
decision actually landed under. `OD-SPEC-013`'s own *Why This Record Is Not `OD-SPEC-011`*
section carries the fuller account.)

It does not touch `work/ledger.json`. A submission is not an item, and `OD-SPEC-008` already
said so.

It does not promise a form, an HTTP API or an MCP tool. It makes each of them a transport if it
is ever built, which is the whole of what it owes them.

## Controls

| Weakening | What it produces |
|---|---|
| the CLI verb validates before calling | the rule set lives in a transport, and the second transport re-derives it |
| each surface persists what it collects | one write door per interface; a submission means something different depending on where it entered |
| the transport merges its defaults into submitted text | `ARC-SPECDB-002`'s submitted-versus-inferred separation is destroyed upstream of every reader |
| the submission carries no contract version | a request written under older questions is reinterpreted rather than read as what it was |
| a draft is stored as an incomplete submission | validation stops being a refusal and becomes a later reader's problem, which is what born-structured was supposed to buy out of |
| the door is named as the CLI | the fifth surface reopens this decision, and the list decides it again |

## Consequences

`OD-SPEC-013` can be written against a fixed seam: one accept function, one submission type, one
form contract version travelling with it. That was the last thing the layout item was waiting on
besides the rule set.

An intake surface added later is an adapter with a known obligation, and a surface that does
anything more than adapt is a defect somebody can name.

`P10-SERVICE-SEAM` gains a second case rather than a competitor. It asks where orchestration
lives given one adapter and the work verbs; intake arrives with the seam already required and no
adapter to migrate, which is evidence for that item rather than a decision taken ahead of it.

## Status

Accepted. The rule set and the physical layout remain open as `OD-SPEC-010` and `OD-SPEC-013`,
and where the accept function lives remains open as `P10-SERVICE-SEAM`.
