---
id: OD-GATE-002
type: decision
title: The public surface check is derived in this workspace rather than by a tool nobody has
status: accepted
version: 3
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
- **Named re-exports of another crate's items.** Resolution is per crate: `Public_Surface`
  is given one package and its root, and there is no second crate's module tree to follow
  `pub use nomos_model::Subject_Of_Path` into. The name is reported as an unresolved
  re-export rather than resolved to the declaration it names. This is the one entry that
  is *reported* rather than merely absent, which is the difference version 2 bought and the
  section below records.
- **Blanket and generic impls.** An `impl<T> Trait for T` is recorded by its written form,
  not expanded over the types it covers.
- **Trait inheritance.** A trait's supertrait methods are surface and are recorded against
  the supertrait, where they are written.
- **`#[cfg]`.** Every branch is read, so a surface that differs by platform or feature is
  reported as the union. This workspace has no `#[cfg]`-gated exports.
- **Macro-generated items.** A `macro_rules!` expansion that declares a `pub` item is
  invisible to the scan. *There is no such macro here* was wrong when it was written:
  `nomos-contracts` declares its twelve identity newtypes through `Digest_Identity!` and
  `Named_Identity!`, each expansion carrying a `pub struct` and its inherent methods. The
  expansion is still invisible. What changed in version 2 is that those twelve are
  re-exported by name from the crate root, so they are now *reported* as unresolved
  re-exports instead of being absent from the snapshot altogether.
- **Type aliases and inference.** The declaration is recorded as written, so
  `pub type Guard = …` shows the alias rather than what it resolves to.

Each of these is a false negative — the surface under-reports rather than over-reports —
and under-reporting is the direction that flatters. That is the cost, and it is the reason
this record exists rather than a comment in the file.

Each of those is an absence. The entry above them is not, and the distinction is the whole
of version 2: a name that is *reported* is a line in the snapshot, so adding or removing it
still fails the test, while a name that is *absent* changes nothing and nobody hears.

What is *not* given up is the property the item asked for. Adding a `pub fn`, widening a
field, adding a variant, adding a method to a public type, changing a signature, or adding
a name to a `pub use` list all change a committed file, and the test fails until somebody
commits that change with the change that caused it.

That last clause was false for a cross-crate target until version 2, and it was false in
the way that is hardest to notice: not by failing loudly, but by being true of every name
somebody happened to test.

## Version 2: A Name Is Reported, Not A Declaration

`P11-REEXPORT-GRAIN`. `Emit_Re_Export` resolved each name in a `pub use` list separately
and then asked one question about the whole list — had *anything* resolved. A name that
resolved to nothing was reported only when every name beside it also did.

A glob is the sole name in its declaration, so the promise three paragraphs up held for the
one case this record thought to name, and held by accident. It failed for every other:
`pub use corpus::{Corpus, SourceFile, Subject_Of_Path, Walk}` had three names declared in
the crate and one re-exported from `nomos-model`, so the three made the fourth look handled
and it left `tests/contract/surface/nomos-integration-tests.txt` entirely — blessed away by
`a6f0e9c` while the crate went on exporting the name and `slice.rs` went on calling it.

**The report is per name.** A list is split across as many lines as it has unresolvable
names, spelled as the `pub use` a reader would go and look at — `pub use
corpus::Subject_Of_Path`, carrying `as` where the source renamed. The route is the one the
source wrote, not the one the target eventually lives at, because that first hop is where
somebody checking the report has to begin and the second hop is the resolution this reader
is recorded as not doing.

Reporting whole lists was refused for the mirror reason: one line naming four names says
four items are outside this crate when one is, which is over-reporting, and a snapshot that
cries wolf is re-blessed unread.

Measured when the grain changed: fifteen names in three crates were being dropped. One in
`nomos-integration-tests`, two in `nomos-contract-tests` re-exported from `nomos-rules`,
and twelve in `nomos-contracts` from the macro above. Three snapshots moved for one
comparison, which is the shape of the defect — it was never about one crate.

