---
id: OD-ANALYSIS-007
type: decision
title: Whether the first program-semantics capability is picked and built now, or waits for a rule that needs the evidence it would produce
status: closed
version: 2
authority: canonical-normative-record
tags:
  - analysis
  - capability
  - architecture
relations:
  - target: OD-ANALYSIS-004
    type: relates-to
  - target: OD-CAPABILITY-008
    type: relates-to
  - target: D-135
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-ROADMAP-002
    type: relates-to
  - target: OD-RULES-008
    type: relates-to
  - target: OD-ANALYSIS-010
    type: relates-to
  - target: OD-CAPABILITY-015
    type: relates-to
---

# Whether the first program-semantics capability is picked and built now, or waits for a rule that needs the evidence it would produce

## Question

`OD-ANALYSIS-004` settled what a program-semantics capability must say — the five existing
epistemic types (`FactVariant`, `Guarantee`, `EvidenceClass`, `Applicability`, `Observation`),
never a domain-local substitute — and what `FactVariant::SemanticallyResolved` obliges a
producer to have actually done. It deliberately does not pick which capability gets built
first: its five worked shapes (ownership crossing a closed boundary, lifetime/allocation
against a performance policy, escape past a module boundary, a synchronization ordering a
determinism policy forbids, an effect a package or authority policy restricts) are named as
"illustration of the boundary test... not a closed list," and its "What This Record Does Not
Do" section states plainly: "The first program-semantics producer is separate work, reserving
its own territory."

`crates/capabilities` holds exactly one crate, `nomos-cap-syntax`, filing every fact it
produces at `FactVariant::Syntactic`. `FactVariant`'s ordering — `Predicted`, `Approximate`,
`Syntactic`, `SemanticallyResolved`, `RuntimeObserved` — exists so a rule needing resolved
names can refuse a weaker answer, but today it orders one value against nothing: no rule in
this workspace has ever asked for `SemanticallyResolved` or `RuntimeObserved` evidence, and
neither of `nomos-rules`' two real rules (`Check_Naming_Convention`,
`Check_Completeness_Mirrors`) reads past `Syntactic`/`Approximate`. Picking one of the five
illustrative shapes and building a crate under `crates/capabilities` for it now — the first
question a session reading `OD-ANALYSIS-004` reaches for — would be exactly the shape `D-135`
already named a mistake elsewhere: inferring genericity, or in this case a whole new capability
domain, from a wish rather than a demonstrated concrete need. The open question is whether that
choice gets made now, on the strength of `OD-ANALYSIS-004`'s worked examples alone, or waits
for a real consumer to force it.

## Current Position

`OD-ANALYSIS-004` names no trigger for this choice — neither it nor `OD-ANALYSIS-002`,
`OD-ANALYSIS-003`, `ARC-CONFORMANCE-001` nor `OD-CAPABILITY-002` states a concrete condition
under which the first program-semantics capability should be selected. Nothing in this
workspace today is blocked on the absence: no rule, native or from a package, has a subject
that needs an ownership, lifetime, escape, concurrency-structure or effects claim to reach a
verdict it cannot reach otherwise. `nomos-cap-syntax` remains the only capability contract;
`OD-CAPABILITY-007`'s decline-with-reason question and `OD-CAPABILITY-008`'s provider-trait
question were both left open for the identical reason — a population of one (or, for
`OD-CAPABILITY-008`, two) provider instances is not enough to check a design against, and the
same is true here for a population of zero real consumers of the two unused `FactVariant`
levels.

Building any one of the five illustrative shapes now would fix a choice — which fact, in
which payload shape, against which of this repository's own policies — before a real rule
exists to hold that choice to account. `OD-ANALYSIS-004`'s own worked examples are explicit
that each is a *shape* of claim, not a specific one this workspace has committed to; treating
one of them as pre-selected would be reading a decided answer into a record that named the
opposite.

## What Would Decide It

A rule — native, or from a future rule package — that needs a program-semantics fact to reach
a verdict it cannot reach at `Syntactic` or `Approximate` today. That rule's own subject would
name which of `OD-ANALYSIS-004`'s five shapes (or a sixth this record does not anticipate) is
the real one to build, the same way `Check_Naming_Convention`'s arrival was what let
`OD-RULES-006` compare two rules' placement rationale on real evidence instead of one. Until
such a rule exists, any of the five shapes is equally unmotivated, and picking among them would
be a design choice with no case to check it against.

