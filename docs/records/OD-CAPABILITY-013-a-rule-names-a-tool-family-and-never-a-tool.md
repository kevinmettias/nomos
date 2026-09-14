---
id: OD-CAPABILITY-013
type: decision
title: A rule names a tool family and never a tool
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - capability
  - packages
  - architecture
relations:
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: OD-PACKAGE-014
    type: relates-to
---

# A rule names a tool family and never a tool

## Question

`code-standards`' `kernel/toolspec` package (read directly from
`C:/Users/kmett/source/repos/kevinmettias/code-standards` on 2026-09-05) names three
things this workspace has two of: FAMILY, what a check asks for; TOOL, what can answer it;
SELECTION, which one a repository chose. A person required a decision on whether this
workspace adopts the same closed vocabulary above its capability contracts, what a rule may
name under it, whether delivery becomes an explicit axis, and how the answer applies to
today's real providers — not whether to build a selection mechanism, which is a separate,
sibling question.

## What Was Measured

**Nomos's own invariant is already stricter than the property `toolspec` exists to
enforce, in the dimension that matters.** `toolspec`'s own governing rule is "a check may
name a FAMILY and must never name a TOOL" — the predecessor's checks called a vendor
directly (`tsanalyzer.Exhaustive`) before this existed. No rule in `nomos-rules` has ever
named a tool, or a family: every rule asks for a capability contract — `nomos.cap.
dependency.edges`, `nomos.cap.lint.diagnostics` — a fact shape, resolved by the registry to
whichever provider is admitted. `OD-CAPABILITY-001`'s registry already does, structurally,
what `toolspec`'s rule was written to prevent by convention. Adopting FAMILY does not close
a gap in what a rule may name; nothing needs closing there.

**What is genuinely missing is a classification above the capability contract, and a
repository-level choice among competing providers — and this record is not the one that
decides the second.** A capability contract is one fact shape; a family is a kind of
answering tool. `nomos.cap.lint.diagnostics` is one shape; LINTER is a kind of tool that
could answer several different shapes across languages. Nothing in this workspace names
that kind today, and nothing lets a repository choose among competitors for one capability.
Version 1 gave a reason for that second half — "because none exist yet: every capability in
this workspace has exactly one admitted provider" — and **that reason was false when it was
written, contradicted by this record's own provider table three paragraphs below, which
lists three offers against `nomos.cap.syntax.items`** (amended at version 3; the measurement
and what it changes are under "What This Record Does Not Do"). Competitors do exist. What is
absent is a repository's way of choosing between them, which is a different statement and
the one this paragraph needed. `P54-A-REPOSITORY-CANNOT-CHOOSE-ITS-TOOLS` is the sibling
item for that half — since closed by `OD-HOST-009` — and this record answers only the
taxonomy question its own `done_when` asked for.

**Applied to the real providers, `toolspec`'s LanguagePackage/ToolProvider split by
delivery is confirmed as the axis that does not matter, on a live example rather than a
hypothetical.** Eight real providers exist today (the item's own count of seven is one
short — `nomos-lang-rust-compiler`, `P40-COMPILER-BACKED-PROVIDER`'s delivery, was built
after this item was authored):

| Provider | Capability | Family | Delivery |
|---|---|---|---|
| `nomos-lang-rust` | `nomos.cap.syntax.items` | PARSER | in-process (`syn`) |
| `nomos-lang-rust` | `nomos.cap.controlflow.reachability` | SEMANTIC_MODEL | in-process (`syn`) |
| `nomos-lang-rust-scan` | `nomos.cap.syntax.items` | PARSER | in-process |
| `nomos-lang-go` | `nomos.cap.syntax.items` | PARSER | in-process (`tree-sitter-go`) |
| `nomos-lang-rust-cargo` | `nomos.cap.dependency.edges` | PACKAGE_MANAGER | external (`cargo metadata`) |
| `nomos-lang-rust-clippy` | `nomos.cap.lint.diagnostics` | LINTER | external (`cargo clippy`) |
| `nomos-lang-rust-deny` | `nomos.cap.dependency.policy` | PACKAGE_MANAGER | external (`cargo deny`) |
| `nomos-lang-go-modules` | `nomos.cap.dependency.edges` | PACKAGE_MANAGER | in-process (reads `go.mod`/`go.work` text directly) |
| `nomos-lang-rust-compiler` | `nomos.cap.rust.copy_clones` | SEMANTIC_MODEL | in-process (`rustc_interface`/`rustc_driver`) |

