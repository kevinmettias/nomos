---
id: OD-CAPABILITY-011
type: decision
title: The syntax payload gains a small, closed set of typed shapes for what four rules already re-derive by hand; it does not become a general tree
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - syntax
  - determinism
  - rules
relations:
  - target: OD-ANALYSIS-007
    type: relates-to
---

# The syntax payload gains a small, closed set of typed shapes for what four rules already re-derive by hand; it does not become a general tree

## Question

`nomos.cap.syntax.items`'s payload writes one tab-separated line per item: ordinal, kind,
visibility, qualified name, documentation and a free-text `shape: Option<String>`. `syn`
builds a full parse tree in-process before this encoding runs, and most of that tree is
discarded: an enum's own variants are never recorded as items at all, a function's own
parameter names and types collapse to an arity count, and a struct field of `u32` and one of
`String` both collapse to `shape: "value"`. Four rules re-derive structure the discarded tree
already had, by re-parsing the same raw text a second time by hand: `enum_shape.rs`,
`constant_scope.rs`, `domain_type_alias.rs` and `facade.rs`. One of those hand parsers
(`Split_At_Top_Level_Commas`) was checked directly and found wrong on a real, ordinary case
— a variant holding a function-typed member — because a bracket-depth tracker treats a
function arrow's own closing angle as a bracket it never opened. The question this record
answers: does the payload move to carrying a typed tree, or stay a flat item index, and what
does either answer cost.

## What was checked, not assumed

Read all four rules directly rather than trusting the summary that motivated this item.

- **`enum_shape.rs`** re-derives, via its own whole-file brace-depth scan, which variants an
  enum has and whether each is tuple-shaped, with the member type list split by
  `Split_At_Top_Level_Commas` — the exact function already shown wrong on a callback-typed
  variant member. `syn`'s own walker (`walk.rs`'s `visit_item_enum`) never emits a per-variant
  `Item` at all; an enum's own `shape` is always `None`. **Nothing today carries this fact.**
- **`domain_type_alias.rs`** re-derives a type alias's own visibility (`Strip_Rust_Visibility`)
  and whether it sits inside a still-open `impl`/`trait` body, via a second whole-file
  brace-depth scan. Both facts **already exist**, unused by this rule, in every `Item`
  produced today: `item.visibility` is a first-class typed field, and `walk.rs`'s
  `visit_item_impl` already pushes the impl's own self-type onto `item.scope` for exactly
  this nesting question. This rule's re-derivation of these two facts can stop **today**,
  independent of any payload change — it is not re-parsing something absent, it is
  re-parsing something already sitting in the payload it was already handed.
- **`facade.rs`** re-derives `pub mod`/`pub use` declarations, including a re-export's own
  alias. Checked against `shape.rs`'s own doc comment for the `Use` item's shape: it states
  directly that what a re-export aliases — the source path being re-exported — is "on the
  other side of a name resolution this provider does not perform," and is never captured.
  Unlike the other three, this rule's re-derivation recovers a fact **no version of this
  payload has ever carried**, not one the tree already had and the encoding dropped.
- **`constant_scope.rs`** re-derives, via its own frame-stack scan, which function body a
  local `const` declaration sits inside. This is not module-item structure at all: `syn`'s
  own `Visit` walker here only visits top-level items, never descends into a function body's
  own statements, and no `Item` this crate produces today represents a statement inside a
  function. Fixing this rule's re-derivation is not "expose more of the tree already built
  per item" — it is "walk somewhere this visitor does not walk today," a materially larger
  change than adding a field to an existing item shape.

Also checked: the payload's determinism footprint. `SyntaxFactProduction`'s own `Strategy`
declaration is reused, not independently declared, by three domains in
`tests/integration/tests/determinism/declarations.rs`: `syntax-fact-production`,
`module-index-rollup`, `controlflow-reachability-production`. A byte-format change touches
exactly these three goldens, not an unbounded set.

## Decision

The payload does not become a general typed tree — nothing here needs arbitrary recursive
structure, and `nomos-lang-rust-scan` exists specifically to answer `nomos.cap.syntax.items`
*without* a real parser, over files `syn` itself refuses (a stated byte-order-mark case in
its own module doc). A format that only a real parser could produce would make that crate's
whole reason for existing — keeping `nomos_capability::Registry`'s fallback and contention
branches reachable over a corpus a stronger parser cannot fully cover — impossible to
satisfy honestly.

