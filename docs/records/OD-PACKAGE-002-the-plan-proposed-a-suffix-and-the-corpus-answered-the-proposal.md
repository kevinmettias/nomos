---
id: OD-PACKAGE-002
type: decision
title: The plan proposed a suffix and the corpus answered the proposal, so four kinds lose it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - packages
  - protocol
  - corpus-authority
  - wire-format
relations:
  - target: D-132
    type: affected_by
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-GATE-002
    type: relates-to
---

# The plan proposed a suffix and the corpus answered the proposal, so four kinds lose it

## The Question

`OD-PACKAGE-001` measured `PackageKind` in `crates/contracts/nomos-contracts/src/package.rs`
while answering a different question, found that four of its sixteen labels carry a `Package`
suffix the corpus taxonomy does not, and deliberately did not act. Its reason was bounded and
good: renaming a protocol surface was outside a versioning item's `done_when`, the evidence
had been weighed once, and there was no consumer to migrate and nobody to notify.

`P10-KIND-TAXONOMY` is the item that weighs it a second time, with nothing else to decide.

The question is narrow and the stakes are not. `PackageKind::Label` is the serialized form.
`nomos-contracts` is band 0 and depends on `serde` and nothing else precisely so that a
knowledge service in OCaml or a client in TypeScript can reimplement these types without
vendoring a Rust crate — the crate's own module doc says so, and
`tests/contract/contracts_names_nothing.rs` enforces it. A peer never sees the variant; it
sees the label. So a divergent label is a divergent wire format, and the only cheap moment to
notice that is while nobody has read it.

## What The Corpus Says

The package taxonomy sentence, quoted in full from line 21 of
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/new docs/03_Packages_Providers_Rules_and_Applicability.md`
(header line 7: `Domain-owned edition v14.20 • 7 August 2026`):

> Package taxonomy - "Plugin" is a user-facing umbrella, not the only architectural kind.
> Installable units shall declare package_kind as one of KernelModule, ServiceModule,
> FeatureModule, LanguagePackage, RulePackage, ToolProvider, MetricProvider,
> RepositoryProvider, RuntimeProvider, ModelBackendPackage, AgentExecutorPackage,
> ClientPackage, IntegrationPackage, SdkPackage, ProjectionPackage, or FeaturePack. New kinds
> require protocol-governed semantics rather than ad hoc loader behavior.

Sixteen kinds. `PackageKind` declares sixteen and its membership already matched: **no kind is
missing and none is extra.** The whole divergence was four spellings.

Two of the four are also named, unsuffixed, by numbered normative requirements. `ARCH-003`,
quoted in full from
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/nomos-spec-internal-artifacts/01_authoring/artifacts/requirements/ARCH-003.md`
(`status: normative`, `authority: canonical-normative-record`, `maturity: accepted`), restated
verbatim at line 35 of the volume:

> ARCH-003 ToolProvider implements a Nomos capability contract and declares guarantees,
> quality rank, supported operations, environment requirements, provenance, and coverage
> behavior.

`ARCH-004`, from `.../01_authoring/artifacts/requirements/ARCH-004.md`, same three front-matter
fields, restated at line 37:

> ARCH-004 MetricProvider emits provider-neutral observations and descriptors; Atlas and gates
> consume the same observations.

And `SDK-001` at line 188 of the volume lists what the Package SDK scaffolds:

> SDK-001 The Package SDK shall scaffold and validate LanguagePackage, RulePackage,
> ToolProvider, MetricProvider, debugger/profiler, client, repository-host, and
> executor-adapter packages.

Measured across the whole v14 artifact tree and every volume available as text, the strings
`ToolProviderPackage`, `MetricProviderPackage`, `RepositoryProviderPackage` and
`RuntimeProviderPackage` occur **zero** times in the corpus.

## The Suffix Is Not Decoration