The live case the item's own `why` predicted is exactly `nomos-lang-rust-cargo` against
`nomos-lang-go-modules`: identical family (PACKAGE_MANAGER), identical capability
(`nomos.cap.dependency.edges`), and *different* delivery — one shells to a subprocess, the
other reads a manifest directly. Under the current `PackageKind` split (`P47-TOOLPROVIDER-
HAS-NO-PACKAGE-2`'s own subject), only `nomos-lang-rust-cargo` is a candidate `ToolProvider`
and `nomos-lang-go-modules` is not, purely because of where it runs — the two most alike
providers in this table, split by the one axis this record finds does not matter.

**`nomos-lang-rust-deny`'s own family is a real judgment call, not a mechanical lookup, and
is worth stating as one.** `toolspec`'s own LINTER is "applies a rule set of its own" —
true of `cargo deny`'s bans/licenses/sources checks in one reading. But its subject is the
dependency graph, not source text, the identical subject `nomos-lang-rust-cargo` answers
for direction rather than policy; classified here as PACKAGE_MANAGER for that reason, a
dependency-graph auditor beside a dependency-graph reader, not a source-code rule engine
beside `clippy`.

**`nomos-lang-go-modules`'s delivery is a real gap in `toolspec`'s own two-value axis, not
an oversight in applying it.** IN_PROCESS is defined there by consequence, not mechanism:
"it cannot be absent, so a probe that reports it unavailable is describing missing DATA...
never a missing installation." Reading `go.mod` text directly shares that consequence
exactly — there is no external program to be absent, wrong-versioned, or broken — even
though it is not literally "a library linked into the caller" the way `go/types` is for the
predecessor. This record reads IN_PROCESS by its stated purpose rather than its literal
mechanism: no external tool to install is what the axis is *for*, and a direct manifest
read qualifies on that test as fully as a linked library does.

## The Decision

**This workspace adopts a closed FAMILY vocabulary, reusing `toolspec`'s twelve names
verbatim** (`PARSER`, `SEMANTIC_MODEL`, `COMPILER`, `LINTER`, `TYPE_CHECKER`, `FORMATTER`,
`LANGUAGE_SERVER`, `REFACTORING_ENGINE`, `INDEXER`, `TEST_RUNNER`,
`DOCUMENTATION_GENERATOR`, `PACKAGE_MANAGER`) rather than inventing a second taxonomy for
an identical idea: every real provider in the table above classifies cleanly under it, and
the twelve names are already field-tested against a larger, five-language population than
this workspace has today.

**A family classifies a *provider's registration*, never something a rule names.** This is
narrower than `toolspec`'s own rule, deliberately: a rule already names only a capability
contract, and a family is metadata a human (or a future selection mechanism) reads off a
provider's own declaration to answer "what kind of tool is this" — `PROVIDER`, `Declared_
Guarantee` and `FactContext`'s own siblings, not a fourth thing a rule's `Requirement`
carries. Loosening a rule to name a family instead of a capability would be a real
regression from what this workspace already has.

**Delivery becomes an explicit second axis on a provider's own registration, with
`IN_PROCESS` read by consequence rather than literal mechanism**: no external tool exists
to be absent, wrong-versioned, or broken, whether because the analysis is a linked library
(`syn`, `tree-sitter-go`, `rustc_interface`) or because it is a direct read of a manifest
format nothing external produces (`go.mod`/`go.work`). `EXTERNAL` names every subprocess-
backed provider, unchanged from `toolspec`'s own definition.

**Applied by name to today's real population**, as the table above states, correcting the
item's own count from seven providers to eight and naming `nomos-lang-rust-compiler` as the
instance built after this item was authored.

**The classification is carried by a provider declaration in the registry, never by
`ProviderOffer`** (amended at version 2, from the measurement below). `ProviderOffer` is
keyed per *capability*: `crates/substrate/nomos-capability/src/provider_offer.rs` carries
`provider`, `capability`, `version` and `guarantee`, and `Registry` holds `offers:
BTreeMap<CapabilityId, Vec<ProviderOffer>>` with no provider-keyed structure beside it.
Measured 2026-09-14: a provider identity exists nowhere in this workspace exactly once. It
exists only as a field repeated across that provider's own offers, and `Provider_Offer` is
constructed at **18 sites across 7 crates**, not the eight this record's own table states --
`nomos-lang-rust` constructs three (`src/guarantee.rs`, `src/reachability/guarantee.rs`,
`src/rollup/contract.rs`) and `nomos-repo-policy` constructs five. A `family`/`delivery`
field on `ProviderOffer` would give one provider three to five independently writable copies
of a fact this record says it has exactly one of, with nothing preventing them from
disagreeing. Version 1 suggested `nomos-capability` "alongside `Registry` and
`ProviderOffer`" as the natural home and was right about the crate and wrong about the
neighbour.

**What makes a disagreeing pair unrepresentable rather than merely discouraged**: a provider
is declared once, carrying its family and delivery; an offer names a `ProviderId` and
carries no classification field at all, so there is no second place to write one. This is
the shape `Registry` already has one key over. A `CapabilityContract` is declared once
through `Declare`, an offer references it by `CapabilityId`, and `Register_Offer`'s own
`Refuse_Unofferable` refuses an offer against an undeclared capability with
`OfferRefusal::ForUndeclared`. A declaration keyed by `ProviderId`, and an offer from an
undeclared provider refused the same way, is that existing mechanism applied to the other
key rather than a second one invented beside it. `Register_Offer` already holds the
per-`(capability, provider)` uniqueness this sits above, refusing a provider's second offer
against one capability as `OfferRefusal::Duplicate`.

**The cardinality this must hold for is the measured one**, not the table's: `nomos-lang-
rust` with three offers and `nomos-repo-policy` with five each have exactly one writable
classification, and every one of their offers resolves back to it by `ProviderId`.

This record names the carrier's *role and key*, not its type's spelling. Which type carries
a provider declaration, and whether it arrives through a new registry verb or an extension
of an existing one, is chosen by the implementation against the registration path it edits.

## What This Record Does Not Do

**No provider or package moves here.** It does not add a `Family`/`Delivery` field to any
real Rust type, and does not touch `PackageKind`, `ProviderOffer`, or any provider's own
registration. That is real code, named precisely enough for a follow-up item: a `Family`
enum, a `Delivery` enum, and the provider declaration that carries them, in
`nomos-capability` beside `Registry` — which already sits above every provider and below
every rule — plus one classification per real provider. Where that classification lives is
decided above rather than left to that item, because version 1 left it underdetermined and
the only shape version 1 gestured at is the one the measurement rules out.

It does not decide `P54-A-REPOSITORY-CANNOT-CHOOSE-ITS-TOOLS`. That item is about
`Selection` — a repository choosing among competing providers of one capability — which
this record does not build. This record's own family/delivery classification is what a
selection mechanism would need to exist first, not a substitute for building one.

**The reason version 1 gave for setting that aside was false when it was written, and the
question it set aside has since been decided elsewhere** (amended at version 3). Version 1
said `Selection` "does not exist yet because no capability has more than one admitted
provider."

**The count is wrong.** Measured 2026-09-14 against
`crates/orchestration/nomos-check-orchestration/src/composition.rs`:
`Declare_Syntax_Capability` declares `nomos.cap.syntax.items` and then offers three
providers against it, and has offered more than one since `nomos-lang-rust-scan` was
composed in. Each guarantee is read from its own declaring site rather than restated:

| provider | declared at | `FactVariant` | soundness | completeness | granularity |
|---|---|---|---|---|---|
| `nomos.lang.rust.syn` | `crates/languages/nomos-lang-rust/src/guarantee.rs` | `Syntactic` | `Sound` | `Unknown` | `File` |
| `nomos.lang.rust.scan` | `crates/languages/nomos-lang-rust-scan/src/guarantee.rs` | `Approximate` | `Unsound` | `Unknown` | `File` |
| `nomos.lang.go.tree-sitter` | `crates/languages/nomos-lang-go/src/guarantee.rs` | `Syntactic` | `Sound` | `Sound` | `File` |

`FactVariant::SemanticallyResolved` appears in the first of those files as what a future
compiler-backed Rust provider *would* offer. No such provider is composed into the
production registry, so nothing in this workspace offers this capability at that strength
today.

**The question is decided, and only the mechanism is unbuilt.**
`P54-A-REPOSITORY-CANNOT-CHOOSE-ITS-TOOLS` is closed, and `OD-HOST-009` is what closed it —
accepted, and citing this record for the twelve family names its declaration is keyed by. It
decides that a repository may declare, per language and per family, which tool answers it or
that none should run; that the declaration lives in `standards.json`; and that it travels as
a capability fact the way the five existing policy families do. What `OD-HOST-009` did not
do is build any of that. So the honest statement is that the selection mechanism is unbuilt,
not that the question is open, and this record names no trigger for it because `OD-HOST-009`
declares none.

**Why the competition that does exist has never had to be resolved by a repository**, which
is the observation the count was a poor proxy for. Two of the three offers are over the same
Rust subjects, and no caller here has ever had both to choose between:
`nomos_rules::Syntax_Requirement` states soundness `Assurance::Sound`, so
`nomos.lang.rust.scan` is refused rather than ranked —
`nomos_contracts::Guarantee::Satisfies` rejects the offer and `Registry::Resolve` reports
`Unmet::BelowRequirement` — and that requirement's own documentation calls soundness "the
axis that separates the two providers and the only one this rule cannot compromise on." The
third offer does not compete at all: it partitions by subject, which `OD-CAPABILITY-009`
decided and which the caller carries in through `Syntax_Requirement_For`'s `Preferring`
rather than the registry inferring. Where a caller will accept nobody else,
`Resolve_Requiring` refuses substitution and names who else was usable, which
`OD-CAPABILITY-001` reserved to the caller deliberately. A floor, a caller-side narrowing
and a required name are the ranking working, not cases it cannot express.

It does not resolve `P47-TOOLPROVIDER-HAS-NO-PACKAGE-2`. That item's own gap — `ToolProvider`
has no manifest crate — is a `PackageKind` question this record's family/delivery
classification does not answer by itself, though a future manifest crate for `ToolProvider`
would likely carry a provider's family the same way a `LanguagePackage` manifest carries a
language.

It does not decide whether a ninth or later provider might need a thirteenth family. The
twelve reused here were sufficient for every real provider measured; a family this
workspace's own population cannot fit is a decision for whenever such a provider is
proposed, the same "wait for a need, not a wish" discipline this workspace already applies
elsewhere.

## Status

Accepted. A closed, twelve-name FAMILY vocabulary is adopted, reused verbatim from
`toolspec` rather than reinvented; it classifies a provider's own registration, never
something a rule names, since a rule already names only a capability contract. Delivery is
a second explicit axis, `IN_PROCESS` read by consequence (no external tool to install)
rather than literal mechanism. Applied to today's real eight providers, naming
`nomos-lang-rust-cargo` and `nomos-lang-go-modules` as the live instance of one family split
wrongly across two `PackageKind`s by delivery alone. No code moves here.

Amended at version 3 on one point, in two places that stated it: version 1's claim that
every capability in this workspace has exactly one admitted provider was false when written,
and the record's own provider table contradicted it. `nomos.cap.syntax.items` has three
admitted offers, measured above. Nothing the record decided rests on the count — the
taxonomy, the cardinality and the carrier are untouched — and the deferral that cited the
count is replaced by what became of the question instead: `P54-A-REPOSITORY-CANNOT-CHOOSE-
ITS-TOOLS` is closed, `OD-HOST-009` decided it, and the mechanism is unbuilt rather than the
question open.
