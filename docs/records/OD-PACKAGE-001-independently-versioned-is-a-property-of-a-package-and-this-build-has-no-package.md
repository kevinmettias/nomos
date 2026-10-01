---
id: OD-PACKAGE-001
type: decision
title: Independently versioned is a property of a package, and this build has no package
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - packages
  - versioning
  - protocol
  - corpus-authority
relations:
  - target: D-132
    type: affected_by
  - target: D-134
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
---

# Independently versioned is a property of a package, and this build has no package

## The Question

`P10-PACKAGE-SEAM` asked whether this workspace intends to satisfy `ARCH-001` and `ARCH-002`,
and it asked in the strongest form the question has. Both are corpus requirements rather than
plan preferences, and `D-132` puts the corpus above the plan. Neither identifier appeared
anywhere in this repository when the item was written; the only occurrence at HEAD is inside
the item's own `why` in `work/ledger.json`, and this record is the second.

Three measurements were offered as evidence that the seam is declared and unrealized.
`PackageKind` in `crates/contracts/nomos-contracts/src/package.rs` declares sixteen kinds and
has no consumer outside its own crate. `Cargo.toml` sets `publish = false` for the whole
workspace. The two crates the requirements name — `crates/languages/nomos-lang-rust` and
`crates/rules/nomos-rules` — sit beside the engine and neither can be released on its own.

All three measurements hold at HEAD. Only one of them is about the requirement, and it is not
the one about `publish`.

## What The Corpus Actually Says

`ARCH-001`, quoted in full from
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/nomos-spec-internal-artifacts/01_authoring/artifacts/requirements/ARCH-001.md`
(`status: normative`, `authority: canonical-normative-record`, `maturity: accepted`),
restated verbatim at line 31 of
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/new docs/03_Packages_Providers_Rules_and_Applicability.md`:

> ARCH-001 LanguagePackage is an independently versioned registration and translation
> boundary for one language ecosystem. It declares package and language identity, supported
> language versions, file/project recognition, language features, canonical mappings,
> provider registrations, capability claims, runtime/tool requirements, protocol
> compatibility, and executable conformance tests. It shall not own repository policy, rule
> semantics, prompt assembly, or client rendering.

