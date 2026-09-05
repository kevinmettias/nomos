---
id: OD-PACKAGE-015
type: decision
title: A crate earns its boundary by independent versioning, isolation, or installation, not by naming a concept
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - package
  - layering
  - crates
relations:
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-019
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# A crate earns its boundary by independent versioning, isolation, or installation, not by naming a concept

## Question

Ten capability-contract crates (`nomos-cap-syntax`, `nomos-cap-dependency`,
`nomos-cap-controlflow`, `nomos-cap-lint`, `nomos-cap-dependency-policy`, `nomos-cap-naming-
policy`, `nomos-cap-limits-policy`, `nomos-cap-scripting-policy`, `nomos-cap-words-policy`,
`nomos-cap-goals-policy`) and six repository crates (`nomos-repo-standards-document` and its
five providers `nomos-repo-standards`, `nomos-repo-limits`, `nomos-repo-scripting`, `nomos-
repo-words`, `nomos-repo-goals`) exist as sixteen separate compilation units. A Rust module
already gives namespace and responsibility separation inside one crate; a crate should buy
something a module cannot. This record names that test and applies it, by name, to both
families.

## The test

**A crate earns its own `Cargo.toml` when at least one of three things is true of it today,
not hypothetically: it is versioned independently of the rest of this workspace, it enforces
an isolation boundary a module inside a shared crate could not, or something depends on it
without also depending on its current siblings.** None of the three is answered by what the
crate is *about* — a coherent concept, a single capability's name, a single file's contents —
because a module answers that identically. The third clause is the one this workspace can
check today: name every crate that currently depends on this one, and see whether any of them
would need to change if this crate's contents moved into a module of a sibling instead.

Concretely: **independent versioning** means the crate is published, tagged, or released on
its own cadence rather than moving in lockstep with the workspace `Cargo.toml`. **An enforced
isolation boundary** means the crate boundary is doing work a `pub`/`pub(crate)` split inside
one crate cannot — a dynamically loaded plugin, a process boundary, a compilation target that
does not build the rest of the workspace. **Independent installation** means a real consumer
depends on this crate and not on its current siblings, so merging them would force that
consumer to compile code it does not use and to rebuild on changes it does not care about.

## Applying it

**The measurement.** Grepping every `Cargo.toml` in this workspace for each of the sixteen
crates' own name finds:

- Each of the ten `nomos-cap-*` crates has its own workspace version (`0.1.0`, moving with
  every other crate here) — no independent versioning anywhere in this family.
