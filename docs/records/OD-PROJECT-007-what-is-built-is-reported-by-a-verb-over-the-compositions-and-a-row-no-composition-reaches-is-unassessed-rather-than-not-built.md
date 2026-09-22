---
id: OD-PROJECT-007
type: decision
title: What is built is reported by a verb over the compositions, and a row no composition reaches is unassessed rather than not built
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - projection
  - documentation
  - capability
  - completeness
  - host
relations:
  - target: OD-PROJECT-001
    type: relates-to
  - target: OD-PROJECT-002
    type: relates-to
  - target: OD-PROJECT-003
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-TRACE-002
    type: relates-to
  - target: OD-TRACE-005
    type: relates-to
  - target: OD-GATE-020
    type: relates-to
  - target: OD-HOST-007
    type: relates-to
  - target: OD-HOST-014
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-AGENT-004
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
---

# What is built is reported by a verb over the compositions, and a row no composition reaches is unassessed rather than not built

## Question

Two external reviews of this repository, at `80e0bbbe` and `bc0aaacf`, each reconstructed by
hand a table of what Nomos has built: one row per product capability, one word per row —
built, partial, first increment, designed-but-not-built, not built. The second asked that the
table come from Nomos itself. Nothing here produces it. `nomos gate plan` prints the rule
registry, `nomos spec profiles` prints the render catalogue, `README.md` carries the zone table
under `OD-PROJECT-001`, and `tests/contract/tests/requirement_trace/` counts committed
assessments. Each is a fragment, each is a different shape, and a reviewer sums them by eye.

The cost is not that the table is missing. It is that every review re-derives it and the
derivations disagree, and that a hand-written "Not built" is an inference from silence —
which `OD-COMPLETENESS-001` already refuses one scale down, for a guard that passes by not
looking.

Three questions, then. Which of the review's rows could a mechanism in this workspace derive
today, and from which source. Where a derived view lives, given that every candidate placement
is already governed by a record with a stated criterion. And what a row may honestly say on a
runner that has no corpus, which is every runner CI has.

## What Was Measured

Measured 2026-09-21 at `8338c6ec`, by reading the compositions and running the committed
binary read-only from the repository root. Where a figure needs the v14 corpus it says so and
names the revision it was read at; nothing else here depends on a corpus. Every count the
item that opened this question carried was re-measured rather than copied, and four of them had
moved: fifty-six composed rules are seventy-one, sixteen capability identifiers are fourteen
declared contracts, five render profiles are nineteen, and `OFFERINGS` no longer exists.

### The registry `nomos-check-orchestration` composes

`Registered()` in `crates/orchestration/nomos-check-orchestration/src/composition.rs`
declares **fourteen** capability contracts and offers **seventeen** providers against them,
pinned by `Test_Registered_Should_Declare_Every_Composed_Capability`. `Registry::Declared()`
and `Registry::Offers()` are public, and `Resolved_Configuration` already walks both to digest
the run's effective policy — so every row below is one the binary can already enumerate, and
no verb prints.

| Capability | Ceiling (variant, soundness, completeness) | Offers, with the guarantee each declares |
|---|---|---|
| `nomos.cap.syntax.items` | Syntactic, Sound, Sound | `nomos-lang-rust` Syntactic Sound Unknown; `nomos-lang-rust-scan` Approximate Unsound Unknown; `nomos-lang-go` Syntactic Sound Sound |
| `nomos.cap.dependency.edges` | SemanticallyResolved, Sound, Sound | `nomos-lang-rust-cargo` at the ceiling; `nomos-lang-go-modules` SemanticallyResolved Sound Unknown |
| `nomos.cap.controlflow.reachability` | SemanticallyResolved, Sound, Sound | `nomos-lang-rust` reachability Syntactic Sound Unsound |
| `nomos.cap.lint.diagnostics` | SemanticallyResolved, Sound, Sound | `nomos-lang-rust-clippy` SemanticallyResolved Sound Unknown |
| `nomos.cap.dependency.policy` | SemanticallyResolved, Sound, Sound | `nomos-lang-rust-deny` SemanticallyResolved Sound Unknown |
| `nomos.cap.naming.policy` | Syntactic, Sound, Sound | `nomos-repo-policy` naming, at the ceiling |
| `nomos.cap.limits.policy` | Syntactic, Sound, Sound | `nomos-repo-policy` limits, at the ceiling |
| `nomos.cap.scripting.policy` | Syntactic, Sound, Sound | `nomos-repo-policy` scripting, at the ceiling |
| `nomos.cap.goals.policy` | Syntactic, Sound, Sound | `nomos-repo-policy` goals, at the ceiling |
| `nomos.cap.words.policy` | Syntactic, Sound, Sound | `nomos-repo-policy` words, at the ceiling |
| `nomos.cap.test.material.policy` | Syntactic, Sound, Sound | `nomos-repo-policy` test_material, at the ceiling |
| `nomos.cap.architecture.declaration` | Syntactic, Sound, Sound | `nomos-repo-policy` architecture, at the ceiling |
| `nomos.cap.review.finding` | RuntimeObserved, Sound, Sound | `nomos-connector-coderabbit` RuntimeObserved Sound Unknown |
| `nomos.cap.requirement.trace` | Syntactic, Sound, Sound | `nomos-cap-requirement-trace`, at the ceiling |

