---
id: OD-HOST-003
type: decision
title: An editor surface is a client of the canonical services, not a parser of the CLI's rendered output
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - editor
  - diagnostics
  - orchestration
  - architecture
relations:
  - target: OD-HOST-001
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
---

# An editor surface is a client of the canonical services, not a parser of the CLI's rendered output

`OD-HOST-001` gave the work group a seam so a second adapter could call it without
duplicating platform choice, verb execution or rendering. `OD-HOST-002` stated what any
seam must guarantee once it exists — a surface holds no state a canonical service cannot
reconstruct — and named nine state families, most of them still without a seam. An editor
surface is where both records stop being about a CLI that exits and start being about a
process that does not: it is the second adapter `OD-HOST-001` anticipated, and it is the
worst case on both counts that record raises, because it duplicates platform choice, verb
execution and rendering if the seam is skipped, and unlike `nomos-cli` it can accumulate
the privileged state `OD-HOST-002` forbids simply by staying open.

The content an editor surface would project already exists and is currently unreachable
from outside `nomos-cli`. `nomos_contracts::Finding`
(`crates/contracts/nomos-contracts/src/finding.rs:56`) and the vocabularies it carries —
`Applicability` (`finding/applicability.rs:33`), `EvidenceClass` (`finding/evidence.rs:24`),
`Guarantee` (`guarantee.rs:23`) — are consumed today only inside `nomos-cli::check::report`
and `vacuity.rs`. `nomos_capability::Registry` and `Requirement`
(`crates/substrate/nomos-capability/src/registry.rs`, `requirement.rs:14`) resolve which
provider satisfies which capability, and only `nomos-cli::check::composition` builds one.
The governing-record corpus is readable through `nomos-spec-store` and `nomos-spec-project`
today only via `nomos spec markdown` / `nomos spec record`, both CLI verbs. This is exactly
`OD-HOST-002`'s families 2, 4 and 8 — capability resolution, finding/requirement state, and
decision/record state — named there as gaps, not closed there. This record does not close
them. It states the shape an editor surface must have once they are closed, and what it
must not do while they are not: fall back to running `nomos` as a subprocess and parsing
what it printed, which would make the CLI's presentation a protocol and be very hard to
undo once an editor depended on the text.

## What an editor surface is

An editor surface is a client of the same canonical services any other client calls —
`nomos-work-orchestration` where a seam exists (`OD-HOST-001`), and, as each is built, the
crates `OD-HOST-002` already named for the families that do not yet have one:
`nomos-capability` for provider resolution, `nomos-contracts` for findings and their
vocabulary, `nomos-spec-store` / `nomos-spec-project` for the record set, and
`nomos-ledger` (via whatever orchestration crate `WorkCommand::Add`'s `ItemKind::Correction`
ends up routed through) for opening a correction item. Concretely, three obligations:

1. **It consumes canonical services, not a rendered surface.** Every fact an editor shows —
   an architecture component, a governing requirement, a record, an evidence class, an open
   item — is obtained by calling the library crate that owns it, the same way
   `nomos-work-orchestration::Run` is called today. It is never obtained by invoking `nomos`
   as a subprocess and parsing stdout. A subprocess call onto a rendering command
   (`nomos check`, `nomos spec markdown`) is the same defect `OD-HOST-001` fixed for the work
   group, arriving through an integration instead of a copy — the third duplication in that
   record's list, not a new one.
2. **Every action is a command through a canonical service.** Moving a dependency behind a
   canonical capability, creating a trace entry a rule like
   `nomos-spec-validate::EveryStatementTracesToSource`
   (`crates/spec/nomos-spec-validate/src/rule/every_statement_traces_to_source.rs:24`) found
   missing, replacing a leaked vendor type (the family `OD-CONNECTOR-001` names for
   canonical-service boundaries generally), or opening a correction item
   (`nomos_ledger::ItemKind::Correction`) — each is issued as a call into the service that
   already performs it, not as logic that lives only inside the editor adapter. This is
   `OD-HOST-002`'s second clause, applied to the same surface its ninth state family already
   flagged as unmet: `CheckCommand`, `SpecCommand` and `request::Command` are parsed and
   executed directly inside `nomos-cli` today, with no orchestration crate of their own. An
   editor surface does not fill that gap by reimplementing those commands' logic in the
   editor process; it is unmet for the editor the same way it is unmet for any second
   adapter, until an orchestration crate exists for each and the editor calls that.
3. **It holds no state the services cannot rebuild.** `OD-HOST-002`'s test applies directly,
   and applies harder here than to any surface that record examined, because an editor is
   exactly the long-lived process that record says makes accumulation cheap: a resolved
   capability registry, a `Finding` set, or a rendered record kept in the editor's memory
   instead of asked for again is privileged state, whether or not it happens to be correct
   at the moment it is shown. What survives `OD-HOST-002`'s "not privileged" carve-out is
   unchanged by being in an editor: a cache of an answer a service already gave and can give
   again, which panel is open, a cursor position, an unsent action the user has not yet
   confirmed. The line is the same test that record states: if the editor exited right now,
   is any fact lost that no other client could get back by calling the same canonical
   service?

## What a diagnostic carries

A message and a span is the floor any linter or language server already provides — and
notably, it is more than `nomos_contracts::Finding` states positionally today:
`Finding::locations` is `Vec<String>`, repo-relative paths with no line or column, because
`identity.rs`'s rule for this workspace is that nothing is identified by its path and line.
A precise span, where an editor needs one to place a squiggle, comes from the editor's own
environment — its language server's buffer coordinates — not from a `Span` type this record
invents; none exists anywhere in this workspace, and this record does not add one.

