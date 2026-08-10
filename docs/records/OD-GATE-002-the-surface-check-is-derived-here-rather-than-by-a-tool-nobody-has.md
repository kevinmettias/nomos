---
id: OD-GATE-002
type: decision
title: The public surface check is derived in this workspace rather than by a tool nobody has
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - boundary
  - tooling
relations:
  - target: OD-GATE-001
    type: relates-to
  - target: OD-LEDGER-003
    type: relates-to
---

# The public surface check is derived in this workspace rather than by a tool nobody has

## Question

Five anti-leak assertions were named for this workspace. Three are in
`tests/contract/tests/boundaries.rs`, module reachability shipped, and the transport check
has no transports to check. The fifth was a `cargo public-api` snapshot, and nothing in the
tree mentioned it.

So the question was not whether to build it. It was where the snapshot comes from, and the
obvious answer does not work here.

## Why Not `cargo public-api`

Three reasons, in the order they bite.

**It is on no runner this repository has.** It is not installed on the machine this was
written on and `.github/workflows/gate.yml` installs nothing. So the check would be a
declared skip on every pull request — an honest one, countable by `P9-SKIP`'s mechanism,
and still zero enforcement in the only place enforcement matters. `OD-GATE-001` already
records sixty-eight assertions in that position. Adding a sixty-ninth to close an item
about checks that silently do not run would be the item's own defect, committed by the fix.

**It cannot be called from where the check lives.** The check has to be a test, and a test
that shells out to `cargo` runs inside a `cargo test` that already holds the target
directory's lock. The nested invocation blocks until the outer one finishes, which it
cannot, because it is waiting for the test. Pointing the child at its own `--target-dir`
avoids the deadlock by rebuilding the entire workspace from scratch on every run.

**The snapshot would move on its own.** `cargo public-api` renders rustdoc JSON, and both
the tool's rendering and the JSON's shape change between versions. A snapshot committed
under one version fails under the next with a diff nobody caused, which is how a check
becomes something people re-bless without reading.

## The Decision

**Each crate's exported surface is derived from its own source by
`tests/contract/src/surface.rs`, committed under `tests/contract/surface/<crate>.txt`, and
compared by `tests/contract/tests/public_surface.rs`.**

That is the same shape this workspace already uses for every other property it watches:
`BANDS` for the dependency order, `Corpus_Gates` for the corpus-gated tests, and
`Declared_Universes` for completeness. All of them read the tree rather than asking a tool
to read it, and all of them run wherever `cargo test` runs.

There is no skip. `P9-PUBLIC-API`'s `done_when` allows a declared one and this needs
neither: the check has no precondition beyond the source files it is about.

## What This Gives Up

The derivation is a source-level reading, not rustdoc's resolution. It is right about what
this workspace writes and would be wrong about things this workspace does not do. Stated
so that the next person does not have to find out:

- **Glob re-exports.** `pub use module::*` names nothing, so nothing resolves. It is
  reported as an unresolved re-export rather than dropped. There are none today.
- **Blanket and generic impls.** An `impl<T> Trait for T` is recorded by its written form,
  not expanded over the types it covers.
- **Trait inheritance.** A trait's supertrait methods are surface and are recorded against
  the supertrait, where they are written.
- **`#[cfg]`.** Every branch is read, so a surface that differs by platform or feature is
  reported as the union. This workspace has no `#[cfg]`-gated exports.
- **Macro-generated items.** A `macro_rules!` expansion that declares a `pub` item is
  invisible. There is no such macro here.
- **Type aliases and inference.** The declaration is recorded as written, so
  `pub type Guard = …` shows the alias rather than what it resolves to.

Each of these is a false negative — the surface under-reports rather than over-reports —
and under-reporting is the direction that flatters. That is the cost, and it is the reason
this record exists rather than a comment in the file.

What is *not* given up is the property the item asked for. Adding a `pub fn`, widening a
field, adding a variant, adding a method to a public type, changing a signature, or adding
a name to a `pub use` list all change a committed file, and the test fails until somebody
commits that change with the change that caused it.

## Blessing Is Not A Passing Run

`NOMOS_SURFACE_BLESS` rewrites the snapshots and then fails. A bless that passed would be a
check any environment could switch off by exporting a variable, and a CI job with it left
set would report green having compared nothing — the same failure this whole item is about,
one level out.

## What Would Change This

A runner that installs `cargo public-api`, and a way to invoke it that is not nested inside
`cargo test` — a separate gate step rather than a test would satisfy both. At that point
the two derivations should be compared against each other before either is trusted alone,
the way `P9-ONE-DIRECTION` compared the corpus-gate count against the source: two
independent readings that agree are worth more than one authoritative one.

## Status

Closed by `P9-PUBLIC-API`.
