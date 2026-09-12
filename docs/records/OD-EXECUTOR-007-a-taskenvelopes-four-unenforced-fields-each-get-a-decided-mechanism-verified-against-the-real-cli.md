---
id: OD-EXECUTOR-007
type: decision
title: A TaskEnvelope's four unenforced fields each get a decided mechanism, verified against the real CLI
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - agent
  - executor
  - security
relations:
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-EXECUTOR-002
    type: relates-to
  - target: OD-CONTRACTS-003
    type: relates-to
---

# A TaskEnvelope's four unenforced fields each get a decided mechanism, verified against the real CLI

## Question

`OD-EXECUTOR-001` measured that `TaskEnvelope.scope`, `.prohibited_changes` and
`.available_tools` were accepted and read by nothing, and declined to close the gap: "when a
real task needs a real tool, naming what enforces that field and how a grant is checked is a
later record's question, measured against a real need rather than designed ahead of one." A
person has now required exactly that question answered, for those three fields and a fourth
this record's own why also names, `applicable_rules`.

Closing this honestly needs two things reading the code cannot supply. First, whether the
real `claude` CLI's documented flags actually confine a subprocess the way their own help
text claims — `OD-EXECUTOR-001`'s own history is that a deny-list which looked complete
leaked twice before an allow-list was tested and held, so a claim about this CLI's behavior
is not settled by its `--help` text alone. Second, `Execute_Task`'s own signature has no root
path to resolve `Territory`'s repo-relative paths against, and widening it reaches real
callers — `nomos-cli::agent::dispatch` and `nomos-workflow-orchestration::run` — outside any
one crate's territory. Both are measured before this record decides anything.

## What Was Measured

**`--restricted` plus `--add-dir` structurally confines file-tool reach, verified
adversarially against the real CLI.** `claude --help` documents `--restricted` as confining
"the file tools to the working directories (`--add-dir` included)." A first real invocation —
`--restricted --add-dir <allowed>` with `Read`/`Write`/`Edit` allowed, prompted to write and
read a file under a sibling `forbidden` directory — reported `permission_denials: []` and
completed with a free-text refusal. That is not evidence the boundary holds: it is the exact
"trust in what the process is asked to do" failure `OD-EXECUTOR-001`'s own third run already
found — the model chose not to try, which proves nothing about what happens when it does. A
second invocation, prompted explicitly not to decline and to report the raw tool result,
reached the real tool calls: `permission_denials` recorded both a `Write` and a `Read`
outside `allowed`, each with the CLI's own reason ("is outside ...; `--restricted` confines
the file tools to the working directory"), and the filesystem confirmed it directly — the
forbidden file was never created, the secret file's contents were unread and unchanged. This
is the same allow-list-tested-adversarially shape `OD-EXECUTOR-001`'s own rule already uses
for tool names, now confirmed for path confinement too.

**`Execute_Task` cannot use this today: it has no root.** `scope` and `prohibited_changes`
are `Territory` values — repository-relative path strings — and `Execute_Task(task, launcher)`
receives nothing to resolve them against. `--add-dir` needs real, absolute directories.
Threading a root through means changing a signature two real callers outside
`nomos-agent-executor-claude-code` depend on.

**`available_tools: Vec<CapabilityId>` names Nomos capabilities, not CLI tools or paths, and
nothing wires either to the other.** `nomos.cap.syntax.items`, `nomos.cap.dependency.edges`
and their siblings are facts this workspace's own registry resolves; `--allowedTools` grants
CLI-native tool names (`Read`, `Bash(git *)`) that have no relationship to a `CapabilityId`.
The one bridge that could carry a Nomos capability to an external subprocess is MCP —
`nomos-mcp` exists and serves three Gate verbs — but this executor passes
`--strict-mcp-config` with no `--mcp-config`, so no MCP server is reachable regardless, and
`nomos-mcp` does not serve capability facts today in any case. There is no real path from a
declared `available_tools` list to anything the subprocess could act on without first
building that bridge.

**`applicable_rules: Vec<RuleId>` has no output for a validation boundary to check yet.**
`OD-CONTRACTS-003` already found this crate does not assemble a `WorkResult`; it returns
free text. A rule judges code (`nomos_analysis::FactReader` over `&[SourceFile]`), and this
invocation, even once `scope` is real, produces no `WorkResult.plan` a correction pipeline
would apply and no code change of its own for a rule to run against — `nomos-corrections`'
own stage/validate/commit chain still has zero real callers, per `OD-CORRECTIONS-001`.

## The Decision