- None of the sixteen crates enforces a real isolation boundary: all sixteen compile into the
  same binaries (`nomos-cli`, `nomos-check-orchestration`'s own test binaries), none is loaded
  as a plugin, none targets a separate process.
- The ten `nomos-cap-*` crates each have a **distinct, disjoint provider** that depends on
  that one contract alone: `nomos-cap-syntax` → `nomos-lang-go`, `nomos-lang-rust`, `nomos-
  lang-rust-scan`; `nomos-cap-dependency` → `nomos-lang-go-modules`, `nomos-lang-rust-cargo`;
  `nomos-cap-controlflow` → `nomos-lang-rust`; `nomos-cap-lint` → `nomos-lang-rust-clippy`;
  `nomos-cap-dependency-policy` → `nomos-lang-rust-deny`; and the five `nomos.cap.*.policy`
  contracts each to their one `nomos-repo-*` provider. `nomos-check-orchestration` and `nomos-
  rules` also depend on all ten, but the language and repository providers do not — `nomos-
  lang-rust` never depends on `nomos-cap-goals-policy`, and `nomos-repo-goals` never depends on
  `nomos-cap-syntax`.
- The six `nomos-repo-*` crates have exactly the opposite shape. `nomos-repo-standards-
  document` is depended on only by its five siblings — never by a consumer outside its own
  family. The five providers (`nomos-repo-standards`, `nomos-repo-limits`, `nomos-repo-
  scripting`, `nomos-repo-words`, `nomos-repo-goals`) are each depended on by exactly the same
  two crates: `nomos-check-orchestration` and `tests/integration`. No consumer anywhere in this
  workspace depends on one of the five without depending on all five.

**The capability-contract family (ten `nomos-cap-*` crates) passes on the third clause and
stays crates.** Each contract's disjoint downstream provider is a real, present-tense
consumer that would be forced to compile and relink on every change to nine contracts it does
not implement if the ten were folded into one crate's modules. This is the same shape `OD-
RULES-011` already built the family to have — one contract, one provider, composed by
identity rather than by inheritance — and the crate boundary is what lets `nomos-lang-rust`
depend on `nomos-cap-syntax` and `nomos-cap-controlflow` without also depending on `nomos-cap-
goals-policy`. Splitting the family further, or merging any two contracts, is not indicated
either: each of the ten already has exactly one provider, no more and no fewer.

**The repository-policy family (`nomos-repo-standards-document` and its five providers) fails
all three clauses and should become one crate with six modules.** No independent versioning
exists or is proposed. No isolation boundary is enforced — the five providers and their shared
read step are always linked into the same two consumers together. And the third clause is the
one measurement actively contradicts a crate boundary here: `nomos-check-orchestration` and
`tests/integration` depend on all five providers unconditionally, every time, so a merge costs
neither of them a single unnecessary rebuild — there is no case, real or hypothetical, of one
of the five being wanted without its siblings. `nomos-repo-standards-document`'s own doc
already states its role as "the shared acquisition step beneath the five" — a description of
an internal helper module, not of a crate with its own consumers. The five capability
contracts these providers implement (`nomos-cap-naming-policy` and its four siblings) are
unaffected by this verdict: they keep their own place in the capability-contract family above,
each with the one provider crate — or, after a future increment carries this verdict out, the
one provider module — that materializes it.

## Reconciling with OD-RULES-019

`OD-RULES-019` measured the same six crates this record does and reached, on its face, the
opposite packaging outcome: "a shared read/parse boundary is warranted, beneath the five
capability contracts and providers, **which stay exactly as separate as `OD-RULES-011` built
them**." Read at that sentence alone, this record looks like a silent reversal rather than an
answer to an open question.

It is not a reversal, because `OD-RULES-019` was never asked, and did not answer, the question
this record's test poses. `OD-RULES-019`'s own measurement is explicit about what problem it
was solving — "what is duplicated is mechanism, not the five capabilities' own authority" —
and its own decision extracted exactly that mechanism into a new crate, `nomos-repo-standards-
document`, by the same "share the mechanism once a real population justifies it, without
collapsing the semantics" reasoning `OD-PACKAGE-006` used first. Nowhere in that reasoning is
independent versioning, an enforced isolation boundary, or a consumer wanting one provider
without its siblings — this record's three clauses — put forward as the reason a *crate*,
rather than a *module*, was the right container for the extracted mechanism. `OD-RULES-011`'s
own stated reason for crate-per-provider is narrower still: "grouped by what kind of input a
crate reads, not by band alone" — an organizational convention borrowed from `crates/languages/`,
not a claim that packaging as separate crates buys anything a shared crate's modules would not.
Both records answer "does this capability need its own identity, its own provider, its own
registration" — five times, correctly, and this record leaves all five answers exactly as they
were built. Neither record asks "does that identity need its own `Cargo.toml`," which is the
only question this record's test poses.

**This record revises `OD-RULES-019`'s packaging choice and leaves its semantic decision
untouched.** The five `nomos.cap.*.policy` contracts, the five `ProviderId` registrations, and
the one shared read step stay exactly as separate as `OD-RULES-011` and `OD-RULES-019` built
them — as capabilities, as providers, as facts. What changes, if a future item carries this
verdict out, is only how many compilation units implement that separateness: six modules
inside one crate answer `OD-RULES-019`'s own duplication measurement identically to six crates
do, at the packaging cost this record's test was written to notice.

## What This Does Not Do

**No crate moves under this record.** It states the test and the six-crate verdict; carrying
the repository-policy family's consolidation out — choosing the surviving crate's name,
moving five crates' source into its modules, updating the dependency graph, `README.md`'s band
table and every generated projection — is a future item's own territory, scoped against this
record's finding rather than re-deriving it.

It does not reopen `OD-RULES-011`'s or `OD-RULES-019`'s decision that the five policy families
are, and remain, five separate capabilities with five separate providers and one shared read
step — the paragraph above is exactly the boundary of what this record touches and does not.

It does not examine any crate outside the sixteen this session's own why named. A future
session applying this test to a different crate does not need a second record to state the
test again.

## Status

Accepted, version 2. Ten capability-contract crates stay crates; six repository-policy crates
are found to be paying crate cost for what a module would give free, reconciled explicitly
against `OD-RULES-019`'s own packaging choice for the same six crates, with the consolidation
itself left to a future item.
