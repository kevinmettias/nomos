---
id: OD-CAPABILITY-009
type: decision
title: Resolve never sees a subject to partition on; a subject-partitioned capability is the caller's Preferring to make
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - capability
  - resolution
  - languages
relations:
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-CAPABILITY-006
    type: relates-to
---

# Resolve never sees a subject to partition on; a subject-partitioned capability is the caller's Preferring to make

## Question

`nomos-lang-go` (`P14-LANG-GO-SYNTAX-PROVIDER`) offers `nomos.cap.syntax.items` with
`Assurance::Sound` on both axes -- strictly stronger than `nomos-lang-rust`'s `Sound/Unknown`
-- but answers only for Go files, disjoint from `nomos-lang-rust`'s Rust files. Registering it
as a third offer in `nomos-check-orchestration::Declare_Syntax_Capability` was tried and
reverted: `nomos_rules::Syntax_Requirement`'s own deliberately-unpreferenced resolution then
picked `nomos_lang_go` for `.rs` files too, because its guarantee dominates and
`Registry::Resolve` ranks purely by guarantee strength. Four real tests in
`nomos-check-orchestration`'s own `tests.rs` went from green to `DependencyUnavailable`
findings -- empirical, not theoretical.

`OD-CAPABILITY-001` closed with this gap already named under "What This Does Not Do": "the
registry ranks and does not dispatch... per-subject fallback changes which provider answers for
which file... stated here so that the gap is a plan and not an oversight." `OD-CAPABILITY-006`
does not cover it either -- it is explicit that its cross-language case is two *different*
languages' subjects joined by a declared correspondence, and says plainly that two
same-capability offers ranked by strength is `OD-CAPABILITY-001`'s case, not its own.
`nomos-lang-rust` and `nomos-lang-go` are offers of the *same* fact, `nomos.cap.syntax.items`,
so `OD-CAPABILITY-006` does not answer this either. This record decides what `Resolve` owes a
capability whose real offers partition by subject rather than compete over one, against the
concrete failure above, not hypothetically.

## What Was Read

**Where the resolved offer's identity actually lands.** `nomos_analysis::Reader::Require`
calls `self.registry.Resolve(need)` and feeds the result into `Key_For`, which calls
`Key_From(capability, subject, inputs, resolution.Offer()?)`. `Key_From`'s own doc: "The
provider and its guarantee are components of the key, which is the reason per-subject fallback
is admissible at all: two providers answering about one file are two addresses rather than two
values at one." `FactKey.provider` is `offer.provider.clone()` -- whichever offer `Resolve`
picked. This is the exact mechanism behind the observed failure: `Resolve` picked
`nomos_lang_go`, `Key_For` built a key stamped with `nomos_lang_go`'s provider id, and the fact
in the store -- written by `Materialize_Syntax`, which calls `nomos_lang_rust::Materialize`
directly and does not consult the registry at all -- was filed under `nomos_lang_rust`'s
identity. The two addresses never agreed and every lookup missed.

**What `SubjectId` actually is.** Every call site constructs it the same way:
`SubjectId::From_Digest(Content_Digest(path.as_bytes()))`. It is a one-way content digest with
no recoverable path, extension, or language. `Registry::Resolve(&Requirement)` does not take a
subject at all; `Reader::Require`/`Require_Any` do receive `subject: &SubjectId`, but by the
time either reaches the registry, the only thing in hand is the opaque digest. There is no
"which subjects does this offer recognize" question a subject-scoping field on `ProviderOffer`
could answer at resolution time, because resolution time never receives anything a domain
predicate could be evaluated against. A `Domain`/`Recognizes` field on `ProviderOffer` would be
answering a question `Resolve` is structurally never asked.

**A mechanism for exactly this already exists, built and tested, unused in production.**
`Requirement::Preferring(provider: ProviderId)` and `Selection::Over`'s honoured-preference
branch: "The offer that no usable offer is strictly stronger than answers. A named preference
outranks that, because a caller saying what it wants is not something the registry is in a
position to overrule." `Selection::Weaker`'s own doc goes further: "What a caller that lowered
its floor for coverage actually spends... The registry has no subject and so cannot make that
call." Both are exactly the shape this gap needs. Neither is called from any production code
path today -- every use of `.Preferring(` in the tree is a test fixture.