**`scope`: enforced by `--restricted` plus one `--add-dir` per path `scope` names, resolved
against a caller-supplied root.** `Execute_Task` and `Execute_In` gain a `root: &Path`
parameter. `Command_For` adds `--restricted` and one `--add-dir <root-joined-path>` per entry
in `task.scope.paths` — the parent directory when an entry names a file, since `--add-dir`
confines by directory and `Territory::Of_Files` may name either. `Territory::Empty()` (the
common case today, per every existing caller's own fixture) adds no `--add-dir` at all,
matching `OD-EXECUTOR-001`'s own "nothing enumerated, nothing granted" reading — an empty
scope keeps today's exact behavior, structural denial of everything, unchanged. `Read`,
`Write` and `Edit` join the allow-list once at least one directory is granted; the
placeholder-only allow-list stays exactly as it is when scope is empty, so a caller that
never populates `scope` sees no behavior change at all.

**`prohibited_changes`: contained by comparison, not prevented by a flag — `--add-dir` has
no documented carve-out syntax, and this record does not invent one to test unverified.**
Before dispatch, hash every real file `prohibited_changes.paths` names (root-relative,
skipping any that do not exist — nothing to protect there yet). After the subprocess
returns, hash them again. Any difference is not read as `Execute_Task` succeeding with a
violation buried in a report: it is `AgentExecutionError`'s own new variant,
`Prohibited_Change`, naming which path changed, so a caller sees a structural refusal rather
than a `WorkResult` it has to separately audit. `OD-EXECUTOR-001`'s own "read for what
structurally permitted, never for what free text claims" is the identical principle applied
to the after-state rather than the invocation.

**`available_tools`: enforcement is refusal, not silent tolerance, until MCP carries a real
capability.** An executor that cannot honor a declared need must say so rather than proceed
as if it could — the same "unknown is not pass" reading `Applicability`'s own module doc
already states for a rule's judgment. `Execute_Task` refuses with a new
`AgentExecutionError::Unsupported_Tools` when `task.available_tools` is non-empty, naming
the capabilities it cannot grant, rather than dispatching a task whose declared requirement
this crate silently drops. An empty list — every real caller today — dispatches exactly as
now. Building the real grant (`nomos-mcp` serving capability facts, this executor supplying
`--mcp-config` scoped to exactly `available_tools`) is not decided here: it is a
capability-serving increment `nomos-mcp`'s own territory would need first, not a fact about
this executor alone.

**`applicable_rules`: context now, validation deferred.** The rule identifiers `task.
applicable_rules` names are appended to the invocation via `--append-system-prompt`, so the
model is told what it is bound by before its one turn runs — real, buildable today, and
consistent with everything else this crate already does structurally rather than by
instruction alone: telling the model is not enforcement, and this record does not claim
otherwise. The validation half — running `nomos check` with exactly `applicable_rules`
selected over whatever `scope` granted, once a real `WorkResult` exists to check — stays
undecided; `OD-CONTRACTS-003`'s own gap (no `WorkResult` assembly) is the blocker, not this
field, and is a different record's question.

## What This Record Does Not Do

**No code changes here.** `Execute_Task`'s new `root` parameter, `Command_For`'s new flags,
the two new `AgentExecutionError` variants, and a test per mechanism that fails when the
constraint is removed are a follow-up item's own territory — this record states the shape,
the same way `OD-RULES-020` stated zones before `P41-ZONES-MIGRATION-3` built them.

It does not update `nomos-cli::agent::dispatch` or `nomos-workflow-orchestration::run` to
supply a real root; both currently have none to give, since no caller in this workspace
constructs a `TaskEnvelope` with a real `scope` or `prohibited_changes` value today. Naming
where that root comes from for each real caller is the follow-up item's own measurement, not
assumed here.

It does not build MCP capability-serving, `nomos-mcp` wiring, or a `WorkResult`-assembly
step. Both are named as the real, specific blockers for `available_tools` and the validation
half of `applicable_rules`, not built toward speculatively.

It does not claim the mechanism above is exhaustively adversarial-tested. Two real
invocations verified the one property this record leans hardest on — that `--restricted
--add-dir` structurally refuses reach outside its own list — over a single path shape (an
absolute directory, one level of nesting). A follow-up implementing this against real
`Territory` values with nested paths, nonexistent paths, or nomos-specific edge cases is
that item's own verification, not re-derived here.

## Amendment: The XVPE Dispatch Migration Removed `scope`'s Primitive, Left The Rest Standing

Added at version 2. Commit `9b3e9683` (2026-09-12, same day as this amendment,
`OD-PLATFORM-003`/`OD-HOST-013`) moved this crate's dispatch down into
`xvpe-agent-backend-claude-code` and rewrote its command-line construction wholesale.
`P42-EXECUTOR-ENVELOPE-IMPLEMENTATION-2`, filed to build this record's four mechanisms,
found this before writing any code and declined rather than build against a stale premise.
This amendment measures which of the four mechanisms that migration actually broke, rather
than let the decline's own broad claim stand unexamined.

### What actually broke

**`scope`, and only `scope`.** The engine's `AgentCapability` now offers exactly
`AgentWorkspace::{Isolated, Existing(one PathBuf)}` and `ToolGrant::{Nothing, Edits}` --
grepped across `crates/backends/agent` and `xvpe-agent-execution` in the sibling `xvpe`
checkout: zero real hits for `restricted` or `add-dir` anywhere, only two stale comments in
unrelated backends (`deepseek`, `kimi`) explaining why nothing is restricted in *their*
adapters. `Push_Capability_Flags` in `claude_code_executor.rs` emits only
`--allowedTools`/`--permission-mode`/`--permission-prompts`/`--max-budget-usd` -- no flag
grants or confines a subset of a directory's contents. `AgentWorkspace::Existing`'s own doc
says plainly: "An existing directory, with whatever it already contains" -- the directory
itself is the entire boundary, not a list of paths within it. `Territory`'s shape (an
arbitrary file list, possibly scattered across unrelated directories) has no primitive left
to translate onto.

