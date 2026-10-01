---
id: OD-PACKAGE-003
type: decision
title: An IntegrationPackage materializes a peer's connection to Nomos, not the content that crosses it
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - packages
  - integration
  - ecosystem
  - protocol
relations:
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-PACKAGE-002
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-AGENT-001
    type: relates-to
  - target: D-132
    type: affected_by
---

# An IntegrationPackage materializes a peer's connection to Nomos, not the content that crosses it

## The Question

`PackageKind` in `crates/contracts/nomos-contracts/src/package.rs` declares
`IntegrationPackage` as one of sixteen kinds, with the one-line doc comment "A connection to
a peer system." Nothing else in this workspace says what one contains. `OD-PACKAGE-001`
measured the same enum and found the deeper gap beneath that silence — no package manifest
exists on disk, nothing installs or resolves one, and `PackageKind` has no consumer at all.
That is `OD-PACKAGE-001`'s subject and it stays open there; this record does not reopen it,
does not propose a reader, and changes nothing about when `PackageKind` gains a consumer.

The question here is narrower and does not wait on a manifest reader to be answerable: when
a repository adopts Nomos, what does an `IntegrationPackage` put into that repository? This
repository is the case in hand, because it is itself a repository under Nomos's own
engineering process, and every surface a peer needs to work here — an agent contract file,
a per-agent adapter, procedural skills, a CI gate — already exists, hand-placed, without a
package or an installer having produced any of them.

## What The Corpus Says An IntegrationPackage Is

Volume 02 of the v14 corpus, `03 August 2026` edition
(`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/new docs/02_Core_Architecture_Identity_and_Configuration.md`,
header line 7: `Domain-owned edition v14.20 • 7 August 2026`), line 159, introducing the
subsystem-ownership table:

> Recommended distribution taxonomy: KernelModule for foundational runtime contracts;
> ServiceModule for independently hostable application services; FeatureModule for optional
> product capabilities; ClientPackage for CLI/desktop/web/mobile/IDE surfaces;
> **IntegrationPackage for peers and external systems**; SdkPackage for embedding/authoring;
> provider package families for languages/tools/models/runtimes; and FeaturePack for
> curated dependency manifests.

That sentence has no numbered requirement artifact of its own — like the package-taxonomy
sentence `OD-PACKAGE-002` weighed, it is volume prose rather than a `canonical-normative-record`
requirement — but it draws the boundary this record needs cleanly: `ClientPackage` is a
Nomos-facing surface (a CLI, a desktop app, an IDE panel), `IntegrationPackage` is the
*peer's* side of a connection, and the provider families (`ToolProvider`, `RepositoryProvider`,
`RuntimeProvider`, `ModelBackendPackage`) are the capability implementations standing behind
one, distinct from the connection itself.

Volume 07, same edition
(`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/new docs/07_Clients_Interfaces_Deployment_and_Administration_cross_system_protocol_addressed.md`),
line 463, gives the one worked example the corpus itself provides, describing the
KnowledgeWorkbench composition operations:

> These operations retrieve or publish through the optional KWB IntegrationPackage. They are
> not a document-authoring API for Nomos. KWB owns narrative/design publishing; Nomos exposes
> only policy import/export, source references, context retrieval, and evidence handoff
> required by Nomos workflows.

Narrative tier again, but load-bearing: the corpus's own instance of an `IntegrationPackage`
is the seam to KWB, and it is explicit that the package carries the *handoff* — import/export,
references, retrieval, evidence — and not KWB's own content. That is the same shape this
record finds by reading the repository directly, for a different peer.

