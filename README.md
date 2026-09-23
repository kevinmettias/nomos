# Nomos

A persistent software-engineering intelligence and control plane.

Nomos builds and maintains a canonical, multi-resolution model of software, and uses it
to keep intent, architecture, implementation, runtime evidence, automated
transformations and agent work aligned.

Three things exist.

The coordination substrate every later phase needs — the protocol vocabulary, the
canonical model kernel, the platform port, the work ledger, and the boundary tests that
keep the architecture from eroding.

The specification system: the corpus lives in a database behind a preservation ledger
that makes silent content loss fail rather than pass. It exists because the previous
specification revision destroyed 282 markdown table rows, all 6 code blocks and 132
sections of narrative, and the mechanism that would have caught it was present and never
ran. See `docs/records/ARC-SPECDB-001-the-specification-is-a-database.md`.

And the analysis kernel, which is young: a capability registry whose answer is never a
bare no, a fact store keyed on identity rather than on a workspace snapshot, capability
contracts with independent providers that are able to disagree, and a rules crate whose
rules judge real code and are reachable as `nomos check`.

What is built and what is not is the ledger's answer rather than this paragraph's. Run
`nomos work list`.

## Layout

Crates are ordered into twelve named zones — `OD-RULES-020` replaced the numeric band
table this section used to carry with a declared set of zones and the zones each may
depend on, plus a short, named list of the same-zone edges a real crate needs; a total
order over one number line forced two crates with no real precedence between them to be
given one anyway the moment either legitimately depended on the other, which is why a
crate had to be renumbered more than once for a dependency nobody disputed. All three of
those parts are declared once, as data, in `nomos-architecture.json` at the repository
root — `OD-RULES-003`'s third prerequisite asked for a declared architecture as data
rather than a Rust `const` table, and `OD-RULES-024` removed the last component and crate
name of this workspace from `nomos-rules`, which is where the table used to live.
`nomos-cap-architecture` carries the contract that file answers to, `nomos-repo-policy`'s
`architecture` module is the one provider that reads it, and `tests/contract` reaches it
through `Declared_Architecture` in `boundaries/bands.rs` and asserts against that one
declaration rather than a second copy of its own. What it declares is in the file; a
reader wanting the zones, the lattice or the edge list reads them there, because restating
any of them here would be the second copy this sentence exists to refuse.

Four rows below are marked `[repo tooling]`: `nomos-ledger`, `nomos-work-orchestration`,
`nomos-spec-orchestration` and `nomos-surface-provenance` exist to develop or preserve this
repository, not to answer a question an end-user repository would ask Nomos.
`ARC-ECOSYSTEM-001` names why a shared zone does not by itself mean shared product
ownership, `OD-LEDGER-036` settles the ledger specifically, and `OD-PROJECT-004` decides
that the tools governing this repository are separated from the product they govern. What
that record decides is ownership, not placement: version 1 named a `crates/repo-tooling/`
directory, version 2 retires it as never built and records the zone declaration two
readers judge as the boundary instead, so these four sit exactly where version 1 measured
them and no move is owed. The `nomos-spec-*` family below the main table carries
the identical distinction in its own prose already. The mark here does not move a crate,
does not change a dependency, and is not itself a decision — it only makes a decision other
records already made visible where a reader would otherwise have to infer it from zone
position alone.

