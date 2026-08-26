---
id: OD-ANALYSIS-010
type: decision
title: The sound control-flow reachability tier is a crate-local call resolver, not a compiler or language-server integration
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - analysis
  - capability
  - architecture
  - rules
relations:
  - target: OD-RULES-008
    type: relates-to
  - target: OD-ANALYSIS-007
    type: relates-to
  - target: OD-ANALYSIS-004
    type: relates-to
  - target: ARC-CONFORMANCE-001
    type: relates-to
---

# The sound control-flow reachability tier is a crate-local call resolver, not a compiler or language-server integration

## Question

`OD-RULES-008` specified `nomos.cap.controlflow.reachability`'s sound tier exactly: "resolving
every call the `Err` arm reaches, including into helper functions elsewhere in the crate, to
confirm each actually constructs or propagates a `Finding`," and named it as this workspace's
first real, already-motivated need for a `FactVariant::SemanticallyResolved` program-semantics
fact — then explicitly declined to say how: "Neither tier exists in this workspace today.
Building either is not this record's territory." Nothing in this workspace names a language
tool this fact would come from, and this workspace has never integrated a compiler frontend or
a language server. Building the sound tier now, without deciding that first, would fix the
heaviest-looking answer (embed `rustc` or `rust-analyzer`) into the first real program-semantics
provider by nobody having checked whether the fact actually needs it.

## What Was Measured

**`ARC-CONFORMANCE-001`'s own test rules out delegating this specific fact to a compiler tool,
regardless of tier.** Read directly: "the test for native analysis is that no provider exposes
the fact... because the fact is about this repository's own architecture, its own requirement
corpus, its own record set or its own render history, none of which any external tool has a
model of." Whether a control-flow path "reaches a `nomos_contracts::Finding` construction" is
exactly that shape — no compiler or language server has a notion of this workspace's own
`Finding` type or what constructing one means, the identical reasoning that record already
applies to a declared band or a corpus requirement. `rustc`, Clippy and `rust-analyzer` could
at most supply raw name resolution; none could answer the actual claim `Check_Unread_Reaches_A_
Finding` needs. This is true independent of which mechanism supplies the name resolution
underneath it.

**What tier 2 actually needs to resolve is narrower than general Rust name resolution.**
`OD-RULES-008`'s own three worked examples (`naming.rs`'s `Payload_Of`, `mirror/index.rs`'s
`Unread_Of`, `dependency.rs`'s `Payload_Of`) are all plain, non-generic, non-trait free
functions called by path within their own crate. Resolving *that* shape of call — an
identifier or a module-qualified path, matched against `fn` items and `use` imports visible in
the calling crate's own module tree — does not require type inference, trait resolution, or
borrow checking; it requires exactly the same kind of scoped, syntactic-plus-module-path
matching this workspace's own `syn`-based tooling already does for `nomos.cap.syntax.items`.
What a hand-rolled crate-local resolver structurally cannot resolve — a call through `dyn
FactReader` (the trait `Check_Unread_Reaches_A_Finding`'s own sources call through), a stored
closure invoked elsewhere, a macro-generated call site, or anything crossing a crate boundary
— is exactly the set `OD-RULES-008` already named for `Applicability::AgentRequired`: "the path
forwards into a call the provider cannot resolve statically... not `MissingCapability` or
`ProviderUnavailable`; a provider is present and ran, and the honest answer is that no
mechanical method decides the question for this specific path." The provider does not need to
resolve those cases to be sound; it needs to say honestly that it did not.

**`nomos-lang-rust` already has the shape this extends, not a shape it would replace.**
Verified directly: `crates/languages/nomos-lang-rust/src/reachability.rs`'s tier-1 provider is
`syn::visit::Visit` over one file's parse tree, and the crate's own doc states its guarantee
model precisely for this reason — "[`Declared_Guarantee`] says the resolution level, whether
the output is sound, whether it is complete." A tier-2 offer is the same discipline at a wider
input (a crate's module tree instead of one file) and a narrower claim (only the calls it can
actually bind), not a new analysis engine replacing what tier 1 already does.

**The capability's own declared `IncrementalGranularity::File` ceiling does not survive tier
2.** `nomos-cap-controlflow::contract::Ceiling()`'s own doc reasons: "a function body lives in
one file — there is no coarser unit a change here could force a re-derivation across." That
reasoning is sound for tier 1's single-file pattern match, and false for tier 2: resolving a
call "into helper functions elsewhere in the crate" means a change to a helper function in a
*different* file can change whether a site in *this* file is sound. A tier-2 offer claiming
`IncrementalGranularity::File` would be claiming an independence the resolution it performs
does not have.

## The Decision

**The sound tier is a native, crate-local call resolver built as a further increment of
`nomos-lang-rust`'s own `syn`-based reading, not an embedding of `rustc`, `rust-analyzer`,
Roslyn, Clang, or any LSP server.** Concretely, when built:

1. It reads a crate's full module tree (every file the crate's own `mod` declarations reach),
   the same "corpus this crate did not write" `nomos-lang-rust`'s own doc already frames its
   job around, widened from one file to one crate.
2. It resolves a call site only when it is a direct call to a free function reachable by path
   or by a `use` import within that same crate — no generics, no trait dispatch, no `dyn`, no
   macro-expanded call sites.
3. Every other call shape is left unresolved *for that path*, reported through
   `Applicability::AgentRequired` or `PartiallySupported` exactly as `OD-RULES-008` already
   specified — never approximated, never silently treated as clean.
4. `nomos_cap_controlflow::contract::Ceiling()`'s `IncrementalGranularity` moves from `File` to
   `Project` when a tier-2 offer is built against it, because that is what the resolution this
   tier performs actually depends on.

Compiler and language-server integration is declined for this specific fact, not for program-
semantics work in general: `ARC-CONFORMANCE-001` already states the boundary this record
applies rather than invents, and nothing here forecloses a future rule whose own subject
genuinely needs a fact only a real compiler frontend can produce — generic instantiation, trait
resolution, or borrow-checker output, none of which this candidate's own scope touches.

## What This Does Not Do

It does not build the tier-2 provider. `OD-RULES-008` already named that as separate,
reservable work with its own territory; this record answers only the mechanism question that
record explicitly left open. It does not touch `nomos-rules::reachability`'s tier-1 rule, its
`Requirement`, or its `Applicability::PartiallySupported` reporting — those stay exactly as
they are until a tier-2 offer exists for a caller to ask for. It does not amend
`OD-ANALYSIS-007`'s own text; `OD-RULES-008` already narrates, in prose, the narrowing this
record's own premise depends on, and rewording that record's stale "no rule needs
`SemanticallyResolved`" sentence is separate, smaller work this record does not claim. It does
not decide anything about a future fact that *does* need generics, trait resolution, or
borrow-checker output — that would be a different, heavier forcing case, decided against its
own real rule when one names it, the same way this one was.

## Status

Accepted. Names the mechanism `OD-RULES-008` traced but declined to choose, checked directly
against `ARC-CONFORMANCE-001`'s own test for when native analysis is owed, against what tier 2
actually needs to resolve (narrower than general Rust name resolution), and against what
`nomos-lang-rust` already is. The provider itself is a following item.