A second, independent trigger: this repository's own architecture, requirement or policy
records naming a specific claim in this family as something a conformance check must make —
for example a future band or authority-class rule that can only be enforced by knowing whether
a value's ownership crosses a closed boundary once resolved. A named requirement of that shape
would pick the first capability by naming the fact it needs, rather than by a session choosing
among `OD-ANALYSIS-004`'s illustrations for its own reasons.

## Amendment: The Trigger Has Fired Twice, By Routes Version 1 Did Not Anticipate

Version 1 was true at the revision it measured. At `8338c6ec` every sentence it rests on is
false; the item that reserved this amendment reports an external review that read the record
at `bc0aaacf` and concluded program semantics was designed but not built; and
`OD-ANALYSIS-010` had already named "rewording that record's stale "no rule needs
`SemanticallyResolved`" sentence is separate, smaller work this record does not claim". This
amendment is that work: it re-measures, says which rule and which record fired the trigger,
decides the one question the new population puts to this record, and closes it. Nothing in
version 1 is rewritten; it is quoted so the correction can be checked against it.

**What version 1 said.** "`crates/capabilities` holds exactly one crate, `nomos-cap-syntax`,
filing every fact it produces at `FactVariant::Syntactic`." "no rule in this workspace has ever
asked for `SemanticallyResolved` or `RuntimeObserved` evidence, and neither of `nomos-rules`'
two real rules (`Check_Naming_Convention`, `Check_Completeness_Mirrors`) reads past
`Syntactic`/`Approximate`." "Nothing in this workspace today is blocked on the absence: no rule,
native or from a package, has a subject that needs an ownership, lifetime, escape,
concurrency-structure or effects claim to reach a verdict it cannot reach otherwise." And its
Status section: "No rule in this workspace needs a `SemanticallyResolved` or `RuntimeObserved`
fact today, so nothing picks among `OD-ANALYSIS-004`'s five illustrative shapes yet."

**The capability population at `8338c6ec`, counted rather than recalled.** `crates/capabilities`
holds thirteen crates: `nomos-cap-architecture`, `nomos-cap-controlflow`,
`nomos-cap-dependency`, `nomos-cap-dependency-policy`, `nomos-cap-goals-policy`,
`nomos-cap-limits-policy`, `nomos-cap-lint`, `nomos-cap-naming-policy`,
`nomos-cap-requirement-trace`, `nomos-cap-scripting-policy`, `nomos-cap-syntax`,
`nomos-cap-test-material-policy` and `nomos-cap-words-policy`. Sixteen capability id strings
are declared in library code under `crates/capabilities` and `crates/languages` together: the
thirteen above, `nomos.cap.module.index` in `nomos-lang-rust`'s rollup, and
`nomos.cap.rust.copy_clones` and `nomos.cap.rust.nested_locks` in `nomos-lang-rust-compiler`
(`src/contract.rs` and `src/nested_lock/contract.rs`). A seventeenth, `nomos.cap.review.finding`,
is declared under `crates/connectors`, and `tests/integration` declares
`nomos.cap.module.surface` for its own rollup. Every other `nomos.cap.…` string in the tree
names a test or example fixture rather than a contract. `nomos-check-orchestration`'s
composition declares fourteen of the real ones — `DECLARED_CAPABILITY_COUNT`, held by
`Test_Registered_Should_Declare_Every_Composed_Capability` — and neither compiler-backed
capability is among them. `nomos-rules::DESCRIPTORS` carries seventy-one rules, held against
`Composed_Rules` by `Test_Every_Composed_Rule_Should_Have_A_Descriptor`; version 1 counted two.

**Every producer of `SemanticallyResolved` and `RuntimeObserved` outside test code.** A
producer is a contract's `Ceiling()` or a provider's `Declared_Guarantee()`. Every site under
`#[cfg(test)]` or in a crate's `tests/` directory was read and set aside — `nomos-corrections`'
three, `nomos-contracts`' three, `nomos-capability`'s two in `src`, and the over-ceiling refusal
fixtures in `nomos-lang-rust`, `nomos-lang-go`, `nomos-lang-go-modules`,
`nomos-lang-rust-clippy` and `nomos-connector-coderabbit` are all test code and produce nothing.

