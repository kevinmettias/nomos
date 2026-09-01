---
id: OD-RULES-011
type: decision
title: A rule's parameters are a fact a repository declares, not a constant the rule compiles with
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - capability
  - naming
  - configuration
relations:
  - target: OD-RULES-001
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
---

# A rule's parameters are a fact a repository declares, not a constant the rule compiles with

## Question

Every rule `crates/rules/nomos-rules` ships judges its subject against a value baked into
its own source: a casing predicate (`Is_Pascal_Snake_Case`, `Is_Upper_Snake_Case`,
`Is_Lowercase_First_Letter_Convention`), a line-count threshold (`REVIEW_TRIGGER_LINES`,
`GO_HARD_TRIGGER_LINES`), a suffix or a symbol-kind filter. A repository that wants a
different convention, a different threshold, or the same rule scoped to a different
symbol kind has no lever to pull short of forking this crate. Whether that is acceptable —
whether "which case a name takes" is properly this crate's own decision, compiled once for
every consumer — or whether it is a fact each repository declares and this crate resolves,
is undecided, and every naming/threshold rule built so far assumed the first answer without
the question ever being asked.

## What Was Measured

This workspace's own `standards.json` already declares a `naming` block —

```json
"naming": { "function": "upper-snake", "method": "upper-snake" },
"languages": { "rust": { "naming": { "function": "upper-snake", "method": "upper-snake" } } }
```

— schema-validated by this same file's own `require` section (`standards.json`'s
`data_contracts` entry for `standards.json` itself lists `naming` as a required `object`
field). It has held this declaration since before any of `crates/rules/nomos-rules`'
casing rules existed. Nothing in `nomos-rules`, `nomos-check-orchestration`, or any
capability provider in this workspace reads it: `grep -r "naming" crates/rules` finds only
the crate's own hardcoded predicates, never a reader of this file. The repository already
states its own convention as data and every rule that judges that convention ignores the
statement and recompiles a guess instead.

The guess is not even self-consistent. `standards.json`'s `"upper-snake"` is a real,
externally defined vocabulary word — `C:\Users\kmett\source\repos\kevinmettias\code-
standards\rules\general\style\shared\naming\case.go` names eight closed case styles
(`upper-camel`, `lower-camel`, `underscore-camel`, `lower-snake`, `upper-snake`,
`screaming-snake`, `mixed-snake`, `lower-kebab`) and `case_validation.go` gives each one an
exact shape: `upper-snake` is `Compute_Total` (`^[A-Z][A-Za-z0-9]*(_[A-Z0-9][A-Za-z0-9]*)*$`),
`screaming-snake` is `COMPUTE_TOTAL` (`^[A-Z0-9]+(_[A-Z0-9]+)*$`) — two different shapes, not
two names for one shape. `nomos-rules`' own `Check_Naming_Convention`
(`checks/naming/violations.rs::Is_Pascal_Snake_Case`) correctly implements `upper-snake`.
Its own Go sibling shipped in this same build window,
`Check_Exported_Go_Functions_Use_Upper_Snake_Case`
(`checks/naming/go_function_names.rs::Is_Upper_Snake_Case`), is named for the same
convention and implements `screaming-snake` instead — checked directly against `case.go`'s
own worked example (`Compute_Total` vs `COMPUTE_TOTAL`) while drafting this record. A second
rule shipped in the same window, `Check_Unexported_Go_Functions_Lowercase_Only_The_First_
Letter`, mirrors that same wrong shape rather than the `mixed-snake` shape
(`compute_Total`, `^[a-z0-9]+(_[A-Za-z0-9]+)*$`) code-standards' own vocabulary already
names for exactly "the first word lower, the rest cased normally." Two real, shipped,
ledger-verified rules drifted from the vocabulary their own rule id names, in the direction
a hand-rolled predicate always drifts: plausible, untested against the source of truth, and
wrong in a way `cargo test` cannot see because nothing checked it against anything external.

`OD-RULES-010` already answered the adjacent question for a different shape of input: a
`ToolProvider`'s output is a fact a native rule reads through `FactReader::Require` and
judges, never a value the rule embeds or a `Finding` a tool emits directly. The reasoning
transfers without alteration. A repository's declared naming/threshold policy is exactly as
external to a rule's own judgment as `cargo clippy`'s diagnostics are — the rule's job is to
compare a subject against a standard, and a standard a repository can restate is data the
rule must be handed, not a constant it was compiled to already agree with.