`ARCH-002`, quoted in full from
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/nomos-spec-internal-artifacts/01_authoring/artifacts/requirements/ARCH-002.md`
(same three front-matter fields), restated verbatim at line 33 of the same volume:

> ARCH-002 RulePackage is independently versioned and separates the normative rule contract
> from its implementations. It contains rule identity/version, normative specification,
> applicability semantics, required canonical capabilities, deterministic judgment
> implementations, optional enhanced implementations, external diagnostic mappings,
> correction and suppression contracts, evidence schema, examples/counterexamples,
> conformance fixtures, evaluation corpus, agent-guidance fragments, and
> presentation/protocol metadata.

Read past the first clause of each. The summary that reached this item was *"independently
versioned"*, and the contents lists are what settle the question. Neither list describes a
Rust API. `ARCH-001` describes a declaration: identity, recognition, mappings, registrations,
requirements, compatibility. `ARCH-002` describes a bundle of contract, implementations,
fixtures, corpus and fragments. These are artifacts on disk, not exported symbols.

Three further requirements in the same corpus, all `canonical-normative-record`, say what the
versioning clause means. `PKG-007`, from
`.../01_authoring/artifacts/requirements/PKG-007.md`:

> PKG-007 A LanguagePackage shall distinguish package version, Nomos protocol range,
> supported language/dialect versions, and provider/tool versions. These version domains
> shall not be conflated.

`PKG-022`, from `.../PKG-022.md`:

> PKG-022 LanguagePackage, RulePackage, and ToolProvider package schemas shall publish
> complete machine-readable manifests. The specification and SDK shall include normative
> illustrative manifests demonstrating identity, version domains, protocol compatibility,
> recognition, capabilities, canonical mappings, provider registrations, environment
> requirements, projections, quality objectives, and conformance suites.

`PKG-014`, from `.../PKG-014.md`:

> PKG-014 Judgment implementations, external diagnostic mappings, correction
> implementations, agent guidance, generated documentation, UI metadata, and MCP metadata
> shall be traceable to one RulePackage version and mechanically checked for semantic
> consistency with its normative contract.

And volume 03 line 45, on composition:

> Language package composition - A language package coordinates language knowledge and
> provider discovery; individual parser, semantic, compiler, formatter, refactoring, test,
> documentation, and debugger implementations remain replaceable ToolProviders.

## The Decision — Accepted

**`ARCH-001` and `ARCH-002` bind this build.** They are normative, accepted, and carry
`canonical-normative-record` authority; `D-132` makes the corpus authoritative over the plan
and nothing about this workspace's shape licenses narrowing that. Reading them as not
reaching this build was available and is refused: the thing being built is the thing they
govern, and "the enforcement engine is exempt from the corpus it enforces" is an argument that
would dissolve any requirement inconvenient at the moment it is measured.

**And the workspace does not fail them in the way the third measurement suggests. It fails
them earlier than that: there is no package here at all.**

Measured at HEAD:

- Twenty-two crates, twenty under `crates/` and two under `tests/`. Every one carries
  `version = "0.1.0"` and `publish.workspace = true`; `[workspace.package]` in `Cargo.toml`
  sets `publish = false`.
- No package manifest of any kind exists on disk. Nothing installs, resolves, activates or
  versions an installable unit, because there is no installable unit.
- `PackageKind` is named outside its own module by exactly two things: the re-export at
  `crates/contracts/nomos-contracts/src/lib.rs:67`, and twenty-two lines of
  `tests/contract/surface/nomos-contracts.txt` that exist because it is public.
- `PackageId` is declared at `crates/contracts/nomos-contracts/src/identity.rs:261` and
  re-exported at `lib.rs:64`. It is constructed nowhere in the workspace.

A thing that has not been declared cannot be versioned independently, and cannot be versioned
dependently either. That is a stronger and more useful statement than "the versioning property
is unmet", because it names the missing artifact rather than the missing attribute.

**Neither crate the item nominates is the artifact the requirements name.**
`nomos-lang-rust` is a `ToolProvider` in the corpus's own sense — volume 03's composition
paragraph puts parser implementations on the provider side of the boundary explicitly, and the
crate's module doc describes itself as a provider that declares a guarantee and registers with
`nomos_capability::Registry`. Publishing it would produce a published `ToolProvider`, not a
`LanguagePackage`. `nomos-rules` holds one rule's judgment implementation; its normative
contract exists and lives in `docs/records` as `D-134` (amended at version 2) and
`OD-RULES-001`. So `ARCH-002`'s *separation* is present in substance and absent in mechanism:
`RuleId::New(COMPLETENESS_MIRROR)` in `mirror.rs` carries no version, and nothing ties the
implementation to a contract version, which is exactly the traceability `PKG-014` requires be
*mechanically checked* and `PKG-020` requires be semantically versioned.

## What Independently Versioned Would Mean For A Cargo Workspace, And Why That Is Not It

Stated plainly, because the item asks for it. For a Cargo workspace it would mean: each crate
carries its own `version` rather than a uniform one, bumped on its own schedule under its own
compatibility policy, with a declared API stability expectation, downstream compatibility
tests, changelogs and migration notes, and a distribution channel — the game plan's own list,
written about XVPE at
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/new docs/nomos full game plan.txt`
line 20205, "remove `publish = false` from genuinely reusable crates; assign meaningful
independent or coordinated versions".

That is a real and reasonable repair, and it is **neither sufficient nor necessary** for
`ARCH-001` and `ARCH-002`.

