---
id: OD-RULES-003
type: decision
title: A declared architecture is data, the observed dependency graph is a fact a capability establishes, and a rule compares the two
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
  - capability
  - conformance
relations:
  - target: ARC-CONFORMANCE-001
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
---

# A declared architecture is data, the observed dependency graph is a fact a capability establishes, and a rule compares the two

## Question

`P12-DRIFT-CAPABILITY` opened against a measurement about this repository's own strongest
check. `tests/contract/tests/boundaries/bands.rs` authors `BANDS`, a table of every workspace
member's band, with a doc comment giving the reason by hand: "a band is a design decision and
there is nothing in the source to infer it from." `tests/contract/tests/boundaries/graph.rs`
loads the real workspace through `Workspace::Load()` and asserts, among other things, that
`Test_Dependencies_Should_Run_Strictly_Downward` — every edge in the observed graph runs to a
strictly lower band than its source — and that `Test_Every_Member_Should_Declare_A_Band` — no
crate joins the workspace outside the table.

That is real architecture enforcement, and the class of finding it produces — a forbidden
dependency direction, a same-band edge between two providers of one capability, a crate
outside the declared ordering — is unavailable to any compiler or linter, because the
declaration the finding is checked against is not written anywhere the language can read. But
the check itself has no contract, no provider, no `Applicability`, no `Guarantee`. It knows
this workspace's crate names by literal string, and `Workspace::Load()` reads `Cargo.toml`
files directly from a hard-coded root. A second repository wanting the same property has
nothing to point at; it would have to copy `bands.rs` and `graph.rs` and edit the table by
hand for its own crates.

The question this record answers is the seam `P12-DRIFT-CAPABILITY`'s own text names: where a
declared architecture is stated, how the observed one is established as facts, and which of
those two Nomos owns.

## The Decision

### A declared architecture is expressed as data, not as a bespoke Rust test

`bands.rs` already says what a declared architecture *is*, in its own doc comment, without
meaning to state it generally: a design decision, authored because there is nothing in the
source to derive it from. Stripped of being a Rust `const` table compiled into one crate's
test binary, what `BANDS` holds is three things — a finite set of named components, a partial
order over them expressed as a rank per component (the "scaled by ten" comment exists so the
order can be edited without renumbering everything), and a finite set of named exceptions to
the strict-downward rule that the order alone cannot express: `CONTRACTS_ALLOWLIST` (one
component may reach a fixed external set and nothing else), `PLATFORM_ADAPTER` (exactly one
named component may cross a boundary no other may), and the same-band prohibition
`graph.rs`'s comments give a reason for twice — two providers of one capability must not be
able to name each other, because a band forbids edges between its own members and that is
what stops the second answer from being derivable from the first.