- Six ceilings at `SemanticallyResolved`: `nomos-cap-controlflow`, `nomos-cap-dependency`,
  `nomos-cap-dependency-policy` and `nomos-cap-lint`, each in its own `src/contract.rs`; and
  the two contracts `nomos-lang-rust-compiler` houses beside their provider.
- Six offers at `SemanticallyResolved`, from five provider crates: `nomos-lang-rust-cargo` and
  `nomos-lang-go-modules` against `nomos.cap.dependency.edges`; `nomos-lang-rust-clippy`
  against `nomos.cap.lint.diagnostics`; `nomos-lang-rust-deny` against
  `nomos.cap.dependency.policy`; and `nomos-lang-rust-compiler`'s one `ProviderId`,
  `nomos.lang.rust.compiler`, offering against both of its own contracts through `ra_ap_hir`.
- One ceiling and one offer at `RuntimeObserved`: `nomos-connector-coderabbit`'s
  `nomos.cap.review.finding`, in its `src/contract.rs` and `src/guarantee.rs`.
- `nomos-cap-controlflow`'s ceiling is not matched by its offer: `nomos-lang-rust`'s
  `reachability` provider declares `Syntactic`, which is `OD-RULES-008`'s tier 1 and the gap
  `OD-ANALYSIS-010` decides the mechanism for.

**Every reader outside test code.** A reader is a `Requirement`, and a `Requirement` is a
floor: `nomos-capability`'s own type says `minimum` "is what the caller cannot do without; an
offer below it is unusable, not merely weaker", and `Guarantee::Satisfies` compares "Every
axis, not a score." So a rule naming `SemanticallyResolved` in a `Requirement` is not
describing a ceiling it happens to sit under; it is refusing every offer beneath that level.

- `crates/rules/nomos-rules/src/checks/dependency/reading.rs`, `Dependency_Requirement`:
  `SemanticallyResolved`, sound, complete, `Project`, against `nomos.cap.dependency.edges`. Its
  own doc says "The ceiling itself" — the floor is set equal to the contract's ceiling, so only
  a provider at the ceiling is admissible. This is resolution of Cargo's package graph, which
  `OD-RULES-008` already distinguished from resolution of program code.
- `checks/lint.rs`, `Lint_Requirement`, and `checks/policy.rs`, `Policy_Requirement`: both
  `SemanticallyResolved`, against `nomos.cap.lint.diagnostics` and
  `nomos.cap.dependency.policy`, each relaying another tool's report at the level that tool
  reached.
- `checks/review.rs`, `Review_Requirement`: `RuntimeObserved`, against
  `nomos.cap.review.finding` — the only reader at the top level.
- `crates/languages/nomos-lang-rust-compiler/src/check.rs`, `Copy_Clones_Requirement`, and
  `src/nested_lock/check.rs`, `Nested_Locks_Requirement`: both `SemanticallyResolved`, sound,
  completeness unknown, `Project`, against the two compiler-backed contracts.
- The remaining `Requirement`s in `nomos-rules` — `Syntax_Requirement`,
  `Reachability_Requirement`, `Architecture_Requirement` — name neither level;
  `Reachability_Requirement` asks for exactly `Syntactic`, deliberately below its capability's
  ceiling.

Five readers at `SemanticallyResolved` and one at `RuntimeObserved`, then, against version 1's
none; and the only readers of the two compiler-backed facts live inside the provider crate
that produces them. No rule under `nomos-rules` reads either.

**Whether this record's own trigger has fired, and by what.** Version 1 named two triggers.
Both have fired, and neither arrived the way version 1 pictured.

The first — "a rule … that needs a program-semantics fact to reach a verdict it cannot reach
at `Syntactic` or `Approximate` today" — was answered by a record before any rule asked.
`OD-RULES-008` (version 2) named `Check_Unread_Reaches_A_Finding`'s sound tier as needing an
intraprocedural control-flow fact at `SemanticallyResolved`, read it as `OD-ANALYSIS-004`'s
Effects shape in the negative, and drew the distinction this amendment inherits: a rule had
already asked for `SemanticallyResolved` build-graph evidence, and none had asked for
`SemanticallyResolved` program-code evidence in `OD-ANALYSIS-004`'s sense.
`P13-CONTROLFLOW-REACHABILITY-CAPABILITY` then built the tier-1 half, and `OD-ANALYSIS-010`
decided the sound tier is a crate-local call resolver rather than a compiler integration. That
rule still asks for `Syntactic`, so the program-semantics *need* is recorded there and the
program-semantics *reader* for it is not yet built.