Instead, `shape` stops being free text for the specific item kinds where free text is
currently standing in for a structure the crate already knows how to write once (`shape.rs`'s
own `Impl_Shape`/`Type_Shape`/`Struct_Shape`/`Function_Shape` family), and gains three
concretely named, closed extensions:

1. **A real per-variant list for `Enum` items** — name and either an arity (tuple) or a real
   field list (struct-variant), the fact `enum_shape.rs` re-derives and the one whose hand
   equivalent (`Split_At_Top_Level_Commas`) is measured wrong today.
2. **A real parameter list and return type for `Fn` items**, replacing the current bare
   arity — nothing named in this record's own investigation depends on this one directly
   today, but `Function_Shape`'s own doc already states the arity-only choice as provisional,
   and the closed extension costs nothing extra once the `Enum` and struct-field work below
   establishes the pattern.
3. **The real, unresolved source path on a `Use` item's own alias** — text only, no name
   resolution, staying `FactVariant::Syntactic`: `facade.rs` is the one rule of the four
   recovering a fact this payload has never carried at any point in its history, and the
   fix is to carry it rather than keep a fifth crate re-deriving it.

`Struct`/`enum`-field types beyond arity are **not** decided here as a fourth extension:
`domain_type_alias.rs`'s only genuine unmet need is the alias's own right-hand-side type
name, which the `Type_Shape` collapse already loses ("every type to either slice or value")
independent of whether an enum or a function ever gets richer shapes. Whether `Type_Shape`
itself grows a real type-name field is real, separate, unscoped work this record does not
commit to; what it does commit to is that `domain_type_alias.rs`'s *other* two
re-derivations (visibility, impl/trait nesting) have no reason to wait for it.

## What a second language provider owes

`nomos-lang-go` is a real parser (tree-sitter) and owes the equivalent three extensions for
Go's own shapes it already covers (a Go `type X struct{}`'s own field list, a function's own
parameter/return list, an import/re-export's own path) wherever Go has an analogous
construct — decided per-construct when that crate's own provider is next touched, not
enumerated here.

`nomos-lang-rust-scan` owes nothing new. It is not a parser and was never meant to answer
what only a parser can; its offer stays weaker on these three fields exactly as it is
already weaker in the ways its own module doc already states, and `Registry::Resolve`'s
existing fallback and applicability vocabulary is what already lets a caller ask for the
richer shape and accept a weaker one when that is the best a subject's own admitted provider
can honestly give.

## What happens to the content-addressed encoding

`Encode_Payload`'s byte format changes: the three extensions above are new fields or a
richer `shape` encoding, so the bytes for any file exercising an enum, a function, or a
re-export change. `syntax-fact-production`, `module-index-rollup` and
`controlflow-reachability-production` — the three declared domains built on
`SyntaxFactProduction::STRENGTH` — need their golden bytes re-baselined in the same commit
that changes the encoding. The declared determinism *property* does not change (still
whatever `DeterminismStrength`/`ReproducibilityScope`/`TraceEquivalence` triple
`SyntaxFactProduction` states today); only the golden value the goldens compare against
does, the same re-baselining any byte-format change to a `BitIdentical` producer already
requires.

## Which of the four hand-parsing rules this removes, and which keep their own parser

- **`enum_shape.rs`**: removed once the per-variant extension exists — the rule reads the
  typed list instead of re-scanning, and the specific bracket-depth bug measured against
  `Split_At_Top_Level_Commas` stops mattering because nothing calls that function anymore.
- **`facade.rs`**: removed once the `Use` source-path extension exists, for the alias half;
  the `pub mod` half was already redundant with existing typed fields (name, visibility) and
  did not need to wait.
- **`domain_type_alias.rs`**: partially removed **now**, independent of any payload change —
  its visibility and impl/trait-nesting re-derivation should read `item.visibility` and
  `item.scope` today. Its remaining re-derivation (the alias's own right-hand-side type
  name) stays a hand parser until `Type_Shape` itself is decided separately.
- **`constant_scope.rs`**: keeps its own parser. Its need — which function body a local
  `const` sits inside — is a different kind of gap than the other three: it needs the syntax
  walker to descend into function bodies at all, which nothing here does today, not a richer
  shape on an item the walker already visits. Deciding whether and how the walker should
  visit function-body-local items is real, separate, larger work this record declines to
  fold in.

## Status

Accepted. No code moves under this item. The three named extensions, the two independently
actionable fixes in `domain_type_alias.rs`, and the standing gap in `constant_scope.rs` are
each their own future item's scope.
