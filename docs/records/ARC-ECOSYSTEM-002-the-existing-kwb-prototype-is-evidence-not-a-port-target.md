---
id: ARC-ECOSYSTEM-002
type: architecture
title: The existing C# KWB is an evidence source and not a port target, and an extracted finding lands by its own kind
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ecosystem
  - kwb
  - evidence
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: D-136
    type: relates-to
---

# The existing C# KWB is an evidence source and not a port target, and an extracted finding lands by its own kind

## Question

`D-136` already decided that KnowledgeWorkbench is rebuilt from scratch in Rust rather than
ported, and named, at a domain-model grain, what survives the rewrite: the universal
epistemic kernel, content-derived claim and concept identity, the `CoverageOutcome` and
derivation-ledger primitives, and the five recorded data-loss incidents. What it does not
say is where what is extracted goes once somebody has it, or what would ever change the
call — and both questions are load-bearing the moment a person or an agent actually opens
the .NET tree and finds something worth keeping.

Absent an answer, the default behavior for a codebase that does roughly the right thing is
to port it, structure and all. That is the wrong answer here for the same reason
`ARC-ECOSYSTEM-001` already gives for any subsystem: the C# implementation's class layout
was decided by a boundary that predates `ARC-ECOSYSTEM-001`'s own boundary, so importing
that layout imports an old boundary and then argues about the new one from inside it. What
must survive is not the shape of the code; it is the findings the code paid to learn.

## Decision

The existing C# KWB (`C:/Users/kmett/source/repos/KnowledgeWorkbench`) is an evidence
source, never a port target. This is not a fresh call — `D-136` already made it, and the
new `f:/repos/kwb` repository has already recorded its own acknowledgement of it (`D-001`
there, which states plainly that where the two disagree, `D-136` is the one that was
actually deliberated). This record is the piece neither of those states: what an extracted
finding becomes, and what would reopen the question.

### What is extracted

At the grain `D-136` already uses: proven invariants, the failure cases the prototype
actually hit — its five recorded data-loss incidents, not hypothetical ones — semantic
models that survived contact with real data, replay mechanisms, project-integration
lessons, authentication and connector mistakes, and other operational experience. What is
not extracted is anything about how the C# code is organized: namespaces, class hierarchy,
dependency injection, EF Core mapping. Organization is precisely the boundary question
`ARC-ECOSYSTEM-001` answers, and the old codebase answered it before that boundary existed.

### Where an extraction lands

An extracted finding is not one kind of artifact; which kind depends on what was found, and
the rule `ARC-ECOSYSTEM-001` already states for a crossing applies here too — the direction
is what is governed, not just the destination:

- A failure mode or an invariant load-bearing for this repository's own build — the
  specification store, the ledger, or another Nomos subsystem whose current address is
  bootstrap-shaped, per `ARC-ECOSYSTEM-001`'s "Current Placement Does Not Prove Permanent
  Ownership" — lands as a governing record here, the same way `D-136` itself did.
- A domain fact about the corpus this repository ingests lands as a corpus artifact or a
  `nomos-spec-*` fixture, not a record, because it is evidence about content rather than a
  decision about architecture.
- Work implied but not yet done lands as a ledger item, exactly as any other gap does.
- A finding whose subject is KWB's own domain — rationale, semantic intent, requirements
  elicitation — is KWB knowledge, and does **not** land here at all. It crosses from this
  repository's extraction work into KWB the same way any generalization would, which is the
  crossing `P11-ECOSYSTEM-UPWARD` is about. Until that crossing is governed, a finding of
  this kind is written down where it was found rather than asserted as KWB knowledge on its
  own authority. This record does not open that crossing; it only says that this is the kind
  of thing that would use it once it exists.

### What would change the answer

Not a blanket never. The call is reopened if either becomes true:

- `f:/repos/kwb`'s own governing decision reverses course and calls for the .NET tree to be
  ported rather than read — at which point this record is wrong about the destination and
  should be corrected to match KWB's own decision, the same relationship `D-001` already
  states toward `D-136`.
- A specific piece of the .NET implementation is shown to be substantially unrecoverable
  from its behavior alone — recoverable only by reading its exact code shape rather than
  what it does — in which case porting that piece is a narrower, evidenced exception, not a
  reversal of the general rule.

## Why This Is An Ecosystem Record And Not Only A KWB One

The interesting part is not whether KWB ports its prototype — that is `D-136`'s question and
it is closed. The interesting part is what happens to something learned while extracting
from a legacy system that predates the current boundary, which is a shape this repository
will meet again: `code-standards` is this repository's own version of exactly this
relationship, and a lesson learned reading it has the same landing-spot question this record
answers for KWB's prototype. `ARC-ECOSYSTEM-001` is cited as the boundary this decision
targets rather than restated, because restating it here would be the second-authority
mistake `AGENTS.md` warns against.

## Conflicts With Existing Decisions

`D-136` is untouched. This record adds a landing-spot and a revisability clause to a
decision `D-136` already made; it does not reopen or restate the rewrite call itself.

`ARC-ECOSYSTEM-001` is untouched. This record cites it and applies it to one artifact; the
seam itself is unchanged.

`P11-ECOSYSTEM-UPWARD` is untouched and unclosed. This record explicitly declines to open
the observations-to-knowledge crossing that item governs — it only names that a KWB-domain
finding would use it once opened.

## Status

Closed by `P12-KWB-EVIDENCE`.