The first rules that actually read `SemanticallyResolved` program-code evidence are
`Check_Copy_Clones` and `Check_Nested_Locks`, and they arrived by version 1's second route
inverted. That trigger was a record of this repository's own naming a claim a check must
make. What happened instead was a Required item: `P40-COMPILER-BACKED-PROVIDER`, whose `why`
is that none of the existing providers "consults a compiler for resolved semantics" and whose
`done_when` requires that "at least one rule judges what it produces", built
`nomos-lang-rust-compiler` at `484951cb` with `nomos.cap.rust.copy_clones`, its provider and
its rule in one commit; `P42-SEMANTIC-FACT-FAMILY`, also Required, added
`nomos.cap.rust.nested_locks` the same way at `3eb32860`. The capability was picked to prove a
seam — a real compiler semantic API on stable Rust — and the rule was written because the item
required one, which is the opposite of version 1's model, in which a rule's own need names the
fact. Version 1 also warned that picking a shape now "would fix a choice — which fact, in
which payload shape, against which of this repository's own policies — before a real rule
exists to hold that choice to account". That is what happened, and it cost less than version 1
feared, because both choices were kept narrow enough to be re-derived: the crate's own doc says
it "picked the narrowest real question a compiler frontend answers that a syntax tree cannot".

**Where the two capabilities sit on `OD-ANALYSIS-004`'s map, by their own contract text.**
`nomos.cap.rust.copy_clones` answers which `.clone()` calls "duplicate a value that was already
cheap to copy" — a resolved `Copy` implementation on a resolved receiver type. That is the
ownership family: `OD-ANALYSIS-004` names "a move/borrow/copy tri-state" as that family's own
vocabulary. `nomos.cap.rust.nested_locks` answers whether "a lock guarding a value that is
already, itself, behind a lock" exists once `T` is resolved past aliases — the concurrency-
structure and synchronization family, which the crate's own doc names for it. Both are static
facts a resolved model states without executing anything, so `SemanticallyResolved` rather than
`RuntimeObserved` is the right level under `OD-ANALYSIS-004`'s own split. And both sit on the
*evidence* side of that record's boundary, not the claim side: `OD-ANALYSIS-004` says "A
provider's resolved model of ownership, borrowing, lifetime, allocation, escape, concurrency
structure, synchronization or effects is the *evidence* such a claim composes with", and every
claim shape it worked was a resolved fact composed against a policy of this repository's own —
a boundary `bands.rs` declares closed, a determinism policy under `Strategy`. Neither
compiler-backed rule composes with any such policy; each is a 1:1 relay of the provider's
verdict at `GateCategory::Advisory`. One measured fact bears on the boundary test itself and
is recorded without being decided: `cargo clippy` ships `clippy::clone_on_copy`, on by default
in `clippy::all`, and `nomos.cap.lint.diagnostics` already carries clippy's diagnostics into
this workspace, so whether `copy_clones` re-derives "a raw diagnostic a language provider
already answers exhaustively" is `OD-ANALYSIS-004`'s own test applied to a capability built to
prove a seam rather than to make a claim. Neither `P40-COMPILER-BACKED-PROVIDER` nor the
crate's docs address it, and it is not what version 1 asked. The first program-semantics
*claim* in `OD-ANALYSIS-004`'s sense is still unwritten; the first program-semantics *evidence*
is built, twice.

**Whether the family now needs a shared contract crate.** No, and the answer is measured
against the one item that reserved one. `P42-SEMANTIC-FACT-FAMILY`'s territory was
`crates/capabilities/nomos-cap-semantics/src/lib.rs`; its commit created no such crate, and
nothing by that name has ever existed under `crates/capabilities`. What the two capabilities
share is provider mechanics — `reading::Load_Crate`, sysroot discovery and per-crate file
filtering — and nothing contract-shaped: two ids, two schemas, two payload types, two
guarantees, and no `Observation`. `OD-CAPABILITY-002`'s criterion is per contended capability,
never per family — "Crates appear per *contended* capability, which is a much smaller number
than per capability" — and a family crate would be a home for exactly the shared vocabulary
`OD-ANALYSIS-004` forbids a program-semantics capability from introducing. So each contract
stays beside its provider, as both contract files already say, until a second party names it.

