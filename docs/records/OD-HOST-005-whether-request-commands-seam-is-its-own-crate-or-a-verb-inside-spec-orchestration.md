---
id: OD-HOST-005
type: decision
title: Whether request::Command's orchestration seam is its own crate or a verb inside nomos-spec-orchestration
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - host
  - orchestration
  - architecture
relations:
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-001
    type: relates-to
---

# Whether request::Command's orchestration seam is its own crate or a verb inside nomos-spec-orchestration

## Question

`OD-HOST-002`'s family 9 names four control-command groups a surface holds no privileged
logic for once each has a seam: `WorkCommand`, `CheckCommand`, `SpecCommand` and
`request::Command`. The first three each closed as their own dedicated crate at band 40 —
`nomos-work-orchestration`, `nomos-check-orchestration`, `nomos-spec-orchestration` — and
`P13-HOST-002-SPEC-DONE` narrowed family 9's remaining gap to `request::Command` alone,
still one hand-parsed verb (`Submit`) inside `nomos-cli::request` with no orchestration
crate a second adapter could depend on.

Following the same shape for a fourth time — a new `nomos-request-orchestration` crate at
band 40 — runs into a constraint the first three never faced: `crates/host/nomos-cli/src/
request.rs` already assembles the specification store through `nomos_spec_orchestration::
corpus::{Assemble, Assembly, CorpusRequest}` and writes its submission through
`nomos_spec_store::Accept_Submission`, the exact store `nomos-spec-orchestration` already
owns. `tests/contract/tests/boundaries/graph.rs::Test_Dependencies_Should_Run_Strictly_
Downward` asserts `dependency_band < band`, strictly — equal-band edges fail the test, the
same rule that keeps `nomos-lang-rust` and `nomos-lang-rust-scan` from naming each other. A
`nomos-request-orchestration` at band 40 could not depend on `nomos-spec-orchestration`,
also band 40, to reach the corpus assembly and store submission `request::Command`
actually needs — unlike `work`, `check` and `spec`, which never cross-depend on each
other's domains at all.

Two shapes both close the gap without breaking that rule, and nothing on the board or in a
governing record has chosen between them:

**A fourth crate at a band strictly above 40** (41, or a new intermediate band), depending
on `nomos-spec-orchestration` the way `nomos-cli` (band 90) already does. This keeps
`request::Command`'s seam as its own crate, matching the other three command groups in
kind, but breaks the pattern that all four orchestration crates so far sit at one band as
peers, and needs its own placement rationale rather than reusing band 40's.

**A `Submit` verb added to `nomos-spec-orchestration` itself**, alongside `SpecCommand`'s
nine. This reuses the corpus assembly and `Accept_Submission` call directly, at no new
band and no new crate, but narrows what "one crate per `OD-HOST-002` command group" means:
`nomos-spec-orchestration`'s own module doc and `README.md` row currently describe it as
exactly `SpecCommand`'s crate, and either would need to say it answers a second command
type as well — the same store, but a `nomos request submit` invocation is not a `nomos
spec` verb by the CLI's own naming.

## What Would Decide It

Whether `request::Command` is, architecturally, a fifth `SpecCommand`-adjacent verb over
the same store — in which case folding it in is the smaller, more defensible move — or a
genuinely separate command group that happens to share a dependency, the same way a
`WorkCommand` verb and a `CheckCommand` verb might one day both need `nomos-capability`
without either becoming the other's sibling. `request.rs`'s own module doc calls it
"submitting a feature request, design spec or feature result through the one accept door
`OD-SPEC-009` decided" — a description that reads as the specification store's own
front door, not a clearly separate domain, which is why this record does not resolve the
question by inspection alone.

## Resolution

Accepted, by reading `crates/host/nomos-cli/src/request.rs` in full rather than by the
inspection-by-description this record's own "What Would Decide It" section declined to
settle for. That file is short enough to read whole, and it answers its own question in its
first line: `//! nomos request — submitting a feature request, design spec or feature result
through the one accept door OD-SPEC-009 decided`, and further, in its own module doc, "This
is a transport and nothing else... It validates nothing and persists nothing on its own
behalf."

`Command` has exactly one variant, `Submit`. `Submit`'s real work — the part that is not CLI
argument parsing — is three calls: `nomos_spec_orchestration::corpus::Assemble` (the same
corpus assembly `SpecCommand` already wraps), `nomos_spec_store::Accept_Submission` (the
store `nomos-spec-orchestration` already owns), and, when `--into` is given,
`nomos_spec_project::{Catalogue::Shipped, Build}` through the `subject-dossier` profile (the
same render primitive `SpecCommand::Render` already wraps). There is no fourth primitive:
`request.rs` contributes a `SubmitRequest` parse and a `Submission` construction — a
different *shape* of input than `SpecCommand`'s verbs take, not a different *domain* of
work. That is the direct evidence for this record's own "fifth `SpecCommand`-adjacent verb"
reading, measured against the file rather than inferred from its module doc alone, and it
was not available when this record was first written at a stage before `request.rs`'s
current shape existed to read.

**The decision:** `request::Command`'s seam is a `Submit` verb added to
`nomos-spec-orchestration`, alongside `SpecCommand`'s existing verbs — the second shape this
record named, not a fourth peer crate. No code moves as part of this resolution;
`request.rs` keeps running exactly as it does today until a follow-on item performs the
move, at which point `nomos-spec-orchestration`'s own module doc and `README.md` row need to
say it answers a second command type, the update this record's "Question" section already
anticipated would be needed under this shape.

## Status

Accepted. `request::Command`'s seam does not build itself until this is decided; it is
decided now, by direct evidence read out of `request.rs` rather than by inspection of its
module doc's description alone. The seam build itself — moving `Submit`'s orchestration
logic into `nomos-spec-orchestration` and updating that crate's own description — is
separate work this resolution unblocks rather than performs.