Ten offers sit at their contract's ceiling on all three assurance axes and seven sit below it
on at least one. That is not a defect column: `OD-CAPABILITY-002` decided a ceiling that
restates the incumbent's guarantee is a ceiling that silently forbids a better second provider,
so a gap between offer and ceiling is the contract working. It is recorded because the
review's adjectives came from here without saying so — "read-only Observed" against the
CodeRabbit row is `FactVariant::RuntimeObserved`, printed from the offer.

**Twenty `Provider_Offer` functions are exported and seventeen are composed.** The three that
are not: `nomos.cap.rust.copy_clones` and `nomos.cap.rust.nested_locks`, both declared and
offered by `crates/languages/nomos-lang-rust-compiler` — this workspace's one compiler-backed
provider, over `ra_ap_hir` — and `nomos.cap.module.index`, declared by `nomos-lang-rust`'s
rollup. All three are composed only under `tests/integration`. No rule in `DESCRIPTORS` requires
any of them, which is the same fact `OD-ANALYSIS-007` records as its open trigger. The review
wrote "Compiler semantics: Partial, Rust-specific" for this row; the mechanism says something
more exact — two contracts declared by a member, offered by it, and composed into no run.

### The rule table, and the registry that derives from it

`nomos_rules::DESCRIPTORS` carries **seventy-one** rules. `nomos gate plan` prints exactly
those seventy-one with the authority each cites, and the citation classes split fifty-nine
`code-standards v0`, one `README.md v0`, and eleven versioned records (`D-134`,
`OD-CAPABILITY-010`, `OD-CAPABILITY-016`, `OD-RULES-003` twice, `OD-RULES-008`,
`OD-RULES-010` three times, `OD-RULES-023`, `OD-TRACE-001`). The plan's own usage line says what
it is: "plan composes this gate's rule registry and reports what it holds."

`OFFERINGS`, which the item that opened this question names as a source, was deleted by
`P52-COMPOSED-RULES-BECOME-DECLARATIONS-3`; `OD-GATE-020`'s own amendment records the deletion.
`nomos-gate-orchestration::Registered()` now types no row of its own and derives every offer
from `DESCRIPTORS`, and `Test_Registered_Should_Offer_Every_Composed_Rule` holds it against
`Composed_Rules`, as `Test_Every_Composed_Rule_Should_Have_A_Descriptor` holds the descriptor
table from the other side. The rule half of the view therefore already exists as a verb, is
derived rather than typed, and is the shape the rest of this record follows.

### The package manifest crates

Six under `crates/packages`. Three carry a `KNOWN_PROVIDERS` allowlist and each entry is a
composed offer: `nomos-lang-rust-package` names `nomos-lang-rust` and `nomos-lang-rust-scan`,
`nomos-lang-go-package` names `nomos-lang-go`, `nomos-tool-package` names
`nomos-lang-rust-clippy` under `Family::Linter` and `nomos-lang-rust-deny` under
`Family::PackageManager`. `nomos-model-package` and `nomos-rule-package` enumerate no provider
by construction — a model backend does not register one, and a rule package names a
capability rather than a tool. `nomos-package` is the generic core.

