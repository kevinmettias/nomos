---
id: OD-EXECUTOR-007
type: decision
title: A TaskEnvelope's four unenforced fields each get a decided mechanism, verified against the real CLI
status: accepted
version: 1
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

## Status

Accepted. `scope` and `prohibited_changes` each have a real, evidence-backed mechanism;
`available_tools` is refused rather than silently dropped until a capability-serving bridge
exists; `applicable_rules` reaches the run as context now and stays undecided as a
validation boundary until a `WorkResult` exists to validate.
