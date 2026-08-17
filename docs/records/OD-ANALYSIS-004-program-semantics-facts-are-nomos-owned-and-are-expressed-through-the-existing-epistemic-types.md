---
id: OD-ANALYSIS-004
type: decision
title: Program-semantics facts are Nomos-owned and are expressed through the existing epistemic types
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - analysis
  - capability
  - contracts
  - boundaries
relations:
  - target: ARC-CONFORMANCE-001
    type: relates-to
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-ANALYSIS-002
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
---

# Program-semantics facts are Nomos-owned and are expressed through the existing epistemic types

## Question

`FactVariant` (`crates/contracts/nomos-contracts/src/guarantee/fact_variant.rs`) orders five
resolution levels — `Predicted`, `Approximate`, `Syntactic`, `SemanticallyResolved`,
`RuntimeObserved` — weakest first, so that a rule needing resolved names can refuse a
syntactic answer instead of silently accepting a weaker fact than it needs. Two of the five,
`SemanticallyResolved` and `RuntimeObserved`, have no producer anywhere in this workspace.
`crates/capabilities` holds exactly one crate, `nomos-cap-syntax`, and `OD-ANALYSIS-002`
already recorded why: the one capability this build ships, `nomos.cap.syntax.items`, is a
per-file leaf by construction, and every fact filed under it is `Syntactic`. The ordering
that is supposed to let a rule demand resolved names is currently an ordering over one value.

Program semantics is named as the expansion that will exercise the other two levels:
ownership, borrowing, lifetime, allocation, escape, async and concurrency structure,
synchronization, and effects. Every one of those is a fact about a program that no syntactic
reading establishes, and each is the kind of claim Nomos exists to make rather than a
language server's private business to hold.

Two things have to be settled before the first such capability is written, because both are
cheap to get right now and expensive to unwind once a capability crate exists with its own
vocabulary already shipped in it:

1. What vocabulary a program-semantics fact is stated in — whether it reuses the epistemic
   types this workspace already has, or a new capability is free to arrive with its own.
2. What `FactVariant::SemanticallyResolved` obliges a producer to have actually done, since
   today the level is defined by nothing but a doc comment and no producer has ever had to
   satisfy it.

## Program-Semantics Facts Are Nomos-Owned

A program-semantics fact — that a value is moved rather than borrowed at some point, that a
reference cannot outlive the frame that produced it, that a `Future` holds a lock across a
suspension point — is a fact about a program's own behavior that this repository's own rules
may need to hold a subject to, exactly as they already hold a subject to a fact about its
syntax, its declared band, or its requirement history. `ARC-CONFORMANCE-001` states the
general shape once: **a Nomos conformance claim composes architecture, requirements, history,
runtime evidence and policy, checked against each other, and native analysis is owed only
where no provider exposes the fact.** Program semantics does not sit outside that shape; it
is one more evidence domain a claim may compose with the others, filed the same way syntax
facts already are.

So the answer to the first question is: **yes, Nomos owns the claim, and no, that ownership
does not license a new vocabulary.** A program-semantics capability's payload — its own
per-item record shape, the way `nomos-cap-syntax`'s `SyntaxPayload` is a per-item record shape
— is free to be domain-local, exactly as `SyntaxPayload` is. What is not free to be
domain-local is the five questions every capability in this workspace already answers about
any fact it produces, because those five questions already have one vocabulary and a second
one answering the same questions is a second authority for something already governed:

- **What was established, and how strongly ordered against every other resolution level** —
  `FactVariant`. A program-semantics fact states its level exactly as a syntax fact does, and
  `Guarantee::Satisfies` compares it against a requirement's the same way, on the same axis.
- **What kind of sameness, across what environment, compared how** — the `Guarantee` triple
  (`variant`, `soundness`, `completeness`, `incremental`) plus `DeterminismStrength`,
  `ReproducibilityScope` and `TraceEquivalence` for the reproducibility question, which
  `Guarantee`'s own module doc already states is orthogonal to resolution level and answered
  by a different type for exactly that reason.
- **How the claim was come by** — `EvidenceClass`. A program-semantics fact produced by
  consuming a compiler's resolved model is `Derived` or `Verified` depending on whether the
  method would have caught a wrong answer; one produced by an agent's reading of the code
  is `AgentJudged`, and `EvidenceClass::Is_Mechanical` already refuses to let that report as a
  machine having checked something. No new confidence enum states this more precisely than
  the existing ordering already does.
- **Whether a rule reached a judgment about this subject, and if not, why not** —
  `Applicability`. A program-semantics rule that cannot get an ownership fact for a subject
  reports `MissingCapability` or `ProviderUnavailable` exactly as a syntax rule does; it does
  not invent its own "partial coverage" notion, because `Applicability::PartiallySupported`
  and `Applicability::Is_Coverage_Debt` already state what partial and missing mean, and a
  second notion of partial coverage is precisely the drift the item that reserved this record
  was written to catch before the fact.
