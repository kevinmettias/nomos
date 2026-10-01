---
id: ARC-CONFORMANCE-001
type: architecture
title: A Nomos conformance claim composes system-level evidence, and native analysis is owed only where no provider exposes the fact
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - conformance
  - evidence
  - boundaries
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: ARC-ECOSYSTEM-002
    type: relates-to
---

# A Nomos conformance claim composes system-level evidence, and native analysis is owed only where no provider exposes the fact

## Question

Nomos has a check surface — `nomos check`, `nomos-rules`, `Finding`, `EnforcementReach`,
`GateCategory` — and nothing states what kind of claim it makes that an existing analyzer
does not. Without that, every capability argument is decided case by case, and the
case-by-case answer trends one way: toward restating what `rustc`, Clippy or
`rust-analyzer` already said, because that is the work whose value is obvious and whose
provider is already written. A check surface that cannot say what it is for will keep
being filled with what is easiest to justify rather than with what only it can supply.

The distinction is stateable, and this repository has already built four things that state
it without ever writing it down: `tests/contract/tests/boundaries/readme.rs` asserts the
README's band table against the real workspace, in both directions, by parsing the table
back out of the file a reader edits and comparing it to `BANDS`; `graph.rs`'s
`Test_Dependencies_Should_Run_Strictly_Downward` asserts a dependency ordering no compiler
enforces, because a band is a design decision authored in `bands.rs` rather than inferred
from `Cargo.toml`; `tests/contract/tests/requirement_trace/committed.rs`'s
`Test_Every_Assessment_Should_Name_A_Site_That_Exists` asserts that a corpus requirement's
committed site still exists in the tree it was assessed against; and `nomos spec
freshness` asserts that a rendered projection still matches the record set it was rendered
from. None of those four is a language question, and none has a provider — a compiler, a
linter, a language server — that could be asked to answer it instead.

## What A Conformance Claim Is

A Nomos conformance claim is not a diagnostic about source text. A diagnostic answers a
question language semantics alone can settle — does this parse, does this type-check, does
this borrow, is this idiomatic — and the answer is complete once the language and its
standard library are known. A conformance claim answers a different kind of question: does
*this system*, built the way its own records say it should be built, still hold the shape
and the history its own architecture, requirements and policy commit it to. That question
cannot be settled by the language, because the language has no notion of a band, a
requirement's site, a governing record, or which projection was rendered from which commit.

The four worked examples above are the evidence the distinction is real. Each composes
inputs no compiler carries:

- **Architecture** — `Test_Dependencies_Should_Run_Strictly_Downward` judges the edges the
  workspace actually has against `BANDS`, a table authored because "a band is a design
  decision and there is nothing in the source to infer it from" (`bands.rs`). The claim is
  about a dependency *ordering* this repository decided to hold itself to, not about
  whether any one edge compiles.
- **Requirements, and their history** — `Test_Every_Assessment_Should_Name_A_Site_That_Exists`
  judges a committed assessment against the corpus requirement it was written against and
  the place in the tree it named. The claim is that a specific prior judgment about a
  specific requirement still points at something real, which is a question about this
  repository's own record of itself, not about the code the site happens to be written in.
- **Runtime evidence and policy, composed together** — `readme.rs`'s band-table check is a
  runtime read of the actual workspace (the member list, its declared bands) checked
  against a policy document (the README a reader edits), in both directions, so that
  either one drifting from the other is caught regardless of which one moved.
- **A projection's own history** — `nomos spec freshness` judges a rendered file against
  the sidecar stamping what record set and what profile produced it, which is a claim about
  *this repository's own prior act of rendering*, not about the rendered text in isolation.

So the general shape: **a Nomos conformance claim is composed from architecture (what
depends on what, and what band it must sit at), requirements (what a corpus site is
supposed to satisfy, and what was already assessed about it), history (what this
repository's own prior commits, renders and assessments said), runtime evidence (what the
workspace actually is right now), and policy (what a governing record or a maintained
document commits this repository to) — checked against each other rather than against a
language's grammar or type system.** A finding lands only once that composition has run;
listing the four checks above is illustration, not the definition, because a fifth check
built the same way would be another instance of the same claim, and one built by parsing
source text for a language-level property would not be, however useful.

## The Negative Half

A record that only says what Nomos does will be read as licence to do all of it, so the
boundary is stated as plainly as the claim.

