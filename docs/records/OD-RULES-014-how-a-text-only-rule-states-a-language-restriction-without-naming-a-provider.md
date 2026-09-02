---
id: OD-RULES-014
type: decision
title: How a text-only rule states a language restriction without naming a provider
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - capability
  - applicability
  - languages
relations:
  - target: OD-CAPABILITY-009
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
---

# How a text-only rule states a language restriction without naming a provider

## Question

A rule whose norm is intrinsically about one language — Go's `//nolint` needing a reason,
Rust's `unsafe` needing a justification — has to decide whether the file in front of it is
that language. `nomos-rules` gives it no way to ask, so eleven functions across nine modules
have each answered it privately by looking at the file extension.

Whether that is a defect is not obvious, and the obvious repair is the one `OD-CAPABILITY-009`
already rejected. `SourceFile` carries `preferred_syntax_provider` precisely so a rule does not
recompute recognition, so the apparent fix is to compare that field against a language
provider's identity. That would reintroduce the thing `nomos-rules`' own manifest exists to
prevent: a band-30 rule naming a language crate.

## What Was Measured

**Eleven private classifiers, four names, two signatures, for one question.**

| Module | Function | Takes |
|---|---|---|
| `checks/rust_text.rs:279` | `Is_Rust_Source` | `&SourceFile` |
| `checks/concurrency_text.rs:129` | `Is_Rust_Source` | `&SourceFile` |
| `checks/error_text.rs:195` | `Is_Rust_Source` | `&SourceFile` |
| `checks/go_text.rs:232` | `Is_Go_Source` | `&SourceFile` |
| `checks/placement.rs:171` | `Is_Rust_File` | `&str` |
| `checks/placement.rs:179` | `Is_Go_File` | `&str` |
| `checks/structure.rs:223` | `Is_Go_File` | `&str` |
| `checks/naming/go_data_names.rs:97` | `Is_Go_File` | `&str` |
| `checks/naming/go_function_names.rs:117` | `Is_Go_File` | `&str` |
| `checks/naming/go_type_names.rs:64` | `Is_Go_File` | `&str` |
| `checks/function_shape.rs:264` | `Source_Has_Extension` | `&SourceFile, &str` |

Every one of them is the same three lines — `Path::new(path).extension()` compared
case-insensitively against a literal. The bodies do not disagree, which is what makes this
duplication rather than eleven considered judgments.

**The pattern is replicating, not settled.** `checks/error_text.rs` was committed while this
record was being written and arrived carrying a fresh copy of `Is_Rust_Source`. A rule author
today has no declared alternative to copying it, so each new language-specific rule family
adds one.

**`preferred_syntax_provider` cannot answer the question, because a provider identity is not
a language identity.** Three offers are registered against `nomos.cap.syntax.items`:
`nomos.lang.rust.syn`, `nomos.lang.rust.scan` and `nomos.lang.go.tree-sitter`. Two of the
three are the same language. The identities name the implementation technology that reads the
file — `syn`, `scan`, `tree-sitter` — and a rule asking "is this Rust" against one of them is
asking a narrower question than it means.

Today that narrowness is latent rather than live: `composition::Recognized_Syntax_Provider`
tries `nomos_lang_rust` first and `nomos_lang_go` second and never returns
`nomos.lang.rust.scan`, so a rule testing for `nomos.lang.rust.syn` would currently be right by
accident. It stops being right the first time recognition prefers the scan provider for any
path, and it is silent when it does: the rule returns no findings and reports no absence.

**`Recognition` is per-provider and binary.** `nomos_lang_go::Recognition::Of_Path` answers
`Recognized` or `Unrecognized { extension }` — "is this mine", not "what is this". There is no
type anywhere below band 30 that names a language, and `nomos-contracts` has none.

**The prohibition the classifiers are working around is deliberate and documented at length.**
`nomos-rules/Cargo.toml` states it once per dependency: a rule names a capability contract and
lets the registry choose who answers it, "never `nomos-lang-rust`, which is where the answer
happens to come from today". `OD-CAPABILITY-009` kept `preferred_syntax_provider` opaque for the
same reason, and all three call sites that read it — `checks/crosslang/reading.rs:33`,
`checks/mirror/index.rs:77`, `checks/naming/reading.rs:31` — pass it straight to
`Syntax_Requirement_For` without ever inspecting it.

## The Decision