What a Nomos diagnostic adds on top of message-and-span is what `Finding` already carries
and a rendered line already throws away, by that type's own documentation: "every one of
those answers is lost the moment it becomes prose." Concretely, a diagnostic in this
architecture is a projection of:

- **the finding itself** — `rule` (`RuleId`) and `subject`/`subject_name` (`SubjectId`
  and its preimage), naming which rule and which subject, never derived from where the
  subject currently sits in a file;
- **whether the rule reached its subject** — `Applicability`, so a diagnostic never renders
  "clean" and "not evaluated" the same way; `MissingCapability`, `ProviderUnavailable` and
  the rest of that enum are load-bearing states, not edge cases a text message would
  collapse;
- **how the claim was come by** — `EvidenceClass`, from `AgentJudged` (the floor) to
  `Authoritative`, so a reader can tell a measurement from a guess without asking a second
  question;
- **what the wiring would really do about it** — `GateCategory` (`Review`, `Unreachable`,
  `Advisory`, `Blocking`), which is not implied by severity and is the field that tells a
  reader whether ignoring this diagnostic is safe or is a build about to go red;
  `Finding::Can_Fail_A_Build` is the existing predicate an editor surface calls rather than
  re-deriving from the other fields;
- **the governing requirement or record** — the `RuleId` resolved to whatever decided the
  rule exists, read through `nomos-spec-store` / `nomos-spec-project`, the same way `nomos
  spec markdown --id <node-id>` reads it today. `Finding` carries no direct pointer to a
  record id yet; that resolution is part of the seam family 4 and family 8 of `OD-HOST-002`
  still need, not a claim this record makes as already wired;
- **the guarantee behind it, where the rule depended on a capability** —
  `nomos_capability::Requirement::minimum` and the `Guarantee` an installed provider actually
  offered (`variant`, `soundness`, `completeness`, `incremental`), so a diagnostic backed by
  a weaker-than-nominal provider says so instead of reading identical to one backed by the
  strongest;
- **the correction, if one exists** — an open `nomos_ledger` item of `ItemKind::Correction`
  addressing this finding's subject, the same kind this item and its own predecessors
  (`P12-EDITOR-PROJECTION`, `-2`) are, surfaced through the work group's existing seam rather
  than a new one.

A diagnostic carrying only text gives up every one of these, which is the gap `Finding`
already exists to close for `nomos check`'s own output; an editor diagnostic that does not
project this is a linter's diagnostic wearing this repository's name.

## Which navigations and actions are in scope

In scope, each naming the service it is a projection or command onto:

- showing, for a symbol under the cursor, the finding(s) about it, their rule, evidence,
  gate and governing record — a projection of `Finding` plus the record store;
- showing the open item(s) touching a subject — a projection of `nomos-ledger` through
  whatever seam exposes `work list`/`show`'s filtering, the same shape
  `nomos-work-orchestration::BoardView`/`ShowView` already return;
- moving a dependency behind a canonical capability — a command through `nomos-capability`'s
  resolution machinery;
- creating a missing trace entry a rule such as `EveryStatementTracesToSource` found absent —
  a command through `nomos-spec-validate`'s own mechanism, not a text edit the editor
  performs by pattern-matching the rule's message;
- replacing a leaked vendor type — a command through whichever canonical-service boundary
  the type should have crossed, per `OD-CONNECTOR-001`'s vocabulary for that class of defect;
- opening a correction item for a finding — a command through the work group's seam,
  constructing a `WorkCommand::Add` with `ItemKind::Correction`, not a locally-authored
  ledger edit.

**Explicitly not in scope:** this does not reimplement completion, type information or
responsiveness that a language server already provides. Symbol resolution, go-to-definition
for ordinary code references, incremental reparse on keystroke, and the positional
correlation between a byte offset and a line/column are a language server's job, done by a
language server, and an editor integration for this architecture composes with one rather
than replacing it or shadowing its index with a second one computed by this project's own
tooling. Where this record's diagnostic needs a position, it rides on the coordinates the
language server the editor is already running supplies; it does not stand up a parallel
indexer to get one.

## What this does not decide

It does not build the editor, or the orchestration crates families 2, 3, 4, 8 and 9 of
`OD-HOST-002` still lack — `nomos-capability`, the check pipeline, and the record store each
need a seam of `nomos-work-orchestration`'s shape before an editor can call them the way
this record requires, and building any of those crates is not this item's territory. It does
not pick a transport (a language-server custom extension, IPC, an editor plugin's own RPC) —
the same way `OD-HOST-001` left rendering and transport to the adapter, this record leaves
protocol choice to whichever concrete integration is built against the seams once they
exist. It does not design the correlation between `Finding::locations`' repo-relative paths
and a language server's positional coordinates; that is part of the concrete integration,
not of what a diagnostic conceptually carries.

## What would make this wrong

If an editor integration is ever built by shelling out to `nomos check` or `nomos spec
markdown` and parsing what was printed, that is the failure this record exists to name,
whether or not the parsing happens to work today. Equally, if an editor session keeps a
resolved `Finding` set, a capability registry, or a rendered record in memory across calls
instead of asking the owning service again, that is `OD-HOST-002`'s privileged state,
arrived at by the surface this record predicted would be the first to have a reason to keep
it.