**Why it is not simply a matter of calling it.** `nomos_rules::Syntax_Requirement`'s own doc is
explicit and deliberate: "There is deliberately no `Preferring`. Naming a provider would be the
rule deciding what the registry exists to decide." That stance is correct for what it guards --
`nomos-lang-rust` and `nomos-lang-rust-scan` are genuine competitors over the same subject
space, any `.rs` file, and letting a rule pin one by name would let it silently prefer the
weaker, faster scanner forever regardless of what a stronger parser could answer, defeating the
reason `OD-CAPABILITY-001` gave the registry that ranking job in the first place. Applying
`Preferring` naively to unblock `nomos-lang-go` would either contradict this stance outright or
require deleting it -- and deleting it breaks the genuine competition it protects.

## What Decided It

The tension resolves once the two questions `Syntax_Requirement` and its call sites are
actually asking are told apart, because they are not the same question:

- **"Given this floor, which competing offer is best?"** -- a quality judgment, subject-
  agnostic, correctly the registry's to rank by guarantee strength, and correctly excluded from
  needing a name. This is what `Syntax_Requirement()` states today, unchanged, still with no
  `Preferring` baked in.
- **"Given this concrete file, which offers can even attempt it?"** -- an applicability fact,
  not a quality judgment, and one the registry structurally cannot answer because it never
  receives anything recognizable about the subject -- only a digest already stripped of the
  path that would answer it.

The second question can only be answered where a real, un-digested path is still in hand: at
the call site, before `SubjectId::From_Digest` is ever computed. `nomos_lang_rust::recognition`
and `nomos_lang_go::recognition` both already expose exactly this as `Recognition::Of_Path(path:
&str)`. So the rule is:

**A capability whose real, registered offers partition by subject rather than compete over one
is not resolved by widening what the registry ranks. It is resolved by the caller, at the point
it still holds the subject's real path, narrowing the `Requirement` with `.Preferring(provider)`
chosen from that path's own `Recognition` -- before the path is digested into the `SubjectId`
`Resolve` will only ever see as opaque.** This does not touch `ProviderOffer`, `Registry::Resolve`,
or `Selection`'s ranking, all of which stay exactly as `OD-CAPABILITY-001` left them, correctly
reserved for genuine same-subject competition. It does not touch `Syntax_Requirement()`'s own
"no `Preferring`" stance either -- that function states a subject-agnostic floor and is right to
carry no preference. What is missing is not in the floor; it is in every call site that turns a
floor into a `Require` call for one concrete file, which today discards the very information
(the real path) that would let it narrow correctly, before the registry ever sees a subject it
cannot use.

## Why Not A Subject-Scoping Field On `ProviderOffer`

Considered and rejected, for the reason under "What Was Read" above: `Resolve` and `Selection`
operate over `ProviderOffer` values and a `Requirement`, and neither carries a subject at the
point ranking happens. A `Domain` field on `ProviderOffer` would need `Resolve` (or a new
subject-aware variant of it) to also receive the subject to filter by, and the only subject
available anywhere near that call is already an opaque digest -- a `Domain` predicate would
have nothing decidable to test it against without also being handed the pre-digest path, which
means the caller already has to do the applicability work itself, making a registry-side field
redundant with the caller-side `Preferring` call it would still need to make. Splitting the
same decision across two places, one of which cannot act on it alone, is worse than leaving it
in the one place that can.

## What This Does Not Do

It does not register `nomos_lang_go::Provider_Offer()` in
`nomos-check-orchestration::Declare_Syntax_Capability`. It does not change
`nomos_capability::{ProviderOffer, Registry, Requirement, Selection}` in any way -- every type
and every ranking rule `OD-CAPABILITY-001` decided stands untouched. It does not change
`nomos_rules::Syntax_Requirement()`'s signature or its "no `Preferring`" doc, both of which
remain correct statements about a subject-agnostic floor.

## What Would Unblock `nomos-lang-go`'s Registration

A further, real increment, not built here:

- Every caller that resolves `nomos.cap.syntax.items` for one concrete subject --
  `nomos_rules::mirror::index` and `nomos_rules::naming::reading` today -- must compute
  `Recognition::Of_Path` against the real path it already holds before that path is digested,
  and attach `.Preferring(ProviderId::New(nomos_lang_rust::PROVIDER))` or
  `.Preferring(ProviderId::New(nomos_lang_go::PROVIDER))` to `Syntax_Requirement()` accordingly,
  falling through to the registry's own ranking only for a path neither recognizes (today,
  never -- both providers are total over `.rs`/`.go`, so an unrecognized path is a third case
  this increment must also decide, not silently swallow).
