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

Crates are ordered into bands. A crate may depend only on crates in a strictly lower
band, and `tests/contract` asserts it.

Four rows below are marked `[repo tooling]`: `nomos-ledger`, `nomos-work-orchestration`,
`nomos-spec-orchestration` and `nomos-surface-provenance` exist to develop or preserve this
repository, not to answer a question an end-user repository would ask Nomos.
`ARC-ECOSYSTEM-001` names why a shared band table does not by itself mean shared product
ownership, and `OD-LEDGER-036` settles the ledger specifically. The `nomos-spec-*` family
below the main table carries the identical distinction in its own prose already. The mark
here does not move a crate, does not change a dependency, and is not itself a decision — it
only makes a decision two other records already made visible where a reader would otherwise
have to infer it from band position alone.

| Band | Crate | Owns |
|---|---|---|
| 0 | `nomos-contracts` | Protocol truth. What earns a place in it is `OD-CONTRACTS-001`. Depends on `serde` and nothing else. |
| 10 | `nomos-model` | Subjects, composite identity, evidence, and the `SubjectSet` exclusion primitive. |
| 12 | `nomos-store` | Content-addressed documents, with one write door per authority. |
| 15 | `nomos-platform` | Port traits: clock, filesystem, cross-process lock, process launcher. |
| 16 | `nomos-platform-std` | The std implementation of those traits. |
| 18 | `nomos-workspace` | Snapshots, build variants, and the single change door. |
| 19 | `nomos-scope-verification` | `Territory` and `VerificationPredicate` -- the two ledger-agnostic primitives `OD-LEDGER-037` found underneath `nomos-agent-contracts`'s reuse of `nomos-ledger`, moved verbatim to their own crate below both. `nomos-ledger` re-exports both for its own claim/overlap and finish logic, exactly as `nomos-lang-rust-package` re-exports `nomos-package`'s domains (`OD-PACKAGE-007`). |
| 20 | `nomos-ledger` | `[repo tooling]` Territory-based mutual exclusion over `work/ledger.json`. |
| 21 | `nomos-capability` | The contract registry whose answer is never a bare no. |
| 22 | `nomos-analysis` | Fact identity, fact readers, the fact store, and invalidation. |
| 23 | `nomos-cap-syntax` | A capability contract, housed below every provider that offers against it. |
| 23 | `nomos-cap-dependency` | The `nomos.cap.dependency.edges` contract — a workspace member's own first-party dependency edges, resolved by Cargo. Housed below its provider and below the rule that reads it: `nomos-rules` is a real second party from the day it was written, so this did not wait beside `nomos-lang-rust-cargo` the way `nomos.cap.module.index` waits inside `nomos-lang-rust`. |
| 23 | `nomos-cap-controlflow` | The `nomos.cap.controlflow.reachability` contract — whether a control-flow path forward from a fact-read failure reaches a `Finding`. Housed below its provider (`nomos-lang-rust`) and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason `nomos-cap-dependency` did not wait beside its own provider. |
| 23 | `nomos-cap-lint` | The `nomos.cap.lint.diagnostics` contract — one workspace member's own diagnostics from an external lint tool. Housed below its provider (`nomos-lang-rust-clippy`) and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason `nomos-cap-dependency` did not wait beside its own provider. `OD-RULES-010`, the first real `ToolProvider`. |
| 23 | `nomos-cap-dependency-policy` | The `nomos.cap.dependency.policy` contract — the workspace's own bans/licenses/sources verdict from `cargo deny`, over the resolved dependency graph as a whole (`IncrementalGranularity::WholeWorkspace`, unlike `nomos-cap-lint`'s per-member `Project`). Housed below its provider (`nomos-lang-rust-deny`) and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason its four siblings above did not wait beside their own providers. `OD-RULES-010`'s second real `ToolProvider`. |
| 23 | `nomos-cap-naming-policy` | The `nomos.cap.naming.policy` contract — a repository's own declared naming convention, read from `standards.json` as data rather than compiled as a constant, over code-standards' own closed eight-style `Case` vocabulary. Housed below its provider (a future `crates/repository/` crate) and below the rules that read it (`nomos-rules`), the same real-second-party-from-day-one reason its five siblings above did not wait beside their own providers. `OD-RULES-011`. |
| 23 | `nomos-cap-limits-policy` | The `nomos.cap.limits.policy` contract — a repository's own declared numeric thresholds (file-size triggers, parameter-count caps and their like), read from `standards.json` as data rather than compiled as a constant, over scope-qualified rows rather than `nomos-cap-naming-policy`'s closed `Case` vocabulary since a threshold is a bare number, not a value drawn from a fixed style set. Housed below its own future provider and below the rules that read it (`nomos-rules`), the same real-second-party-from-day-one reason its six siblings above did not wait beside their own providers. `OD-RULES-011`'s own threshold-family instance. |
| 23 | `nomos-cap-scripting-policy` | The `nomos.cap.scripting.policy` contract — a repository's own declared tooling language and forbidden script extensions, read from `standards.json` as data rather than compiled as a constant. Unlike its two siblings, its payload is a plain optional scalar plus a list rather than scope-qualified rows: `check-script-discipline`'s own `spec.go` states that refining "which language is my tooling" *by* language would be circular. Housed below its own future provider and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason its seven siblings above did not wait beside their own providers. A third `OD-RULES-011` instance. |
| 23 | `nomos-cap-words-policy` | The `nomos.cap.words.policy` contract — a repository's own additions to code-standards' default approved-abbreviation vocabulary, read from `standards.json`'s `words.approved_abbreviations` as data rather than compiled as a constant. A bare list, simpler than any of its three siblings: a vocabulary addition has no scope to qualify. Removal is not supported — a repository extends the default list, per code-standards' own stated philosophy, rather than replacing it. Housed below its own future provider and below the rule that reads it (`nomos-rules`), the same real-second-party-from-day-one reason its eight siblings above did not wait beside their own providers. A fourth `OD-RULES-011` instance. |
| 24 | `nomos-package` | The language-agnostic manifest core: `PackageId`, `PackageKind` and `PKG-007`'s four version domains, minus any typed version-domain abstraction or provider allowlist a specific language would supply. Depends on nothing above `nomos-contracts`. |
| 25 | `nomos-lang-rust` | Recognition and syntax facts from `syn`, and the module rollup derived from them — the one fact in this workspace computed from other facts. |
| 25 | `nomos-lang-rust-scan` | The second provider of that capability. Same band, so neither may name the other. |
| 25 | `nomos-lang-go` | The first real second-language provider of `nomos.cap.syntax.items`: recognition and syntax facts from `tree-sitter-go` in place of `syn`. Declares `Assurance::Sound` on both axes rather than one — Go has no macro system, so there is no construct where its parse tree ends in an unexpanded token stream the way a Rust macro invocation does, a claim checked against the real grammar (generics, build tags) before it was written. Same band as its two Rust-reading siblings; none of the three may name either of the others. |
| 25 | `nomos-lang-rust-cargo` | The one provider of `nomos.cap.dependency.edges` — runs `cargo metadata` and reads the filesystem, the only I/O any provider in this workspace performs. Composed into `nomos-check-orchestration::Run`, which materializes its edges and hands them to `Check_Dependency_Direction`. |
| 25 | `nomos-lang-rust-clippy` | The one provider of `nomos.cap.lint.diagnostics` — runs `cargo clippy --message-format=json` and reads the filesystem, the same subprocess-backed shape as `nomos-lang-rust-cargo`. Composed into `nomos-check-orchestration::Run`, which materializes one fact per workspace member and hands them to `Check_Lint_Diagnostics`. `OD-RULES-010`. |
| 25 | `nomos-lang-rust-deny` | The one provider of `nomos.cap.dependency.policy` — runs `cargo deny --format json check bans licenses sources` (its own JSON stream lands on stderr, not stdout) and reads the filesystem, the same subprocess-backed shape as `nomos-lang-rust-clippy`. `advisories` is deliberately excluded: it is the one `cargo deny` check that fetches the RustSec database over the network. `OD-RULES-010`'s second real `ToolProvider`. |
| 25 | `nomos-lang-go-modules` | A second provider of `nomos.cap.dependency.edges`, for a Go workspace rather than a Cargo one. Reads `go.work`/`go.mod` text directly — a module path is already Go's own unambiguous identity, so no subprocess resolution is needed the way Cargo's manifest text requires. Composed into `nomos-check-orchestration::Run` alongside `nomos-lang-rust-cargo`; its honestly weaker completeness never clears `Check_Dependency_Direction`'s own floor, so the two never compete for one subject. Same band as its three siblings; none of the four may name any other. `OD-CAPABILITY-009`. |
| 25 | `nomos-repo-standards` | The one provider of `nomos.cap.naming.policy` — reads `standards.json`'s `naming` and `languages.*.naming` blocks through `nomos_platform::FileSystem`, an absent file declaring nothing rather than failing. The first crate under `crates/repository/` rather than `crates/languages/`: it reads one repository-wide configuration file, not one language's source or manifest format. `OD-RULES-011`. |
| 25 | `nomos-repo-limits` | The one provider of `nomos.cap.limits.policy` — reads `standards.json`'s `limits` and `languages.*.limits` blocks through `nomos_platform::FileSystem`, alongside `nomos-repo-standards` for the identical reason. This item wrote the `limits` block itself: no such block existed in this repository's `standards.json` before it, so its values are set equal to `nomos-rules`' prior hardcoded file-size/parameter-count defaults. `OD-RULES-011`'s threshold-family instance. |
| 25 | `nomos-repo-scripting` | The one provider of `nomos.cap.scripting.policy` — reads `standards.json`'s `scripting.tooling_language` and `scripting.forbidden_extensions` through `nomos_platform::FileSystem`, alongside `nomos-repo-standards` and `nomos-repo-limits` for the identical reason. An empty declared language is treated as undeclared, matching `check-script-discipline`'s own equivalence. `OD-RULES-011`'s third instance. |
| 25 | `nomos-repo-words` | The one provider of `nomos.cap.words.policy` — reads `standards.json`'s `words.approved_abbreviations` through `nomos_platform::FileSystem`, alongside its three siblings for the identical reason. `OD-RULES-011`'s fourth instance. |
| 26 | `nomos-lang-rust-package` | The Rust `LanguagePackage` manifest format and its refusing reader — wraps `nomos-package`'s generic core with `RustEdition` resolution and this workspace's two Rust providers. |
| 26 | `nomos-model-package` | The first `ModelBackendPackage`/`AgentExecutorPackage` manifest maturity: identity, `PackageKind`, `PKG-007`'s first two version domains reused from `nomos-package` unchanged, and `ModelSelection` replacing the fourth. A peer of `nomos-lang-rust-package`, not a dependent of it. Also carries a growing declared-not-computed execution/routing vocabulary, now ten maturities answering twenty-five `MODEL-ROUTE` requirements `OD-PACKAGE-011` v3 licensed — none of it wired into a real consumer yet. |
| 26 | `nomos-rule-package` | The first `RulePackage` manifest maturity, measured field by field against four real shipped rules rather than invented (`OD-PACKAGE-008`). A third peer wrapping `nomos-package`'s generic core, not a dependent of `nomos-lang-rust-package` or `nomos-model-package`. |
| 26 | `nomos-lang-go-package` | The first real second consumer of `nomos-package`'s generic core: the Go `LanguagePackage` manifest format, wrapping it with `GoVersion` resolution and `nomos-lang-go`'s own syntax provider in place of `RustEdition` and Rust's two. A fourth peer of the three above; none of the four names another. `OD-PACKAGE-006`, `OD-PACKAGE-007`. |
| 30 | `nomos-rules` | A rule as a pure function whose subject is an argument: source it is handed, and facts it reads through a `FactReader`. |
| 35 | `nomos-corrections` | `CorrectionCandidate`, `CorrectionPlan`, and the deterministic preview, stage, validate, commit and rollback lifecycle over a workspace change. No agent or model backend decides anything here; `Commit` takes the caller's `Evidence` but does not judge it. |
| 36 | `nomos-agent-contracts` | `AGT-001`'s `TaskEnvelope` and `AGT-002`'s `WorkResult` — the typed input and output shape an agent-assisted operation carries, bundling what was scattered across `nomos-scope-verification`, `nomos-contracts` and `nomos-corrections`. Neither type computes anything; a caller fills a `TaskEnvelope` in and an agent's own response fills a `WorkResult` in. |
| 37 | `nomos-agent-executor-claude-code` | The first real `AgentExecutor`, and the one concrete implementation this workspace has today — its name says which, so a second one (a different agent CLI, a human, a replay) has an honest name left to take rather than inheriting a canonical-sounding one it never earned. Dispatches a `TaskEnvelope`'s `goal` to Claude Code as a subprocess through `nomos-platform`'s `ProcessLauncher`, bounded by `OD-EXECUTOR-001`'s structural capability boundary — an isolated working directory, no MCP config, an allow-list naming no real tool, one `--print` turn — and reads the result for what it structurally permitted, never for its own free-text claims. Does not assemble a `WorkResult`: `CorrectionPlan::New` refuses an empty candidate list, so a judgment-only task has no plan to report yet. |
| 37 | `nomos-model-backend-ollama` | The first real `ModelBackend` adapter, not a second `AgentExecutor` (`OD-PACKAGE-013`) — built and named as the latter, then measured against `PackageKind::ModelBackendPackage`'s and `PackageKind::AgentExecutorPackage`'s own doc comments and found to match the former: a fixed model, one forwarded field, every other `TaskEnvelope` field read and ignored, no tool-use loop, no MCP surface. Structurally parallel to `nomos-agent-executor-claude-code`, no shared trait, no shared package kind. Dispatches a `TaskEnvelope`'s `goal` to a local Ollama model (`ollama run`) as a subprocess through `nomos-platform`'s `ProcessLauncher`, bounded by `OD-EXECUTOR-004`'s structural capability boundary, measured against `ollama run`'s own real mechanism rather than inherited from `OD-EXECUTOR-001` by analogy: never `--experimental`/`--experimental-yolo`/`--experimental-websearch` — the only flags that open any tool-use capability, so their absence is the entire boundary, because there is no tool subsystem to grant into in the first place. No per-call dollar cost (inference is local); a wall-clock timeout stands in its place. |
| 40 | `nomos-work-orchestration` | `[repo tooling]` Runs a `nomos work` verb against a caller-chosen platform and hands back a typed outcome — generic over `nomos-platform`'s traits, so a second adapter can depend on it without also depending on `nomos-platform-std` or on how `nomos-cli` renders an answer. |
| 40 | `nomos-check-orchestration` | Composes the capability registry, ingests already-walked source into facts and judges it, and hands back a typed outcome — apart from choosing a platform, walking a tree or rendering the answer. |
| 40 | `nomos-spec-orchestration` | `[repo tooling]` Assembles the specification store from the embedded governing records and a caller-named corpus, and answers all nine `SpecCommand` verbs plus `nomos request submit`'s `Submit`, generic over `nomos-platform`'s traits where a verb reads or writes — apart from choosing a platform or rendering the answer. |
| 41 | `nomos-gate-orchestration` | The seam for the first-class Gate object `ARC-ROADMAP-001` names: `Plan` composes a real rule registry and reports what it holds, `Run_Gate` composes `nomos-check-orchestration` into a judged disposition — apart from selecting scope, choosing a platform or rendering the answer. Above `nomos-check-orchestration`'s band so it may depend on it. |
| 90 | `nomos-cli` | The `nomos` binary. |
| 90 | `nomos-api` | A second real caller of `nomos-gate-orchestration`'s `Run_Gate`, `Plan` and `Explain` (all three Gate verbs), `nomos-work-orchestration`'s `Run` for all eleven `WorkCommand` verbs (`List`, `Show`, `Validate`, `Audit`, `Claim`, `Renew`, `TakeOver`, `Abandon`, `Decline`, `Finish`, `Add`), and `nomos-spec-orchestration`'s `Run` for all nine `SpecCommand` verbs (`Profiles`, `Sources`, `Record`, `Table`, `Markdown`, `Freshness`, `Preview`, `Render`, `Commit`) plus `Submit` — apart from choosing a platform or wiring an actual transport over it. |
| 91 | `nomos-surface-provenance` | `[repo tooling]` A report over this repository's own git history, run on demand and invoked from nowhere else: `OD-STORE-002`'s Worked Case join between a crate's surface snapshot and `docs/records/`, for a caller-given commit range. Never a gate. |
| 100 | `nomos-contract-tests` | The assertions in `tests/contract`. Observes the workspace; nothing observes it. |
| 100 | `nomos-integration-tests` | The vertical slice, driving the product through its seams. Its peer, not its layer. |