`ARCH-006`, quoted in full from
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/nomos-spec-internal-artifacts/01_authoring/artifacts/requirements/ARCH-006.md`
(`status: normative`, `authority: canonical-normative-record`, `maturity: accepted`), restated
verbatim at line 41 of volume 03:

> ARCH-006 Generated docs, support matrices, task-context fragments, skill fragments, and MCP
> metadata are projections of canonical packages, not separate policy sources.

This is the one normative sentence that names the concrete surfaces the item asks about —
skill fragments and MCP metadata among them — and it settles their authorship question before
this record has to invent one: they are *projections*, generated from canonical package
content, not content an installing mechanism authors on its own. `ARCH-005`, same file and
tier, restated at line 39 — "IDE clients remain thin. Analysis and product logic live in the
daemon/application services, not in Lua, TypeScript, Kotlin, or C# plugin code" — states the
same discipline one surface over: the peer-side artifact stays thin, and the substance stays
where it was authored. A narrative-tier companion, line 43 of the same volume (`ARCH-006A`,
which has no requirement artifact of its own under `01_authoring/` and is therefore volume
prose rather than a numbered requirement, the same tier `OD-PACKAGE-002` found the taxonomy
sentence to be), makes the same point about ownership rather than mechanism: "Human-authored
product, feature, architecture, rationale, alternative, research, and historical design
documents are KnowledgeWorkbench publishing artifacts or external repository documents and
shall not be promoted into Nomos package truth."

## This Repository Is The Worked Example

Measured at HEAD, before this item's own change:

| Surface | Present here | Where |
|---|---|---|
| Agent contract file | Yes | `AGENTS.md` |
| Per-agent adapter | Yes, one, for Claude Code | `CLAUDE.md` |
| Procedural skills | Yes, two | `.claude/skills/nomos-task/`, `.claude/skills/nomos-spec-change/` |
| CI gate projection | Yes | `.github/workflows/gate.yml` |
| Subagent definitions | No | — |
| Hooks | No (`.claude/settings.local.json` carries only `permissions`, and is itself personal and untracked per `CLAUDE.md`) | — |
| MCP server registration | No | — |
| Connector configuration | No dedicated surface; the closest analogue is `--corpus` / `NOMOS_V14_CORPUS`, which configures *this repository's own development* rather than a consuming repository's peer connections | `README.md:182` |

The absence of the last three is itself evidence, not a gap in the reading. Nothing installed
them and nothing is missing them: they are optional surfaces this particular adoption does not
need, which is exactly what "optional" has to mean for a materialization list to be honest
about a real repository rather than an idealized one.

Every surface that *is* present was placed by a person, once, and is maintained by hand. None
of the four carries a rendered-projection sidecar the way `spec/domain-specification.md` and
`diagrams/relations.mmd` do — there is no `AGENTS.md.nomos-projection.json`. That is consistent
with `OD-PACKAGE-001`: the same absence of an installer or a reader that record found for the
manifest is true here for every one of these files. `ARCH-006` says what they are supposed to
be — projections of canonical package content — and this repository is not yet built to that
target; it hand-authors what a mature `IntegrationPackage`, rendered through a `ProjectionPackage`,
would generate. That gap is named because it bears on the boundary below, not because closing
it is this record's job.

## What An IntegrationPackage Materializes

| Surface | Placed / merged / declared-only | Owner |
|---|---|---|
| Agent contract file (`AGENTS.md`-shaped) | Placed — one file, Nomos-authored content, generic across peers | Nomos. Software-engineering process and how to act on it safely is exactly `ARC-ECOSYSTEM-001`'s "software-specific reality and the engineering of change." |
| Per-agent adapter (`CLAUDE.md`-shaped) | Placed — one file per peer harness present, thin, routing to the agent contract file rather than restating it | Nomos authors the content; `OD-AGENT-001` is why it stays thin. Which adapters exist is dictated by which peer harnesses are installed, which is the `IntegrationPackage`'s own subject. |
| Procedural skills / task-context / skill fragments | Placed — whole files, at the peer's own directory convention (`.claude/skills/` for Claude Code) | Nomos authors the procedure; `ARCH-006` names skill fragments and task-context fragments as canonical-package projections, so the mature form is a `ProjectionPackage` output that the `IntegrationPackage` places at the peer's path. Hand-authored today for the reason above. |
| Subagent definitions | Declared-only unless the peer harness's own extension model calls for one; then placed the same way as a skill | Same routing as skills. |
| Hooks | Declared-only unless the peer harness's own hook mechanism is present; then merged into that mechanism's own configuration file, carrying only the trigger a Nomos policy names (for example, "run the gate before the harness stops") | The trigger's *content* is Nomos policy; the hook *mechanism* — the engine that fires it — belongs to the peer harness product itself, which is outside all four of `ARC-ECOSYSTEM-001`'s owners. An `IntegrationPackage` places into somebody else's format; it does not become that format's owner. |
| MCP server registration | Declared-only unless an MCP host is present; then placed as the peer-side pointer at a Nomos-hosted gateway | The gateway is Nomos's own — volume 02's ownership table names an `API/CLI/MCP/LSP Gateway` subsystem that exposes "the same application services through versioned structured interfaces." `ARCH-006` names MCP metadata as a projection of canonical packages. The `IntegrationPackage` places the registration; it does not implement the gateway and does not carry the protocol, which belongs to neither Nomos, XVPE, nor KWB — MCP is a peer transport this ecosystem adopts, not one it owns. |
| CI projections (`gate.yml`-shaped) | Placed at the peer CI system's conventional path, wired to run; the artifact's *content* comes from `nomos spec render` profiles, which is `ProjectionPackage`'s mechanism, not the `IntegrationPackage`'s | `ProjectionPackage` renders; `IntegrationPackage` places the rendered result where the peer CI product expects it and triggers it in that product's own idiom. Neither product is the CI system, which sits outside the four the same way an MCP host or a hook engine does. |
| Connector configuration | Declared-only — names which already-existing `RepositoryProvider`, `ToolProvider`, `RuntimeProvider` or `ModelBackendPackage` instances back a given peer connection | The providers implement the capability and are already separate kinds in the taxonomy at line 159 of volume 02 (`"provider package families for languages/tools/models/runtimes"`, distinct from `IntegrationPackage` in the same sentence). The `IntegrationPackage` only wires a peer to a provider it does not itself define. |
| Rules / providers (optional) | Declared-only — may name a `RulePackage` or provider a given peer integration depends on | Content ownership stays with `RulePackage` / the relevant provider kind; naming a dependency is not carrying it. |

The pattern across every row: an `IntegrationPackage` **places or wires**, and where content is
involved, the content was authored or rendered by something else — Nomos's own engineering
process, a `ProjectionPackage`'s render, or a `RulePackage`'s contract. Nothing in this table
has the `IntegrationPackage` originate policy, a rule, a rendering mechanism, or a peer's own
execution semantics. Where none of that applies and the surface is genuinely absent from the
target repository, the correct answer is declared-only: the package records that the surface
exists and what it would carry, without placing a file that has no reader.

## Declaring A Placement Is Not Performing One

The table above says "placed" for a row and stops there, which leaves one question
unanswered: does the `IntegrationPackage` itself write the bytes, or does it say that a write
should happen and hand the description to something else? The distinction matters the moment
a real mechanism exists to do either, so it is worth drawing now rather than after the first
implementation has already picked one by default.

**An `IntegrationPackage` declares a materialization intent — source, target path, the
ownership class `OD-PACKAGE-004` gives that target, and the publication scope `OD-PACKAGE-005`
gives it. It does not itself perform the write.** Atomicity, the conflict handling each
ownership class requires, staging, and rollback are mechanics: they refer to nothing that
knows what a crate, a rule, a gate, or a peer connection is, the same test `ARC-ECOSYSTEM-001`
already applies to every other crossing. A `Composed` file's owned-region check, a
`GeneratedOwned` file's unconditional overwrite, a `UserOwned` file's refusal — none of that
logic differs because the asset happens to belong to an `IntegrationPackage` rather than a
`ProjectionPackage` or any future package kind that materializes something. Reimplementing it
once per package kind is the reuse `D-090` already refuses on cheaper grounds than this one,
and `ARC-ECOSYSTEM-001`'s adopted `D-091`/`D-122` split says where the mechanics belong once
they are built: a generic materializer, mechanism-owned, consuming a materialization intent no
matter which package kind declared it. An `IntegrationPackage` that performed its own writes
would be teaching itself write mechanics the same way a generic execution primitive is never
taught what a crate is — the crossing runs backward.

This changes nothing about `OD-PACKAGE-001`'s four-step path or about the placement table
above; declaring is still the `IntegrationPackage`'s whole job, and the table's "placed" rows
name what gets declared, not a claim about which component ends up calling a filesystem write.
No materializer exists in this workspace today, generic or otherwise, and this record does not
build one — it states the shape a later one would consume so that the first `IntegrationPackage`
implementation is not also where write mechanics get invented ad hoc.

**Left open:** whether the placement table's per-surface rows — agent contract file, per-agent
adapter, skills, hooks, MCP registration, CI projections, connector configuration — should
become a named typed axis (something like an `IntegrationSurfaceKind`) once more than one
`IntegrationPackage` exists to compare, or whether the table itself remains the right shape
indefinitely. Nothing today argues either way; it is a question for whoever builds the second
`IntegrationPackage`; and finds out whether the first one's rows generalize.

## Ownership, Routed Through `ARC-ECOSYSTEM-001`

`ARC-ECOSYSTEM-001` decides ownership by semantics, not by which repository a file sits in,
and gives two governed crossings that this table is an instance of rather than a departure
from:

- **KWB semantic intent → governed projection → Nomos executable contract.** The KWB
  `IntegrationPackage` volume 07 names is exactly this crossing materialized: KWB owns the
  rationale and narrative on its side, and what an `IntegrationPackage` carries across is
  "policy import/export, source references, context retrieval, and evidence handoff" — the
  governed projection, never the rationale itself. That the corpus's own worked example lines
  up with `ARC-ECOSYSTEM-001`'s diagram independently, rather than by this record's
  construction, is evidence for the crossing rather than an application of it.
- **XVPE generic execution primitive ← adapted by ← Nomos-specific service.** A hook engine,
  an MCP host, and a CI runner are generic execution primitives of whatever product hosts
  them — most of the time, a peer entirely outside this ecosystem's four owners, such as a
  particular agent harness or a CI vendor. Where such a primitive is itself XVPE's (a task
  scheduler, a process substrate), the same rule applies: `IntegrationPackage` never teaches
  the generic primitive what a crate, a rule, a gate, or a finding is. It adapts to the
  primitive's own shape and carries Nomos-specific content across that adapter, exactly as
  `D-130`'s single named adapter already does for linking.

The agent contract file and per-agent adapter fall under Nomos's ownership row directly —
"what does the software actually do" and "does an implementation satisfy an executable rule"
are Nomos's questions, and the harness-facing files exist to let an agent act on Nomos's own
authorities correctly, which is `AGENTS.md`'s and `CLAUDE.md`'s entire content today. Nothing
in this record moves either file or claims a different owner for them; it names the owner
they already have.

## What An IntegrationPackage Is Not

**Not a `LanguagePackage`.** A `LanguagePackage` recognizes and canonically maps one language
ecosystem — `ARCH-001`, quoted in `OD-PACKAGE-001`, lists identity, recognition, canonical
mappings, provider registrations. None of that is about a peer harness, a CI system, or an
MCP host; a repository needs a `LanguagePackage` for the languages it is written in and an
`IntegrationPackage` for the peers that work on it, independently and without either
subsuming the other. A Rust repository worked by two different agent harnesses needs one
`LanguagePackage` and two `IntegrationPackage`s.

**Not a `RulePackage`.** `ARCH-002` gives a `RulePackage` a normative contract, judgments,
corrections, fixtures, an evaluation corpus. An `IntegrationPackage` carries none of that: the
"declared-only" rows above name a dependency on a `RulePackage` at most, and never a judgment.
An `IntegrationPackage` that started carrying its own correction logic or its own normative
statements would be exactly the undifferentiated bucket `PackageKind`'s own doc comment warns
against — a `Plugin` in substance, wearing a more specific label.

**Not a `ProjectionPackage`.** This is the boundary the corpus states most directly, in
`ARCH-006`: generated docs, support matrices, task-context fragments, skill fragments, and MCP
metadata are projections of canonical packages, and an `IntegrationPackage` is where several of
those land, not where they are produced. A `ProjectionPackage` answers "what does this
canonical record become"; an `IntegrationPackage` answers "which peer receives it, in what
shape, at what path." Collapsing the two would mean every peer connection re-derives its own
rendering logic instead of sharing one, which is the many-installers outcome the item's own
`why` names as the cost of leaving this undecided.

## What This Costs If Left As Read Today

Nothing changes at HEAD. `AGENTS.md`, `CLAUDE.md`, `.claude/skills/`, and `.github/workflows/gate.yml`
stay exactly what they are: hand-placed files with no installer, no manifest, and no
`IntegrationPackage` behind them, the same condition `OD-PACKAGE-001` found for the whole
package seam. What changes is that a later reader — or a later item building the first
`IntegrationPackage` manifest — has a place-or-decline table to build against instead of
inventing surfaces one at a time, and an ownership routing that does not have to be re-argued
per surface.

## What This Is Not

**Not a settlement of `OD-PACKAGE-001`'s question.** No manifest reader exists after this
record, `PackageKind` gains no consumer, and `PackageId` is still constructed nowhere. This
record answers what an `IntegrationPackage` would contain if one existed; it does not make one
exist, and it does not touch the four-step path `OD-PACKAGE-001` laid out for the seam's first
consumer.

**Not a rename or a repair of `AGENTS.md`, `CLAUDE.md`, or any file this repository already
carries.** Nothing under those paths changes. This record documents their existing ownership
and shape; it does not edit them.

**Not a decision about hooks, MCP hosts, or CI vendors this repository does not use.** Their
rows above are declared-only because the surfaces are declared-only here; adopting one is a
separate, later measurement against whichever peer is actually present, not a conclusion this
record reaches in advance of it.

**Not a claim that XVPE, KWB, or repository tooling must implement anything.** `ARC-ECOSYSTEM-001`
is routed to rather than re-decided, and nothing here schedules a crossing, a crate move, or a
new dependency. The routing states which product's semantics a future surface would answer to
if built, not that it will be built.