- `nomos-check-orchestration::facts::materialize::Materialize_Syntax`'s write side must gain the
  matching per-`Recognition` dispatch, generalized from its current hardcoded
  `nomos_lang_rust::Materialize` call, so that whichever provider identity the read side
  requests via `Preferring` is the same identity the write side actually filed the fact under.
  Read and write already have to agree on provider identity for `Key_From` to find anything;
  today they agree only because there is one language to hardcode.
- Only once both sides dispatch on the same `Recognition` does adding
  `registry.Offer(nomos_lang_go::Provider_Offer())` to `Declare_Syntax_Capability` become safe --
  at that point it is no longer relying on `Resolve`'s unpreferenced ranking to do work this
  record found it cannot do.

Until that lands, `nomos-lang-go` remains built, tested, and correctly *not* wired into the
composed registry. This record decides the shape of the fix; it does not build it.

## Amendment: The Unblock Plan's Own Call Sites Would Regress `nomos-rules`' Provider-Blindness, And It Named Only One Of Two Prerequisites

"What Would Unblock `nomos-lang-go`'s Registration" above says the fix lives in "every caller
that resolves `nomos.cap.syntax.items` for one concrete subject -- `nomos_rules::mirror::index`
and `nomos_rules::naming::reading`" -- naming `nomos_lang_rust::recognition::Recognition::Of_Path`
and `nomos_lang_go::recognition::Recognition::Of_Path` as what each must compute before calling
`.Preferring(...)`. Read against `nomos-rules`' own `Cargo.toml`, that plan cannot be built as
written without a real regression.

**What was read.** `nomos-rules/Cargo.toml` states, once per capability, exactly the same
stance: it depends on `nomos-cap-syntax` and not `nomos-lang-rust`, on `nomos-cap-dependency`
and not `nomos-lang-rust-cargo`, on `nomos-cap-controlflow` and not `nomos-lang-rust`'s
reachability module, on `nomos-cap-lint` and not `nomos-lang-rust-clippy` -- each comment
giving "the identical reasoning" as the one before it: a rule states a floor and the registry
chooses the provider, so this crate never needs to know a provider's name to add a fourth
capability, and would not have needed to for a fifth. Computing `Recognition::Of_Path` inside
`mirror::index` or `naming::reading` requires importing `nomos_lang_rust` and `nomos_lang_go`
directly -- the exact dependency this crate's own manifest has, four times over, declared it
does not take. The plan as written would regress that stance for precisely the one capability
it was written to hold for, in the act of fixing the one gap that stance was blocking.

`SourceFile` (`crates/rules/nomos-rules/src/lib.rs:186`) already carries the correction. Its
`subject` field is documented as "carried rather than derived... the composition root's own
convention... carrying it makes rule and root agree by construction," and `path` is documented
as "reporting only... nothing in this crate keys anything on it." The composition root
(`nomos-check-orchestration::composition::Declare_Syntax_Capability`) already depends on
`nomos-lang-rust` today and would gain `nomos-lang-go` the moment its offer is registered --
it is already the one place in this dependency graph allowed to know both providers by name.

**The correction.** The caller-side preference this record's original text assigns to
`mirror::index`/`naming::reading` is computed by the composition root, not by those two
functions -- the same division `subject` already draws. The root resolves each source's
`Recognition` against both languages once, before the two rule-owning modules ever see it, and
carries the result to them as already-resolved data (a `ProviderId`, e.g. a field beside
`subject` on `SourceFile`, populated the same construction-time way `subject` is). `nomos-rules`
then only needs `nomos_capability::ProviderId` -- already a dependency, via `nomos-capability`
itself -- to call `.Preferring(id)` when the field is present, never a language crate's
`recognition` module. This does not touch `Syntax_Requirement()`'s "no `Preferring`" stance,
which still correctly describes the subject-agnostic floor that function alone states; only the
*caller* narrows it, exactly as the original decision below already says, just not from inside
`nomos-rules`.

