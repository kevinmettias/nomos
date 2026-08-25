---
id: OD-CAPABILITY-010
type: decision
title: The first cross-language capability compares two providers' declared fields against a doc-comment-declared correspondence
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - cross-language
  - syntax
  - rules
relations:
  - target: OD-CAPABILITY-006
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-SYNTAX-002
    type: relates-to
---

# The first cross-language capability compares two providers' declared fields against a doc-comment-declared correspondence

## Question

`OD-CAPABILITY-006` fixed what a cross-language claim is and classified five claim families
by reachability against the resolution vocabulary this workspace already has. Family 2 —
producer/consumer serialization format agreement — is "the one family with real reach at
`Syntactic` today, and only partly": comparing two sides' *spelled* declarations, field
names and arity, needs no name resolution and no execution. That record explicitly declined
to build the capability, define the correspondence-declaration shape, or pick which crate it
lives in, naming all three "the first cross-language capability's own work, judged against
this record."

Two things stood in the way of that work existing. First, `nomos.cap.syntax.items` — the one
capability both `nomos-lang-rust` and `nomos-lang-go` already offer — carries no field-level
data for a struct: `PayloadItem.shape` is `Observation::Absent` for every `Struct`-kind item
in both providers today, verified directly against `visit_item_struct`
(`crates/languages/nomos-lang-rust/src/syntax/walk.rs`) and `Record_Type_Spec`
(`crates/languages/nomos-lang-go/src/syntax/walk.rs`) before this record was written. A
field-by-field comparison has nothing to read. Second, no correspondence-declaration
convention exists to say which Rust struct is claimed to share a wire shape with which Go
struct — `OD-CAPABILITY-006`'s own criterion is that this must be declared, never inferred,
because neither provider can see the other's subject.

## What Was Measured

`nomos.syntax.items.v2`'s own grammar (`crates/capabilities/nomos-cap-syntax/src/payload.rs`)
already answers the first question without a schema version bump. `shape` is documented as
"open... meaningful relative to the kind" — `fn/<arity>` for a function, `slice`/`value` for
a typed declaration, `inherent`/`trait` for an impl block, `.` for "the shape has nothing to
say about this form." A kind that has said nothing yet saying something now is exactly the
extension this vocabulary was built to admit, and it is invisible to a build compiled before
this record: an `Observation::Present` value the shape's own `Escape`/`Unescape` round-trip
already carries opaquely for any string, tabs and newlines included, whether or not that
build's `shape`-per-kind table happens to interpret `Struct`'s new content.

The second question already has a live answer inside this workspace, not merely a candidate
one: `nomos-rules::universe`'s "declared mirror," read off a doc comment
(`/// Mirrored by \`Test_Name\`.`) rather than a config file, a Rust attribute, or an IDL.
`Test_A_Declared_Mirror_Should_Be_Read_Off_The_Doc_Comment` is the worked precedent that a
`documentation: Observation` field already carries a resolvable name a rule parses out and
looks up against real source, and that a blind provider reports unobserved rather than
silently "no mirror" — the identical shape `OD-CAPABILITY-006` requires of a cross-language
correspondence: declared at the site, not inferred, and honest about a provider that cannot
see it.

`nomos-lang-rust` depends on `syn` with the `printing` feature deliberately absent —
"nothing here renders tokens back out," its own `Cargo.toml` states. A field's full,
generic-aware type spelling is therefore not cheaply reachable from this provider without
either widening that boundary or hand-rolling a token renderer duplicating what `printing`
already gives; `Type_Head` (`crates/languages/nomos-lang-rust/src/syntax/shape.rs`) already
exists as this crate's stated compromise for the identical problem one layer up — a type's
head, as written, without its generic arguments — and is reused rather than a new renderer
built to avoid re-deciding a boundary this crate already drew. `nomos-lang-go` faces no such
constraint: `tree-sitter`'s node API returns exact source bytes for any span, so a Go field's
type is captured verbatim. The two sides are not symmetric in what they can spell, the same
way they are not symmetric in `Assurance` today, and the comparison this record licenses is
bounded by the weaker side.

## The Decision