- **What a provider saw when it looked, including that it could not look** — the
  `Observation` three-state (`NotObserved` / `Absent` / `Present`, currently
  `nomos-cap-syntax`'s `payload::observation` and generalized by `OD-SYNTAX-002` as "not
  observed is not absent"). A program-semantics payload that records, per item, whether an
  escape or a lock-acquisition was seen uses the same three-state shape rather than an
  `Option`, for the reason `OD-SYNTAX-002` already gives: the two empty answers are not the
  same answer, and one of them is a silent downgrade from admitted gap to phantom coverage.
  `Observation` itself may be promoted out of `nomos-cap-syntax` the day a second capability
  needs the identical shape — `OD-CONTRACTS-001`'s band-0 admission criterion is what that
  promotion is judged against, the same test `SyntaxPayload` was judged against and did not
  meet, because today only one provider needs it.

`OD-CONTRACTS-001` is why the negative statement matters as much as the positive one. Its
criterion for band 0 is that a type is admitted only when it crosses a subsystem, process or
plugin boundary and the parties on both sides must agree about it in the same words; a
domain-local concept stays in the domain that owns it. Ownership analysis, borrow-checking and
async-structure analysis each carry vocabulary of their own — a move/borrow/copy tri-state, a
lifetime-region model, a happens-before ordering — and every one of those is the natural thing
for a first implementer to reach for, because it is the vocabulary the underlying analysis
already speaks. None of it answers what was established, how strongly, by what, or over what;
it answers what the analysis found, which is domain content and belongs in the capability's own
payload the way `SyntaxPayload`'s `kind`, `shape` and `documentation` vocabulary belongs in
`nomos-cap-syntax`. A capability that let its domain vocabulary migrate into the five questions
above — a `MoveConfidence` enum standing in for `EvidenceClass`, a `PartialOwnershipCoverage`
type standing in for `Applicability::PartiallySupported` — would be exactly the second
authority `OD-CONTRACTS-001` and `ARC-CONFORMANCE-001` both already forbid, reached for the
reasons `OD-CONTRACTS-001`'s Question section names: the domain vocabulary reads like it
belongs in the fact type, and the module boundary that says otherwise is easy to not read.

## What The Family Covers, And Where It Ends

The family is the eight questions the item that reserved this record named: ownership,
borrowing, lifetime, allocation, escape, async and concurrency structure, synchronization, and
effects. `ARC-CONFORMANCE-001`'s boundary test governs every one of them the same way it
governs the four worked examples it already cites: **native analysis is owed only where no
provider exposes the fact, never because writing it natively would be convenient.** Applied to
this family specifically, the test separates two kinds of question that use the same words:

- **A raw diagnostic a language provider already answers exhaustively is never re-derived.**
  Whether code parses, whether it type-checks, whether it borrow-checks, what a reference's
  inferred lifetime is, what a `Future`'s inferred `Send`/`Sync` bound is — `rustc` and
  `rust-analyzer` already answer every one of these completely, and a program-semantics
  capability that reimplemented borrow-checking or lifetime inference to restate one of these
  answers would fail the test regardless of how easy it would be to write, exactly as
  `ARC-CONFORMANCE-001` already forbids for parsing and type resolution.
- **A claim this repository's own architecture, requirements, history or policy makes about a
  program-semantics fact, that no provider states because no provider has a model of this
  repository's own commitments, is what the family is for.** A provider's resolved model of
  ownership, borrowing, lifetime, allocation, escape, concurrency structure, synchronization
  or effects is the *evidence* such a claim composes with — brought in at `SemanticallyResolved`
  or `RuntimeObserved` exactly as `ARC-CONFORMANCE-001` already states compiler and
  language-server facts flow in at the level `FactVariant` names — never the claim itself.

Concretely, and only illustratively — none of the examples below is a capability this record
authorizes building, and each is a *shape* of claim rather than a specific one this workspace
has committed to:

- **Ownership / borrowing** — not "does this value's ownership satisfy Rust's rules" (settled
  completely by `rustc` compiling or refusing to); a claim of this family's shape is that a
  value's ownership, once resolved, crosses a boundary `bands.rs` declares closed, which is an
  architecture fact no compiler has a model of.
- **Lifetime / allocation** — not "what is this reference's inferred lifetime" or "does this
  allocate" as raw facts a profiler or the compiler already state; a claim of this family's
  shape is that a capability's own declared `IncrementalGranularity` or a workspace performance
  policy is actually honored given the lifetimes or allocations a resolved model reports.
- **Escape** — not "does this reference outlive its frame" as rustc's borrow checker already
  settles; a claim of this family's shape is that a value provably escapes past a module
  boundary this repository's own package or band table declares closed.
- **Async / concurrency structure, synchronization** — not "is this future `Send`", which
  `rustc` answers completely; a claim of this family's shape is that a task structure holds a
  synchronization primitive across a suspension point in an ordering this repository's own
  determinism or reproducibility policy forbids for a claim made under `Strategy`.
- **Effects** — not a raw "does this function perform I/O", which an effect-tracking tool could
  state on its own account; a claim of this family's shape is that an effect occurs from code
  this repository's own `AuthorityClass` or `PackageKind` policy restricts it from performing.

Static facts in this family — the ones a resolved compiler model states without executing
anything: ownership, borrowing, lifetime, allocation-as-decided, escape-as-provable — are the
shape of fact `SemanticallyResolved` exists for. Facts that can only be established by running
the program and watching what it actually does — actual concurrency interleaving, actual
synchronization order, actual effects performed — are the shape `RuntimeObserved` exists for.
That split is why this family, and not some other one, is what tests whether the two unused
levels can be produced through the existing seam at all: it has a producer-shaped candidate for
each.

## What `FactVariant::SemanticallyResolved` Obliges

The variant's doc comment says a fact at this level is "established with resolved names, types
and references." That sentence has never had to hold against a producer, so this record fixes
its meaning before one exists to fix it by example instead.

**A producer claiming `SemanticallyResolved` must have resolved every name occurrence in its
subject to the declaration it actually binds — including across module and crate boundaries —
and assigned every typed expression its checked or inferred type, not the syntactic annotation
where the two differ (a generic parameter, an elided lifetime, a type inferred from usage).**
Concretely, that means consuming a resolved semantic model — the compiler's own resolved IR, or
an equivalent tool's semantic model built the same way — rather than a producer's own
approximation of name or type resolution built by pattern-matching syntax. A producer that
guesses a name's target from lexical scoping rules it re-implements, or that reports a type
annotation verbatim without resolving generics or inference, has not met the level regardless
of how often the guess is right; `Syntactic` is exactly the level for a fact read from the text
or its parse tree with no name resolution, and a producer that stops at a good guess belongs
there; overclaiming `SemanticallyResolved` for it is what
`Guarantee::Satisfies`("the comparison that decides so is this ordering") exists to make a rule
refuse.

**It does not oblige borrow-checking, lifetime inference, or any runtime observation.** Those
are either a further static fact a `SemanticallyResolved` producer may or may not also resolve
— stated separately, per the family grain above, not folded into what the level means generally
— or they are `RuntimeObserved`'s territory entirely. A producer that resolves names and types
but does not attempt borrow analysis has still met `SemanticallyResolved`; it has simply not
produced every fact the family could eventually offer.

**It obliges the same honesty about partial resolution that `Observation` and `Applicability`
already state elsewhere.** A producer that resolved most but not all of a subject's names —
because a macro expansion was opaque to it, or a cross-crate reference could not be followed —
does not report the resolved names as though the subject were fully resolved. It reports
`Applicability::PartiallySupported` for the subject and states, in its own payload, what was
not covered — the same shape `Applicability::PartiallySupported`'s own doc comment already
requires ("what was not covered is recorded separately and is not implied to be clean") — and
does not fall back to `Syntactic` for the whole subject either, since some of the subject
genuinely was resolved. A producer that cannot tell whether it resolved a given name — no
method available to check its own answer — records that item's field as `Observation::Absent`
or `NotObserved` per `OD-SYNTAX-002`'s distinction, never as a resolved answer it is not sure
of.

## What This Record Does Not Do

It does not build a capability, a payload schema, or a crate under `crates/capabilities`. The
first program-semantics producer is separate work, reserving its own territory, and this record
is what that work is measured against — the same relationship `ARC-CONFORMANCE-001` holds to
every capability argument made under it.

It does not add a sixth `FactVariant` level, reorder the five that exist, or change
`Guarantee`, `Applicability`, `EvidenceClass` or `Observation` in any way. Every type this
record names is unchanged by it.

It does not decide which crate a program-semantics capability lives in, or whether it is one
crate or several. `OD-CAPABILITY-002`'s criterion — a capability contract is not a provider's
property, and lives beside its provider only while contention has not yet named a second one —
governs that question exactly as it already governs `nomos-cap-syntax` and
`nomos.cap.module.index`, unchanged by anything here.

It does not enumerate every claim this family may eventually make. The five worked shapes above
are illustration of the boundary test, not a closed list, exactly as `ARC-CONFORMANCE-001`'s own
four worked examples are illustration and not the definition of a conformance claim.

It does not promote `Observation` out of `nomos-cap-syntax`. That crate is still the only
provider that needs it, `OD-CONTRACTS-001`'s admission criterion is still not met by one
provider, and this record only states that a second provider needing the identical shape is
what would meet it — the same deferred promotion `OD-ANALYSIS-002` already recorded for
`nomos.cap.module.index`'s contract.

## Status

Closed by `P12-PROGRAM-SEMANTICS`.
