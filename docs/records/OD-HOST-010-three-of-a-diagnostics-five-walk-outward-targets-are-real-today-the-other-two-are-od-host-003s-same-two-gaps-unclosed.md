---
id: OD-HOST-010
type: decision
title: Three of a diagnostic's five walk-outward targets are real today; the other two are OD-HOST-003's same two gaps, unclosed
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - editor
  - diagnostics
  - lsp
  - architecture
relations:
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-003
    type: relates-to
  - target: OD-CAPABILITY-012
    type: relates-to
  - target: OD-RULES-020
    type: relates-to
  - target: OD-GATE-020
    type: relates-to
---

# Three of a diagnostic's five walk-outward targets are real today; the other two are OD-HOST-003's same two gaps, unclosed

## Question

`P42-LSP-PROJECTION` asked for a language server exposing Nomos findings as diagnostics,
with no analysis logic of its own, where a diagnostic can be walked to its governing rule,
supporting facts and subject, its requirement, its architectural component, and what
correction is available. Five targets, and whether each has a real, mechanical answer
today needed checking directly rather than assumed from the shape of the ask -- the same
discipline `OD-RULES-023` and `OD-RULES-024` already applied to a declared-architecture
rule's own two halves.

This is not the first record to ask the question. `OD-HOST-003` ("An editor surface is a
client of the canonical services, not a parser of the CLI's rendered output") already
described almost this exact diagnostic, before `nomos-check-orchestration` had a caller-
callable `Run`, and named two of its seven listed projections as gaps rather than claims:
"the governing requirement or record... `Finding` carries no direct pointer to a record id
yet" and "the guarantee behind it, where the rule depended on a capability... `Requirement::
minimum` and the `Guarantee` an installed provider actually offered." `OD-HOST-002`'s state
families 4 and 8 are the same two gaps by a different name. What this record adds is a
fresh check, against today's tree, of whether either has closed since -- and a real, partial
build of the three that have.

## What was measured

**The governing rule is real and richly answered.** `nomos_contracts::Finding::rule` is a
`RuleId` directly, and `nomos_rules::DESCRIPTORS`
(`crates/rules/nomos-rules/src/rule_descriptor.rs:184`) gives every composed rule's
`SubjectKind`, its `requires: &[RequiredFact]` (each resolving to a real
`nomos_contracts::CapabilityId` through `RequiredFact::Capability`), and the governing
record or ported-standard citation `OD-GATE-020` put on this table. This is the one target
that was already this well answered before this item, and this record adds nothing to it
beyond building `crates/host/nomos-lsp/src/walk_outward/governing_rule.rs` to read it.

**Supporting facts are not.** `nomos_check_orchestration::CheckOutcome::Judged`
(`crates/orchestration/nomos-check-orchestration/src/examined/check_outcome.rs:28`) carries
exactly `findings`, `examined` and `claim` -- no fact store, no per-finding read trail.
`RunContext.store: &mut MemoryFactStore` is caller-supplied and outlives one `Run` call in
principle (`OD-ANALYSIS-009`'s own first increment let a caller reuse it), but re-deriving
which specific fact backed one finding means reconstructing the exact `FactKey` a rule's own
internal `Reader::Require`/`Require_Any` call resolved -- capability, subject, the
`InputDigest` of whatever bytes the rule hashed, the chosen provider and its `Guarantee`,
and, most of all, the `nomos_capability::Requirement` the rule asked for. `RuleDescriptor`
names the *capability family* a rule reads (`RequiredFact`), never the `Requirement` floor
or the resolved offer -- that is private to each `Check_*` function's own call in
`crates/rules/nomos-rules/src/checks/*.rs`. `nomos_analysis::Reader::On`
(`crates/substrate/nomos-analysis/src/reader.rs:28`) is also built once per `Judged_Findings`
call and shared across every rule in one run (`run_context.rs:590`), so even its own
`Into_Dependencies` trail is a flat list for the whole run, not attributable to one finding.
This is exactly `OD-HOST-003`'s "guarantee behind it" gap, still open: nothing between a
`Finding` and the fact store lets an outside caller ask "which fact, at what guarantee,
backed this."

One instance of the same gap surfaced concretely while building the parts that are ready:
`nomos_cap_lint::LintDiagnostic::line` (`crates/capabilities/nomos-cap-lint/src/payload/
lint_diagnostic.rs:28`) is the real, tool-reported line this item's own `why` names --
but `Check_Lint_Diagnostics`'s own `Finding_For_Diagnostic`
(`crates/rules/nomos-rules/src/checks/lint.rs:135`) files `locations: vec![diagnostic.file.
clone()]`, the file alone, dropping the line the fact it read a moment earlier already
carried. The line survives only inside `summary`'s free text
(`format!("{} [{lint}]: {} ({}:{})", ...)`). This is not a "supporting facts" design gap in
the abstract; it is a live, first-party rule the LSP crate this record's build produces
cannot show a precise line for today, and the fix is one line in `checks/lint.rs`, not in
this record's own territory -- named here as evidence, not corrected here.

**The requirement link does not exist, and `OD-HOST-003` already said it would not.**
`tests/contract/requirements/*.assessment` files (over 300, one per v14-corpus requirement
id such as `AGT-007`, `CHK-003`) are hand-authored prose: a `verdict`, an optional `record`,
and `site:`/`gap:` lines naming `file#Symbol` locations -- never a `RuleId`.
`tests/contract/tests/requirement_trace/main.rs`'s own module doc states the design
directly: an assessment is "a declared entry committed to this repository, compared against
the workspace by this suite, and never derived from the corpus at check time." Grepping
every rule module under `crates/rules/nomos-rules/src/checks/` for a v14 requirement id
(`AGT-\d{3}`, `CHK-\d{3}`) finds exactly two incidental prose mentions, neither a
declared field. There is no reverse index from a `RuleId` to an assessment file, and
building one is not a detail this item's own single-file territory could invent --
`OD-HOST-003`'s own words ("that resolution is part of the seam family 4 and family 8 of
`OD-HOST-002` still need") describe precisely this population, and nothing has closed it
since that record shipped.

**The architectural component is real today, and it was not when `OD-HOST-002` and
`OD-HOST-003` were written.** Neither record mentions a zone: `OD-RULES-020`'s eleven named
zones and `nomos_rules::ZONES` (`crates/rules/nomos-rules/src/checks/dependency/zones.rs`)
postdate both. `Finding::locations` is repo-relative and forward-slashed, and every crate in
this workspace lives at `crates/<zone-directory>/<crate-name>/...`, so a location under
`crates/` resolves to a crate name by the path's third segment, and `Zone_Of(crate_name)`
(a real, already-externally-read public function -- `tests/contract/tests/boundaries/
bands.rs` imports it the same way) answers the zone. This is real and mechanical exactly
when a location is a `crates/...` source path; it is honestly `None` for a package-name
subject (`dependency/violations.rs`'s own `subject_name`), a doc or test path, or a finding
with no location at all (`GOALS_AND_PARTS_LINE_UP`'s workspace-level rule, `checks/
goals.rs:243`, files a fixed declaration-file name that is not a crate path either).
`crates/host/nomos-lsp/src/walk_outward/architectural_component.rs` is this answer, built
and tested against both a Rules-zone and a Host-zone real path.

**A correction is available for two rules, narrowly, and the mechanism is not the one
`OD-HOST-003` imagined.** That record's own scope named "an open `nomos_ledger` item of
`ItemKind::Correction` addressing this finding's subject" as the correction projection --
a ledger-bookkeeping concept. What was actually built since, `P40-CORRECTIONS-CANONICAL-
SEAM` and `P40-CORRECTIONS-SECOND-FAMILY-3`, is a different, more concrete mechanism:
`nomos-correction-orchestration::run::Judged` composes exactly two rules --
`nomos_rules::COMPLETENESS_MIRROR` and `nomos_rules::NO_TRAILING_WHITESPACE` -- and its own
private `Phantom_Claim`/`Trailing_Whitespace_Claim` recognizers turn a matching `Finding`
into a real, staged, previewable `nomos_corrections::CorrectionCandidate`. Neither the two-
rule membership nor a "does rule X have a family" query is exported from that crate's own
`lib.rs` (only `CorrectionCommand`, `CorrectionOutcome`, `CorrectionEnvironment` and
`Run_Correction` are). `crates/host/nomos-lsp/src/walk_outward/available_correction.rs`
declares the two rule ids directly rather than invent an export that crate does not have --
a real, narrow, correct-today answer, and a named judgment call: a third rule added to that
crate's own private `selected` array would not be noticed by this declaration until someone
updates it by hand. The record of that judgment call is this paragraph and the module's own
doc comment, not a silent duplication.

## The decision

**Build the three that are ready; name the two that are not, rather than force either.**
`crates/host/nomos-lsp` is a real crate: `lsp-server` 0.10.0 and `lsp-types` 0.97.0 (neither
previously in `Cargo.lock`; grepped for `tokio`/`async-std` first and found neither
anywhere in this workspace, so the synchronous pairing was the one that introduces no
runtime) over stdio, calling `nomos_check_orchestration::Run` -- the identical seam
`nomos-cli::check` and `nomos-correction-orchestration::run` already call -- with no rule
judgment of its own. `Diagnostics_For` translates each `Finding` into an `lsp_types::
Diagnostic`: severity from `Finding::Can_Fail_A_Build` and `GateCategory`, range from
`Finding::locations`' own `"path:line"` convention (a whole-line span; `range.rs`'s own doc
cites `nomos_cap_lint::LintDiagnostic`'s "no span beyond a line" as the design point that
combination must not cross), and every real walk-outward answer attached at `Diagnostic::
data` -- LSP's own standard per-diagnostic extension point, so this crate invents no second
protocol beside LSP's own. `Diagnostics_For` and the three `WalkOutward` sub-answers are
pure functions with no I/O; `server.rs` is the one impure layer, and it does nothing
`OD-HOST-002` forbids: it holds a `HashSet` of previously-published paths purely so a
finding that clears gets its diagnostic cleared too, which is a cache of an answer `Run`
already gave and can give again, not a fact no other client could reconstruct.

**The other two remain undecided, not declined.** Supporting facts need either
`CheckOutcome` widened to expose a per-finding read trail (a `Reader` built per rule rather
than once per run, and a decision about whether `RuleDescriptor` should also carry each
rule's own `Requirement` floor, not only its capability family), or a narrower `FactKey`-
reconstruction capability nothing in this workspace offers today. The requirement link needs
a real cross-reference from `RuleId` to a v14 requirement id, which is a design question of
the same weight `OD-CAPABILITY-010` and `OD-RULES-010` were, not a detail this item's single-
file territory could invent -- and, per `requirement_trace`'s own module doc, would still
have to answer for a corpus CI cannot read, the same "a green run is not evidence" caution
`OD-TRACE-001` already states for every other requirement-shaped check in this workspace.
Neither is declined: both are real properties a diagnostic could honestly carry once the
missing fact exists, named here rather than improvised inside `crates/host/nomos-lsp` on the
way past.

**This does not reopen `OD-CAPABILITY-012` or `OD-HOST-003`.** `OD-CAPABILITY-012` answered
the opposite direction -- Nomos calling out to a language server as a provider -- and found
`nomos_platform` has no port for a long-lived, bidirectional subprocess. `nomos-lsp` needs no
such port: it is the long-lived process itself, launched once by an editor and speaking over
the stdio pipes it already owns, the same shape `nomos-mcp`'s own binary already has for
MCP. `OD-HOST-003`'s own caution -- "does not reimplement... symbol resolution... composes
with [a language server] rather than replacing it... rides on the coordinates the language
server the editor is already running supplies" -- is about an editor plugin correlating a
subject to a live cursor position, and is not crossed here: `nomos-lsp` performs no symbol
resolution, no completion, and no column-precise span (`range.rs`'s whole-line-only `Range`
is the same "no editor-grade span" ceiling `LintDiagnostic` already holds itself to, carried
forward rather than widened). Two independent LSP servers publishing diagnostics for the
same buffer -- one for language semantics, one for this repository's own norms -- is the
ordinary shape a real editor already composes (a linter's own LSP beside a language
server), not the parallel indexer that record warns against.

## What this record does not do

**No code changes to `nomos-cap-lint`, `nomos-rules`' rule bodies, or `nomos-correction-
orchestration`.** The `LintDiagnostic` line-drop in `checks/lint.rs` is named as found
evidence, not fixed here -- it is a one-line, single-rule defect inside a different item's
territory, and conflating "found while building the LSP crate" with "this record's own
territory" would be the same false widening `OD-GATE-005` already warns a derived
projection's commit against, applied to a decision record instead of a rendered one.

**No fact-provenance capability and no requirement cross-reference are designed here.**
Both are named as real, open questions a future item should claim once someone decides the
shape -- a widened `RuleDescriptor`, a per-rule `Reader`, a `RuleId`-to-requirement-id table
committed the same way an assessment file is -- not answered by naming that they exist.

**It does not withdraw either target.** Both remain real, in the same sense `OD-RULES-024`
kept architecture drift and representation leakage real while declining to build either:
`P42-LSP-PROJECTION`'s own `done_when` asked for five, and this record's honest answer is
three built, two named and still open.

## Follow-up territory, once either gap is decided

A future item closing the fact-provenance gap belongs in `nomos-check-orchestration` (widening
`CheckOutcome` or adding a second, opt-in return shape) and `nomos-rules` (deciding whether
`RuleDescriptor` should carry a rule's own `Requirement` floor) together, not inside
`crates/host/nomos-lsp` -- this crate would then gain one more `walk_outward` module reading
whatever that seam exposes, the same shape its three siblings already have. A future item
closing the requirement-link gap belongs wherever `tests/contract/requirements/*.assessment`'s
own format is decided to carry a `rule:` field (or a sibling table keyed the same way), plus
whatever mechanism keeps that field from silently drifting the way `OD-TRACE-001`'s own
motivating case (`AGT-001`'s stale assessment) already showed a citation can. Neither
follow-up needs to touch `crates/host/nomos-lsp/src/file_diagnostic.rs` itself: `Diagnostics_
For` already builds its `data` payload from `WalkOutward::Of`, and a sixth field there is an
addition to that one function, not a redesign of it.

## Status

Accepted. Three of a diagnostic's five walk-outward targets -- governing rule, architectural
component, and a narrow known-correction-family signal -- are real and shipped in
`crates/host/nomos-lsp`. The other two, supporting facts and the corpus requirement a
finding bears on, are `OD-HOST-003`'s own two named gaps from before this seam existed, and
neither has closed since: both need a real capability or cross-reference decided first, named
here with the follow-up territory each would claim, not built by improvisation inside this
item's own single-file territory.
