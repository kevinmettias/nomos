---
id: OD-HOST-014
type: decision
title: The transport admits a product operation that reads or writes the tree it is given, and refuses one that starts a metered external process
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - api
  - mcp
  - agent
relations:
  - target: OD-HOST-007
    type: relates-to
  - target: OD-CONNECTOR-001
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-HOST-006
    type: relates-to
---

# The transport admits a product operation that reads or writes the tree it is given, and refuses one that starts a metered external process

## Question

`nomos-api` exports thirty `Handle_*` functions. `OD-HOST-007` decided that the twenty-one
repo-tooling verbs -- eleven `Work_*` and ten `Spec_*` -- are not projected, structurally
rather than advisorily. Nine are product operations an end-user repository would ask.

Five of those nine are served: the four Gate verbs and `Correction_Run`. The other four --
`Check_Run`, `Workflow_Run`, `Agent_Execute`, `Agent_Judge_Role` -- are real, tested,
canonical application seams that no wire caller and no MCP client can reach.

The criterion in `OD-HOST-007` cannot decide them. It separates a repository-development verb
from an end-user verb, and by it all nine are end-user verbs. So either something else
separates the four from the five, or nothing does and they are simply unadmitted. This record
says which.

## What Was Measured

Taken at `0f793cc0`.

`ServedMethod::REGISTRY` names five methods and the `ServedTool::REGISTRY` in `nomos-mcp`
projects exactly that array. `tests/contract/tests/boundaries/transport_registry.rs` holds an
`ADMITTED` list of five handler names and quantifies it over the real exported surface of
`nomos-api`, so a handler not named there is excluded by default rather than by a prefix
guess. A widening is therefore a visible edit in three places, which is what
`P62-TRANSPORT-MCP-CORRECTION-SURFACE-2` measured when it admitted `Correction_Run`.

What each of the four unadmitted operations causes on the host:

| Operation | What it does |
|---|---|
| `Check_Run` | Walks and judges a tree. Reads only. |
| `Workflow_Run` | Executes a step sequence. A step body may be `Check`, `Correction`, `Gate` or `Agent`. |
| `Agent_Execute` | Starts a subprocess through a `ProgramLauncher` and spends against a ceiling. |
| `Agent_Judge_Role` | Reads a `README.md` row and a committed surface snapshot, then dispatches an agent as above. |

And what the five already served cause: the four Gate verbs read and judge, and
`Correction_Run` stages and, if asked, commits a change inside the tree it was given. None
starts a process that costs money outside it.

The agent dispatch is not unbounded. The clauses named by `OD-EXECUTOR-001` moved down into
`xvpe-agent-backend-claude-code`: an isolated working directory, an allow-list granting
nothing real, one `--print` turn, a spend ceiling, a wall bound that kills, and an answer read
from schema-validated `structured_output`. Those bound one call. They do not bound how many
calls a wire client makes, and the ceiling is per dispatch rather than per caller.

## Decision

### 1. The criterion is what the operation causes on the host, not who asks it

The question in `OD-HOST-007` was whose verb it is. That question is answered and exhausted:
the twenty-one are excluded and the nine are product operations. The line inside the nine is a
different one, and it is the blast radius of a single call.

An operation that reads the tree it is given, or writes inside it, is admitted. An operation
that starts an external process which costs money is refused, because a transport admitting it
hands an unauthenticated caller a spend decision the host never made. This is stated as a
criterion rather than as four separate judgements, so that the tenth product operation is
decided the day it is written rather than the day somebody notices it is missing.

### 2. Check_Run is admitted

It walks and judges a tree exactly as `Gate_Run` does, and reads nothing else. No property
distinguishes it from the four Gate verbs already served, and refusing it would be a registry
that is merely short -- the absence-as-boundary that `OD-CONNECTOR-001` refuses and
`OD-HOST-007` named.

### 3. Agent_Execute and Agent_Judge_Role are refused