The taxonomy uses four suffixes and each names what the installable unit *is*: `Module` for
machinery this product itself ships, `Package` for a versioned bundle of declarations and
data, `Provider` for an implementation standing behind a capability contract, and `Pack` for a
manifest that only names other units. `FeaturePack` is not a `FeaturePackPackage` for the same
reason `ToolProvider` is not a `ToolProviderPackage`.

That matters because the enum's divergence has the shape of a tidying instinct — four names
made to match twelve — and tidying is exactly what would produce it again. Appending `Package`
to a provider does not regularize the taxonomy; it moves four kinds into a family whose
semantics they do not have, and a provider's distinction from the thing that ships it is what
volume 03's own composition paragraph at line 45 turns on: "individual parser, semantic,
compiler, formatter, refactoring, test, documentation, and debugger implementations remain
replaceable ToolProviders".

## The Three Plan Copies, And Which One This Enum Followed

Verified by line number in
`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/new docs/nomos full game plan.txt`
(40,278 lines, CRLF, a raw reasoning transcript with no section markers).

**Copy A, lines 16070 to 16091.** Heading `# 9. Recommended package taxonomy`, then "Every
installable package should declare a `package_kind`.", then a fenced block whose sixteen lines
are 16075 to 16090:

> KernelModule / ServiceModule / FeatureModule / LanguagePackage / RulePackage / ToolProvider
> / MetricProvider / RepositoryProvider / RuntimeProvider / ModelBackendPackage /
> AgentExecutorPackage / ClientPackage / IntegrationPackage / SdkPackage / ProjectionPackage /
> FeaturePack

**Copy C, lines 23357 to 23378.** The same heading, the same sentence, the same sixteen names
in the same order, at lines 23362 to 23377.

Both agree with the volume exactly.

**Copy B, lines 20757 to 20779.** Under the heading `## Volume 03 — Packages, Providers,
Rules, and Applicability`, the sentence "This volume should gain a complete package taxonomy",
then the subheading `### Add package kinds`, then a fenced block of **fifteen** lines, 20764 to
20778:

> KernelModule / ServiceModule / FeatureModule / LanguagePackage / RulePackage /
> ToolProviderPackage / MetricProviderPackage / RuntimeProviderPackage / ModelBackendPackage /
> AgentExecutorPackage / ClientPackage / IntegrationPackage / SdkPackage / ProjectionPackage /
> FeaturePack

`RepositoryProvider` absent; `ToolProvider`, `MetricProvider` and `RuntimeProvider` suffixed.

`PackageKind` followed copy B. It has the volume's sixteen members, copy B's three suffixed
spellings, and `RepositoryProviderPackage` — a name that exists in no corpus and no plan copy
at all, invented by extending copy B's suffix to the member copy B was missing.

One correction to `OD-PACKAGE-001`, which recorded that the suffix "appears in no other list
anywhere". `ToolProviderPackage` occurs once more, at line 29569, inside a seven-item
illustrative list contrasting Nomos package kinds with a mod-loader's. It is not a fourth
taxonomy copy and it does not change the count of full lists, but the record said "anywhere"
and the string is somewhere, so it is written down here.

## Why Copy B Is The One With The Least Authority — And Why That Is Not The Whole Argument

`D-132` is enough on its own. The plan is a game plan: lineage, authoritative about intent and
never about the corpus, and where the two disagree the corpus wins and the plan's version is
recorded rather than deleted. That settles a two-to-one split among the plan's own copies in
the same direction the volume already points, and this record records copy B rather than
wishing it away, which is `D-132` rule 1 operating exactly as written.

But there is a stronger fact, and it was not available when `OD-PACKAGE-001` was written
because it needs the artifact tree read alongside the volume.

**Copy B is not a taxonomy. It is an instruction to amend volume 03 — and the amendment
happened, and rejected the suffix.**

The v14 corpus artifact tree carries its own copy of the volume at
`.../nomos-spec-internal-artifacts/01_authoring/domain_volumes/03-packages-providers-rules-and-applicability.md`.
Its header at line 20 reads `Domain-owned edition v14.19 • 4 August 2026`, three days older
than the v14.20 edition quoted above. Its section 4.2 contains `ARCH-001` through `ARCH-006`
and the five composition paragraphs, and it contains **none** of this:

| In v14.20 section 4.2 | In v14.19 | Asked for by the plan |
|---|---|---|
| The `package_kind` taxonomy sentence, line 21 | absent | `### Add package kinds`, line 20761 |
| `PKG-MOD-001`, line 25 | absent | "### Add install-time composition requirements", line 20823 |
| `PKG-MOD-002` (lazy activation), line 27 | absent | same list, "packages being lazily activated" |
| `PKG-MOD-003` (FeaturePacks carry only metadata), line 29 | absent | same list, "no feature pack becoming an indivisible monolith" |
| `ARCH-006A`, line 43 | absent | — |
| `INT-MOD-002` (package lifecycle states), line 170 | absent | "```text Available / Downloaded / …```", line 20806 |
| `SDK-MOD-001` (narrow publishable libraries), line 186 | absent | "### Add SDK/embedded use", line 20838 |

None of `PKG-MOD-001`, `PKG-MOD-002`, `PKG-MOD-003`, `ARCH-006A`, `INT-MOD-002` or
`SDK-MOD-001` has a requirement artifact under `01_authoring/artifacts/requirements/`, and the
string `package_kind` appears nowhere in `01_authoring/` at all. They arrived together, in the
v14.20 edition, and they are the section of the plan that begins "This volume should gain a
complete package taxonomy."

So copy B is a proposal and volume 03 line 21 is its adjudication. The editor who executed the
proposal restored `RepositoryProvider` and dropped all three suffixes. That editor also
renamed the proposal elsewhere in the same pass — the plan's package states are `Available`,
`DependencyUnavailable`, `RemovalPending`, `Withdrawn`; `INT-MOD-002` landed as
`NotInstalled`, `DependencyMissing`, `RemovedRetainingConfiguration`, and no `Withdrawn` at
all. Volume 03 renaming what the plan proposed is a pattern, not an accident, and following a
proposal past the document that answered it is the specific mistake `PackageKind` made.

## What Bounds This

Stated because `OD-PACKAGE-001` was right to state it and the facts have not all improved.

The taxonomy sentence is still volume prose. The artifact-tree volume declares
`authority: canonical-explanatory-narrative`, not `canonical-normative-record`, and no
requirement artifact carries `package_kind`. It is a `shall` in a governed volume rather than a
numbered requirement, which is a real tier below `ARCH-001` and `ARCH-002`.

**And two of the four kinds have no normative attestation anywhere.** `RepositoryProvider` and
`RuntimeProvider` occur in the entire v14 artifact tree **zero** times, in any spelling. Their
only corpus attestation is the taxonomy sentence at line 21, plus the two agreeing plan copies.
`ToolProvider` and `MetricProvider` are different: `ARCH-003` and `ARCH-004` are normative,
accepted, `canonical-normative-record`, and spell them unsuffixed.

That asymmetry is recorded rather than smoothed. It does not change the outcome — for all four,
every source that names them at all names them unsuffixed, and there is no source anywhere for
the suffix except a superseded proposal — but a later reader asking "how firmly is
`RepositoryProvider` fixed?" deserves the honest answer, which is: by one narrative sentence
in one volume and two copies of a reasoning transcript, and by nothing normative. If a numbered
requirement ever names that kind differently, this record is the thing it supersedes, and the
test below is where the argument has to be had.

What has changed since `OD-PACKAGE-001` is the other half of its reasoning. It declined to act
partly because the evidence had been weighed once, in passing, by an item about versioning.
This item's whole `done_when` is this question, the sources were re-read rather than
re-cited, and the amendment finding above is new. There is still no consumer to migrate and
nobody to notify, and that is now an argument for acting rather than against it: the labels are
free to correct today and pinned by whoever reads them first.

## The Decision — Accepted

**Four variants and four labels are renamed to the corpus's spelling**, in
`crates/contracts/nomos-contracts/src/package.rs`:

| Was | Is | Label constant |
|---|---|---|
| `ToolProviderPackage` | `ToolProvider` | `TOOL_PROVIDER_LABEL` |
| `MetricProviderPackage` | `MetricProvider` | `METRIC_PROVIDER_LABEL` |
| `RepositoryProviderPackage` | `RepositoryProvider` | `REPOSITORY_PROVIDER_LABEL` |
| `RuntimeProviderPackage` | `RuntimeProvider` | `RUNTIME_PROVIDER_LABEL` |

Both halves, because the variant is what this workspace reads and the label is what a peer
reads, and renaming only the variant would leave the wire format exactly as wrong while
looking repaired.

Membership, order and the other twelve spellings are unchanged, because they were already the
corpus's.

`tests/contract/surface/nomos-contracts.txt` moves with the rename: four lines, re-blessed with
`NOMOS_SURFACE_BLESS=nomos-contracts` so no other crate's snapshot was touched, which is the
scoping the surface check acquired for exactly this reason.

**Nothing gains a consumer.** `PackageKind` having none is `OD-PACKAGE-001`'s subject and stays
open there; a caller invented to discharge a spelling item would be worse than the spelling.
The doc comment that states the condition is kept and corrected: it used to point at
`OD-PACKAGE-001` for a divergence that no longer exists, and now points here for its
settlement while leaving the absent-consumer condition exactly as `OD-PACKAGE-001` wrote it.

## What Holds It

A table in `package.rs`'s test module, `CORPUS_TAXONOMY`, carries the sixteen kinds in the
order volume 03 names them, each paired with the label it must serialize as. It is a
transcription of line 21 and its doc comment says so, so a row is a claim about the corpus
rather than a local preference.

`Test_Every_Kind_Should_Carry_The_Label_The_Corpus_Names` walks it and asserts two things per
kind: that `Label()` returns the transcribed label, and that the kind's position matches the
table's. Relabelling is caught by the first, reordering by the second, and the failure message
names the kind with `Debug` rather than `Display` — `Display` delegates to `Label`, so a
message built from it would print the wrong spelling and name nothing.
`Test_Display_Should_Render_The_Serialized_Label` closes that loop from the other side, because
two spellings of one protocol commitment in circulation is the same defect one level down.

A seventeenth kind is held by the compiler, not by an assertion. `Corpus_Position` is an
exhaustive match, so a new variant makes the test module fail to build, next to the table and
the citation; `Label`'s own match refuses it first, in the library. Observed: adding one
variant produced `error[E0004]: non-exhaustive patterns:
PackageKind::SeventeenthKind not covered` at both sites. That is the mechanism, and its limit
is stated plainly — it forces the author to arrive at the table, it cannot force them to argue
honestly once there. Nothing in a test can.

Negative control, run and restored: setting `METRIC_PROVIDER_LABEL` back to
`"MetricProviderPackage"` fails two tests with

> assertion `left == right` failed: PackageKind::MetricProvider serializes as a label the
> corpus taxonomy does not name; OD-PACKAGE-002 fixes the sixteen
>   left: "MetricProviderPackage"
>  right: "MetricProvider"

## What This Is Not

**Not a naming preference.** If it were, it would not be worth a record. It is a wire format,
and the argument would be identical if the corpus's spelling were the uglier one.

**Not a claim that the enum was wrong about membership.** It was not. Sixteen kinds, the
corpus's sixteen, in the corpus's order. Reporting a suffix as though it were a missing kind
would have been the more alarming and less true finding.

**Not a licence to edit the plan.** `D-132` is explicit: the plan is the approved statement of
intent and stays as written. Copy B is quoted above rather than corrected, and the reason it
lost is recorded rather than assumed.

**Not a settlement of what happens when a manifest arrives.** `PKG-022` requires machine-
readable manifests and nothing in this workspace reads one. When something does, it will pin
these sixteen labels, and that is the item where a wrong one would first cost something — which
is why this was the last moment it was free.