All three allowlists stand in the `UNIVERSES` table under
`tests/contract/tests/completeness_universes/table.rs` as `Unmirrored`, three of the twelve
holes `Test_The_Number_Of_Unmirrored_Universes_Should_Be_Declared` counts, and `nomos check`
reports each as an advisory over this tree. `Check_Package_Conformance` — the check that would
compare a manifest's declared providers against the registry — is exported and, in its own
module doc, "Nothing calls this from a real host today."

`Family` closes twelve tool-family labels. Two are occupied by a known provider and ten are
not. A label with no registrant is a vocabulary term, not an expectation; the last section says
why that distinction decides what a row may print.

### What the hosts serve

`nomos-api` exports thirty `Handle_*` functions. `ServedMethod::REGISTRY` names five —
`GatePlan`, `GateRun`, `GateExplain`, `GateCompare`, `Correction` — `ServedTool::REGISTRY` in
`nomos-mcp` projects the same five, and `ADMITTED` in
`tests/contract/tests/boundaries/transport_registry.rs` quantifies that list over the real
exported surface. `Test_The_Transport_Should_Name_No_Repo_Tooling_Handler` and
`Test_The_Tool_Registry_Should_Name_The_Same_Operations_As_The_Served_Method_Registry` are the
mirrors the `UNIVERSES` table claims for both. Of the twenty-five unserved handlers, twenty-one
are excluded by `OD-HOST-007` and four are decided by `OD-HOST-014`; the fifth product
operation that record admits, `Check_Run`, is admitted and not yet served.

Three binaries: `nomos` with eight verb groups (`work`, `spec`, `check`, `request`, `gate`,
`agent`, `correct`, `workflow`), `nomos-mcp`, and `nomos-lsp`. `nomos agent execute` names
one real `AgentExecutor` (`claude-code`) and one real `ModelBackend` (`ollama`) in its own usage
text; `nomos workflow run` dispatches exactly one step over five bodies.

### The render catalogue and the README

`SHIPPED` in `crates/spec/nomos-spec-project/src/catalogue.rs` carries **nineteen** profiles,
mirrored by `Test_Every_Profile_File_Should_Be_Shipped`; `nomos spec profiles` prints them.
`OD-PROJECT-002` measured four of the then-eighteen rendering over a store seeded only from
this repository's records; `relation-families` arrived later and is not re-measured here. Two
are required and committed. All nineteen select from the store's ten content kinds and nothing
else — none reads a registry, a descriptor table or an assessment.

`README.md`'s zone table has **sixty-eight** rows for sixty-eight workspace members across
twelve zones — fourteen `Capability Contract`, fifteen `Provider`, six `Application Service`,
five `Host`, four `Agent`, three `Repo Tooling` — and
`Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band` compares it both ways against
the declared architecture. It is the README's one owned region. It says a crate exists and
which zone it sits in; it does not say whether anything composes the crate.
`nomos-lang-rust-compiler` has a row and no offer in any run, and the row cannot tell a reader
that.

Two hundred and fifty-four records are registered and two hundred and fifty-four are on disk.

### The committed requirement assessments

**Thirty-nine** files under `tests/contract/requirements/`, read by
`tests/contract/tests/requirement_trace/` and by the `nomos.cap.requirement.trace` provider.
Verdicts: seven `Met`, twenty-nine `Partial`, three `Diverges`, no `NotBinding`. `Partial` is
`OD-TRACE-003`'s verdict, absent from the four `OD-TRACE-001` named, and every `Partial` names a
gap, held by `Test_Every_Partial_Should_Name_A_Gap`. The identifier prefix is the corpus
family, and nine families have an entry:

| Family | Assessed | Met | Partial | Diverges | Family size in the corpus | Unassessed |
|---|---|---|---|---|---|---|
| `AGT` | 15 | 1 | 13 | 1 | 18 | 3 |
| `AGT-EXEC` | 4 | 0 | 4 | 0 | 5 | 1 |
| `CAP` | 2 | 2 | 0 | 0 | 4 | 2 |
| `CHK` | 1 | 1 | 0 | 0 | 7 | 6 |
| `COR-EXEC` | 2 | 2 | 0 | 0 | 8 | 6 |
| `EVID` | 1 | 1 | 0 | 0 | 2 | 1 |
| `MODEL-ROUTE` | 10 | 0 | 10 | 0 | 13 | 3 |
| `WF` | 2 | 0 | 2 | 0 | 11 | 9 |
| `WORK-LEDGER` | 2 | 0 | 0 | 2 | 6 | 4 |

