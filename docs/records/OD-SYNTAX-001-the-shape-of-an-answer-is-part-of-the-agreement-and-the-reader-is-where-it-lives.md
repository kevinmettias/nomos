---
id: OD-SYNTAX-001
type: decision
title: The shape of an answer is part of the agreement, and one reader is where it lives
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - schema
  - providers
relations:
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
---

# The shape of an answer is part of the agreement, and one reader is where it lives

## Question

`OD-CAPABILITY-002` moved the `nomos.cap.syntax.items` contract out of the provider that
happened to write it first: an agreement is not the property of one party to it, and while
the ceiling lived in one provider, that provider could raise or lower what its peer was
permitted to claim.

The schema identifier moved with it, and the schema did not. `nomos.syntax.items.v1` was a
`SchemaId` string in `nomos-cap-syntax` and nothing else. What a payload with that stamp
actually *is* — which records, in what order, with which fields, and what makes one
malformed — was written down nowhere and implemented five times: two providers authoring
the bytes, and three consumers reading them back.

## What Five Implementations Cost

The two writers were duplicated on purpose and the reason was good. The three readers were
duplicated by accident and disagreed.

They disagreed about exactly the thing a schema exists to settle. The rule crate's reader
refused an unknown record tag, a wrong field count and a non-numeric field, each with a
line number. The slice's reader accepted any line whose first field was `unexpanded` or
`item`, ignored field counts entirely, and — the one that matters — decoded the **empty**
payload to a directory that declares nothing. A fact carrying no bytes is not a file with
nothing in it, and a rollup that folds the two together reports a file it could not read as
a file that declared nothing. That is `OD-GATE-001`'s shape inside a decoder.

Nothing in the tree compared the three, because there was nothing to compare them against.

## The Decision

**The grammar of `nomos.syntax.items.v1` is written in `nomos-cap-syntax`, together with
the one reader every consumer uses. The writers stay with their providers.**

The asymmetry is the substance of this record, so it is stated rather than left to look
like an inconsistency:

**A shared writer would prove nothing.** What makes two providers of one capability
interchangeable is that each authors the format independently and a third party can read
both. Sharing an encoder makes that true by construction, and the property the slice exists
to check evaporates while appearing to hold.

**A shared reader proves the same thing better.** A consumer writing its own decoder is not
demonstrating that the format is an interface; it is re-deciding what *well-formed* means,
and three answers to that question is how one capability comes to decode differently
depending on who is asking. The reader belongs to neither provider, so a consumer using it
is not agreeing with either writer by construction.

What the duplication was actually missing was a check, not less duplication. Each provider
now decodes its own output through the canonical reader in its own tests. That is the
agreement being verified at both ends, which is what nothing did before.

## What The Schema Declines To State

**Whether a `Function` record is a definition or a signature.**

It looks stateable. `nomos-lang-rust` writes `NotApplicable` for a trait member, because a
trait method declares no visibility of its own and recording it as private would invent a
declaration the source does not contain — so under that provider, a `Function` marked
`NotApplicable` *is* a signature.

It is not a property of the schema, because it is not a property of every conforming
answer. `nomos-lang-rust-scan` has no `NotApplicable` value at all: a line reader cannot see
the enclosing trait, and it writes `Private` for the same construct. Both payloads conform.
The presence of the mark is an observation; **its absence is not evidence of anything**, and
a consumer reading absence as "this is a definition" is reading a weaker provider's
blindness as a measurement.

Both halves are now asserted at the providers, one each, rather than described here: the
parser's test shows the mark on a trait member, and the scanner's test shows that it does
not write one. That is the divergence being measured instead of assumed.

## What This Makes Load-Bearing

`OD-RULES-001` set a guarantee floor that the approximate provider does not meet, and gave
a reason about accuracy. This record adds a second reason that is stronger, because it is
about correctness rather than quality.

`nomos-rules` excludes trait-method signatures from its index of checks by reading the mark.
That filter is **sound only because of the floor**. A caller that lowered it would not
receive a slightly weaker answer — it would receive signatures counted as checks, silently,
because the blind spelling is indistinguishable from a private definition. The floor is
what keeps the blind spelling from ever reaching the filter, and that is now written at the
filter, at the schema, and here.

Carrying the item's form in the payload is what would make the distinction statable. That is
a v2 question and `P10-SYNTAX-V2` holds it, together with the doc comments and declared type
shapes that `OD-RULES-001` names as the condition for `nomos-rules` to stop parsing Rust.

## What This Does Not Do

**No byte of any payload changed.** The encoders were not touched, so every fact already in
a store keeps its digest and nothing is invalidated. The reader was made strict where two of
the three old readers already were; the third was the lax one, and the strictness it gains
is the empty payload no longer decoding to zero items.

The kind and visibility vocabularies are still open, and deliberately. A provider that can
distinguish fewer forms writes fewer labels, and fixing the vocabulary in the contract would
make the weaker provider unable to answer honestly. Three labels are reserved — the ones
consumers branch on — and the rest belongs to whoever can observe it.

A `Render_Payload` exists beside the reader for the round trip that tests the grammar
against itself. It is documented as not for providers, and that is a note rather than a
mechanism: nothing stops a provider calling it, and the day one does, the interchangeability
the slice measures is gone without a test failing. Naming it here is the alternative to
pretending the risk is absent.

## Status

Closed by `P10-SYNTAX-SCHEMA`.