Which second party will come first is worth stating, because it is not the one
`OD-CAPABILITY-002` names. That record's trigger is a second provider. This workspace's zone
rule supplies an earlier one: `nomos-lang-rust-compiler` is zoned `Provider` in
`nomos-architecture.json`, `Permits` forbids `Rules` from naming `Provider`
(`OD-CAPABILITY-015` quotes the edge), and `nomos-rules` reaches no `nomos-lang-*` crate outside
a dev-dependency. `check.rs` plans "Wiring this rule into `nomos-check-orchestration::Run` and
`nomos-rules::DESCRIPTORS`", and the moment a descriptor in `nomos-rules` names either
capability, a second party is naming the contract from a zone that cannot see the file it
lives in. At that moment the contract moves to its own crate under `crates/capabilities`, the
shape the five policy contracts already have — one crate per capability, not one for the
family — and `Test_A_Capability_Id_Should_Be_Written_In_One_Crate` is what holds the id to one
home before and after. Until then it does not move. Whether a crate that declares a contract
and bundles its only provider belongs in `Provider` or in `Capability Contract` is
`OD-CAPABILITY-015`'s criterion, not this record's; that the compiler crate is zoned `Provider`
while `nomos-connector-coderabbit`, the same shape, is zoned `Capability Contract` is recorded
here as measured and routed there.

**Wiring either capability into `Run` is not this record's to schedule.** Neither is composed:
`composition.rs` names `nomos_lang_rust_compiler` nowhere, `RequiredFact` has no variant for
either fact, `README.md`'s row for the crate says "Not yet composed into a real gate run", and
the only crate depending on it is `tests/integration`, for its determinism declarations. That
wiring is exactly the "additional one-off capability orchestration" `OD-ROADMAP-002` pauses:
"A capability may still be built; what waits is wiring it in by extending the hand-written
mapping", lifted when the run planner it names lands, or lapsing if that successor is abandoned.
Where that stands is also not this record's. `P41-RUN-PLANNER` is declined under
`OD-RULES-009`, whose latest amendment finds a planner speculative rather than merely unbuilt
now that materialization derives its demand "from the union of
`nomos_rules::RuleDescriptor::requires` over the selected rules", and `OD-RULES-027` finds that
"The derivation is available, and it is not the planner." Whether that derivation is the
successor `OD-ROADMAP-002` waits for, or the pause has lapsed with the planner's decline, is a
question for those records. This record says only that the wiring waits on them, and schedules
nothing.

## Status

Closed at version 2. The question version 1 held open — whether the first program-semantics
capability is picked now or waits for a rule that needs it — was answered by events rather than
by this record: `OD-RULES-008` named the first rule that needs a `SemanticallyResolved`
program-semantics fact, and `P40-COMPILER-BACKED-PROVIDER` and `P42-SEMANTIC-FACT-FAMILY`, both
Required, built the first two capabilities that produce one, each with the rule that reads it
in the same commit. At `8338c6ec` five `Requirement`s outside test code ask for
`SemanticallyResolved` and one asks for `RuntimeObserved`; six contracts and six offers stand at
the former and one of each at the latter. Neither compiler-backed capability needs a shared
family crate: each stays beside its provider under `OD-CAPABILITY-002` until a second party
names it, and the party that will is the first composed rule in `nomos-rules`, at which point
that contract gets a crate of its own under `crates/capabilities`. Composing either into `Run`
waits on `OD-ROADMAP-002`'s capability-orchestration pause and on the records that decide
whether it has lifted. What version 1 was guarding — that a shape not be fixed before a rule
holds it to account — has moved rather than lapsed: the shapes are fixed and re-derivable, and
the first program-semantics *claim* composed against a policy of this repository's own is still
unwritten, which is `OD-ANALYSIS-004`'s to measure when a rule states one. No revisit condition
remains here.