**`nomos.syntax.items.v2`'s `shape` gains a real vocabulary entry for `Struct`.** A struct
with named fields records `shape` as `Observation::Present`, encoding one `name` and one
`type` per field, tab-separated, one field per line, in declaration order — read and written
by a shared pair of helpers (`Struct_Shape`/`Struct_Fields`) in `nomos-cap-syntax`, the same
crate that already owns `Function_Shape`/`Function_Arity` for the identical reason: the
vocabulary belongs to the contract, not to either provider. A struct with no named fields
(Rust's unit and tuple forms; Go has no such form) records `Observation::Absent` — observed,
and there is nothing to say, the same as any other kind's default. Both `nomos-lang-rust` and
`nomos-lang-go` populate it for a real named-field struct: the second real party is not
optional groundwork here, it is the whole reason `nomos.cap.syntax.items` is a capability
contract rather than one provider's own opinion. A field's `type` is `Type_Head`'s answer on
the Rust side (a type's head as written, generics and references stripped) and the tree's own
source-text slice on the Go side — carried through into a finding's summary for a person to
read, never compared to the other side's type spelling as a pass/fail condition, because
nothing in this workspace declares a mapping between the two languages' type vocabularies and
inventing one is not this record's question to answer.

**A cross-language correspondence is declared by a doc comment, on the struct claiming a
counterpart:** `/// Corresponds to \`<QualifiedName>\`.` — the identical marker shape,
resolved the identical way, as `universe.rs`'s `Mirrored by`. One-directional: the struct
that declares the correspondence names the other side; the other side declares nothing back,
the same asymmetry a mirror's own declaration already has. `<QualifiedName>` is resolved
against every source's own `nomos.cap.syntax.items` fact by exact match, language-blind by
construction — the rule does not ask which language a hit came from, only whether one exists,
because the correspondence names a struct, not a file or a language.

**The rule compares field *names and arity*, set-wise, not declaration order.** `OD-
CAPABILITY-006`'s own worked example — "side A declares four fields, side B declares three,
one name differs" — is a claim about which names exist on each side, not about the order they
were written in; a wire format built from named fields (the shape this family's own text
scopes to) does not generally require positional agreement the way a tuple would. A name
present on one side and absent on the other is the finding; a name present on both is not
reported, the same "clean is silent" convention `Check_Lint_Diagnostics` already holds for a
tool that already decided.

**A named counterpart nowhere in the sources reports `Applicability::MissingCapability`**,
naming which side is missing — `OD-CAPABILITY-006`'s own resolution, applied rather than
re-decided. A counterpart that is found but carries no field data (a Rust tuple or unit
struct named as a wire counterpart, a struct form neither provider enumerates fields for)
reports `Applicability::Unparseable`: the fact exists and was read, but it is not the fact
this comparison needs, which is a different failure than no fact existing at all.

**No new capability crate.** Both sides of the comparison are already `nomos.cap.syntax.items`
facts — the rule reads it twice, once per subject, the way any rule already reads one
capability for one subject, only over a subject *pair* a declared correspondence names rather
than a subject the walk handed it directly. `OD-CAPABILITY-002`'s contention criterion never
triggers: there is no second capability contract for a second party to contend over, because
there is no second capability at all.

## What This Does Not Do

It does not touch the other four families `OD-CAPABILITY-006` named. FFI ownership, runtime
error propagation, cross-side threading and the `SemanticallyResolved`/`RuntimeObserved`
halves of family 2 and 3 all still wait on producers this workspace does not have; nothing
here builds one.

It does not compare the two sides' field types to each other, license a Rust-to-Go type
mapping, or claim `u32` and `int` are or are not the same wire type. That is a declared
correspondence of its own kind — `OD-CAPABILITY-006`'s own criterion, applied a second time,
to a question this record does not reach.

It does not add a `printing` dependency to `nomos-lang-rust`, or otherwise widen what that
provider renders. `Type_Head`'s existing boundary is reused, not renegotiated, and a future
record that wants a Rust field's full generic-aware spelling has to make that case on its own
terms.

It does not change `nomos.syntax.items.v2`'s `SCHEMA` constant or its grammar. `shape`'s
vocabulary was already open per kind; giving `Struct` a real answer is the extension that
vocabulary exists to admit, not a new version of it.

It does not decide how this rule is selected by `OD-GATE-017`'s `Wants(selected, ...)`
mechanism beyond composing it the same way every other unconditional rule already is — that
is composition, per `OD-HOST-004`, not a question this record reopens.

## Status

Accepted.
