---
id: OD-HOST-008
type: decision
title: Source discovery is a canonical service, not a walker each host keeps its own copy of
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - orchestration
  - architecture
  - discovery
relations:
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-001
    type: relates-to
  - target: OD-PLATFORM-002
    type: relates-to
---

# Source discovery is a canonical service, not a walker each host keeps its own copy of

## Question

`nomos-api`'s own `sources.rs` calls itself "a deliberate twin of
`crates/host/nomos-cli/src/gate/sources.rs`, not a shared dependency of it," and cites
`OD-HOST-002`'s family 3 ("the directory walk itself stays a composition-root concern") as
why. `nomos-lsp`'s own `sources.rs` gives the identical reason from the other side:
`nomos-cli::check::sources` is `pub(super)`, so a second composition root "carries its own
copy of the same walk rather than reaching into another crate's private module." Four
citations of the same reasoning, at four sites, is what this record measures rather than
assumes.

## What Was Measured

Read directly at `c0e5c79f`, all four files in full: `crates/host/nomos-api/src/sources.rs`,
`crates/host/nomos-cli/src/check/sources.rs`, `crates/host/nomos-cli/src/gate/sources.rs`,
`crates/host/nomos-lsp/src/sources.rs`.

**The walk is byte-for-byte the same shape four times over.** An explicit `Vec<PathBuf>`
stack over `std::fs::read_dir`, not `WalkDir`; `target` and `.git` skipped by name; a nested
git worktree's own root (a `.git` *file*, not a directory) skipped by the identical test
`P67-SELF-CHECK-WALK-CROSSES-NESTED-WORKTREE-BOUNDARY-2` added to all four independently;
a `SourceFile::New(relative_forward_slash_path, Subject_Of_Path(&relative), text)`
construction at the leaf. Three of the four (`nomos-api`, `check::sources`, `gate::sources`)
additionally hard-code the same five-element `SCRIPT_EXTENSIONS` literal for
`check-script-discipline`'s own population; `nomos-lsp` alone omits it, a real behavioral
difference this record does not paper over -- see "What This Does Not Close" below.

**The extension check is a documented, deliberate literal, not an oversight.**
`check::sources::Is_Recognized_Extension`'s own doc: "The extension check is a literal, the
same as `nomos_lang_rust::RUST_EXTENSION` and `nomos_lang_go::GO_EXTENSION` already state,
rather than a dependency on either crate: this walk decides which bytes are worth reading at
all, not which registered provider answers for them." That reasoning is what this record
measures against `nomos_lang_rust`/`nomos_lang_go`'s own public surface and finds no longer
load-bearing: `RUST_EXTENSION`/`GO_EXTENSION` (`pub const &str`) and `Recognition::Of_Path`
already exist on both crates, and `nomos-check-orchestration::composition::
Recognized_Syntax_Provider` already depends on both by name to answer the second, real
recognition question over a path a walk has already let through. A walk consulting the same
two constants directly is not a new dependency this workspace has avoided elsewhere; it is
the one this documented rationale argued against without the constants existing to name.

**`OD-HOST-002` family 3's own claim is narrower than the citation reads it.** Family 3
(amended by `OD-PLATFORM-002`) asserts two things: a `CheckOutcome` is reconstructable by
any client that walks the same tree, and the walk is a composition-root concern. The first
is unaffected by this record -- nothing here changes what a `CheckOutcome` is reconstructable
from. The second is what this record revises: "composition-root concern" was read at each of
the four sites as "each composition root's own copy," but nothing in family 3's own text
requires four separate implementations rather than one shared library both a first and a
second composition root call. `OD-HOST-001` already established that shape for the work
group (`nomos-work-orchestration`) and `OD-HOST-002` itself for capability resolution and
fact/analysis state; this record applies the identical seam to the one family 3 left as a
composition-root-local walk.

## The Decision

**The walk becomes one crate, `nomos-workspace-discovery` (Zone: Application Service), that
all four hosts call instead of carrying their own copy.** `Registered_Extensions()` names
what a registered language package recognizes -- `nomos_lang_rust::RUST_EXTENSION` and
`nomos_lang_go::GO_EXTENSION`, read from those crates rather than retyped -- the one edit a
third language package costs this crate instead of a fifth host-side copy. `Walked_Sources`
takes the recognized extension set as its own parameter rather than baking in
`Registered_Extensions`'s answer, so a caller needing more than registered-language
recognition composes its own wider set instead of this crate inventing a second
registration surface.

**`check-script-discipline`'s script extensions are not folded into `Registered_Extensions`.**
They are a rule's own applicability data (`standards.json`'s `forbidden_extensions`), not a
registered language package's recognition -- a different kind of "registered" than this
record's own question asks about. Centralizing them anyway, as a plain, honestly-labeled
constant (`SCRIPT_EXTENSIONS`) rather than inventing package registration for them, is still
in scope: it is the same literal three of the four hosts already duplicated, and leaving it
duplicated a third time while claiming to close this exact class of duplication would be the
same defect this record exists to close, worn thin.

**Each host's own `sources.rs` becomes a thin wrapper**, keeping its existing
`pub(crate)`/`pub(super)` `Walked_Sources(root: &Path) -> Option<Vec<SourceFile>>` signature
so no caller elsewhere in that host needed to change -- `nomos-lsp`'s `server.rs`,
`nomos-cli`'s `check.rs` and `gate.rs`, and `nomos-api`'s five call sites (`agent.rs`,
`check.rs`, `correction.rs`, `response/gate_explain_response.rs`,
`response/gate_run_response.rs`, `workflow.rs`) were all grepped directly and confirmed to
call only `Walked_Sources`, never `Read_Sources`/`Read_Entry`/`Read_Source`/`Relative_Path`
directly.

## What This Does Not Close

`nomos-lsp`'s own pre-existing gap -- it alone never recognized `check-script-discipline`'s
five script extensions -- is unchanged by this record. `nomos-lsp` calls
`Registered_Extensions()` alone, matching its behavior before this record exactly; giving it
script recognition too would be a real behavior change to a host this record's own scope is
architectural consolidation, not a feature addition, and is left for whoever decides
`nomos-lsp` should surface script-discipline diagnostics at all.

`check-script-discipline`'s own extension set is not verified against `standards.json`'s
`forbidden_extensions` by any test -- it never was, at any of the three sites that duplicated
it before this record, and centralizing the literal into one constant does not by itself
close that hole. `tests/contract/tests/completeness_universes/table.rs` records it as an
honest, counted, `Unmirrored` universe rather than a closed one.

## Status

Accepted. Four independently-maintained copies of one walk, and the "rather than a
dependency on either crate" reasoning each cited for its own hard-coded extension check,
are both measured directly against the tree and found to be a real, closeable duplication
rather than a necessary consequence of `OD-HOST-002`. `nomos-lsp`'s narrower recognition and
`check-script-discipline`'s own unverified extension list are named as open, not closed, by
this record.