Not sufficient. `PKG-007` requires four version domains kept distinct — package version,
protocol range, language/dialect versions, provider/tool versions. A Cargo `version` field is
one number and can carry at most one of them; conflating the rest into it is the thing
`PKG-007` forbids by name. `PKG-022` requires a machine-readable manifest, which a `Cargo.toml`
is not, since it declares dependencies and features and none of identity, recognition,
canonical mappings, provider registrations, quality objectives or conformance suites.
`ARCH-002`'s contents — evaluation corpus, conformance fixtures, examples and counterexamples,
agent-guidance fragments — are data, and a Rust crate can ship them only as opaque bytes that
no version comparison can reason about.

Not necessary. Nomos distributes packages through its own registry: `PKG-MOD-001` in volume 03
gives the Package Registry an installation contract, and `SDK-003` at line 192 of the same
volume gives publishing its own act — "Publishing shall generate manifests, content hashes,
signatures, migration metadata, protocol compatibility, dependencies, permissions, licenses,
and deprecation/replacement information". A `LanguagePackage` can be independently versioned
in that channel while every Rust crate in this tree stays `publish = false`, because
`publish` answers a different question: whether a Rust *library consumer outside this
workspace* exists. Today nobody is that consumer.

So `publish = false` is evidence about intent, and it is honest evidence — about crates.io. It
is not evidence for or against `ARCH-001` and `ARCH-002`, and this record declines to read it
as either. That correction is the substance of this decision.

## What Would Have To Become True

Not what is true now. Four things, in order, and none of them is in `nomos-contracts`:

1. **An installable unit exists that is not a Cargo crate.** A declared manifest, on disk,
   carrying a `PackageId`, a `PackageKind`, and `PKG-007`'s four version domains as four
   fields that cannot be collapsed into one.
2. **Something reads it and refuses what it cannot resolve.** A registry, however small, in
   the shape `nomos-spec-store`'s registration reader already demonstrates: every malformed
   input a refusal and not a skip. This is the step that gives `PackageKind` and `PackageId`
   their first consumer, and it must land *with* the first manifest rather than before it. A
   manifest with no reader would be this item's own finding reproduced one level up.
3. **The one rule this tree has acquires a contract version its implementation cites**, so
   `PKG-014`'s traceability can be checked rather than asserted. `D-134` is already at
   version 2 and `nomos-rules` does not name that number anywhere.
4. **Only then does the `publish` question arise**, separately, answered by whether a Rust
   consumer outside this workspace exists. No crate's `publish` setting is changed by this
   record, and changing one would not advance items 1 through 3.

The first step is item 1 and item 2 together, with `nomos-lang-rust` and
`nomos-lang-rust-scan` as the natural first subject: a `LanguagePackage` that *registers* both
as providers, which is what volume 03's composition paragraph describes, rather than a rename
of either.

## PackageKind Keeps No Consumer, And Says So

No consumer is added. Inventing a caller to discharge an item is the defect this board keeps
opening items about, and a caller invented here would be worse than most: `nomos-contracts` is
band 0, reimplemented by peers that will never compile it, so a fabricated consumer would
teach a reader that the seam is live.

Instead the declaration states its own condition at the site, in the idiom `D-134` established
when a universe was made to declare its mirror next to itself. The doc comment on
`PackageKind` now says that nothing consumes it, that the reason is the absence of any
installable unit rather than a forgotten call site, and what change would end that — a reader
that resolves a declared manifest to a `PackageId`, a `PackageKind` and `PKG-007`'s version
domains. "Unused" would have been a label. A condition is a thing a later reader can check.

## A Spelling Divergence Found While Measuring

Recorded here rather than acted on, because `D-132` rule 1 is that a disagreement between the
plan and the corpus is *recorded, not deleted*.