**The second, independent prerequisite the original text did not name.** Ground-truthed against
the real walk: `nomos-cli::check::sources::Read_Entry`, `nomos-cli::gate::sources` and
`nomos-api::sources` (the last two identical by the second's own module doc, "a deliberate twin
... not a shared dependency") each read a directory entry only `if path.extension().is_some_and(
|extension| return extension == "rs")`. No `.go` file reaches a `SourceFile` at all today, in
any of the three, independent of how `nomos_capability::Registry` is composed or how
`Preferring` is wired. Registering `nomos_lang_go::Provider_Offer()` after the correction above
still materializes zero Go facts in a real `nomos check`, `nomos gate run` or API run, because
nothing ever hands a `.go` path to either side of the pipeline. This is not the registry gap
`OD-CAPABILITY-001`/`OD-CAPABILITY-006` scoped, and it is not the resolution gap this record's
own body decided either -- it sits earlier, in three walkers this record never read.

**What this amendment does not do.** It does not change what the body above decided:
`nomos_capability::{ProviderOffer, Registry, Requirement, Selection}` still need no
subject-scoping mechanism, `Syntax_Requirement()` still correctly carries no preference, and
the registry still ranks purely by guarantee strength with no notion of subject. It corrects
only *where* the caller-side narrowing is computed, and it names, without building, the walker
gap alongside the call-site work the original "What Would Unblock" section already named as not
built here. Both remain a further, real increment.

## Amendment: Both Named Increments Are Built

Both gaps the amendment above named are now built, against the correction it decided, not the
original text's.

**The corrected call-site fix.** `nomos_rules::SourceFile` gains a
`preferred_syntax_provider: Option<ProviderId>` field, carried the same way `subject` already
is. `nomos-check-orchestration::composition::Recognized_Syntax_Provider(path)` is the one
function that computes `Recognition::Of_Path` against `nomos_lang_rust` and `nomos_lang_go` by
name; `crate::run::Recognized` calls it once per source before any rule runs, to populate the
field, and `crate::facts::materialize::Materialize_Syntax`'s write side calls the identical
function to decide which provider's `Materialize` to run. `nomos_rules::Syntax_Requirement_For`
takes the field's value directly and never computes recognition itself, so `nomos-rules/
Cargo.toml` gained no dependency on either language crate. `Declare_Syntax_Capability` registers
`nomos_lang_go::Provider_Offer()` as a real third offer. An end-to-end test proves a clean
`.go` source is judged under `nomos_lang_go`'s own identity while the existing `.rs` tests stay
green -- the regression this record's own body describes does not recur.

**The walker gap.** `nomos-cli::check::sources::Read_Entry`, `nomos-cli::gate::sources::
Read_Entry` and `nomos-api::sources::Read_Entry` each now admit a path whose extension is `rs`
*or* `go`, the same hardcoded-literal style each already used for `rs` alone -- not a new
dependency on either language crate, since a walk decides what is worth reading at all, not
which registered provider answers for it. A real `nomos check`, `nomos gate run` or API run now
discovers a `.go` file and judges it through `nomos_lang_go`'s real offer, the first time this
has been true anywhere in this workspace.

Neither increment changed anything the body above or the first amendment decided:
`nomos_capability::{ProviderOffer, Registry, Requirement, Selection}` remain untouched,
`Syntax_Requirement()` still carries no preference, and the walker's extension check is
unrelated to `Registry::Resolve`'s own ranking. This amendment records that both were built; it
does not reopen how.

## Status

Accepted. Decided against the concrete failure recorded in `P14-LANG-GO-SYNTAX-PROVIDER`'s own
commit, by reading `Reader::Require`/`Key_From` (where a resolved offer's identity becomes a
lookup key), `SubjectId`'s construction (a one-way digest, never a recognizable path once
inside the registry's reach), `Selection::Over`/`Requirement::Preferring` (the existing,
tested, unused-in-production mechanism this decision points call sites at), and
`Syntax_Requirement`'s own doc (whose "no `Preferring`" stance is correct for the floor and is
left unchanged). `nomos_capability` needs no subject-scoping mechanism; the fix lives entirely
above the registry, in the call sites that still hold a real path before it is digested away.
`nomos-lang-go` is registered and, as of the second amendment above, reachable from a real walk:
both increments this record named as not built here are now built.
