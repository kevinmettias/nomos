---
id: OD-CAPABILITY-014
type: decision
title: An impl block's own generic parameters join OD-CAPABILITY-011's closed set of typed shape extensions
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - syntax
  - determinism
  - rules
relations:
  - target: OD-CAPABILITY-011
    type: relates-to
  - target: OD-CAPABILITY-013
    type: relates-to
---

# An impl block's own generic parameters join OD-CAPABILITY-011's closed set of typed shape extensions

## Question

`P68-SINGLE-LETTER-NAMES-READS-A-BLANKET-IMPLS-GENERIC-PARAMETER` measured this directly
against a real, ordinary third-party fixture (`hex` 0.4.3's `impl<T: AsRef<[u8]>> ToHex for
T`): `single-letter-names` reported `T` as an unjustified single-letter name, but `T` is not
a struct field or an ordinarily-declared identifier a human abbreviated — it is the impl
block's own already-declared generic parameter, used again as the block's Self type. The
rule cannot tell this case apart from `impl Trait for X` where `X` is a real, single-letter
struct name someone chose and still deserves the finding, because nothing in the syntax
payload says whether an `Implementation` item's own name is one of its own declared generic
parameters. The question this record answers: does that fact join `OD-CAPABILITY-011`'s
closed set of typed extensions, the same way an enum's per-variant list and a function's
parameter list did, or is it a different, larger kind of gap the way `constant_scope.rs`'s
need was.

## What was checked, not assumed

Read the walker and the shape encoding directly rather than trusting the finding's own
description.

**The generic parameter list is already computed at the exact call site that discards it.**
`nomos-lang-rust/src/syntax/walk.rs`'s `visit_item_impl` receives `node: &'ast
syn::ItemImpl`, which carries `node.generics` — the impl's own `<T: AsRef<[u8]>>` — and never
reads it. The function reads exactly two things off `node`: `Type_Head(&node.self_ty)` (the
name) and `node.trait_.is_some()` (fed to `Impl_Shape`, which encodes only `TRAIT` or
`INHERENT`). This is `OD-CAPABILITY-011`'s "small, closed, cheap" case, checked directly
rather than assumed: the fact is sitting in the same `&syn::ItemImpl` the walker already
holds, at the same call site, the identical shape enum variants and a function's parameter
list were in before that record's fix — not a walk somewhere the visitor does not walk today
(`constant_scope.rs`'s harder case, which this record does not resemble).

**A rule-side heuristic with no new data would hide a real case, not just miss one.**
Checked whether `single-letter-names` could instead exempt every Implementation item whose
own name is a single uppercase letter, with no payload change at all. Rejected: `impl Trait
for X` for a real, single-letter-named struct `X` is exactly the case this rule exists to
catch, and a blanket exemption on shape (`Type_Head` being a bare identifier) cannot tell it
apart from a blanket impl's own generic parameter without knowing what the impl actually
declared. The distinction is genuinely a fact about the impl's own generics, not a shape a
name-only heuristic can approximate.

**The existing shape encoding for `Implementation` items is a single fixed label, not yet a
composite.** `Impl_Shape(serves_a_trait: bool) -> String` (`nomos-lang-rust/src/syntax/
shape.rs`) returns exactly `TRAIT` or `INHERENT` — one of two constants, unlike `Struct_Shape`
or `Function_Shape`, which already encode a variable-length body behind a header
(`STRUCT_SHAPE_HEADER` plus a tab/newline-delimited, escaped field list, in
`nomos-cap-syntax/src/payload/observation.rs`). Extending `Implementation`'s own shape to
also carry a generic parameter list is the same kind of change `OD-CAPABILITY-011`'s enum and
function extensions already made to their own item kinds, using the identical
escape/delimiter machinery this payload already has, not a new mechanism.

**No second question is being smuggled in.** `domain_type_alias.rs`'s own nesting check
already reads `item.scope`, which `visit_item_impl` already pushes the impl's own self-type
onto — a fact this record's own subject (the impl's *generic parameter list*, not its
self-type or its nesting) does not touch and does not need to re-decide.

## Decision

**An `Implementation` item's own declared generic parameter names join `OD-CAPABILITY-011`'s
closed set of typed shape extensions**, on the same footing as that record's three: a small,
closed, already-available fact the walker discards today, not a general tree and not a
second walk. `Impl_Shape`'s encoding gains the impl's own generic type-parameter identifiers
(the names bound by `<...>` — `T` in `impl<T: AsRef<[u8]>> ToHex for T`, not their trait
bounds, which nothing measured here needs), alongside the existing `TRAIT`/`INHERENT` label,
using the same header-plus-escaped-list convention `Struct_Shape` already established for a
variable-length body. Lifetime and const generic parameters are not part of this extension:
nothing measured against this record's own subject (a name collision between an
Implementation item's own name and one of its generics) needs them, and `OD-CAPABILITY-011`'s
own discipline is to extend for a checked need, not a hypothetical future one.

A consumer (`single-letter-names`, or any future rule) can then ask "is this Implementation
item's own name one of its own declared generic parameters" as a direct comparison against a
typed list, the same shape `enum_shape.rs`'s consumer will ask "what are this enum's own
variants" once `OD-CAPABILITY-011`'s extension lands.

## What a second language provider owes

`nomos-lang-go` has no equivalent construct: Go has no `impl` block and no blanket
implementation over a bare type parameter the way `impl<T> Trait for T` is method syntax with
receivers, not a trait/self-type pair. This record names no obligation for it. If a future Go
construct raises the identical name-collision question (a generic function's own type
parameter used as some other declared name, say), that is a new, separate measurement against
Go's own real code, not an extension this record pre-commits to by analogy.

`nomos-lang-rust-scan` owes nothing new, the identical reasoning `OD-CAPABILITY-011` already
gives for its own three extensions: it is not a parser, and its offer stays weaker on this
field exactly as it is already weaker on the others its own module doc names.

## What happens to the content-addressed encoding

`Encode_Payload`'s byte format changes for any file declaring a generic `impl` block: the
extension is a new segment of `Impl_Shape`'s own encoded string, so the bytes for such a file
change the same way `OD-CAPABILITY-011`'s three extensions already changed bytes for a file
exercising an enum, a function, or a re-export. The same three declared determinism domains
that record named (`syntax-fact-production`, `module-index-rollup`,
`controlflow-reachability-production`, all built on `SyntaxFactProduction::STRENGTH`) need
their golden bytes re-baselined in the same commit that changes the encoding — no new domain
is introduced by this extension, since it lands inside the same `shape` field those three
already cover.

## What this record does not do

It does not build the extension, touch `Impl_Shape`, `visit_item_impl`, or
`single-letter-names` itself, or re-baseline any golden. It names the shape a future
increment takes, the same way `OD-CAPABILITY-011` named `enum_shape.rs`'s fix before
`P46`-numbered work built it.

It does not re-open `P70-IMPL-BLOCK-GENERIC-PARAMETERS-NOT-IN-PAYLOAD`'s own sibling
question — trait bounds, lifetime parameters, or const generics on an impl block — beyond
naming that none of them are in scope, because nothing measured here needs them.

It does not decide whether `single-letter-names` itself changes, only that the payload gains
the fact the rule would need to. A follow-up Correction item re-authoring
`P68-SINGLE-LETTER-NAMES-READS-A-BLANKET-IMPLS-GENERIC-PARAMETER` against this extension is
real, separate, bounded work once the extension exists.

## Status

Accepted. An `Implementation` item's own generic parameter names join `OD-CAPABILITY-011`'s
closed set of typed shape extensions, using the same encoding convention its `Struct_Shape`
already established. No code moves here.