The first five columns are read from this repository. The last two are not: they were counted
under `01_authoring/artifacts/requirements` of the v14 corpus at code-standards `f0d820729`,
which holds **363** requirement files in **61** families — the same 363 and 61 `OD-TRACE-001`
recorded. Fifty-two families have no entry at all, so 289 of the 324 unassessed requirements
belong to families this repository does not name anywhere, and the other 35 sit inside the nine
above. Without the corpus, the `Unassessed` column cannot be filled and the fifty-two families
cannot be listed, because a family with no entry and a family that does not exist are the same
absence. `tests/contract/tests/requirement_trace/main.rs` says this of itself: an assessment is
"never derived from the corpus at check time."

Nothing committed enumerates the sixty-one families. `tests/corpus/families/counts.json` is a
register of ingest content families — table rows, code blocks, domain-model rows — and not of
requirement families.

### What no source reaches

The review's rows named Architecture inference, Program semantics, Complexity, Atlas,
Feature/test intelligence and Integration materialization, each as not built or designed but
not built. Measured against every declared list above: no capability contract, descriptor,
allowlist, served registry, profile, README row or assessment carries any of those names.
`ARC-ROADMAP-001` names four of them in its deferred tier — "architecture discovery/inference",
"feature topology and path tracing", "test intelligence built on those models", "Atlas" — as
prose inside a governing record, which the store holds as blocks and no mechanism reads as a
list. The corpus families that carry them (`ATL`, `ARC-AN`, `FEAT`, `TEST`, `MET-COMP`) exist
only where the corpus is mounted. So every one of those rows was a person's mapping from a
product name to an absence, and the review was right about each of them by knowing the
repository rather than by reading a source.

## The Decision

### 1. The view is a host verb over the compositions, in `gate plan`'s shape

Each placement the question lists is decided by the criterion its own governing record already
states, and three of the four fail it.

**Not a spec render profile.** `OD-PROJECT-001`'s criterion: "A file is a projection output
exactly when everything it asserts is in the specification store." A capability, an offer, a
descriptor, an allowlist and an assessment are none of the store's ten content kinds; they are
facts about `Cargo.toml`, the composition roots and `tests/contract/requirements/`. Rendering
them would be the "second system wearing the projection system's name" that record refused,
and putting them in the store to justify the profile would be inventing corpus content to
satisfy a file format.

**Not a committed projection with a sidecar.** `OD-PROJECT-003` measured that "`--require` only
changes what happens when an output is *absent*", and that a stale committed output fails the
freshness step whether or not it was required. The sidecar the step reads is store-fed — its
`inputs` are store rows hashed by identity — so a workspace-fed body would need a second
freshness mechanism, which is a second authority for the one question `spec freshness`
answers. And the view changes on every commit that adds a capability, a rule, a provider or an
assessment, which is most weeks' commits; that is `OD-PROJECT-003`'s tax paid on the most
volatile artifact in the tree.

