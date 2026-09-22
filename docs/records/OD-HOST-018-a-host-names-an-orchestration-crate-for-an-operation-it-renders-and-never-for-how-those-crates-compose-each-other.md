---
id: OD-HOST-018
type: decision
title: A host names an orchestration crate for an operation it renders, and never for how those crates compose each other
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - orchestration
  - architecture
  - layering
relations:
  - target: OD-ROADMAP-005
    type: relates-to
  - target: OD-HOST-001
    type: relates-to
  - target: OD-HOST-008
    type: relates-to
  - target: OD-HOST-011
    type: relates-to
  - target: OD-HOST-012
    type: relates-to
  - target: OD-RULES-020
    type: relates-to
  - target: OD-PACKAGE-015
    type: relates-to
  - target: OD-RULES-028
    type: relates-to
  - target: OD-PROJECT-004
    type: relates-to
---

# A host names an orchestration crate for an operation it renders, and never for how those crates compose each other

## Question

An external architecture review of `dev` at `bc0aaac` asked, in its thirteenth section, that
this workspace establish a rule: an orchestration crate may coordinate domain services, but a
host should not need to know the internal orchestration graph. It observed that the
orchestration crates now compose each other while the hosts also name several of them
directly, and read the result as trending toward a mesh rather than a layer.

`OD-ROADMAP-005` decision item 8 authorizes the rule and bounds it: the one piece of that set
whose output is a record rather than code, superseding nothing.

No record states it. The three nearest each decide a different question on its own
`OD-PACKAGE-015` third-clause trigger -- `OD-HOST-001` that choosing a platform, running a verb
and rendering its outcome are three crates; `OD-HOST-011` that the response twin stays and no
contracts crate is extracted; `OD-HOST-012` that the repo-tooling handlers keep their present
home. Between them they govern what a host *builds*, what it *returns* and what it *holds*.
None of them governs what it may *know*.

## What Was Measured

Measured 2026-09-21 at `53de19a4`, from every workspace member's own production
`[dependencies]` and from `nomos-architecture.json`. The edges are named rather than counted,
because the review's claim is about a shape and a shape is not visible in a total.

**Every host edge into an orchestration crate.** Eight crates in this workspace have a name
ending in `-orchestration` or are `nomos-workspace-discovery`. The six crates under
`crates/host/` reach them twenty-two times:

| Crate under `crates/host/` | Orchestration crates it names |
|---|---|
| `nomos-cli` | `nomos-work-orchestration`, `nomos-check-orchestration`, `nomos-spec-orchestration`, `nomos-gate-orchestration`, `nomos-correction-orchestration`, `nomos-workflow-orchestration`, `nomos-agent-orchestration`, `nomos-workspace-discovery` |
| `nomos-api` | the same eight |
| `nomos-api-transport` | `nomos-check-orchestration`, `nomos-correction-orchestration`, `nomos-gate-orchestration` |
| `nomos-lsp` | `nomos-check-orchestration`, `nomos-correction-orchestration`, `nomos-workspace-discovery` |
| `nomos-mcp` | none -- it reaches everything through `nomos-api-transport` |
| `nomos-surface-provenance` | none |

The population is the directory rather than the zone, so the table is complete in both
directions and the two rows that contribute nothing are named rather than dropped. Five of the
six are `Host` in `nomos-architecture.json`; `nomos-surface-provenance` is `Repo Tooling`
there, and `OD-PROJECT-004` decided where it sits physically. Every one of the twenty-two
edges above is drawn by a `Host`-zone crate.

**Every edge among the orchestration crates. There are six, and all six are real.**

| From | To |
|---|---|
| `nomos-correction-orchestration` | `nomos-check-orchestration` |
| `nomos-gate-orchestration` | `nomos-check-orchestration` |
| `nomos-workflow-orchestration` | `nomos-agent-orchestration` |
| `nomos-workflow-orchestration` | `nomos-check-orchestration` |
| `nomos-workflow-orchestration` | `nomos-correction-orchestration` |
| `nomos-workflow-orchestration` | `nomos-gate-orchestration` |

`nomos-check-orchestration`, `nomos-agent-orchestration`, `nomos-work-orchestration`,
`nomos-spec-orchestration` and `nomos-workspace-discovery` name no orchestration crate at all.
The graph is two levels deep and acyclic, which is what `OD-RULES-020` already measured when
it replaced the number line.

