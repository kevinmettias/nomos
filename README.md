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
crate had to be renumbered more than once for a dependency nobody disputed. `nomos-rules`
declares `ZONES`, `Permits` and `SAME_ZONE_EDGES`, and `tests/contract` asserts against
that one declaration rather than a second copy of its own.

Four rows below are marked `[repo tooling]`: `nomos-ledger`, `nomos-work-orchestration`,
`nomos-spec-orchestration` and `nomos-surface-provenance` exist to develop or preserve this
repository, not to answer a question an end-user repository would ask Nomos.
`ARC-ECOSYSTEM-001` names why a shared zone does not by itself mean shared product
ownership, `OD-LEDGER-036` settles the ledger specifically, and `OD-PROJECT-004` decided
where these four belong physically. The `nomos-spec-*` family below the main table carries
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
| Capability Contract | `nomos-connector-coderabbit` | The `nomos.cap.review.finding` contract, bundled with its one provider in a single crate rather than split the way its nine `nomos-cap-*` siblings above are: `OD-CAPABILITY-002` licenses this while there is exactly one provider and, once `nomos-rules`' `Check_Review_Findings` reads it, one party on each side. A connector under `ARC-CONNECTOR-001`: reads an already-posted CodeRabbit review comment through `gh api` (GitHub is the transport, CodeRabbit is the authority the resulting `Observed` fact is a claim about), never writes (`OD-CONNECTOR-001`, held by omission), and carries one recorded fixture plus one opt-in live test (`OD-CONNECTOR-002`). Capability Contract zone because it declares a capability contract, which is the criterion `OD-CAPABILITY-015` settles — not because `nomos-rules` needs to reach it, which would make the `Rules`-may-not-name-`Provider` edge vacuous by relabelling whatever a rule turns out to need. A consequence of the classification, not its reason: `Permits` forbids Rules from naming Provider, so `Provider` here would leave this fact family unreachable from `nomos-rules`. Running `gh api` is no obstacle either way — `Permits` already grants this zone `Substrate`, where `nomos_platform`'s ports live. |
| Capability Contract | `nomos-cap-requirement-trace` | The `nomos.cap.requirement.trace` contract, bundled with its one provider in a single crate the identical `OD-CAPABILITY-002`-licensed way `nomos-connector-coderabbit` is one row above, and Capability Contract zone on the identical criterion `OD-CAPABILITY-015` settles: it declares a capability contract. That it reads arbitrary files across the repository tree (a `.assessment` file's own named site, gap and governing record) through a `nomos_platform::FileSystem` neither qualifies nor disqualifies it — `Permits` already grants this zone `Substrate`, where that port lives. Promotes `tests/contract/tests/requirement_trace`'s own already-correct parsing and comparison logic — `OD-TRACE-001`'s guard — into a real, gate-composed rule (`nomos-rules`' `Check_Requirement_Trace_Staleness`) rather than leaving it reachable only from a test binary. `P42-REQUIREMENT-TRACE-STALENESS-RULE-2`. |
| Provider | `nomos-package` | The language-agnostic manifest core: `PackageId`, `PackageKind` and `PKG-007`'s four version domains, minus any typed version-domain abstraction or provider allowlist a specific language would supply. Depends on nothing above `nomos-contracts`. |
| Provider | `nomos-lang-rust` | Recognition and syntax facts from `syn`, and the module rollup derived from them — the one fact in this workspace computed from other facts. |
| Provider | `nomos-lang-rust-scan` | The second provider of that capability. Same zone, so neither may name the other without a named exception, and neither has one. |
| Provider | `nomos-lang-go` | The first real second-language provider of `nomos.cap.syntax.items`: recognition and syntax facts from `tree-sitter-go` in place of `syn`. Declares `Assurance::Sound` on both axes rather than one — Go has no macro system, so there is no construct where its parse tree ends in an unexpanded token stream the way a Rust macro invocation does, a claim checked against the real grammar (generics, build tags) before it was written. Same zone as its two Rust-reading siblings; none of the three may name either of the others. |
| Provider | `nomos-lang-rust-cargo` | The one provider of `nomos.cap.dependency.edges` — runs `cargo metadata` and reads the filesystem, one of three subprocess-backed providers in this workspace now, alongside its two `ToolProvider` siblings below. Composed into `nomos-check-orchestration::Run`, which materializes its edges and hands them to `Check_Dependency_Direction`. |
| Provider | `nomos-lang-rust-clippy` | The one provider of `nomos.cap.lint.diagnostics` — runs `cargo clippy --message-format=json` and reads the filesystem, the same subprocess-backed shape as `nomos-lang-rust-cargo`. Composed into `nomos-check-orchestration::Run`, which materializes one fact per workspace member and hands them to `Check_Lint_Diagnostics`. `OD-RULES-010`. |
| Provider | `nomos-lang-rust-deny` | The one provider of `nomos.cap.dependency.policy` — runs `cargo deny --format json check bans licenses sources` (its own JSON stream lands on stderr, not stdout) and reads the filesystem, the same subprocess-backed shape as `nomos-lang-rust-clippy`. `advisories` is deliberately excluded: it is the one `cargo deny` check that fetches the RustSec database over the network. `OD-RULES-010`'s second real `ToolProvider`. |
| Provider | `nomos-lang-rust-compiler` | The one provider of `nomos.cap.rust.copy_clones` — loads a crate through `ra_ap_hir`, rust-analyzer's own semantic-analysis engine published as a library, and resolves which `.clone()` calls duplicate a value whose type already implements `Copy`. This workspace's first provider backed by a real compiler semantic API rather than a syntax tree (its siblings above) or another tool's own report (`nomos-lang-rust-clippy`, `nomos-lang-rust-deny`) — a question needing resolved names and trait implementations, not merely a parse. Not yet composed into a real gate run; that and a shared provider taxonomy, if the evidence ever warrants one, are `nomos-check-orchestration` and `nomos-rules`' own territory. `P40-COMPILER-BACKED-PROVIDER`. |
| Provider | `nomos-lang-go-modules` | A second provider of `nomos.cap.dependency.edges`, for a Go workspace rather than a Cargo one. Reads `go.work`/`go.mod` text directly — a module path is already Go's own unambiguous identity, so no subprocess resolution is needed the way Cargo's manifest text requires. Composed into `nomos-check-orchestration::Run` alongside `nomos-lang-rust-cargo`; its honestly weaker completeness never clears `Check_Dependency_Direction`'s own floor, so the two never compete for one subject. Same zone as its three siblings; none of the four may name any other. `OD-CAPABILITY-009`. |
| Provider | `nomos-repo-policy` | The five `nomos.cap.*.policy` providers (naming, limits, scripting, words, goals), consolidated from six crates into one after `OD-PACKAGE-015` found none of them earned an independent crate boundary — no independent versioning, no enforced isolation, and their two real consumers (`nomos-check-orchestration` and `tests/integration`) already depended on all five together, every time. Each provider keeps its own module, its own capability contract, and its own `ProviderId`; the shared `standards.json` read/parse step `OD-RULES-019` decided the five owe is a private module beneath them, not a crate of its own. `OD-RULES-011`, `OD-RULES-019`, `OD-PACKAGE-015`. |
| Provider | `nomos-lang-rust-package` | The Rust `LanguagePackage` manifest format and its refusing reader — wraps `nomos-package`'s generic core with `RustEdition` resolution and this workspace's two Rust providers. |
| Provider | `nomos-model-package` | The first `ModelBackendPackage`/`AgentExecutorPackage` manifest maturity: identity, `PackageKind`, `PKG-007`'s first two version domains reused from `nomos-package` unchanged, and `ModelSelection` replacing the fourth. A peer of `nomos-lang-rust-package`, not a dependent of it. Also carries a growing declared-not-computed execution/routing vocabulary, now ten maturities answering twenty-five `MODEL-ROUTE` requirements `OD-PACKAGE-011` v3 licensed — none of it wired into a real consumer yet. |
| Provider | `nomos-rule-package` | The first `RulePackage` manifest maturity, measured field by field against four real shipped rules rather than invented (`OD-PACKAGE-008`). A third peer wrapping `nomos-package`'s generic core, not a dependent of `nomos-lang-rust-package` or `nomos-model-package`. |
| Provider | `nomos-lang-go-package` | The first real second consumer of `nomos-package`'s generic core: the Go `LanguagePackage` manifest format, wrapping it with `GoVersion` resolution and `nomos-lang-go`'s own syntax provider in place of `RustEdition` and Rust's two. A fourth peer of the three above; none of the four names another. `OD-PACKAGE-006`, `OD-PACKAGE-007`. |
| Provider | `nomos-tool-package` | The first `ToolProvider` manifest maturity — `PackageKind::ToolProvider` was the one populated `PackageKind` with no manifest crate before this one. Identity, `PackageKind` (restricted to `ToolProvider`), `PKG-007`'s first two version domains reused from `nomos-package` unchanged, and `OD-CAPABILITY-013`'s closed twelve-name FAMILY vocabulary replacing the fourth. Admits this workspace's two real `ToolProvider`s by their own provider identities: `nomos-lang-rust-clippy` (`LINTER`) and `nomos-lang-rust-deny` (`PACKAGE_MANAGER`). A fifth peer of the four above; none of the five names another. `OD-CAPABILITY-013`, `P47-TOOLPROVIDER-HAS-NO-PACKAGE-2`. |
| Rules | `nomos-rules` | A rule as a pure function whose subject is an argument: source it is handed, and facts it reads through a `FactReader`. |
| Agent | `nomos-corrections` | `CorrectionCandidate`, `CorrectionPlan`, and the deterministic preview, stage, validate, commit and rollback lifecycle over a workspace change. No agent or model backend decides anything here; `Commit` takes the caller's `Evidence` but does not judge it. |
| Agent | `nomos-agent-contracts` | `AGT-001`'s `TaskEnvelope` and `AGT-002`'s `WorkResult` — the typed input and output shape an agent-assisted operation carries, bundling what was scattered across `nomos-scope-verification`, `nomos-contracts` and `nomos-corrections`. Neither type computes anything; a caller fills a `TaskEnvelope` in and an agent's own response fills a `WorkResult` in. |
| Agent | `nomos-agent-executor-claude-code` | The first real `AgentExecutor`, and the one concrete implementation this workspace has today — its name says which, so a second one (a different agent CLI, a human, a replay) has an honest name left to take rather than inheriting a canonical-sounding one it never earned. Dispatches a `TaskEnvelope`'s `goal` to Claude Code as a subprocess through `nomos-platform`'s `ProcessLauncher`, bounded by `OD-EXECUTOR-001`'s structural capability boundary — an isolated working directory, no MCP config, an allow-list naming no real tool, one `--print` turn — and reads the result for what it structurally permitted, never for its own free-text claims. Does not assemble a `WorkResult`: `CorrectionPlan::New` refuses an empty candidate list, so a judgment-only task has no plan to report yet. |
| Agent | `nomos-model-backend-ollama` | The first real `ModelBackend` adapter, not a second `AgentExecutor` (`OD-PACKAGE-013`) — built and named as the latter, then measured against `PackageKind::ModelBackendPackage`'s and `PackageKind::AgentExecutorPackage`'s own doc comments and found to match the former: a fixed model, one forwarded field, every other `TaskEnvelope` field read and ignored, no tool-use loop, no MCP surface. Structurally parallel to `nomos-agent-executor-claude-code`, no shared trait, no shared package kind. Dispatches a `TaskEnvelope`'s `goal` to a local Ollama model (`ollama run`) as a subprocess through `nomos-platform`'s `ProcessLauncher`, bounded by `OD-EXECUTOR-004`'s structural capability boundary, measured against `ollama run`'s own real mechanism rather than inherited from `OD-EXECUTOR-001` by analogy: never `--experimental`/`--experimental-yolo`/`--experimental-websearch` — the only flags that open any tool-use capability, so their absence is the entire boundary, because there is no tool subsystem to grant into in the first place. No per-call dollar cost (inference is local); a wall-clock timeout stands in its place. |
| Repo Tooling | `nomos-work-orchestration` | `[repo tooling]` Runs a `nomos work` verb against a caller-chosen platform and hands back a typed outcome — generic over `nomos-platform`'s traits, so a second adapter can depend on it without also depending on `nomos-platform-std` or on how `nomos-cli` renders an answer. |
| Application Service | `nomos-check-orchestration` | Composes the capability registry, ingests already-walked source into facts and judges it, and hands back a typed outcome — apart from choosing a platform, walking a tree or rendering the answer. |
| Specification | `nomos-spec-orchestration` | `[repo tooling]` Assembles the specification store from the embedded governing records and a caller-named corpus, and answers all nine `SpecCommand` verbs plus `nomos request submit`'s `Submit`, generic over `nomos-platform`'s traits where a verb reads or writes — apart from choosing a platform or rendering the answer. |
| Application Service | `nomos-gate-orchestration` | The seam for the first-class Gate object `ARC-ROADMAP-001` names: `Plan` composes a real rule registry and reports what it holds, `Run_Gate` composes `nomos-check-orchestration` into a judged disposition — apart from selecting scope, choosing a platform or rendering the answer. Same zone as `nomos-check-orchestration`, and one of the named same-zone exceptions this workspace's own dependency model declares so it may depend on it. `OD-RULES-020`. |
| Application Service | `nomos-correction-orchestration` | The seam for correction planning and lifecycle both hosts call: `Run_Correction` composes `nomos-check-orchestration` and `nomos-corrections` into a judged, staged, validated (and optionally committed) outcome — apart from choosing a platform, walking a tree or rendering the answer. `nomos-cli`'s own `correct.rs` is a thin renderer over it, the way `gate.rs` already is over `Run_Gate`. Same zone as `nomos-check-orchestration`, a named same-zone exception; `nomos-corrections` is a different zone (Agent) this zone's own edges already permit. `P40-CORRECTIONS-CANONICAL-SEAM`, `OD-RULES-020`. |
| Application Service | `nomos-workflow-orchestration` | The workflow tier's first real execution increment: runs an ordered sequence of `WorkflowStep` declarations, each paired with a real dispatch to one of this workspace's four real dispatch targets (`nomos-agent-executor-claude-code`, `nomos-model-backend-ollama`, `nomos-check-orchestration::Run`, `nomos-correction-orchestration::Run_Correction`), refusing whichever step first declares itself incoherent before that step's body ever dispatches — apart from choosing a platform. Same zone as both, and all three of `Body::Check`, `Body::Correction` and `Body::Gate`'s own edges are named same-zone exceptions this workspace's dependency model declares. `OD-WORKFLOW-005`, `P40-WORKFLOW-CHECK-BODY`, `P40-WORKFLOW-CORRECTION-BODY`, `OD-RULES-020`. |
| Application Service | `nomos-agent-orchestration` | The seam for dispatching one `TaskEnvelope` to a chosen backend both hosts call: `Run_Agent_Execute` composes a bare goal and `Run_Agent_Judgment` composes an already-judged `RoleSurfacePair`/`Finding` pair, each dispatching through `nomos-agent-executor-claude-code` or `nomos-model-backend-ollama` — apart from choosing a platform, reading a crate's declared role or surface, or rendering the answer. `nomos-cli`'s own `agent.rs` is a thin renderer over it, the way `correct.rs` already is over `Run_Correction`. Reaches Agent zone directly, no named exception needed (`Permits`). `P43-AGENT-CANONICAL-SEAM-2`. |
| Application Service | `nomos-workspace-discovery` | The one walk every composition root shares: `Walked_Sources` finds every file under a root whose extension a caller-supplied set names, skipping `target`, `.git`, a nested git worktree's own root and a nested repository root (a directory carrying its own `standards.json`, so a vendored third-party tree is judged against the conventions it declares or not here at all — `P96`) — apart from choosing a platform, judging the result or rendering an answer. `Registered_Extensions` names what a registered language package recognizes (`nomos-lang-rust`'s and `nomos-lang-go`'s own extension constants), the one edit a third language package costs this crate instead of a fifth host-side copy; a caller needing `check-script-discipline`'s script extensions too composes its own wider set. Reaches Provider zone directly (`nomos-lang-rust`, `nomos-lang-go`), no named exception needed (`Permits`). Replaces four independent copies of the identical walk (`nomos-api`, `nomos-cli`'s `check` and `gate`, `nomos-lsp`). `P41-WORKSPACE-DISCOVERY-SERVICE-2`, `OD-HOST-008`. |
| Host | `nomos-cli` | The `nomos` binary. |
| Host | `nomos-api` | A second real caller of `nomos-gate-orchestration`'s `Run_Gate`, `Plan` and `Explain` (all three Gate verbs), `nomos-work-orchestration`'s `Run` for all eleven `WorkCommand` verbs (`List`, `Show`, `Validate`, `Audit`, `Claim`, `Renew`, `TakeOver`, `Abandon`, `Decline`, `Finish`, `Add`), `nomos-spec-orchestration`'s `Run` for all nine `SpecCommand` verbs (`Profiles`, `Sources`, `Record`, `Table`, `Markdown`, `Freshness`, `Preview`, `Render`, `Commit`) plus `Submit`, `nomos-correction-orchestration`'s `Run_Correction`, and `nomos-agent-orchestration`'s `Run_Agent_Execute`/`Run_Agent_Judgment` — apart from choosing a platform or wiring an actual transport over it. |
| Repo Tooling | `nomos-surface-provenance` | `[repo tooling]` A report over this repository's own git history, run on demand and invoked from nowhere else: `OD-STORE-002`'s Worked Case join between a crate's surface snapshot and `docs/records/`, for a caller-given commit range. Never a gate. |
| Host | `nomos-api-transport` | **What this workspace serves over a wire, and nothing about the wire** (`OD-HOST-013`, 2026-09-11). `ServedMethod` names the four verbs `OD-HOST-007` and `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2` admit -- `nomos.gate.plan`, `nomos.gate.run`, `nomos.gate.explain`, `nomos.correction.run` -- and none of the twenty-one `Handle_Work_*` and `Handle_Spec_*` handlers those records exclude, because those belong to the `[repo tooling]` crates below. `NomosApiService` is the dispatch into `nomos_api::Handle_*`, and is an `xvpe_remote_call::RemoteCallStrategy`: the JSON-RPC 2.0 envelope, the line framing, the reserved codes and the socket are all `xvpe-remote-call-backend-json`'s. Not itself `[repo tooling]`: it exists precisely to answer a question an end-user repository would ask. Same zone (Host) as `nomos-api`, and one of Host's own named same-zone exceptions rather than a peer edge -- `OD-RULES-020` measured this exact edge as real. Binds no listener of its own; a caller supplies one. |
| Host | `nomos-mcp` | The MCP half of `AGT-006`'s "neutral versioned contracts and MCP tools" commitment, and still a real, runnable MCP server over stdio. **What it holds is the catalogue, not the protocol** (`OD-HOST-013`, 2026-09-11): `ServedTool` names which four tools exist, the sentence each publishes and the JSON Schema each accepts; `NomosToolCatalog` is an `xvpe_remote_call::ToolCatalogStrategy`, and the whole handshake -- `initialize`, `tools/list`, `tools/call`, `ping`, notification suppression, the framing -- is `xvpe-remote-call-backend-json`'s. Depends on `nomos-api-transport` alone -- never on `nomos-api` or any orchestration crate directly -- and a `tools/call` reaches a handler by calling that crate's own `NomosApiService` under the name the client asked for, inheriting its exclusion of every `[repo tooling]` handler rather than repeating it. (It used to reach it by re-serializing the call into a synthetic JSON-RPC line; the shared contract replaced that round trip.) Same zone (Host) as `nomos-api-transport`, another of Host's own named same-zone exceptions. Ships its own binary: unlike its sibling, a stdio transport has no listener for a caller to bind first. |
| Host | `nomos-lsp` | **What this workspace tells an editor, and nothing about the protocol** (`OD-HOST-013`, 2026-09-11). `NomosDiagnosticProvider` is an `xvpe_diagnostics::DiagnosticProviderStrategy`: it walks a tree, runs `nomos-check-orchestration::Run` over it, and `Diagnostics_For` translates each `Finding` into an `xvpe_diagnostics::SourceDiagnostic`, judging nothing itself. `severity::Severity_Of` is the half no shared vocabulary could hold -- what this workspace's own `GateCategory` and `Applicability` deserve. The handshake, the workspace root, `file://` conversion, whole-line spans and stale-marker clearing are `xvpe-language-server-backend-lsp`'s, so **this crate names no protocol library at all**. Depends on `nomos-check-orchestration` and `nomos-correction-orchestration` directly (both Application Service, within Host's own `Permits`) and on `nomos-rules` for `DESCRIPTORS` and `ZONES` -- reading a rule's own declared contract and this workspace's own declared architecture, never a rule's judgment. Re-judges on `didOpen`/`didSave`, full-document sync only, reusing one workspace and fact store across calls (`OD-ANALYSIS-009`). Three of `P42-LSP-PROJECTION`'s five walk-outward targets are answered; two are named undecided in `OD-HOST-010`. |
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
nomos work list [--state ready|waiting|held|snagged|stranded|claimed|blocked|done|declined]
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
of work the item is — `capability`, `decision`, `validation`, `correction` or `cleanup` —
and `--origin` says whether a person required it or a session proposed it. An unrecognized
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
nomos request submit --kind <feature-request|design-spec|feature-result> --id <node-id>
                      --by <name> [--state draft|accepted] [--contract-version <n>]
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
it does not give up is the tables above: `tests/contract/tests/boundaries.rs` compares them
against `nomos-rules`' own declared `ZONES` in both directions, so a crate that joins the
workspace without joining this page fails the gate. See
`docs/records/OD-PROJECT-001-the-repository-readme-is-not-the-suites-overview.md`.