**Not a second owned README region.** `OD-AGENT-004`'s rule, as `tests/contract/tests/
boundaries/readme.rs` restates it: "enumerate a compiled vocabulary only where a test compares
that enumeration against its authority, otherwise route". A checked table is permitted, and a
checked table of fourteen contracts, seventeen offers, seventy-one rules, five served methods
and thirty-nine verdicts is a hundred-odd hand-typed cells that a test would make every
capability commit re-edit. The zone table earns that cost because membership changes rarely
and a member's zone is a decision with nothing in the source to infer it from. Every cell of
this view is in the source. `Test_The_Readme_Should_Not_Relist_A_Vocabulary_It_Routes_To` is the
test that would then be arguing with the region.

**A verb, derived at run time, never a copy.** `gate plan` already is this for the rule half:
it composes the registry and reports it, and `OD-GATE-020` records what happened when the
same list was maintained by hand instead — parity went false silently three times and failed
at the number the record named. The verb sits beside `Plan` in `nomos-gate-orchestration`,
reads `nomos_check_orchestration::Registered()`, `nomos_rules::DESCRIPTORS`, each package
crate's `KNOWN_PROVIDERS`, `ServedMethod::REGISTRY` and the requirement-trace registry reader
over the root it is given, and types no list of its own. Its output is line-oriented like
`gate plan`'s, so a reviewer runs the binary and reads the answer, which is what the second
review asked for.

The verb has two halves and labels them. **What this binary composes** — the contracts, offers,
rules, allowlists and served methods — is true wherever the binary runs and needs no tree.
**What this tree declares** — the assessments — is true of the root the verb was handed. A
line that does not say which half it belongs to is a line a reader will carry to the wrong
repository.

Under `OD-HOST-014`'s criterion — "An operation that reads the tree it is given, or writes
inside it, is admitted" — the verb is admissible over the transport. Serving it is a separate
item, judged there, and this record does not admit it.

### 2. A row says one of four things, each with a mechanical witness

The verb prints, for every name a source carries, exactly one of these, and never a fifth:

- **built** — a composition witnesses it: an offer in `Registered()`, a descriptor in
  `DESCRIPTORS`, a method in `ServedMethod::REGISTRY`, an entry in a `KNOWN_PROVIDERS`, or an
  assessment reading `Met`. The row carries the guarantee or citation the source declares and
  the verb judges nothing about it.
- **declared and unobserved** — a member declares it as something it provides and no
  composition takes the offer. Today: the three exported `Provider_Offer` functions
  `Registered()` never calls, and `Check_Package_Conformance`. The witness is the export; the
  absence is the composition's.
- **partial** — an assessment reads `Partial`, and the row carries its gap. This is the only
  source whose own vocabulary contains the word, and the verb never manufactures it: an offer
  below its ceiling is built at that guarantee, and five served methods of thirty handlers is
  a count, not a partial.
- **unassessed** — no source carries the name. This replaces "not built", which the verb
  cannot print: "not built" is `OD-COMPLETENESS-001`'s inference from silence, and that record's
  rule is that "a declared universe must have a check comparing it against the reality it
  claims to enumerate." There is no declared universe of product capabilities here, so there is
  no reality against which "not built" could be checked.

`Diverges` and `NotBinding` are printed as the verdict words, with the governing record each
names, because `OD-TRACE-002` already chose those words and a second spelling would be a second
place for them to be wrong.

Two things the verb does not do with these states, because each would be a claim of coverage
the source does not make. An unserved `Handle_*` is not "declared and unobserved": a handler is
an export, not an expectation of service, and twenty-one of them are unserved by decision.
An unoccupied `Family` label is not either: a closed vocabulary names what a registrant may
say, not what one is expected to exist for.

### 3. What a row says when the corpus is absent, which is what CI has

Every build-time row is unchanged; none of it ever needed a corpus. Every assessment row
prints what is committed — verdict, sites, gap, record — and every family row prints its
counts of `Met`, `Partial`, `Diverges` and `NotBinding`. The `Unassessed` column prints as
**uncounted**, in the shape `OD-TRACE-005` gave a run that cannot look: a state the run reports
as its result, never a zero and never a silence.

**The verb never reads the corpus, mounted or not.** `OD-TRACE-001` drew the line: "the corpus
is required to *author* an entry and to *re-verify* one. It must not be required to *run the
guard*." A verb whose output changes with an environment variable is a verb whose CI answer and
local answer differ, which is the one property this view exists to end. The count of what
nobody has looked at, per family, is a corpus-side fact, and `OD-TRACE-005` already decided
where corpus-side facts about assessments are computed: the corpus-gated suite under
`tests/integration`, declared in `corpus_gates.rs`. That suite may print "assessed n of N" per
family and list the families with no entry; the verb may not, and the sixty-one family sizes
are never committed to make it possible — a committed size list would be a declared universe
whose only mirror is corpus-gated, which is `OD-GATE-001`'s hole with a new name.

So on CI a family row reads, for example, `AGT: 15 assessed — Met 1, Partial 13, Diverges 1;
unassessed: uncounted`. The table in this record is what that row becomes when a person
mounts the corpus and runs the integration suite, and it is dated for that reason.

### 4. The verb reproduces no review table, and the mapping stays a person's

The verb emits rows for names its sources carry: capability identifiers, provider identities,
rule identifiers, package allowlists, served operations, requirement identifiers and their
families. It emits no row for "Atlas", "LSP navigation", "Compiler semantics" or any other
product name, because no source carries one and a mapping compiled into the verb would be the
hand-written table this record exists to retire, one indirection further from the reader.

A reviewer who wants the review's shape writes the mapping — this product name is these
capability rows — and every cell the mapping points at is then a line the verb printed, checkable
by running it. What stays a judgment is the mapping, which is the smallest thing that can stay
one. The three rows this record could not reach any source for are the rows where the mapping
points at nothing, and the honest cell there is "unassessed", not "not built".

### 5. The territory a building item would reserve

Stated so the item is authored against this record rather than by re-deriving it. Territory
follows the predicate's dependency cone: a verb beside `Plan` moves `nomos-gate-orchestration`'s
public surface, so `tests/contract/surface/nomos-gate-orchestration.txt` is territory with the
crate; the CLI renderer is `crates/host/nomos-cli/src/gate.rs` and `crates/host/nomos-cli/src/gate/`,
which has no snapshot because it is a binary; the API handler is `crates/host/nomos-api/src/lib.rs`,
one new file under `crates/host/nomos-api/src/response/`, and `tests/contract/surface/nomos-api.txt`.
If the registry census is exported from where `Registered()` lives instead,
`crates/orchestration/nomos-check-orchestration` and its snapshot join. Serving the verb is not
this territory: `OD-HOST-014` names its own three files, and that is a second item.

`README.md` is deliberately not territory. Its Host rows describe the verbs in prose, and
`OD-AGENT-004` says prose routes rather than restates; no table is added.

At the time of writing `crates/host/nomos-api` and its snapshot are held by a live claim on the
board, so a building item queues behind it or lands the orchestration and CLI halves first.

## What This Record Does Not Do

It builds nothing. No verb, no profile, no README row, no assessment, and no change to any
composition root or allowlist.

It does not amend `OD-PROJECT-001`. The README keeps one owned region, and the criterion that
record states is what excluded the second here.

It does not decide the verb's spelling, its group, or its output grammar beyond being
line-oriented and two-halved. `OD-PROJECT-005` left the same things to its building item for the
same reason.

It does not assess a fortieth requirement, backfill a hash, or move a verdict. The family table
above is a measurement, not a registry entry, and `OD-TRACE-002`'s floor is untouched.

It does not decide what the deferred corpus families mean for the roadmap, or whether the two
compiler-backed contracts should now be composed. Both questions are held by items on the board
at the time of writing, and this record's counts will move when they land.

It does not admit the verb over the transport. `OD-HOST-014`'s criterion admits it; admitting is
a separate act with its own three files.

## Controls

| Weakening | What it produces |
|---|---|
| render the view as a spec profile | a renderer over the workspace under the projection system's name, refusing corpus-unset or fed with invented store content |
| commit the verb's output beside a sidecar | a file stale on every capability commit, with no freshness mechanism that can read it, which is `OD-PROJECT-003`'s tax for nothing |
| add the view as a second README table | a hundred-odd hand-typed cells, and a checked-copy rule that makes every capability commit a README edit |
| print "not built" for a name no source reaches | an inference from silence with no universe to check it against |
| commit the sixty-one family sizes so `Unassessed` is a number | a declared universe whose only mirror is corpus-gated, the `OD-GATE-001` hole under a new name |
| let the verb read a mounted corpus | a CI answer and a local answer that differ, for one environment variable |
| print "partial" for an offer below its ceiling | the verb grading providers, which `OD-CAPABILITY-002` reserved to a provider's own tests |
| print "declared and unobserved" for an unserved handler or an unoccupied family label | an export or a vocabulary term reported as a broken expectation, twenty-one of them by decision |
| compile a product-name mapping into the verb | the review's hand-written table, one indirection further from the reader, stale at the next review |

## Status

Accepted. The view is a host verb beside `gate plan`, derived at run time from the
compositions and typed nowhere; a row is built, declared and unobserved, partial, or unassessed,
each with the source that witnesses it, and "not built" is not a word the verb can print. On a
corpus-less runner every build-time row is unchanged and every family row counts what is
committed and reports unassessed as uncounted; the count itself is the corpus-gated suite's to
give and never the verb's. Measured at `8338c6ec`: fourteen contracts, seventeen offers of
twenty exported, seventy-one rules, three allowlists, five served methods of thirty handlers,
nineteen profiles, sixty-eight README rows, thirty-nine assessments in nine of sixty-one
families, and 324 requirements nobody has looked at.