**What the zone model permits, and the asymmetry that falls out of it.** All six edges above
are same-zone: `members` puts both ends of each in `Application Service`, and `permits` does
not grant that zone to itself. Each of the six is therefore listed individually in
`exceptions`, and both directions of that list are mechanically checked -- see
`tests/contract/tests/boundaries/graph.rs`, whose
`Test_Dependencies_Should_Run_Strictly_Downward` refuses an edge that is neither a permitted
crossing nor a named exception, and whose
`Test_Every_Same_Zone_Edge_Should_Be_A_Real_Dependency` refuses a named exception with no
dependency behind it. `Check_Dependency_Direction` makes the same judgment over a real run
from the same declaration.

Every one of the twenty-two host edges is cross-zone. `Host` permits `Application Service`,
`Repo Tooling` and `Specification` outright, so not one host edge is named anywhere, no count
of them is declared, and no check can fail on one however it came to be there. **The
composition graph is fully enumerated and checked in both directions; the reach into it from
above is granted wholesale.** That asymmetry, not the number eight, is what the review saw.

**Whether any host reaches an orchestration crate it renders no outcome for. Exactly one
does.** `nomos-lsp` names `nomos-correction-orchestration` at a single site,
`crates/host/nomos-lsp/src/walk_outward/available_correction.rs`, for `CorrectionFamily::Of`
-- a lookup against that crate's own declaration of which rules it composes a correction for.
The module documents that it never invokes `Run_Correction`, and the LSP surfaces no
correction operation and renders no correction outcome. Every other host edge is answered by
an operation that host offers: `nomos-cli` has seven verb modules against seven orchestration
crates and reaches the eighth, `nomos-workspace-discovery`, for the shared walk `OD-HOST-008`
put beneath every composition root; `nomos-api` declares thirty `Handle_*` entry points across
the same seven families and reaches the walk the same way.

**One near miss, named so the next measurement does not read it as a violation.**
`nomos-api-transport` names its three orchestration crates for their *request* vocabulary only
-- `CheckCommand`, `CorrectionCommand`, `GateCommand`, `FindingQuery`, `RuleSelector`,
`ScopeSelector` -- and names no orchestration outcome type at all, because every answer it
returns is a `nomos_api` response serialized. Each of the three answers a method it serves,
and the twin it renders instead is `OD-HOST-011`'s own decision.

## The Decision

**A host may name an orchestration crate when the host itself offers an operation that crate
answers and renders that operation's outcome. It may not name one for any reason that is a
fact about how the orchestration crates compose each other.**

**The test is removal.** Delete any edge from the composition graph above. If a host's
justification for any of its own edges changes, that host was holding the graph.

Applied at `53de19a4`: deleting `nomos-workflow-orchestration` into
`nomos-gate-orchestration` leaves `nomos-cli` naming `nomos-gate-orchestration` for exactly
the reason it already did, which is that `gate.rs` renders `nomos gate`. All six deletions
leave both large hosts unchanged. **So eight is not a mesh. It is seven operations and one
shared walk**, and the count the review read as coupling is the count of operations the host
offers. The rule is about what a host knows, and a count cannot see that.

**What the rule forbids, stated so it can be checked by hand:**

1. **Naming an orchestration crate the host surfaces no operation for.** The edge has no
   host-side reason, so whatever reason it has is a fact about the other side.
2. **Reaching past a composing crate to one it already composes, to perform by hand what the
   composition performs.** A host calling `nomos-check-orchestration` and then
   `nomos-gate-orchestration` over one tree to assemble one answer has re-derived the
   composition rather than asked for it.
3. **Reading one orchestration crate's declaration in order to describe a different service's
   behaviour.** The host is then answering a question about the graph.
4. **Ordering or sequencing host-side calls because of what one of those crates composes.**
   A host that must call two in an order the orchestration crates already imply holds the
   graph in its control flow rather than in its manifest.

## What This Rule Does Not Forbid

**A host naming a platform composer.** `nomos-composer-std` is how a host chooses an
implementation of the ports without naming one per port; `OD-HOST-001` settled that
composition is the host's job and `OD-RULES-028` settled the zone that keeps a host away from
`nomos-platform-std` directly. Nothing here touches it.

**A host rendering an orchestration outcome type.** That is the third of `OD-HOST-001`'s three
crates and the whole reason the seam exists. A host that names `WorkOutcome` or `CheckOutcome`
and prints it is doing precisely what it is for.