| Zone | Crate | Owns |
|---|---|---|
| Protocol | `nomos-contracts` | Protocol truth. What earns a place in it is `OD-CONTRACTS-001`. Depends on `serde` and nothing else. |
| Substrate | `nomos-model` | Subjects, composite identity, evidence, and the `SubjectSet` exclusion primitive. |
| Substrate | `nomos-store` | Content-addressed documents, with one write door per authority. |
| Substrate | `nomos-platform` | Port traits: clock, filesystem, cross-process lock, process launcher, environment. |
| Backend | `nomos-platform-std` | The std implementation of those traits, and the one crate here that really opens a file, reads the clock or starts a process. Its own zone rather than Substrate's so `Permits` can keep a host away from it: a host names a platform through `nomos-composer-std` below, never an implementation of each port. `OD-RULES-028`. |
| Substrate | `nomos-platform-xvpe` | The XVPE implementation of those traits: any launcher here, seen as the surface XVPE's own adapters take. One adapter, not a replacement for the port. |
| Substrate | `nomos-composer-std` | Which backend set a composition root runs on, named once. Selects `nomos-platform-std` and re-exports `nomos-platform`'s port vocabulary beside it, so a host expresses a platform rather than an implementation of each port. `OD-HOST-001` already permits a root to name a concrete provider; this is the thing to name it *through*. A library, never a composition root: it orchestrates nothing and holds no state. |
| Substrate | `nomos-workspace` | Snapshots, build variants, and the single change door. |
| Substrate | `nomos-scope-verification` | `Territory` and `VerificationPredicate` -- the two ledger-agnostic primitives `OD-LEDGER-037` found underneath `nomos-agent-contracts`'s reuse of `nomos-ledger`, moved verbatim to their own crate below both. `nomos-ledger` re-exports both for its own claim/overlap and finish logic, exactly as `nomos-lang-rust-package` re-exports `nomos-package`'s domains (`OD-PACKAGE-007`). |
| Repo Tooling | `nomos-ledger` | `[repo tooling]` Territory-based mutual exclusion over `work/ledger.json`. |
| Substrate | `nomos-capability` | The contract registry whose answer is never a bare no. |
| Substrate | `nomos-analysis` | Fact identity, fact readers, the fact store, and invalidation. |
| Substrate | `nomos-materialization` | The generic materializer `OD-PACKAGE-003` routed the write mechanics to, and the vocabulary a declaring package and that mechanism both name: `MaterializationIntent`, `OwnedRegion`, `OD-PACKAGE-004`'s `OwnershipClass` and `OD-PACKAGE-005`'s `PublicationScope`. Performs a declared intent against a real tree -- never writes `UserOwned`, overwrites `GeneratedOwned` unconditionally, writes only a `Composed` target's declared owned region and refuses when the free region would be lost -- and undoes every write it has already made when a later one fails. Publication scope is carried and reported and gates no write, because `OD-PACKAGE-005` makes it a property of where an asset may travel rather than of whether it may be written. Below every package kind that declares a placement, which is why the four vocabulary types moved out of `nomos-integration-package` once a second party named them (`OD-CAPABILITY-002`); that crate re-exports all four unchanged, exactly as `nomos-ledger` re-exports `nomos-scope-verification`'s two primitives. Names no package kind, crate, rule, gate or peer connection, so a second package kind needs no change here -- and is named as this repository's own rather than under a platform prefix, per `D-138`. |
| Capability Contract | `nomos-cap-syntax` | A capability contract, housed below every provider that offers against it. |
| Capability Contract | `nomos-cap-dependency` | The `nomos.cap.dependency.edges` contract — a workspace member's own first-party dependency edges, resolved by Cargo. Housed below its provider and below the rule that reads it: `nomos-rules` is a real second party from the day it was written, so this did not wait beside `nomos-lang-rust-cargo` the way `nomos.cap.module.index` waits inside `nomos-lang-rust`. |
| Capability Contract | `nomos-cap-controlflow` | The `nomos.cap.controlflow.reachability` contract — whether a control-flow path forward from a fact-read failure reaches a `Finding`. Housed below its provider (`nomos-lang-rust`) and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason `nomos-cap-dependency` did not wait beside its own provider. |
| Capability Contract | `nomos-cap-lint` | The `nomos.cap.lint.diagnostics` contract — one workspace member's own diagnostics from an external lint tool. Housed below its provider (`nomos-lang-rust-clippy`) and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason `nomos-cap-dependency` did not wait beside its own provider. `OD-RULES-010`, the first real `ToolProvider`. |
| Capability Contract | `nomos-cap-dependency-policy` | The `nomos.cap.dependency.policy` contract — the workspace's own bans/licenses/sources verdict from `cargo deny`, over the resolved dependency graph as a whole (`IncrementalGranularity::WholeWorkspace`, unlike `nomos-cap-lint`'s per-member `Project`). Housed below its provider (`nomos-lang-rust-deny`) and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason its four siblings above did not wait beside their own providers. `OD-RULES-010`'s second real `ToolProvider`. |
| Capability Contract | `nomos-cap-naming-policy` | The `nomos.cap.naming.policy` contract — a repository's own declared naming convention, read from `standards.json` as data rather than compiled as a constant, over code-standards' own closed eight-style `Case` vocabulary. Housed below its provider (a future `crates/repository/` crate) and below the rules that read it (`nomos-rules`), the same real-second-party-from-day-one reason its five siblings above did not wait beside their own providers. `OD-RULES-011`. |
| Capability Contract | `nomos-cap-limits-policy` | The `nomos.cap.limits.policy` contract — a repository's own declared numeric thresholds (file-size triggers, parameter-count caps and their like), read from `standards.json` as data rather than compiled as a constant, over scope-qualified rows rather than `nomos-cap-naming-policy`'s closed `Case` vocabulary since a threshold is a bare number, not a value drawn from a fixed style set. Housed below its own future provider and below the rules that read it (`nomos-rules`), the same real-second-party-from-day-one reason its six siblings above did not wait beside their own providers. `OD-RULES-011`'s own threshold-family instance. |
| Capability Contract | `nomos-cap-scripting-policy` | The `nomos.cap.scripting.policy` contract — a repository's own declared tooling language and forbidden script extensions, read from `standards.json` as data rather than compiled as a constant. Unlike its two siblings, its payload is a plain optional scalar plus a list rather than scope-qualified rows: `check-script-discipline`'s own `spec.go` states that refining "which language is my tooling" *by* language would be circular. Housed below its own future provider and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason its seven siblings above did not wait beside their own providers. A third `OD-RULES-011` instance. |
| Capability Contract | `nomos-cap-words-policy` | The `nomos.cap.words.policy` contract — a repository's own additions to code-standards' default approved-abbreviation vocabulary, read from `standards.json`'s `words.approved_abbreviations` as data rather than compiled as a constant. A bare list, simpler than any of its three siblings: a vocabulary addition has no scope to qualify. Removal is not supported — a repository extends the default list, per code-standards' own stated philosophy, rather than replacing it. Housed below its own future provider and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason its eight siblings above did not wait beside their own providers. A fourth `OD-RULES-011` instance. |
| Capability Contract | `nomos-cap-goals-policy` | The `nomos.cap.goals.policy` contract — the purposes a repository declares it exists to serve, the parts it declares to serve them, and the ceiling on how thinly one purpose may be spread, read from `standards.json` as data rather than compiled as a constant. A small graph rather than rows, a scalar or a list: goals, parts, and the edges between them, because the rule's whole question is whether the two sets line up across those edges — which is also why a part that serves nothing gets a line of its own in the encoding, since that is the defect being looked for. Housed below its own future provider and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason its nine siblings above did not wait beside their own providers. A fifth `OD-RULES-011` instance, and the clearest case for that decision yet: code-standards' own `check-goal-traceability` reads nothing but the declaration, so no version of the rule could have compiled its parameters in and still meant anything. |
| Capability Contract | `nomos-cap-test-material-policy` | The `nomos.cap.test.material.policy` contract — a repository's own fixture locations (repository-relative directory prefixes under which its test material lives), read from `nomos-test-material.json` as data rather than compiled as a constant, and composed with each rule's own toolchain-fixed clauses (`tests/`, `examples/`, `_test.go`, …) rather than replacing them. A bare list, simpler than any of its five siblings: a fixture location has no scope to qualify and no value to carry. Read from `nomos-test-material.json` and not `standards.json`, which another tool decodes with unknown fields disallowed — the convention `nomos-architecture.json` already set. Housed below its provider (`nomos-repo-policy`) and below the rules that read it (`nomos-rules`), the same real-second-party-from-day-one reason its ten siblings above did not wait beside their own providers. A sixth `OD-RULES-011` instance. |
| Capability Contract | `nomos-cap-architecture` | The `nomos.cap.architecture.declaration` contract — a repository's own architecture as data: the components it divides itself into, which member belongs to which, which component may depend on which, the named package pairs that order alone cannot express, and its declared write authorities. `OD-RULES-003` called this its third prerequisite, "a place to author a declared architecture as data rather than a Rust `const` table", and left it unbuilt; `OD-RULES-029` measured what its absence cost and insisted the whole triple travel together, because a membership map shipped without its lattice and its vocabulary externalizes nothing. Deliberately **not** an `OD-RULES-011` family: a family is a repository's policy parameters and this is the description a rule judges against. Nothing in the crate names a component, which is what lets a repository divide itself into `Domain`/`Infrastructure`/`Api` without editing `nomos-rules`. Read from `nomos-architecture.json` and not `standards.json`, which another tool decodes with unknown fields disallowed. |
| Capability Contract | `nomos-cap-review-finding` | The `nomos.cap.review.finding` contract on its own — capability identity, contract version, ceiling, payload schema, payload codec and payload types, with no provider beside them. Unlike its eleven `nomos-cap-*` siblings above, it was not extracted because a second party arrived: `OD-CAPABILITY-002`'s criterion is provider *contention* and this capability still has exactly one provider, and `OD-CAPABILITY-017` measured that the bundled arrangement defeated no boundary. Both of those measurements stand. What moved is `OD-ROADMAP-005`'s fifth decision, which is the owner's own sequencing: a generic rule layer should not have to name a vendor crate to obtain a generic capability, so the contract `nomos-rules` reads is spelled without `CodeRabbit`'s name in it. The vendor crate re-exports none of it — a re-export would leave the old spelling working and preserve the defect the split removes. |
| Capability Contract | `nomos-cap-requirement-trace` | The `nomos.cap.requirement.trace` contract, bundled with its one provider in a single crate the identical `OD-CAPABILITY-002`-licensed way `nomos-connector-coderabbit` is one row above, and Capability Contract zone on the identical criterion `OD-CAPABILITY-015` settles: it declares a capability contract. That it reads arbitrary files across the repository tree (a `.assessment` file's own named site, gap and governing record) through a `nomos_platform::FileSystem` neither qualifies nor disqualifies it — `Permits` already grants this zone `Substrate`, where that port lives. Promotes `tests/contract/tests/requirement_trace`'s own already-correct parsing and comparison logic — `OD-TRACE-001`'s guard — into a real, gate-composed rule (`nomos-rules`' `Check_Requirement_Trace_Staleness`) rather than leaving it reachable only from a test binary. `P42-REQUIREMENT-TRACE-STALENESS-RULE-2`. |
| Capability Contract | `nomos-cap-rust-copy-clones` | The `nomos.cap.rust.copy_clones` contract on its own — capability identity, contract version, ceiling, payload schema, payload types and their canonical codec, with no provider beside them. It lived inside `nomos-lang-rust-compiler` from `P40-COMPILER-BACKED-PROVIDER`, which `OD-CAPABILITY-002` licenses while a capability has one provider — and it still has one. What moved it is `OD-ANALYSIS-007` version 2, for a reason `nomos-cap-review-finding`'s own split did not have: the rule that reads this capability is `nomos-rules`', `Permits` forbids `Rules` from naming `Provider`, so while the contract was bundled there was no arrangement in which this capability could be composed into a real run at all. The provider crate re-exports none of it — a re-export would leave the old spelling working and preserve the defect the split removes. |
| Capability Contract | `nomos-cap-rust-nested-locks` | The `nomos.cap.rust.nested_locks` contract on its own, in the same shape and for the same reason as its sibling one row above. Deliberately **not** half of a shared `nomos-cap-rust-semantics`: `OD-ANALYSIS-007` version 2 refuses a family crate by name, because what these two capabilities share is their provider's mechanics (`Load_Crate`, sysroot discovery, per-crate file filtering) and nothing contract-shaped, and a family crate would be a home for exactly the shared program-semantics vocabulary `OD-ANALYSIS-004` forbids. |
| Provider | `nomos-package` | The language-agnostic manifest core: `PackageId`, `PackageKind` and `PKG-007`'s four version domains, minus any typed version-domain abstraction or provider allowlist a specific language would supply. Depends on nothing above `nomos-contracts`. |
| Provider | `nomos-lang-rust` | Recognition and syntax facts from `syn`, and the module rollup derived from them — the one fact in this workspace computed from other facts. |
| Provider | `nomos-lang-rust-scan` | The second provider of that capability. Same zone, so neither may name the other without a named exception, and neither has one. |
| Provider | `nomos-lang-go` | The first real second-language provider of `nomos.cap.syntax.items`: recognition and syntax facts from `tree-sitter-go` in place of `syn`. Declares `Assurance::Sound` on both axes rather than one — Go has no macro system, so there is no construct where its parse tree ends in an unexpanded token stream the way a Rust macro invocation does, a claim checked against the real grammar (generics, build tags) before it was written. Same zone as its two Rust-reading siblings; none of the three may name either of the others. |
| Provider | `nomos-lang-csharp` | The third language, and the one that tests whether the second was a special case: recognition and syntax facts from `tree-sitter-c-sharp`, over the namespaces, `using` directives, types and type members a C# file declares. Weaker than its Go peer on exactly one axis and stronger in what it counts: a `#if` region is a subtree carrying every branch at once, at most one of which is in any compilation, so it reads none of them, counts the regions into the payload header and declares `Assurance::Sound` soundness with `Assurance::Unknown` completeness — the same unbounded-gap shape `nomos-lang-rust`'s macros produce. Generics, nested types, `partial` and file-scoped namespaces were each checked against the real grammar and none of them weakens the claim. Same zone as its three syntax-reading siblings; none of the four may name any other. `P123-CSHARP-SYNTAX-PROVIDER-AND-PACKAGE`. |
| Provider | `nomos-lang-rust-cargo` | The one provider of `nomos.cap.dependency.edges` — runs `cargo metadata` and reads the filesystem, one of three subprocess-backed providers in this workspace now, alongside its two `ToolProvider` siblings below. Composed into `nomos-check-orchestration::Run`, which materializes its edges and hands them to `Check_Dependency_Direction`. |
| Provider | `nomos-lang-rust-clippy` | The one provider of `nomos.cap.lint.diagnostics` — runs `cargo clippy --message-format=json` and reads the filesystem, the same subprocess-backed shape as `nomos-lang-rust-cargo`. Composed into `nomos-check-orchestration::Run`, which materializes one fact per workspace member and hands them to `Check_Lint_Diagnostics`. `OD-RULES-010`. |
| Provider | `nomos-lang-rust-deny` | The one provider of `nomos.cap.dependency.policy` — runs `cargo deny --format json check bans licenses sources` (its own JSON stream lands on stderr, not stdout) and reads the filesystem, the same subprocess-backed shape as `nomos-lang-rust-clippy`. `advisories` is deliberately excluded: it is the one `cargo deny` check that fetches the RustSec database over the network. `OD-RULES-010`'s second real `ToolProvider`. |
| Provider | `nomos-lang-rust-compiler` | The one provider of `nomos.cap.rust.copy_clones` and of `nomos.cap.rust.nested_locks` — loads a crate through `ra_ap_hir`, rust-analyzer's own semantic-analysis engine published as a library, and resolves which `.clone()` calls duplicate a value whose type already implements `Copy`, and which `Mutex<T>`/`RwLock<T>` guard a `T` that is itself already a lock. This workspace's first provider backed by a real compiler semantic API rather than a syntax tree (its siblings above) or another tool's own report (`nomos-lang-rust-clippy`, `nomos-lang-rust-deny`) — a question needing resolved names and trait implementations, not merely a parse. One provider identity and two offers, because `ProviderOffer` pairs a provider with a capability per offer. Composed into a real gate run as of `P123`: both contracts moved out to their own crates, both rules moved into `nomos-rules`, and `nomos-check-orchestration` declares both capabilities and offers this crate against each. Its cost is real and demand-gated for that reason — each family loads a sysroot and the whole resolved crate graph of the tree under check, so a run whose selection declares neither never loads `ra_ap_hir` at all. Provider zone rather than Capability Contract now that it declares no contract, which is `OD-CAPABILITY-015`'s own criterion. `P40-COMPILER-BACKED-PROVIDER`, `P42-SEMANTIC-FACT-FAMILY`, `OD-ANALYSIS-007`. |
| Provider | `nomos-lang-go-modules` | A second provider of `nomos.cap.dependency.edges`, for a Go workspace rather than a Cargo one. Reads `go.work`/`go.mod` text directly — a module path is already Go's own unambiguous identity, so no subprocess resolution is needed the way Cargo's manifest text requires. Composed into `nomos-check-orchestration::Run` alongside `nomos-lang-rust-cargo`; its honestly weaker completeness never clears `Check_Dependency_Direction`'s own floor, so the two never compete for one subject. Same zone as its three siblings; none of the four may name any other. `OD-CAPABILITY-009`. |
| Provider | `nomos-repo-policy` | The five `nomos.cap.*.policy` providers (naming, limits, scripting, words, goals), consolidated from six crates into one after `OD-PACKAGE-015` found none of them earned an independent crate boundary — no independent versioning, no enforced isolation, and their two real consumers (`nomos-check-orchestration` and `tests/integration`) already depended on all five together, every time. Each provider keeps its own module, its own capability contract, and its own `ProviderId`; the shared `standards.json` read/parse step `OD-RULES-019` decided the five owe is a private module beneath them, not a crate of its own. `OD-RULES-011`, `OD-RULES-019`, `OD-PACKAGE-015`. |
| Provider | `nomos-lang-rust-package` | The Rust `LanguagePackage` manifest format and its refusing reader — wraps `nomos-package`'s generic core with `RustEdition` resolution and this workspace's two Rust providers. |
| Provider | `nomos-model-package` | The first `ModelBackendPackage`/`AgentExecutorPackage` manifest maturity: identity, `PackageKind`, `PKG-007`'s first two version domains reused from `nomos-package` unchanged, and `ModelSelection` replacing the fourth. A peer of `nomos-lang-rust-package`, not a dependent of it. Also carries a growing declared-not-computed execution/routing vocabulary, now ten maturities answering twenty-five `MODEL-ROUTE` requirements `OD-PACKAGE-011` v3 licensed — none of it wired into a real consumer yet. |
| Provider | `nomos-rule-package` | The first `RulePackage` manifest maturity, measured field by field against four real shipped rules rather than invented (`OD-PACKAGE-008`). A third peer wrapping `nomos-package`'s generic core, not a dependent of `nomos-lang-rust-package` or `nomos-model-package`. |
| Provider | `nomos-lang-go-package` | The first real second consumer of `nomos-package`'s generic core: the Go `LanguagePackage` manifest format, wrapping it with `GoVersion` resolution and `nomos-lang-go`'s own syntax provider in place of `RustEdition` and Rust's two. A fourth peer of the three above; none of the four names another. `OD-PACKAGE-006`, `OD-PACKAGE-007`. |
| Provider | `nomos-lang-csharp-package` | The third consumer of `nomos-package`'s generic core, and the first whose language-version domain a `{major, minor}` pair could not have carried: C#'s `LangVersion` is a major with an *optional* minor, so `7.3` and `12` are both whole versions and `CsharpVersion` is the case the generic unresolved `language_versions` field exists for. Refuses `latest`, `preview` and `default` — real `LangVersion` values that name whatever the toolchain is rather than a version a manifest can claim. A seventh peer of the six above; none of the seven names another. `OD-PACKAGE-006`, `OD-PACKAGE-007`. |
| Provider | `nomos-tool-package` | The first `ToolProvider` manifest maturity — `PackageKind::ToolProvider` was the one populated `PackageKind` with no manifest crate before this one. Identity, `PackageKind` (restricted to `ToolProvider`), `PKG-007`'s first two version domains reused from `nomos-package` unchanged, and `OD-CAPABILITY-013`'s closed twelve-name FAMILY vocabulary replacing the fourth. Admits this workspace's two real `ToolProvider`s by their own provider identities: `nomos-lang-rust-clippy` (`LINTER`) and `nomos-lang-rust-deny` (`PACKAGE_MANAGER`). A fifth peer of the four above; none of the five names another. `OD-CAPABILITY-013`, `P47-TOOLPROVIDER-HAS-NO-PACKAGE-2`. |
| Provider | `nomos-integration-package` | The first `IntegrationPackage` manifest maturity — `PackageKind::IntegrationPackage` had a decided shape in three records and no manifest crate before this one. Identity, `PackageKind` (restricted to `IntegrationPackage`), `PKG-007`'s first two version domains reused from `nomos-package` unchanged, and a list of materialization intents in place of the third and fourth: each a source, a repository-relative target, `OD-PACKAGE-004`'s ownership class (undeclared is `UserOwned`) and `OD-PACKAGE-005`'s publication scope (undeclared is `Local`). Declares placements and performs no write — `nomos-materialization` below it is the mechanism that does, and a surface is a free label rather than a typed axis. The four types that vocabulary is spelled in are that crate's and re-exported here unchanged, so no caller's spelling changed when they moved. A sixth peer of the five above; none of the six names another. `OD-PACKAGE-003`, `OD-PACKAGE-004`, `OD-PACKAGE-005`. |
| Provider | `nomos-connector-coderabbit` | The one provider of `nomos.cap.review.finding`, offering against `nomos-cap-review-finding` one table section above. A connector under `ARC-CONNECTOR-001`: reads an already-posted CodeRabbit review comment through `gh api` (GitHub is the transport, CodeRabbit is the authority the resulting `Observed` fact is a claim about), never writes (`OD-CONNECTOR-001`, held by omission), and carries one recorded fixture plus one opt-in live test (`OD-CONNECTOR-002`). Provider zone rather than Capability Contract since `OD-ROADMAP-005`'s fifth decision moved the contract out: `OD-CAPABILITY-015`'s criterion is what a crate *declares*, and this one declares no contract any more. That is what lets `Permits` keep `Rules` away from it outright, where the bundled arrangement needed `OD-CAPABILITY-017`'s measurement and then `tests/contract/tests/boundaries/bundled_contract_half.rs` to hold the same line. |
| Rules | `nomos-rules` | A rule as a pure function whose subject is an argument: source it is handed, and facts it reads through a `FactReader`. |
| Agent | `nomos-corrections` | `CorrectionCandidate`, `CorrectionPlan`, and the deterministic preview, stage, validate, commit and rollback lifecycle over a workspace change. No agent or model backend decides anything here; `Commit` takes the caller's `Evidence` but does not judge it. |
| Agent | `nomos-agent-contracts` | `AGT-001`'s `TaskEnvelope` and `AGT-002`'s `WorkResult` — the typed input and output shape an agent-assisted operation carries, bundling what was scattered across `nomos-scope-verification`, `nomos-contracts` and `nomos-corrections`. Neither type computes anything; a caller fills a `TaskEnvelope` in and an agent's own response fills a `WorkResult` in. Since `P126-A-PORT-STANDS-BETWEEN-THE-GENERIC-AGENT-PATH-AND-ITS-TWO-BACKENDS` the crate also publishes the two ports a generic path dispatches through, `AgentExecutor` and `ModelBackend`, one per `PackageKind`, with the two answer types they return — `AgentExecution`, which carries a `WorkResult` with the spend, denials, error flag and duration an executor can ground, and `ModelAnswer`, which carries a response and nothing a model backend cannot — beside `DispatchRefusal`, `DispatchPort` and `DeclaredTarget`. The asymmetry is the point and is structural: there is no value here on which a model backend's answer can be asked for a cost. |
| Agent | `nomos-agent-executor-claude-code` | The first real `AgentExecutor`, and the one concrete implementation this workspace has today — its name says which, so a second one (a different agent CLI, a human, a replay) has an honest name left to take rather than inheriting a canonical-sounding one it never earned. Dispatches a `TaskEnvelope`'s `goal` to Claude Code as a subprocess through `nomos-platform`'s `ProgramLauncher`, bounded by `OD-EXECUTOR-001`'s structural capability boundary — an isolated working directory, no MCP config, an allow-list naming no real tool, one `--print` turn — and reads the result for what it structurally permitted, never for its own free-text claims. Does not assemble a `WorkResult`: `CorrectionPlan::New` refuses an empty candidate list, so a judgment-only task has no plan to report yet. |
| Agent | `nomos-model-backend-ollama` | The first real `ModelBackend` adapter, not a second `AgentExecutor` (`OD-PACKAGE-013`) — built and named as the latter, then measured against `PackageKind::ModelBackendPackage`'s and `PackageKind::AgentExecutorPackage`'s own doc comments and found to match the former: a fixed model, one forwarded field, every other `TaskEnvelope` field read and ignored, no tool-use loop, no MCP surface. Structurally parallel to `nomos-agent-executor-claude-code`, no shared trait, no shared package kind. Dispatches a `TaskEnvelope`'s `goal` to a local Ollama model (`ollama run`) as a subprocess through `nomos-platform`'s `ProgramLauncher`, bounded by `OD-EXECUTOR-004`'s structural capability boundary, measured against `ollama run`'s own real mechanism rather than inherited from `OD-EXECUTOR-001` by analogy: never `--experimental`/`--experimental-yolo`/`--experimental-websearch` — the only flags that open any tool-use capability, so their absence is the entire boundary, because there is no tool subsystem to grant into in the first place. No per-call dollar cost (inference is local); a wall-clock timeout stands in its place. |
| Repo Tooling | `nomos-work-orchestration` | `[repo tooling]` Runs a `nomos work` verb against a caller-chosen platform and hands back a typed outcome — generic over `nomos-platform`'s traits, so a second adapter can depend on it without also depending on `nomos-platform-std` or on how `nomos-cli` renders an answer. |
| Application Service | `nomos-check-orchestration` | Composes the capability registry, ingests already-walked source into facts and judges it, and hands back a typed outcome — apart from choosing a platform, walking a tree or rendering the answer. |
| Specification | `nomos-spec-orchestration` | `[repo tooling]` Assembles the specification store from the embedded governing records and a caller-named corpus, and answers all nine `SpecCommand` verbs plus `nomos request submit`'s `Submit`, generic over `nomos-platform`'s traits where a verb reads or writes — apart from choosing a platform or rendering the answer. |
| Application Service | `nomos-gate-orchestration` | The seam for the first-class Gate object `ARC-ROADMAP-001` names: `Plan` composes a real rule registry and reports what it holds, `Run_Gate` composes `nomos-check-orchestration` into a judged disposition — apart from selecting scope, choosing a platform or rendering the answer. Same zone as `nomos-check-orchestration`, and one of the named same-zone exceptions this workspace's own dependency model declares so it may depend on it. `OD-RULES-020`. |
| Application Service | `nomos-correction-orchestration` | The seam for correction planning and lifecycle both hosts call: `Run_Correction` composes `nomos-check-orchestration` and `nomos-corrections` into a judged, staged, validated (and optionally committed) outcome — apart from choosing a platform, walking a tree or rendering the answer. `nomos-cli`'s own `correct.rs` is a thin renderer over it, the way `gate.rs` already is over `Run_Gate`. Same zone as `nomos-check-orchestration`, a named same-zone exception; `nomos-corrections` is a different zone (Agent) this zone's own edges already permit. `P40-CORRECTIONS-CANONICAL-SEAM`, `OD-RULES-020`. |
| Application Service | `nomos-workflow-orchestration` | The workflow tier's first real execution increment: runs an ordered sequence of `WorkflowStep` declarations, each paired with a real dispatch to one of this workspace's four real dispatch targets (`nomos-agent-orchestration::Run_Agent_Task`, `nomos-check-orchestration::Run`, `nomos-correction-orchestration::Run_Correction`, `nomos-gate-orchestration::Run_Gate`), refusing whichever step first declares itself incoherent before that step's body ever dispatches — apart from choosing a platform. An agent step declares a `ModelExecutionProfile` and the declared package set selects what answers it, rather than a `Body` variant per backend naming one; `Body::Agent`'s edge joins `Body::Check`, `Body::Correction` and `Body::Gate`'s as a named same-zone exception this workspace's dependency model declares. `OD-WORKFLOW-005`, `OD-PACKAGE-016`, `P40-WORKFLOW-CHECK-BODY`, `P40-WORKFLOW-CORRECTION-BODY`, `OD-RULES-020`. |
| Application Service | `nomos-agent-orchestration` | The seam for dispatching one `TaskEnvelope` to a chosen backend both hosts call: `Run_Agent_Execute` composes a bare goal and `Run_Agent_Judgment` composes an already-judged `RoleSurfacePair`/`Finding` pair, each dispatching through the port its `DeclaredTarget` resolves to — `nomos_agent_contracts::AgentExecutor` or `ModelBackend`, one per `PackageKind`, because an executor and a model backend answer different questions and their answers are not one shape (`OD-EXECUTOR-005`). This crate names neither adapter crate, in its manifest or in a line of source; each host's own `agent` module is the composition root that supplies the concrete pair, and each adapter declares itself. Apart from choosing a platform, reading a crate's declared role or surface, or rendering the answer. `nomos-cli`'s own `agent.rs` is a thin renderer over it, the way `correct.rs` already is over `Run_Correction`. Reaches Agent zone directly, no named exception needed (`Permits`). `P43-AGENT-CANONICAL-SEAM-2`. |
| Application Service | `nomos-workspace-discovery` | The one walk every composition root shares: `Walked_Sources` finds every file under a root whose extension a caller-supplied set names, skipping `target`, `.git`, a nested git worktree's own root and a nested repository root (a directory carrying its own `standards.json`, so a vendored third-party tree is judged against the conventions it declares or not here at all — `P96`) — apart from choosing a platform, judging the result or rendering an answer. `Registered_Extensions` names what a registered language package recognizes (`nomos-lang-rust`'s and `nomos-lang-go`'s own extension constants), the one edit a third language package costs this crate instead of a fifth host-side copy; a caller needing `check-script-discipline`'s script extensions too composes its own wider set. Reaches Provider zone directly (`nomos-lang-rust`, `nomos-lang-go`), no named exception needed (`Permits`). Replaces four independent copies of the identical walk (`nomos-api`, `nomos-cli`'s `check` and `gate`, `nomos-lsp`). `P41-WORKSPACE-DISCOVERY-SERVICE-2`, `OD-HOST-008`. |
| Host | `nomos-cli` | The `nomos` binary. |
| Host | `nomos-api` | A second real caller of `nomos-gate-orchestration`'s `Run_Gate`, `Plan` and `Explain` (all three Gate verbs), `nomos-work-orchestration`'s `Run` for all eleven `WorkCommand` verbs (`List`, `Show`, `Validate`, `Audit`, `Claim`, `Renew`, `TakeOver`, `Abandon`, `Decline`, `Finish`, `Add`), `nomos-spec-orchestration`'s `Run` for all nine `SpecCommand` verbs (`Profiles`, `Sources`, `Record`, `Table`, `Markdown`, `Freshness`, `Preview`, `Render`, `Commit`) plus `Submit`, `nomos-correction-orchestration`'s `Run_Correction`, and `nomos-agent-orchestration`'s `Run_Agent_Execute`/`Run_Agent_Judgment` — apart from choosing a platform or wiring an actual transport over it. |
| Repo Tooling | `nomos-surface-provenance` | `[repo tooling]` A report over this repository's own git history, run on demand and invoked from nowhere else: `OD-STORE-002`'s Worked Case join between a crate's surface snapshot and `docs/records/`, for a caller-given commit range. Never a gate. |
| Host | `nomos-api-transport` | **What this workspace serves over a wire, and nothing about the wire** (`OD-HOST-013`, 2026-09-11). `ServedMethod` names the six verbs `OD-HOST-007`, `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2` and `OD-HOST-014` admit -- `nomos.gate.plan`, `nomos.gate.run`, `nomos.gate.explain`, `nomos.gate.compare`, `nomos.correction.run`, `nomos.check.run` -- and none of the twenty-one `Handle_Work_*` and `Handle_Spec_*` handlers those records exclude, because those belong to the `[repo tooling]` crates below, nor the three `OD-HOST-014` refuses by its own criterion (`Handle_Agent_Execute` and `Handle_Agent_Judge_Role` start a metered subprocess; `Handle_Workflow_Run` would admit them through a step body). `NomosApiDispatch` is the dispatch into `nomos_api::Handle_*`, and is an `xvpe_remote_call::RemoteCallStrategy`: the JSON-RPC 2.0 envelope, the line framing, the reserved codes and the socket are all `xvpe-remote-call-backend-json`'s. Not itself `[repo tooling]`: it exists precisely to answer a question an end-user repository would ask. Same zone (Host) as `nomos-api`, and one of Host's own named same-zone exceptions rather than a peer edge -- `OD-RULES-020` measured this exact edge as real. Binds no listener of its own; a caller supplies one. |
| Host | `nomos-mcp` | The MCP half of `AGT-006`'s "neutral versioned contracts and MCP tools" commitment, and still a real, runnable MCP server over stdio. **What it holds is the catalogue, not the protocol** (`OD-HOST-013`, 2026-09-11): `ServedTool` names which six tools exist -- `ServedMethod::REGISTRY` projected entry for entry, so `nomos.check.run` arrived here the day `OD-HOST-014` admitted it there -- the sentence each publishes and the JSON Schema each accepts; `NomosToolCatalog` is an `xvpe_remote_call::ToolCatalogStrategy`, and the whole handshake -- `initialize`, `tools/list`, `tools/call`, `ping`, notification suppression, the framing -- is `xvpe-remote-call-backend-json`'s. Depends on `nomos-api-transport` alone -- never on `nomos-api` or any orchestration crate directly -- and a `tools/call` reaches a handler by calling that crate's own `NomosApiDispatch` under the name the client asked for, inheriting its exclusion of every `[repo tooling]` handler rather than repeating it. (It used to reach it by re-serializing the call into a synthetic JSON-RPC line; the shared contract replaced that round trip.) Same zone (Host) as `nomos-api-transport`, another of Host's own named same-zone exceptions. Ships its own binary: unlike its sibling, a stdio transport has no listener for a caller to bind first. |
| Host | `nomos-lsp` | **What this workspace tells an editor, and nothing about the protocol** (`OD-HOST-013`, 2026-09-11). `NomosDiagnosticProvider` is an `xvpe_diagnostics::DiagnosticProviderStrategy`: it walks a tree, runs `nomos-check-orchestration::Run` over it, and `Diagnostics_For` translates each `Finding` into an `xvpe_diagnostics::SourceDiagnostic`, judging nothing itself. `severity::Severity_Of` is the half no shared vocabulary could hold -- what this workspace's own `GateCategory` and `Applicability` deserve. The handshake, the workspace root, `file://` conversion, whole-line spans and stale-marker clearing are `xvpe-language-server-backend-lsp`'s, so **this crate names no protocol library at all**. Depends on `nomos-check-orchestration` and `nomos-correction-orchestration` directly (both Application Service, within Host's own `Permits`), on `nomos-rules` for `DESCRIPTORS`, `SourceFile`, `RequiredFact` and `SubjectKind`, on `nomos-cap-architecture` and `nomos-repo-policy` for the architecture declared as data in `nomos-architecture.json` -- `OD-RULES-024` removed the last component and crate name of this workspace from `nomos-rules`, so there is no `ZONES` in any crate for this one to read -- and on `nomos-cap-requirement-trace` for `Assessments_In` -- reading a rule's own declared contract, the architecture the repository under check declares and a committed assessment's own declaration, never a rule's judgment. Re-judges on `didOpen`/`didSave`, full-document sync only, reusing one workspace and fact store across calls (`OD-ANALYSIS-009`). All five of `P42-LSP-PROJECTION`'s walk-outward targets are answered: the three `OD-HOST-010` built, plus which facts a finding's rule read (`OD-HOST-016`, read off the trail `CheckOutcome::Judged` now carries) and which corpus requirements it bears on (`OD-HOST-015`, the `rule` lines an assessment declares). Each answer keeps its own vocabulary for not knowing -- a rule that structurally reads no fact is not a rule whose trail went missing, and an empty requirement list means nobody declared a line rather than that no rule bears. `Diagnostics_For` takes all of them through one `WalkContext`, so a sixth answer is a field rather than a third change to that signature. |
| Host | `nomos-daemon` | **The process that outlives an invocation** (`P128-EVERY-INVOCATION-STARTS-COLD-BECAUSE-NOTHING-OUTLIVES-A-PROCESS`). `Resident` is started against one root and holds the three values `nomos-check-orchestration`'s `Run_Reassessing` takes as caller-supplied parameters -- a `nomos-workspace` `Workspace`, a `nomos-analysis` `MemoryFactStore` and a `RuleReassessmentCache` -- across requests, so a second request over an unchanged tree files no fact at all and an edited one files only the moved subject's. It judges through that one seam and composes no rule of its own, which is why a reused judgment and a cold invocation's agree rather than being compared into agreement; it walks through `nomos-workspace-discovery` with the same extension set `nomos-cli` walks, so the two are over one population. What it watches is that walked set, re-read and compared against the held workspace's own content digests at request time -- there is no watcher and no background thread, and a root that stops being readable is refused rather than answered from what is still held. A residency is confined to one thread and requests to it serialize -- `Answer` takes `&mut self`, and a `Resident` is not `Send` because `nomos-analysis`' store holds a boxed propagation strategy that is not, which was measured by being refused rather than assumed; a concurrent caller starts a resident per thread and those share the root safely, because a resident writes nothing to the tree, which is also why stopping one leaves nothing a later cold invocation could read as current state. A library crate, not a binary, and for a sharper reason than `nomos-api`'s: a wire belongs to `nomos-api-transport` by `OD-HOST-013`, and a Host may not name another Host, so a transport host composes a `Resident` rather than this crate growing a second serving surface. `OD-ANALYSIS-009`, `OD-HOST-002`. |
| Verification | `nomos-contract-tests` | The assertions in `tests/contract`. Observes the workspace; nothing observes it. |
| Verification | `nomos-integration-tests` | The vertical slice, driving the product through its seams. Its peer, not its layer. |

The specification system sits beside the kernel rather than above it. It reaches the
product only through a knowledge capability, so nothing in the product may name it.

| Zone | Crate | Owns |
|---|---|---|
| Specification | `nomos-spec-model` | The canonical normalizer and hashing. The single authority on what content hashes to. |
| Specification | `nomos-spec-store` | Schema, migrations, and this repository's own governing records. |
| Specification | `nomos-spec-bundle` | Deterministic JSONL export and import — the portable authority committed to git. |
| Specification | `nomos-spec-ingest` | Parsers and the manifest gate against the real v14 corpus. |
| Specification | `nomos-spec-validate` | The `NSV-PRESERVE-*` rules and the run that fails closed. |
| Specification | `nomos-spec-project` | Eighteen projection profiles, the renderers, and the freshness stamp. |

The normalizer was **recovered from the corpus, not chosen**: v14's hash generator does
not ship, so the algorithm was reconstructed and verified against all 2,533 recorded
block hashes and all 493 recorded statement hashes. A disagreement in `nomos-spec-model`
makes the entire preservation ledger measure nothing, which is why it is gated by
`crates/spec/nomos-spec-model/tests/normalizer_gate.rs` before anything downstream is
trusted.

## Coordinating concurrent work

Several agents can work this repository at once. The ledger is what keeps them from
overwriting each other: two items are concurrently claimable exactly when their
territories are provably disjoint, and an unanswerable overlap question refuses the
claim rather than granting it.

```
nomos work list [--state <state>]          # `nomos work` names every state it can print
nomos work show   --item <id>
nomos work add     --item <id> --title <text> --why <text> --done-when <text>
                   --kind capability|decision|validation|correction|cleanup
                   --origin required|proposed
                   --territory <path> [--territory <path> …]
                   [--amends <record> …]
                   [--depends-on <id> …]
                   [-- <program> <args…>] [--timeout 2h]
nomos work claim   --item <id> --holder <name> [--lease 2h]
nomos work renew   --item <id> --holder <name> [--lease 2h]
nomos work takeover --item <id> --holder <name> [--lease 2h]
nomos work finish  --item <id> --holder <name>
nomos work abandon --item <id> --holder <name> --reason <text>
nomos work decline --item <id> --holder <name> --reason <text>
nomos work widen   --item <id> --holder <name> --territory <path> [--territory <path> ...]
nomos work validate
nomos work audit
```

**Territory** is written as repository paths, not identifiers, because the ledger is
committed and reviewed in a `git diff` — a diff of digests is a diff nobody reads. Paths
are compared after normalization, so `./crates\A\src\Lib.rs` and `crates/a/src/lib.rs`
are one subject, and a directory contains the files beneath it. An item that reserves
nothing is refused: it would exclude nobody while looking like work.

Territory is paths and not globs, and `--territory-pattern` is a usage error rather than a
flag. Containment already covers what a glob was wanted for — `--territory crates/spec`
reserves everything beneath it, decided from the text with no filesystem access — whereas an
unexpanded pattern compares as *unanswerable* against every other territory, which makes the
item unclaimable by anyone including its author and refuses every other claim on the board
with the code that means stop and fetch a person. `OD-LEDGER-013` records the trade.

**`--kind` and `--origin` are required, and both are closed sets.** `--kind` says what sort
of work the item is and `--origin` says whether a person required it or a session proposed
it; the synopsis above lists both vocabularies, and it is the only place in this file that
does. An unrecognized
value is refused the same way an unrecognized JSON key already is, rather than stored and
ignored. `OD-LEDGER-024` records why these five and not a free-form tag list, and how every
item already on the board was given both when the fields were added.

**A claim is a lease, and a lease lapses.** An agent that dies holding one stops excluding
everybody else the moment the lease runs out, which is what stops one crashed session holding
territory until somebody notices. `list` calls that item `lapsed` rather than `claimed`, because
it is not work in progress. Recovering it costs one deliberate command: the holder that comes
back runs `renew`, and anybody else runs `takeover`, which installs a new claim and keeps the one
it displaced on the item where `show` reports it. `claim` never does this — it refuses a lapsed
item and names the holder it would have displaced — because taking over another agent's
abandoned work is a decision, and a decision belongs in a verb somebody typed.

**Abandoning ends a claim; declining ends an item.** They take the same three arguments and
they are not degrees of one thing. `abandon` says this holder stopped, so the item goes back on
the board with the reason attached — which is right whenever the work is still wanted and
somebody else can finish it, the common case among the abandonments this ledger has recorded.
`decline` says the item is not work at all: superseded by a successor that already landed it, or refused
by name in a record written since it was authored. It takes no claim, because an item nobody
intends to do should not have to be claimed first, and it refuses an item somebody is holding —
that call is the holder's, and the refusal names the two commands. `OD-LEDGER-019` measures what
having only the first of these cost.

**Finishing runs something.** `done_when` is prose for a human; everything after `--` is
an argument vector that gets executed, with no shell between what was written and what
runs. `finish` records the item done only if that exits zero, and it keeps three answers
apart — the predicate failed (the work is not done), the predicate could not be started
or timed out (nobody found out), and there is no predicate at all (nothing was checked,
which must never read like everything checked out).

Exit codes are a contract, because agents branch on them rather than parsing output:

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | validation error |
| 2 | usage |
| 3 | claim unavailable, or a dependency is unfinished — **retryable**, try another item |
| 4 | conflict — a human has to resolve it |
| 5 | the ledger or its lock could not be used at all |

The distinction that earns its own code is 3 versus 5. An agent told the item is taken
should pick up something else; an agent told the ledger is broken should stop and fetch
a person. Collapsing those into "non-zero" makes the first indistinguishable from the
second.

**If a verb exits 5 saying the file is a schema this build does not understand, the
executable is stale, not the ledger.** `finish` runs a predicate that rebuilds the running
process, which Windows will not permit, so sessions copy `target/debug/nomos.exe` and run the
copy — and a copy taken before a field was added used to read the current ledger, drop that
field, write the document back and exit 0. The state change survived and the data did not, so
a lossy write looked exactly like a clean one. It is loud now: a build that cannot account for
every key in the ledger refuses to read it, so it never writes it. Rebuild
(`cargo build -p nomos-cli`), copy the binary again, and retry. `OD-LEDGER-008` records why the
refusal is total rather than partial, and what it does not reach — a copy taken before that
decision landed has none of the guard and stays silent.

`nomos work validate` is the command that answers "is the executable I copied current?". It
reports the file's schema version and the running build's side by side, without having to
provoke a refusal first:

```
ledger is valid (schema 5, and this build understands 5)
```

## Reading the specification

```
nomos spec record    --id <node-id> [--revision <label>]
nomos spec table     --document <path|name> [--block <n>] [--table <n>]
nomos spec render    --profile <id> --into <directory>
nomos spec freshness --into <directory> [--profile <id>]
nomos spec profiles
nomos spec sources
```

The store is assembled per invocation: this repository's governing records are embedded in
the binary, and the v14 corpus is read from `--corpus` or `NOMOS_V14_CORPUS`. A corpus that
is not there is reported as an absence and exits 6, never as a shorter answer. `record` and
`table` write content to stdout and everything about it to stderr, so a redirect captures
exactly what the store holds.

`render` writes a body and a `.nomos-projection.json` sidecar stamping what produced it
and from what. `freshness` compares the two back and keeps the two failures apart — the
store moved (stale) and somebody typed into the file (edited) — exiting 8 for either. A
body with no sidecar beside it is a failure rather than a skip, because otherwise deleting
the sidecar is how an edit stops being caught.

## Submitting a feature request, design spec or feature result

```
nomos request submit --kind <kind> --id <node-id>    # `nomos request` names every kind
                      --by <name> [--state <state>] [--contract-version <n>]
                      --field <name>=<value> [--field <name>=<value> …]
                      [--gap <question>|<blocked-fields,comma-separated>|<blocking|non-blocking>[|<closed-by>]] …
                      [--into <directory>]
```

`OD-SPEC-009` decided there is exactly one door these three born-structured kinds become
durable through, and that every surface — this CLI verb included — is a transport onto it:
`submit` constructs a submission from exactly what was typed, with every `--field` value
carrying origin `submitted`, and hands it to the store's accept function. `OD-SPEC-010` states
the rule set that function checks and `OD-SPEC-013` decides the layout it writes: a submission
is a node, a field is a sequence of attributed values, and a decision gap is a row closable
only by a citation. A refusal names every field that failed, the rule each one failed and what
would satisfy it — exit 9 — and nothing is stored.

Nothing in this repository persists a specification database, so a submission accepted here
exists for exactly as long as the invocation runs. `--into` renders the accepted submission
through the `subject-dossier` profile before the store is gone, which is the one chance a run
has to take the freshness proof `ARC-SPECDB-002` charges a born-structured object with.

## Conventions

Function names are `Pascal_Snake_Case` and control flow uses explicit `return` and
Allman braces, matching the sibling `xvpe` workspace. Types stay `UpperCamelCase`.

`cargo fmt` **cannot** produce this style and damages the tree when run — rustfmt has no
option that puts a control-flow brace on its own line, so it rewrites the workspace's
form back to K&R every time. `cargo fmt --check` is deliberately not a gate step: a gate
containing two contradictory checks is one that can never be green, and a gate that can
never be green is one everybody learns to ignore. See `rustfmt.toml`.

`unwrap_used`, `indexing_slicing`, `arithmetic_side_effects` and `float_cmp` are denied.
A panic on the analysis path is a determinism defect rather than merely a crash: a replay
must reach the same panic at the same step.

## Running the gate

`.github/workflows/gate.yml` is the gate; its `Lint` and `Test` steps are the two commands to
run before pushing. This file does not reproduce them — `OD-GATE-007` is why severity in
particular is stated once, in the workflow and in `[workspace.lints.clippy]` above, and not a
third time here.

## This file is hand-authored, and that is a decision

`D-128` requires `README.md`, `ARCHITECTURE.md` and `ARTIFACT_MAP.md` to be
freshness-validated publication outputs. Those are the *specification suite's* overview
documents: `README.projection.md` from the `github-markdown` profile, and
`spec/architecture.md` from `architecture-document`. Both are corpus-backed profiles —
neither is among the four that render without one (`OD-PROJECT-002`) — and the corpus they
need is not in this repository or on any CI runner, so neither is committed here.

This file is a different document that happens to share a name. It describes the
workspace, and no content kind in the projection system selects a crate's zone or a gate
command — so no profile can render it, and the corpus it would be rendered from is on no
CI runner. It therefore stays hand-authored.

What that gives up is freshness for the prose, and nothing here pretends otherwise. What
it does not give up is the tables above: `tests/contract/tests/boundaries/readme.rs`,
reached through that target's own `main.rs`, compares them in both directions against the
architecture this repository declares as data in `nomos-architecture.json`, so a crate that
joins the workspace without joining this page fails the gate. What that declaration holds is
its own to say and is deliberately not restated here. See
`docs/records/OD-PROJECT-001-the-repository-readme-is-not-the-suites-overview.md`.