They start a subprocess and spend against a ceiling. Every operation served today is bounded
by the tree it was handed; these are not, and the difference is real rather than a matter of
degree. The clauses named by `OD-EXECUTOR-001` make one dispatch safe, which is not the same
as making an open number of them safe from a caller the transport does not authenticate.

`Agent_Judge_Role` carries a second reason, weaker but worth recording: it reads a `README.md`
row and a committed surface snapshot, and it is a convention of this repository that those
exist and mean what this workspace means by them. In a repository that has neither, the
operation answers `NoDeclaredRoleOrSurface`, which is honest and is also the whole of what it
can say. That does not make it a repo-tooling verb under `OD-HOST-007` -- it carries no
repo-tooling mark and lives beside the product handlers -- but it does mean admitting it would
project an operation shaped around a convention the caller may not share.

The refusal is not permanent, and it is not a judgement that the operation is unsuitable. It
is a statement that a transport has no way to bound aggregate spend today, and that admitting
an operation which can incur it would be deciding that question by not asking it.

### 4. Workflow_Run is refused while a step may carry an agent body

Admitting it would admit `Agent_Execute` transitively. `Body::Agent` is one of the four bodies
a step may carry, so a caller who can run a workflow can run an agent by writing a step that
does. That is the widening-by-the-back-door `OD-HOST-007` refuses when it says a later
transport must make its case explicitly rather than by widening a registry nobody is watching.

This refuses the operation as it stands rather than the operation. A transport that admitted
`Workflow_Run` over a step vocabulary it could assert carried no agent body would be a
different proposal, and the criterion in decision 1 would admit it.

### 5. What an admitting increment must edit

Three places, and all three are the mechanism that keeps the exclusion structural:
`ServedMethod::REGISTRY` in `nomos-api-transport`, `ServedTool::REGISTRY` in `nomos-mcp`, and
the `ADMITTED` array in `tests/contract/tests/boundaries/transport_registry.rs`. A widening
that edited fewer than three would be caught by the contract suite, which is the point of
there being three.

### 6. What would reopen decisions 3 and 4

Four events, named so the refusal is falsifiable rather than a standing preference:

- a transport that authenticates callers, so a spend decision has somebody to attribute to;
- an aggregate spend bound a transport can enforce across calls rather than within one;
- a workflow step vocabulary a transport can assert excludes agent bodies, which reopens
  decision 4 on its own;
- a measured request from a real external client, which is the same kind of concrete event
  `OD-HOST-007` used to fire its own trigger rather than waiting for a judgement that the
  surface had become popular enough.

## What This Record Does Not Do

It does not admit `Check_Run`. Admitting an operation is its own item with its own territory,
judged against this record, the way the transport and the correction surface each were.

It does not revisit the exclusion of the twenty-one repo-tooling verbs in `OD-HOST-007`, or
decide whether one could ever be projected under a later framing. That record left the
question open and this one does not answer it.

It does not decide whether the response twins `OD-HOST-011` reasons about should become one
serializable contract layer. A wider registry makes that question larger; it does not answer
it.

## Alternatives Considered

**Admit all four, on the ground that all nine are product operations.** Rejected: it reads the
criterion in `OD-HOST-007` as the only criterion there could be, and the measurement shows a
second line inside the nine that the first never had to consider.

**Refuse all four, keeping the registry at the Gate verbs plus `Correction_Run`.** Rejected
for `Check_Run` specifically: nothing distinguishes it from `Gate_Run`, so refusing it would
be a boundary drawn by inertia, which is the shape `OD-CONNECTOR-001` refuses.

**Admit `Agent_Execute` and rely on the ceiling the executor already carries.** Rejected: that
ceiling is per dispatch. It makes one call safe and says nothing about a caller making many,
and a per-call bound presented as an aggregate one is the kind of guarantee `OD-GATE-001` is
about in its own domain -- the check ran, said something narrower than the reader assumed, and
nothing was listening for the difference.

**Admit `Workflow_Run` and refuse `Agent_Execute`.** Rejected as incoherent: a step may carry
an agent body, so the refusal would be reachable through the admission.