The specification system sits beside the kernel rather than above it. It reaches the
product only through a knowledge capability, so nothing in the product may name it.

| Band | Crate | Owns |
|---|---|---|
| 11 | `nomos-spec-model` | The canonical normalizer and hashing. The single authority on what content hashes to. |
| 12 | `nomos-spec-store` | Schema, migrations, and this repository's own governing records. |
| 13 | `nomos-spec-bundle` | Deterministic JSONL export and import — the portable authority committed to git. |
| 13 | `nomos-spec-ingest` | Parsers and the manifest gate against the real v14 corpus. |
| 14 | `nomos-spec-validate` | The `NSV-PRESERVE-*` rules and the run that fails closed. |
| 14 | `nomos-spec-project` | Eighteen projection profiles, the renderers, and the freshness stamp. |

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
workspace, and no content kind in the projection system selects a crate's band or a gate
command — so no profile can render it, and the corpus it would be rendered from is on no
CI runner. It therefore stays hand-authored.

What that gives up is freshness for the prose, and nothing here pretends otherwise. What
it does not give up is the tables above: `tests/contract/tests/boundaries.rs` compares them
against the declared bands in both directions, so a crate that joins the workspace without
joining this page fails the gate. See
`docs/records/OD-PROJECT-001-the-repository-readme-is-not-the-suites-overview.md`.