**A rule's language restriction is a fact carried to it, not a fact it derives.** `SourceFile`
gains a language field distinct from `preferred_syntax_provider`, populated by the composition
root before any rule runs, exactly as `subject` and `preferred_syntax_provider` already are and
for the reason `SourceFile::subject`'s own documentation already gives: "a rule that computed
its own would be a second answer to that convention, and the two would disagree silently."

Three constraints make it implementable without regressing band 30:

1. **Each language crate declares its own language beside its provider identity.** A
   `LANGUAGE` constant sits next to `PROVIDER` in `nomos-lang-rust`, `nomos-lang-rust-scan` and
   `nomos-lang-go`; the first two declare the same value. A language crate is the only correct
   author of that string, and this is what keeps the mapping out of the root's own head.
2. **The carried type is opaque at band 30.** It lives in `nomos-contracts` and holds a string,
   the way `RuleId` and `ProviderId` already do. A rule compares it against a literal naming
   the one language its norm is about. Naming one language is inherent to a language-specific
   norm; knowing the set of languages is not, and the set stays where recognition is.
3. **Recognition stays in the language crates.** The root keeps asking them, through the single
   `Recognized_Syntax_Provider` seam that already exists, and gains no second answer of its own.

**What becomes of the eleven sites.** All eleven are deleted and replaced by a comparison
against the carried field. `Source_Has_Extension` in `function_shape.rs` is the one that does
not simply fall out, because it is parameterized rather than fixed to one language; it becomes
the same comparison with the expected language passed in. None of the eleven keeps sniffing:
the predicate they compute is not wrong, but it is a second answer to a convention
`Recognition::Of_Path` already owns, and eleven copies of a correct predicate is the defect
here rather than the predicate itself.

## Why Not `preferred_syntax_provider`

Measured above: two of three registered syntax providers are the same language, and the
identities name reading technology rather than language. Overloading the field would also take
it away from the job `OD-CAPABILITY-009` gave it — narrowing a `Require` to the offer that can
attempt a subject — so a future third Rust provider would silently change which rules fire as
well as which provider parses. Those are two different questions and they should not share one
field.

## Why Not A Capability Fact In The `OD-RULES-011` Family

That family carries configuration a repository declares about itself: its naming policy, its
size limits, its tooling language. Which language a file is written in is not a value a
repository chooses, so it is not that kind of fact. `checks/go_text.rs`'s own module doc
already draws this exact line for its own rules — "nothing about whether these markers need a
reason is a value a repository would configure" — and the line holds one level up: nothing
about whether a file is Go is configurable either.

## Why Not An Orchestration-Side Partition Of The Source List

Handing each rule only the sources it applies to would move the knowledge from eleven rules
into the root's hand-composed rule table, where it becomes a per-rule language column — the
independently maintained support matrix the specification forbids, and a second answer to
applicability rather than a first. How that table should generalize is `OD-HOST-004`'s
question, not this one, and this decision is deliberately shaped so that answering it later
changes nothing here: a carried fact on `SourceFile` is correct whether the table stays
hand-written or becomes computed.

## What This Record Does Not Do

It does not change `nomos_capability::{ProviderOffer, Registry, Requirement, Selection}`, or
`Registry::Resolve`'s ranking, or `Syntax_Requirement()`'s subject-agnostic floor — every part
of `OD-CAPABILITY-009` and `OD-CAPABILITY-001` stands.

It does not add a language-crate dependency to `nomos-rules`, which is the constraint that
rejected the obvious repair in the first place.

It does not decide `Is_Test_Or_Example_Source`, duplicated verbatim in `rust_text.rs:286` and
`concurrency_text.rs:136`. That is the same duplication disease answering a different question
— which part of a repository a file sits in, not which language it is — and it needs its own
item rather than being folded in here on the strength of looking similar.

It does not rewire the hand-composed rule table in `nomos-check-orchestration::run_context`,
and it does not change which rules are selected or registered.

## Status

Accepted. Decided by reading all eleven classifier bodies, the three registered
`nomos.cap.syntax.items` offers and their identity constants, `Recognition::Of_Path` in both
language crates, `composition::Recognized_Syntax_Provider`, the three call sites that consume
`preferred_syntax_provider`, and `nomos-rules/Cargo.toml`'s own statement of the prohibition
being worked around. The implementing change is not in this item's territory and needs its own.