The taxonomy sentence at line 21 of
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/new docs/03_Packages_Providers_Rules_and_Applicability.md`
names sixteen kinds:

> Installable units shall declare package_kind as one of KernelModule, ServiceModule,
> FeatureModule, LanguagePackage, RulePackage, ToolProvider, MetricProvider,
> RepositoryProvider, RuntimeProvider, ModelBackendPackage, AgentExecutorPackage,
> ClientPackage, IntegrationPackage, SdkPackage, ProjectionPackage, or FeaturePack.

The game plan carries three lists of the same taxonomy, and they do not agree with each other.
Two of them — the fenced blocks at lines 16075 to 16090 and 23362 to 23377 of
`nomos full game plan.txt`, both headed "Every installable package should declare a
`package_kind`" — are the volume's sixteen kinds spelled exactly as the volume spells them.
The third, at lines 20764 to 20778 under "Add package kinds", is an instruction to amend the
volume and lists **fifteen**: `RepositoryProvider` absent, and `ToolProviderPackage`,
`MetricProviderPackage` and `RuntimeProviderPackage` carrying a `Package` suffix that appears
in no other list anywhere.

`PackageKind` follows the third one. It has the volume's sixteen members and that list's
suffixed spellings, and it supplies `RepositoryProviderPackage` by extending the suffix to the
member the third list omits. So the enum's membership came from the corpus, four of its labels
came from the only list in the plan that the plan itself contradicts twice, and `Label` is the
serialized form a non-Rust peer reimplements.

Which spelling is right is not in much doubt: the volume says one thing, the plan says the same
thing in two places out of three, and `D-132` breaks the remaining tie in the corpus's
favour. Two facts bound how far this record goes anyway. The taxonomy sentence has no
requirement artifact — it appears nowhere under `01_authoring/` in the v14 corpus — so it is
volume prose rather than a numbered normative requirement, unlike `ARCH-001`, `ARCH-002`,
`PKG-007`, `PKG-014`, `PKG-020` and `PKG-022`. And the labels have no consumer to migrate.
Renaming four variants here would be a protocol edit, made on narrative-tier evidence weighed
for the first time, by the item whose `done_when` is a decision about versioning, with no peer
to notify and no migration note with an audience. It is left as measured, and it is work for
the item that gives this enum its first consumer — which is the moment a wrong label first
costs something, and the moment there is somebody to tell.

## What This Costs

**Accepting a requirement and taking no step toward it has a price, and it is not zero.**
Every day `PackageKind` sits unconsumed it is a published protocol commitment that a peer may
reimplement, including the four labels above. The divergence recorded here *is* that risk in
miniature: a spelling adopted from the plan, unnoticed until something measured it, cheap only
because nobody had read it yet.

**This record does not make anything true about packages.** It settles which question is open.
The workspace after it has the same twenty-two crates, the same `publish = false`, the same
uniform `0.1.0`, and the same zero packages. What changes is that "the package seam is
unrealized" is replaced by a named missing artifact and a first step that does not begin with
a `Cargo.toml` edit.

**The count of unconsumed protocol declarations does not fall.** `PackageKind` gains a reason
rather than a caller, and `PackageId` gains neither. It sits in the same crate and inside the
same territory, and it is measured here and left alone because `P10-PACKAGE-SEAM` named one
declaration and a second stated reason written in passing would be a second author's sentence
with nobody's argument behind it. It is named so the next item does not have to rediscover it.

## What This Is Not

Not a decision about directory layout. The item is explicit that an earlier reading citing a
required `packages/` directory could not be verified, and this record found nothing to revive
it: the corpus's requirements are about a versioned artifact and its manifest, and say nothing
about where a Rust crate sits on disk.

Not a deferral. A deferral would have to name a date or a trigger and accept that the question
is closed until then. The trigger here is the first installable unit, which nothing schedules,
and calling that a deferral would be a schedule with no owner wearing a decision's clothes.

Not a licence to flip `publish`. That setting is answered by whether an external Rust consumer
exists, it is a wider territory than this item held, and this record's whole argument is that
it was never the measurement `ARCH-001` and `ARCH-002` were asking for.
