<!--
  Visitor landing page. GitHub shows .github/README.md in place of the root README.md, so this is
  what someone arriving at github.com/kevinmettias/nomos reads first. The root README.md is the
  workspace's own authority on what exists and what owns what, and tests/contract checks it against
  the real workspace; this page says what the project is for and routes there. It deliberately
  restates none of that file's tables, and every count below carries the date it was measured.
-->

# Nomos

**A control plane for software engineering.** Nomos builds a model of a codebase from what
compilers, parsers and existing tools can prove about it, judges that model against the
architecture and rules the repository declares, and coordinates the people and AI agents who
change it so that concurrent work cannot quietly undo what was decided.

Rust · 76 crates · ~300,000 lines · ~5,500 tests · 272 decision records
<sub>(measured September 2026)</sub>

---

## The problem

In any large codebase, four things drift apart. There's the architecture people intend, the
rules they agreed on, the reasons behind past decisions, and the code that actually ships.
Usually nobody notices until the gap is expensive. AI coding agents speed that drift up. Several
agents working one repository can each make a reasonable local change, and together those
changes can erode a boundary nobody wrote down.

Nomos makes that alignment mechanical instead of social:

- **What the code is** comes from facts that independent providers establish, not from
  someone's summary of it.
- **What the code should be** is declared as data (architectural zones, permitted dependencies,
  naming and size policies) and judged by deterministic rules.
- **Who may change what** is decided by a work ledger. It grants two tasks at once only when
  it can prove they touch disjoint files.

## What works today

- **Multi-language analysis with honest uncertainty.** Rust is read through `syn`, through
  rust-analyzer's semantic engine (`ra_ap_hir`) for questions that need resolved types, and
  through clippy, cargo-deny and cargo metadata, which are wrapped as providers rather than
  reimplemented. Go has a tree-sitter parser and a module reader. C# has a tree-sitter provider,
  and wiring it into the check run is the next item on the board. Every provider declares how
  complete and sound its answers are. When two providers of the same fact disagree, Nomos
  reports the disagreement instead of hiding it.
- **Rules and a real gate.** About 70 deterministic rules cover dependency direction between
  declared zones, write authority, naming, lint and dependency policy, and requirement
  traceability. A gate composes them into a build verdict, with baselines, suppressions and
  adoption policies, so an existing codebase can adopt it without fixing everything first.
  `nomos gate compare` shows what a change added, removed or moved.
- **One engine, several front ends.** The same judgment is served through a CLI, a Language
  Server that shows findings in an editor, an MCP server that exposes tools an AI agent can call,
  and a resident daemon that re-judges only what changed since its last answer.
- **Coordination for concurrent agents.** Every task in the ledger reserves a *territory*: the
  paths it may change. Claims are leases, so a crashed agent can't hold work forever. A task is
  recorded done only when its declared verification command actually exits zero, and the exit
  codes separate "taken, try another task" from "broken, fetch a person".
- **Corrections and agent dispatch.** A finding can become a staged, validated change that can
  be rolled back. A task can be handed to Claude Code, or to a local model through Ollama, inside
  an explicit capability boundary: an isolated working directory and no tools beyond the ones
  granted.
- **A specification that can't silently lose content.** The product specification lives in a
  database behind a preservation ledger that fails closed. An earlier markdown revision had
  silently deleted 282 table rows and 132 sections. The content hash that ledger depends on was
  reverse-engineered from the old corpus and verified against all 2,533 recorded block hashes.

## Where it's going

[`ARC-ROADMAP-001`](/docs/records/ARC-ROADMAP-001-nomos-core-s-near-term-boundary-is-the-headless-enforcement-loop-not-architecture-and-feature-intelligence.md)
draws the line. The near-term product is the **headless enforcement loop**: a stable kernel,
incremental analysis, rules, gates, baselines, corrections, workflow orchestration, and parity
across the CLI, API and MCP surfaces. Architecture discovery, feature-path tracing, runtime and
debugger intelligence, and a visual explorer are deliberately deferred until that substrate is
production-grade. The specification already asks for them.

## How it's built

Nomos is developed by the process it implements.

- **Several AI coding agents work this repository at the same time**, coordinated by the ledger
  in [`work/`](/work/ledger.json). More than 1,200 tasks have been claimed, verified and
  finished through it since August 2026.
- **Every architectural decision is a record** in [`docs/records/`](/docs/records/): the
  question, what was measured, what was decided, and what was refused. Records are embedded in
  the binary and cited from code, and when a record and the code disagree, the disagreement is
  filed as work instead of being argued away.
- **Documentation is checked like code.** The architecture tables in the engineering
  [README](/README.md) are asserted against the real workspace in both directions, so a
  crate that joins without being documented fails the build.
- **Panics are treated as determinism defects.** `unwrap`, unchecked indexing and unchecked
  arithmetic are denied by lint on the analysis path. A replay has to reach the same answer at
  the same step.

Most commits are co-authored with Claude. My part is the architecture, the decision records,
the governance the agents work under, and auditing their output against it.

## Part of a larger system

Nomos is one of four products that share one boundary, and
[`ARC-ECOSYSTEM-001`](/docs/records/ARC-ECOSYSTEM-001-four-products-share-one-seam-and-ownership-is-decided-by-semantics.md)
decides which product owns a responsibility by what it *means*, not by where the code
happens to live:

| Project | Owns |
|---|---|
| [**XVPE**](https://github.com/kevinmettias/xvpe) | Generic runtime and platform infrastructure: scheduling, storage, transport. Nothing in it mentions software engineering. |
| **Nomos** (this repository) | Software-specific reality: analysis, architecture, rules, gates, corrections, and evidence that they hold. |
| [**KWB**](https://github.com/kevinmettias/kwb) | Knowledge: rationale, intent, claims, and where each claim came from. |
| Repository tooling | Machinery that exists only to build these repositories, such as the work ledger, kept separate so a scaffold does not become a shipped feature by accident. |

Nomos is a ground-up Rust rewrite of an earlier Go prototype, a code-conformance linter whose
rules are being migrated across one at a time.

## Try it

Requires Rust 1.88 (pinned in [`rust-toolchain.toml`](/rust-toolchain.toml)).

```sh
cargo build --release -p nomos-cli
./target/release/nomos gate plan --root .   # which rules this build composes
./target/release/nomos check --root .       # judge a tree
./target/release/nomos work list            # the live task board
```

## Reading further

| If you want | Read |
|---|---|
| What exists, and which crate owns what | [README.md](/README.md), the engineering README |
| Why something was decided | [docs/records/](/docs/records/) |
| How an agent or a person works this repository | [AGENTS.md](/AGENTS.md) |
| What must pass before a change lands | [.github/workflows/gate.yml](/.github/workflows/gate.yml) |