**A host parsing its own arguments.** `OD-HOST-001` kept `argv` parsing and exit codes out of
the orchestration band deliberately, and a second adapter constructs a command from whatever
its own transport carries.

**A host naming many orchestration crates.** The permitted count is the count of operations it
renders, and nothing here caps it. A host offering twenty verbs may name twenty crates.

**An orchestration crate composing another.** The six edges stand. They are declared, checked
in both directions, and `OD-RULES-020` decided that a named same-zone edge is the correct way
to carry exactly this. This record narrows none of them and adds none.

**The response twin.** `OD-HOST-011` stands as written; a twin is not knowledge of the graph.

## What This Rule Does Not Decide

**Whether a host should depend on one operation surface instead of several.** That is piece 3
of `OD-ROADMAP-005`, and this record does not design it, name its crate, or pre-empt its
shape. The two questions are separable on purpose: this one says what a host may *know*, that
one decides what a host *depends on*. A host could satisfy this rule with twenty-two edges and
break it with one.

Nor would that item, if it lands, make this rule vacuous. The same question is then asked of
the surface itself, and the answer is the same sentence with a different subject.

## What This Costs

**Nothing mechanical holds it, and this record says so rather than implying otherwise.** The
three checks named in the measurement judge whether an edge is *permitted*. `Host` permits
`Application Service` unconditionally, so a host edge cannot fail any of them however it was
justified. What this rule turns on -- whether the host renders an outcome for the crate it
names -- is semantic, and no declaration in this workspace carries it. So the rule is held by
review, and it will be broken quietly before it is broken loudly. That is a real cost, and it
is why the trigger below is stated as something observable rather than as a preference.

**It forbids the cheapest answer to a cross-service question and supplies no replacement.**
When a host needs a fact only another service holds, reaching the crate that holds it is one
line. This rule says no, and deliberately does not say what to do instead, because the thing
to do instead belongs to the item that decides what a host depends on.

## Status Of The Tree Against This Rule

**One live violation, and it is not an accident.** `nomos-lsp` naming
`nomos-correction-orchestration` for `CorrectionFamily::Of` fails the first forbidden form
above: the LSP offers no correction operation, so the edge is justified by what a different
service's pipeline contains. It is stated here rather than left for the next reviewer to
rediscover, because a rule with a known population of violations is a decision and a rule that
pretends it is already satisfied is not.

**The repair is not the state that preceded it.** That module used to carry a literal array of
the same rule identifiers, and `P73-LSP-CORRECTION-FAMILY-DUPLICATED-2` replaced the copy with
a lookup against the declaration -- a real improvement, made for a real reason, which is that
a third family composed over there would have left the copy stale with nothing failing.
Restoring the copy would trade a stated edge for an unstated one and satisfy this rule by
making the workspace worse.

**Where it goes instead is not this record's to say.** Which rules a correction family covers
is a claim about what the product can do rather than about how a pipeline is assembled, and
this record neither assigns it a home nor holds it open against anything else --
`OD-ROADMAP-005` is explicit that a piece needing a further change is a new item and, where it
reaches another record, a new question for the owner. Until it moves, the edge is a measured
exception with its reason written down.

## What Would Decide It Differently

- **A second host edge that fails the removal test.** One is a stated exception with a
  history; two is evidence the rule is not being held, and the question then is whether it
  earns a mechanism or should be dropped rather than left as advice nobody follows.
- **A change inside an orchestration crate's composition that forces a host edit.** This is
  the falsifier. It would prove a host was already holding the graph somewhere the edge list
  above could not see, which is the one thing a dependency measurement cannot rule out.
- **A host whose orchestration edge count exceeds the operations it renders.** Both large
  hosts are seven operations plus one walk today. An eighth edge with no eighth operation is
  the first forbidden form arriving again, and it is countable without reading any code.
- **An application operation surface landing while the hosts keep their direct edges.** Then
  either that surface does not carry what a host needs, or the separation is being paid for
  twice, and both are worth reopening for.

## Status

Accepted. The rule the review asked for, stated at the altitude it asked for and with no
mechanism decided. Measured at `53de19a4`: twenty-two host edges into eight crates, six edges
among the orchestration crates, all six named in `exceptions` while every host edge is granted
by zone without being named, and one host edge -- `nomos-lsp` into
`nomos-correction-orchestration` -- that the rule forbids today.