## The Decision

A rule's configurable parameters are read as a capability fact, the same
`Require`-then-judge-then-emit shape `OD-RULES-010` already establishes, not embedded as a
Rust constant or a hand-rolled predicate. The first instance is `nomos.cap.naming.policy`:
a repository's resolved naming convention, read from `standards.json`'s `naming` and
`languages.*.naming` blocks (falling back to a stated default when a repository declares
none), materialized once for the whole workspace
(`IncrementalGranularity::WholeWorkspace`, `FactVariant::Syntactic` — the fact is the
declaration itself, not an inference over it).

The eight case styles are read off code-standards' own closed vocabulary and its exact
`case_validation.go` shapes, not reinvented: `UpperCamel`, `LowerCamel`, `UnderscoreCamel`,
`LowerSnake`, `UpperSnake`, `ScreamingSnake`, `MixedSnake`, `LowerKebab`, plus `Any` (no
rule). Each wire-round-trips to code-standards' own lowercase-hyphenated spelling
(`"upper-snake"`, not an invented Rust-side name), so a value written in `standards.json` by
someone who has never seen this workspace's Rust source still resolves correctly. A
symbol's required case resolves by the most specific key present — a per-language override
(`languages.<lang>.naming.<symbol>`) before the repository-wide default
(`naming.<symbol>`) before an unconfigured rule's own prior hardcoded behavior, so an
unconfigured repository regresses nothing.

This is a capability contract, not a provider's own type (`OD-CAPABILITY-002`): the
contract lives in its own crate, `nomos-cap-naming-policy`, below both its provider (which
reads `standards.json` through `nomos_platform::FileSystem`, the same port-supplied-by-
caller shape every filesystem-reading provider in this workspace already uses) and the
`nomos-rules` functions that read it. The provider is not a `nomos-lang-*` crate: it reads
one repository-wide configuration file, not one language's source or manifest format, and
naming it as a language provider would misstate what it does. It is the first crate under a
new `crates/repository/` top-level directory, parallel to `crates/languages/` for the same
reason `crates/languages/` exists apart from `crates/capabilities/` — grouped by what kind
of input a crate reads, not by band alone.

`crates/rules/nomos-rules`' six already-shipped casing rules
(`Check_Naming_Convention`, `Check_Project_Owned_Function_Names_Use_Upper_Snake_Case`,
`Check_Data_Names_Stay_Lower_Snake`, `Check_Go_Type_Names_Use_Camel_Case`,
`Check_Exported_Go_Functions_Use_Upper_Snake_Case`,
`Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter`) are refactored onto this
one engine as its first real consumers, each keeping its own `RuleId` and gate behavior —
a repository's `standards.json` gains the power to override any of the six without a second
compiled rule, and the `UpperSnake`/`ScreamingSnake`/`MixedSnake` confusion this record
found is corrected as a direct consequence of routing every one of them through the same
tested vocabulary rather than six independent guesses at it.

## What This Does Not Do

It does not generalize every rule in `nomos-rules` in one stroke. Casing is the first
family because it is the most duplicated shape already shipped and the one this record's
own measurement found a real defect in; a threshold family (the file-size triggers), a
suffix family, and whatever else the code-standards corpus's remaining ~1,120 rule
documents turn out to cluster into are each their own future instance of this same
decision, not decided here.

It does not give `nomos-rules` general file-system access or a second way to read
`standards.json` outside the capability/provider seam `OD-RULES-010` already established —
a rule still takes only `&[SourceFile]` and a `FactReader`, per `crates/rules/nomos-rules/
src/lib.rs`'s own stated invariant, and the provider is still the only place in this
increment that touches a path.

It does not compose the new provider into `nomos-check-orchestration::Run`. Several rules
in this crate already ship additive and unwired for one build cycle before composition
follows (`Check_Declared_Role_Matches_Surface`, `Check_Cross_Language_Correspondence` at
their own first landing); this capability follows the same sequencing, as its own later
item.

It does not decide anything about a repository that declares a case style outside the
eight code-standards names — such a value is a load error at the provider, the same
"a casing scheme this check cannot read is one it would silently ignore" refusal code-
standards' own `spec.go` already states for the identical reason.

## Status

Accepted. `nomos-cap-naming-policy` is this decision's first capability contract.