None of that content is Rust-specific, and none of it needs to be discovered from source. A
declared architecture, generally, is that triple — components, an order over them, and named
exceptions to it — held as data a governing record or a maintained declaration states, the
way this repository already holds other declarations (a capability's contract, a corpus
requirement's site) as data rather than as logic embedded in the test that checks them. The
seam is exactly the one `OD-CAPABILITY-002` drew for a capability's contract: an agreement
does not belong to the party enforcing it, and while a declared architecture lives only inside
the Rust test that reads it, no second party — a second repository, a second language, a
second workspace — can hold the same agreement without a second copy of the file.

### The observed dependency graph is a fact a capability establishes

`OD-RULES-001` already settled the general principle this claim is one instance of: **a rule
states what it needs and is refused an answer beneath it**, and does not open the file itself.
`graph.rs` today violates exactly that shape — `Workspace::Load()` parses `Cargo.toml` files
directly inside the assertion that judges them, the same posture `OD-RULES-001` found and
corrected for `nomos-rules` reading source text straight instead of through a `FactReader`.

The observed dependency graph — which packages exist, and which package each one names as a
direct dependency — is not a property a rule is entitled to establish by opening manifests
itself. It is a fact a capability provider establishes, at a stated `Guarantee`, and a rule
consumes it through a `FactReader` the way `Check_Completeness_Mirrors` consumes
`nomos.cap.syntax.items`. The right resolution level is `FactVariant::SemanticallyResolved`,
not `Syntactic`: a package's real dependency edges are the *resolved* set Cargo's own manifest
and lockfile resolution produces — optional dependencies, target-`cfg`'d dependencies and
dev-dependencies are not the same edge, and a provider that only scanned manifest text for
`dependencies = {...}` blocks could not tell them apart, which is exactly the gap
`OD-RULES-001` refused `nomos-lang-rust-scan` a semantic floor over for the analogous reason.
A provider establishing this fact reads Cargo manifests and build metadata — what
`Workspace::Load()` already does today — but does so behind a capability boundary a rule
requests rather than as code embedded inside the assertion.

### Where the comparison lives

The comparison is a Nomos rule, not a capability and not a fact by itself. It composes two
inputs of different kinds — the declared architecture (policy: data this repository or a peer
commits itself to) and the observed dependency graph (a fact a capability provider
establishes) — into findings, exactly the general shape `ARC-CONFORMANCE-001` already gives a
name: a conformance claim, composed from architecture, requirements, history, runtime evidence
and policy, checked against each other rather than against a language's grammar or type
system. `ARC-CONFORMANCE-001` names this exact pair, `bands.rs` and `graph.rs`, as its first
worked example of the shape. This record does not introduce a new kind of claim; it states
what the two inputs to that already-named claim are, and that one of them must arrive as a
fact rather than as a direct read.

### What happens when a repository declares no architecture

The answer must not be that everything passes, and the vocabulary for saying so already
exists and is used exactly for this. `Applicability::NotApplicable` — "The rule does not bind
this subject. This is the only variant that is a positive statement about the absence of a
judgment" — is the value reported. A repository holding no declared-architecture data gives
the rule nothing to compare the observed graph against; the rule does not bind that
repository, and it says so positively rather than reporting `Supported` over a comparison it
never ran. `Applicability::Display_Label` keeps `NotApplicable` distinct from
`DisplayLabel::Native` at every consumer, so this can never render identically to a clean
pass.

It is deliberately not `MissingCapability`: that variant means no installed provider offers a
capability the rule requires, and the capability establishing the observed graph can be fully
present and correct while no architecture has been declared — the gap is in the policy input,
not the fact input. It is deliberately not `ConfigurationDisabled` either: that variant is a
deliberate human choice to switch a rule off for a subject that otherwise has one, and a
repository that has never declared an architecture has not switched anything off. `NotApplicable`
is the one variant built for a subject the rule was never going to bind in the first place,
which is what an undeclared architecture is.

### What becomes of `boundaries.rs`

It stays a bespoke guard. `P12-DRIFT-CAPABILITY`'s own `done_when` text is explicit that this
item does not widen or change `boundaries/bands.rs` or `boundaries/graph.rs` — `P10-SERVICE-SEAM`
holds that file — and this record does not reach past that boundary. Beyond the territory
line, the reason is substantive and not only procedural: migrating `bands.rs` and `graph.rs`
into instances of the general mechanism this record describes needs three things that do not
exist yet — a capability that establishes the observed dependency graph as a fact at a stated
`Guarantee`, a place to author a declared architecture as data rather than as a Rust `const`
table, and a rule that consumes both through the seam `OD-RULES-001` already established for
every other judgment in this tree. None of those three is built by this record. Stating the
seam and building the mechanism are different items of work, and building it without first
stating where the two halves belong is the "widen `boundaries.rs` into a configurable table"
move `P12-DRIFT-CAPABILITY`'s own `why` text names as the obvious wrong move — the general
mechanism must not be this repository's own guard wearing a parameter.

When the mechanism exists, this repository's own `BANDS`, `CONTRACTS_ALLOWLIST` and
`PLATFORM_ADAPTER` content is the natural first declaration to migrate into it — it is
already exactly the triple the general form needs, authored as data in a table for the reason
`bands.rs` already gives — and `graph.rs`'s five assertions become this repository's own
instance of the general rule, run over its own declaration, rather than a bespoke one. That
migration is not committed to here, has no owner and no schedule; it is left for whichever
future item builds the mechanism and finds this repository the first and most convenient
place to point it at.

## Why This Is A Nomos Claim

`ARC-CONFORMANCE-001` draws the line this record's claim sits on the Nomos side of: a
conformance claim answers whether *this system*, built the way its own records say it should
be, still holds the shape its own architecture commits it to — a question no compiler,
linter or language server has any notion of, because none of them models a declared band, a
same-band prohibition, or a named exception to a strict-downward rule. `bands.rs` and
`graph.rs` are that record's own first worked example, cited by name: "`graph.rs`'s
`Test_Dependencies_Should_Run_Strictly_Downward` asserts a dependency ordering no compiler
enforces, because a band is a design decision authored in `bands.rs` rather than inferred from
`Cargo.toml`." The test `ARC-CONFORMANCE-001` states for native analysis — that no provider
exposes the fact, never that writing it natively would be convenient — is satisfied here for
the same reason it already gave: no compiler has a notion of a declared band, so the
comparison this record describes is Nomos's to make and not a provider's to be asked for
instead. This record does not reopen that test or that worked example; it takes the general
claim `ARC-CONFORMANCE-001` already names and states what its two composed inputs are for this
one instance of it.

## What This Record Does Not Do

It does not touch `tests/contract/tests/boundaries/bands.rs` or
`tests/contract/tests/boundaries/graph.rs`, and it does not widen either file into a
configurable table. `P10-SERVICE-SEAM` holds that territory.

It does not build the capability that establishes the observed dependency graph as a fact, the
data format a declared architecture would be authored in, or the rule that composes the two.
It states where each belongs; building any of the three is separate work this record does not
schedule.

It does not commit this repository's own `BANDS` table to a migration date, or require one at
all. It states only what that table already is, and what it could become once the general
mechanism exists.

It does not reopen `ARC-CONFORMANCE-001`, redraw its four worked examples, or change what a
conformance claim means generally. It applies that record's already-general shape to the one
claim `P12-DRIFT-CAPABILITY` asked about.

It does not change `Applicability`, `Guarantee` or `FactVariant`. `NotApplicable` and
`SemanticallyResolved` are used as they already exist.

## Status

Accepted, landed by `P12-DRIFT-CAPABILITY`.