**`prohibited_changes` did not break.** Its decided mechanism -- hash every real file it
names before dispatch, hash again after, and report `AgentExecutionError::Prohibited_Change`
on any difference -- is nomos's own file comparison around whichever dispatch call runs. It
never depended on `--restricted`, `--add-dir`, or any other flag this migration touched.
Threading `root: &Path` through `Execute_Task`/`Execute_In` to resolve `Territory`'s
repo-relative paths is exactly as buildable today as it was on 2026-09-05.

**`available_tools`'s refusal did not break either.** "Refuse with
`AgentExecutionError::Unsupported_Tools` when `task.available_tools` is non-empty" is a
check against the envelope's own field, made before any dispatch happens. It reads nothing
from `AgentCapability` and needed no flag this migration removed.

**`applicable_rules` lost its named channel, not its disposition.** `--append-system-prompt`
does not exist anywhere in `xvpe-agent-backend-claude-code` (grepped: zero hits). `AgentTask`
itself, in the engine's own module doc, now states the boundary explicitly: "which files are
in scope, which rules apply, what it must not change" are meaningful "only if something
enforces" them, and names exactly two homes for anything else -- `AgentCapability`, where it
is structural, "or in the goal's own text, where it is plainly advisory." `applicable_rules`
was already decided as advisory context, never enforcement (`OD-CONTRACTS-003`'s
`WorkResult`-assembly gap still blocks the validation half, unchanged). Folding the named
rule ids into `task.goal`'s own text reaches the same disposition this record already
decided, through the one channel the engine still offers for it.

### The Decision, Revised

`prohibited_changes`, `available_tools`, and `applicable_rules` (via goal-text folding
rather than `--append-system-prompt`) stand exactly as this record originally decided them
and remain buildable without further engine change.

`scope` has no mechanism left to build against. This record does not invent one now. A
staging mechanism -- nomos copies exactly `scope`'s named files into an isolated directory
it owns, dispatches `AgentWorkspace::Existing` over that copy, and reconciles only what
comes back -- is a plausible next candidate, but this record's own opening measurement
insisted a mechanism claim here is not settled by design alone, only by adversarial
verification against the real CLI, the same discipline that caught the deny-list leaking
twice before an allow-list was tested and held. Electing a replacement without that
verification would be exactly the mistake this record's own history warns against, so
`scope` reverts to undecided: accepted and structurally unenforced, same as `available_tools`
was left before this record, until a future record measures a real candidate the way this
one measured `--restricted`/`--add-dir`.

### Disposition of the stranded dependent

`P42-EXECUTOR-ENVELOPE-IMPLEMENTATION-2`'s decline stranded `P42-SECOND-HARNESS-EXECUTOR-3`,
whose own precondition needed a first real enforced mechanism to compare a second executor's
choices against. That precondition is not gone -- three of the four fields are still
buildable exactly as decided -- so `P42-SECOND-HARNESS-EXECUTOR-3` is declined and
re-authored as `P42-SECOND-HARNESS-EXECUTOR-4`, depending on
`P42-EXECUTOR-ENVELOPE-IMPLEMENTATION-3` (this amendment's own re-authored successor to the
declined `-2`) rather than left stranded behind an id that will never finish.

## Status

Accepted. `prohibited_changes` and `available_tools` each have a real, evidence-backed
mechanism unaffected by the 2026-09-12 XVPE dispatch migration; `applicable_rules` reaches
the run as advisory context, now via `task.goal`'s own text rather than
`--append-system-prompt`, and stays undecided as a validation boundary until a `WorkResult`
exists to validate. `scope` is amended to undecided as of version 2: the migration removed
`--restricted`/`--add-dir`, the only primitive this record's mechanism translated onto, and
no replacement is elected without the same adversarial verification this record's own
version-1 measurement required.