**This does not compete with a language tool on a question the language tool already
answers.** Nomos does not reparse Rust to find what `rustc` already parsed, does not
re-derive a type Roslyn or `rustc`'s own type checker already resolved, and does not chase
`rust-analyzer`'s responsiveness on a question of live editor feedback. Parsing, type
resolution and edit-time responsiveness are language-tool territory in full, and a
capability argument that amounts to "we could check this too" is not a reason to build a
second implementation of a question already answered.

**Compiler and language-server facts are evidence Nomos composes with, not facts Nomos
re-derives.** They flow in at the resolution level `FactVariant` already names —
`Syntactic`, `SemanticallyResolved`, `RuntimeObserved`, and the weaker `Approximate` and
`Predicted` — which exists precisely so that a rule needing a resolved name can refuse a
syntactic answer instead of silently accepting a weaker fact than it needs
(`crates/contracts/nomos-contracts/src/guarantee/fact_variant.rs`). A provider establishes
the fact; Nomos's part starts at composing it with the architecture, requirement, history
and policy inputs above into a claim the provider was never asked and has no vocabulary to
make.

**The test for native analysis is that no provider exposes the fact, never that writing it
natively would be convenient.** Nomos writes native analysis only where the fact a claim
needs is not obtainable from any existing provider at all — because the fact is about this
repository's own architecture, its own requirement corpus, its own record set or its own
render history, none of which any external tool has a model of. The four worked examples
qualify on exactly this test: no compiler has a notion of a declared band, no linter reads
`tests/contract/requirements`, and no language server knows what record a projection was
last rendered from. A capability whose fact *is* obtainable from `rustc`, Clippy or
`rust-analyzer` fails the test regardless of how easy it would be to write a native check
for it instead — ease of implementation is not the criterion, and deciding capability
questions on it is the drift this record exists to stop.

## Where KWB Fits

`ARC-ECOSYSTEM-001` already draws this crossing and this record cites it rather than
re-deciding it: "KWB semantic intent / rationale" projects down into a "Nomos executable
software contract" only through a "governed projection," because "a rationale is not
enforceable; a contract derived from it is." KWB supplies intent — why a design was chosen,
what a decision claimed, what somebody meant — that no static analysis, native or
provider-sourced, can derive from source text or from this workspace's own architecture,
because intent is not a property the tree holds. `ARC-ECOSYSTEM-002` gives the concrete
form of that boundary for the one KWB source this repository currently reads: the existing
C# prototype is "an evidence source, never a port target," and what survives extraction
from it lands as a governing record, a corpus artifact, or a ledger item depending on what
kind of thing was found — never as a Nomos conformance claim asserted on KWB's authority
alone.

So the boundary a conformance claim observes toward KWB is the one this repository's
`EvidenceClass` vocabulary already gives a name to elsewhere: KWB provides evidence, and
Nomos decides the engineering claim. A KWB source can tell a rule what a design intended;
it cannot make the rule true. Whether an implementation actually satisfies that intent is
settled the way every other conformance claim above is settled — by composing the intent
(once it has crossed `ARC-ECOSYSTEM-001`'s governed projection into an executable
contract) with this workspace's own architecture, requirements, history and runtime
evidence. A capability that reads a KWB source and reports its content directly as a
Nomos verdict, skipping the governed projection, is not a conformance claim; it is KWB's
evidence wearing a Nomos finding's shape.

## Conflicts With Existing Decisions

`ARC-ECOSYSTEM-001` is untouched. This record does not redraw the KWB/Nomos crossing; it
states how a conformance claim behaves on the Nomos side of a boundary that record already
governs.

`ARC-ECOSYSTEM-002` is untouched. This record does not reopen what an extraction from the
C# KWB becomes; it cites the evidence-not-authority shape that record already settled.

No existing check is reclassified as non-conforming by this record. The four worked
examples are cited as evidence the distinction already existed in what got built; none of
them changes behavior here.

## What This Record Does Not Do

It does not name a fifth check to build, and it does not change `nomos check`,
`nomos-rules`, `Finding`, `EnforcementReach` or `GateCategory`. It does not define a
mechanical gate that rejects a capability argument automatically — the test above ("no
provider exposes this fact") is a criterion for a reviewer to apply, not a type this
workspace has built a checker for. It does not open or narrow `P11-ECOSYSTEM-UPWARD`; a
KWB-derived fact's path into a Nomos claim still runs through `ARC-ECOSYSTEM-001`'s
governed projection exactly as before. It does not reduce a language tool's remit: nothing
here claims Nomos should or will replace `rustc`, Clippy or `rust-analyzer` on any question
they already answer.

## Status

Closed by `P12-CONFORMANCE-SEAM`.