## Version 3: A Path Attribute Is Not A Concession

`P114-SURFACE-PATH-DECLARED-MODULES-STAY-UNRESOLVED-2`. A module declared with
`#[path = "…"]` is known to the language by the name its `mod` writes and lives in the file
the attribute names. The reader in `tests/contract/src/reading/module_tree.rs` knew only the
two ordinary layouts, `name.rs` and `name/mod.rs`. It found no file, registered the module
holding nothing, and every route through it failed to resolve.

**A same-crate module declared with a path attribute now resolves.** The file the attribute
names is tried first, against the directory the *declaring* file sits in — the base the
language uses, and not the one that module's other children resolve against. An attribute
naming a file that is not there falls through to the two layouts rather than being taken at
its word, so a declaration that genuinely cannot be followed is still reported as it was.

Two spellings were in the tree and reading the attribute only fixed one. Where the attribute
sits on its own line above the `mod`, the walk already saw the `mod` and needed only the
file. Where the two are written together — `#[path = "fact_reuse.rs"] mod determinism;` — the
line opens with an attribute, so no declaration keyword starts it and the walk steps over the
whole line: the module was never a child, and supplying its file fixed nothing. The first
repair without the second measures as working on thirteen crates out of fourteen, which is
the kind of result that gets believed. `nomos-analysis` is the fourteenth and writes all
fifteen of its sites that way.

**This was a defect, not a concession, so the version 1 list is complete without it.** That
list says what a source-level reading gives up against rustdoc's resolution: glob targets,
another crate's declarations, macro expansions, `#[cfg]` branches, blanket impls, aliases. A
path attribute is none of them. It is a plain module of this crate, written here on purpose
at 156 sites across 18 crates, and the reader did not know the third layout.
`P22-REACHABILITY-PATH-ATTR` had already fixed the same blind spot in `Declared_Modules`, a
different reader — which is what makes this a defect with a precedent rather than a limit
with a reason.

**The final claim above was false for a type reached that way, and is now true.** Adding a
variant, a public field, a method, or changing a signature on such a type changed no
committed file. The re-exported *name* was reported in the unresolved section, so adding or
removing the name still failed the test while everything hanging off it stayed invisible —
the same shape version 2 found for the cross-crate case, true of every name somebody happened
to test.

Measured at the repair, and against the tree rather than against the numbers the item was
filed with, which had moved: 27 snapshots carried an unresolved section holding 188 entries
between them, of which 123 named a module their own crate declares with a path attribute.
Seventy-five of those cleared here and the twelve snapshots that held them went to none —
`nomos-lang-rust-compiler` 18, `nomos-analysis` 12, `nomos-lang-rust-cargo` and
`nomos-lang-rust-clippy` and `nomos-lang-go-modules` 7 each, `nomos-lang-rust-scan` and
`nomos-lang-rust-deny` 6 each, `nomos-lang-rust` and `nomos-lang-go` 4 each,
`nomos-connector-coderabbit` 2, `nomos-capability` and `nomos-workspace` 1 each. The other 48
are `nomos-ledger` and `nomos-repo-policy`, whose snapshots are reserved by other live items
and stay stale until their holders re-bless them.

What the twelve gained is the point rather than what they lost. `pub use
contract::CapabilityContract` was one bare line and is now the struct with its four public
fields; `pub use determinism::SnapshotSerialization` is now the struct, its three associated
constants and the `Strategy` it implements.

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

Version 1 closed by `P9-PUBLIC-API`. Version 2 closed by `P11-REEXPORT-GRAIN`, which
re-authored `P11-REEXPORT-SURFACE` once the reading found the defect in three crates rather
than the one it was opened for. Version 3 closed by
`P114-SURFACE-PATH-DECLARED-MODULES-STAY-UNRESOLVED-2`, itself a re-author of
`P114-SURFACE-PATH-DECLARED-MODULES-STAY-UNRESOLVED` with two snapshots dropped from its
territory because live items held them.
