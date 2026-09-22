---
id: OD-EXECUTOR-009
type: decision
title: Knowledge context cannot bridge to a peer that has not exposed an interface yet
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - agent
  - executor
  - ecosystem
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: D-137
    type: relates-to
  - target: OD-EXECUTOR-007
    type: relates-to
  - target: OD-PACKAGE-003
    type: relates-to
  - target: OD-CONTRACTS-005
    type: relates-to
---

# Knowledge context cannot bridge to a peer that has not exposed an interface yet

## Question

`TaskEnvelope.knowledge_context: Vec<KnowledgeReferenceId>` is a declared field the executor
ignores. A person required a real bridge: acquisition from a real source, provenance per
item, task-scoped selection decided by the system rather than guessed by a caller, and
promotion back across the boundary either implemented or recorded as refused, all without
normative authority leaving Nomos. `OD-EXECUTOR-007` named this same field as one of
`TaskEnvelope`'s unenforced four and declined to build it speculatively, alongside `scope`,
`prohibited_changes` and `available_tools`, which that record did decide. Whether
`knowledge_context` is now buildable needed checking directly against KWB itself, not
assumed from the field's own shape.

## What Was Measured

**`KnowledgeReferenceId` was admitted on exactly this understanding, and nothing has changed
it.** `D-137`'s own title says so outright: "`KnowledgeReferenceId` is admitted to
nomos-contracts as the one shape a KWB citation needs, and nothing yet produces or consumes
one." Two committed requirement assessments cite it as their gap for the identical reason:
`AGT-007.assessment` — "the KWB half... is unreachable: `KnowledgeReferenceId` exists (D-137)
but, by that record's own text, 'nothing yet produces or consumes one'" — and
`AGT-017.assessment`, the same citation for a handoff's cross-system links.

**`ARC-ECOSYSTEM-001` draws the boundary precisely and deliberately builds nothing across
it.** KWB owns knowledge — rationale, semantic intent, decision context, long-term epistemic
memory; Nomos owns software-specific reality. The record diagrams the crossing (KWB semantic
intent through a governed projection into a Nomos executable contract) and states the rule a
bridge must honour — a rationale is not enforceable, only a derived contract is, and the
derivation must be a recorded step — but names no concrete interface, protocol, or wire
format, and says outright: "No KWB or XVPE integration is implemented, and none is scheduled
here." The boundary is a governance decision, not yet a crossing anything could build
against.

**KWB itself was read directly, not assumed from its own architecture documents.** It is
real and structured — ten crates, band-organized, densely commented with the same
contract-first discipline this workspace uses — and every one of them is an unimplemented
scaffold today: `kwb-cli/src/main.rs` prints "no verbs are wired yet"; `kwb-mcp/src/main.rs`
prints "no tools are wired yet" and names nine planned read-only tools none of which exist;
`kwb-platform-std/src/lib.rs` says plainly "Nothing is implemented yet"; `kwb-contracts`'s own
crate stays deliberately empty because, in its own words, the shape a citation identity
string takes "is a decision rather than a guess" until `kwb-model` exists to derive it from.
There is no CLI, no HTTP endpoint, no file export, and no library surface on the KWB side
today that anything could call or read.

**This is not a design gap Nomos can close by itself.** `OD-RULES-024` and `OD-RULES-025`
both found a missing fact or convention *this workspace* could eventually decide and
materialize. Here the blocker is external: KWB has not yet decided what a
`KnowledgeReferenceId`'s own string content is, has not built retrieval or query, and has
wired no host at all. A Nomos-side `nomos-knowledge-context` crate built now would have no
real peer to acquire from, select against, or promote to — it would mean inventing both ends
of a protocol against an interface that does not exist, the identical speculative-building
`D-137` already declined when it admitted the identifier alone and stopped there.

## The Decision

**Acquisition and selection are not built here.** Both presuppose a KWB-side interface —
something that answers a query, something that names a citation's shape — that this
workspace does not control and cannot honestly stand in for. Building either now would be
guessing at a peer's contract before the peer has one, which `ARC-ECOSYSTEM-001`'s own
ownership split exists to prevent as much as to enable.

**Promotion back across the boundary is recorded as refused, and this much is genuinely
free.** `ARC-ECOSYSTEM-001` already forbids normative authority leaving Nomos, and since
nothing yet produces a `knowledge_context` entry, there is nothing a promotion path would
carry. Refusing it costs nothing and states a true fact rather than an aspiration: there is
no promotion mechanism because there is no acquisition mechanism for it to promote from.

## What This Record Does Not Do

**No code changes here.** It does not build `crates/orchestration/nomos-knowledge-context`,
which is not authored — that territory presumed a real KWB peer that does not exist yet, the
same way `OD-RULES-024`'s architecture-drift territory presumed a capability that was never
built.

It does not amend `ARC-ECOSYSTEM-001` or `D-137`. Both already say precisely what this
record measured; this record confirms their claims are still current rather than
superseding them.

It does not withdraw the requirement. `AGT-007` and `AGT-017` stay `Partial` for the
identical reason they already are: the gap is real, and closing it is gated on KWB's own
progress, not on a decision Nomos has been withholding.

It does not say KWB will never be ready. Revisiting this is conditioned on an observable
KWB-side fact — a real host binary that answers a query, or a decided citation-identity
shape in `kwb-contracts` — not on a timer or another Nomos-side design pass.

## Amendment: A KWB Host Answers A Query Now, And One Trigger Is Not Enough For Every Half

Added at version 2, 2026-09-21. Version 1 named two observable KWB-side facts as its revisit
condition, and the first of them has happened. This amendment re-measures KWB read-only at
one named commit, says which of version 1's sentences that commit has made false, and
decides acquisition, selection and promotion one at a time — because a host that answers a
query turns out to be enough for none of the three on its own, and the reason differs for
each.

Measured against `F:/repos/kwb` at commit `4eeb04ad4ea42b1709b8d4c37d374f0e739b3832`
(`KWB-107`, 2026-09-21), working tree clean. Nothing under that repository was created,
modified or deleted, and no code here changes.

### What version 1 said that is no longer true

Quoted so the reader can see what expired, not to say it was wrong when written; on
2026-09-05 every one of these was a direct reading of the KWB tree.

> It is real and structured — ten crates, band-organized, densely commented with the same
> contract-first discipline this workspace uses — and every one of them is an unimplemented
> scaffold today: `kwb-cli/src/main.rs` prints "no verbs are wired yet";
> `kwb-mcp/src/main.rs` prints "no tools are wired yet" and names nine planned read-only tools
> none of which exist; `kwb-platform-std/src/lib.rs` says plainly "Nothing is implemented
> yet"

Twelve crates now, and none of those three strings is in any of those three files at
`4eeb04a`. The two hosts dispatch; `kwb-platform-std` carries a directory content store and
a file record log.

> There is no CLI, no HTTP endpoint, no file export, and no library surface on the KWB side
> today that anything could call or read.

Two of the four are false: a CLI exists (`kwb`, four verbs), and a library surface exists
(`kwb_mcp`, the `kwb-mcp` crate's `[lib]`, exported so a test can call a tool). Two still
hold: no HTTP endpoint, and no MCP transport — `Print_Absences` in
`crates/host/kwb-mcp/src/lib.rs` prints "No transport is wired." The publication log a store
directory holds is KWB's own persistence, replayed by both hosts, not an export offered to a
peer.

> Here the blocker is external: KWB has not yet decided what a `KnowledgeReferenceId`'s own
> string content is, has not built retrieval or query, and has wired no host at all.

All three halves have moved. KWB's `D-002` decides the string; `kwb-retrieval` exists with
one query type per world; two hosts are wired.

> Knowledge-context acquisition and selection are blocked on KWB exposing a real interface,
> which it has not done

Superseded by the decisions below: the interface exists and is not the one acquisition
needs.

> Revisiting this is conditioned on an observable KWB-side fact — a real host binary that
> answers a query, or a decided citation-identity shape in `kwb-contracts` — not on a timer or
> another Nomos-side design pass.

The first fired. The second could not fire as worded, and the reason is measured under *The
identity* below.

### What exists at `4eeb04a`, measured and not restated

Only what the three questions this amendment was asked need. What KWB is for, how it is
banded and why it decided any of this is KWB's own `README.md` and `docs/records/`, and none
of it is repeated here.

**`crates/host/kwb-cli/src/main.rs` dispatches four verbs**, from one `VERBS: [Verb; 4]`
table:

- `admit <file> [--store <dir>] [--scope <name>] [--says <concept> <claim>]...` — writes the
  file's bytes through the store's one write door and publishes whatever `--says` supplied.
  Its help (`usage.rs`) says "This command does not read the document. An extractor exists --
  kwb-extract" and "--says is how a passage's claims get in"; the crate's manifest names
  `kwb-ingest` and not `kwb-extract`, so no path through this binary invokes a reader other
  than the person typing. The lineage `admission.rs` records for a `--says` reading is fixed:
  `ReadingProtocol::Named("stated-by-a-person")`, `ReaderName::Named("the operator of kwb
  admit")`. The report prints the document's content address as `source` and four counts;
  it prints no concept or claim identity.
- `retire <concept> --store <dir> --because <reason>` and
  `supersede <concept> --into <concept> --store <dir> --because <reason>` — close a concept,
  refusing a closure with no recorded reason, and print counts.
- `history --store <dir> [--through <count> | --as-of <unix seconds>]` — replays a prefix of
  the publication log and prints counts; refuses a count past the log's end.

**`crates/host/kwb-mcp/src/lib.rs` answers five read-only tools**, from one
`TOOLS: [Tool; 5]` table, each fixed to the world it reads, through
`Answer_Tool_Call(graph, ToolName, argument) -> Option<Vec<String>>`; the binary is
`kwb-mcp <store-dir> [<tool> [<argument>]]`, a command line over that function, and with no
tool it prints the table. What each returns, read from the arms of `Answer_Tool_Call`:

| tool | world | argument | each answer line |
|---|---|---|---|
| `search` | current | words | a claim's text — "claims whose text contains every word of a query" |
| `get_concept` | current | words | a concept's canonical name |
| `neighbours` | current | an exact canonical name | `concept  <name>`, then `claim    <text>` per claim, then `cited    <source> [<scope>]` per assertion |
| `merge_losers` | historical | none | `<name> -> <successor identity> (<reason>)` |
| `held_neighbours` | historical | an exact canonical name | the `neighbours` lines, each suffixed with what became of it |

The `<source>` on a `cited` line is the content address of the document the claim was read
out of — `kwb-ingest`'s `Assertions_Citing` sets it from the written document's
`Identity().Render()` — never the claim's own identity.

**`crates/contracts/kwb-contracts/src/lib.rs` still admits nothing.** The file is a module
comment and `#![forbid(unsafe_code)]`. It still says that "no HTTP route, MCP tool name, CLI
spelling or client SDK method is authoritative for these meanings; they are all projections
of what this crate says", and still ends "This crate stays empty until that decision has
something concrete to name." With nothing in it, no vocabulary a peer reads from either host
above is promised to that peer by KWB's own band-0 rule.

**The identity.** `crates/kernel/kwb-model/src/content_identity.rs` declares
`ContentIdentity([u8; 32])` — the untruncated SHA-256 of a self-describing, delimited layout
— and `Render()` is its one string form: `IDENTITY_CHARACTERS = 64` lowercase hexadecimal
digits, with `Parse` refusing uppercase "so that one identity has exactly one rendering".
`Render`'s own doc says: "This is the string form `D-002` promises Nomos: stable, opaque, and
the same for the same content forever." `Claim::Identity()`, `Concept::Identity()` and
`Assertion::Identity()` in `crates/domain/kwb-domain/src/epistemic/` each return one. So the
answer to the third question is yes: a claim's or a concept's identity renders as a stable
string that `KnowledgeReferenceId` — a `Named_Identity` over a string in
`nomos-contracts/src/digest128.rs` — can carry without Nomos computing anything, which is
exactly `D-137`'s condition for it.

KWB's `D-002` says the same from its side and decides where version 1's second trigger went:
"`kwb-contracts` does not define a matching wrapper type: the agreement between the two
repositories is about the string's meaning and stability, not about either side compiling
the other's Rust." The shape is decided; it is decided in a record and in `Render`, and
`kwb-contracts` will not carry it by KWB's own decision. Version 1 was right that the shape
was undecided and right to name `kwb-contracts` as where KWB's own charter said it would
land; KWB then decided the shape without minting a type. The trigger's substance has fired
and its wording cannot.

**The fact that decides everything below: no host renders a claim's identity, and no tool
accepts one.** Every `ContentIdentity` either host prints was found: the document's, on
`kwb admit`'s `source` line and on every `cited` line; and a successor concept's, in
`Merge_Loser_Line` and `Standing_Of` (`by.Render()`), which is the only concept identity
either host emits and only for a concept that was merged away. `search` answers text,
`get_concept` answers names, `neighbours` answers text and names, and every tool's argument
is words or an exact name — `Neighbourhood_Lines` finds the concept by `Canonical_Name()`,
`Held_Neighbourhood_Lines` derives the identity from the name it was given. The string
`D-002` promises exists and stays inside the process that computed it.

**What did not change on either side.** `TaskEnvelope.knowledge_context`
(`crates/agent/nomos-agent-contracts/src/task_envelope.rs`) is constructed with
`Vec::new()` by every real caller and with a `kwb:decision:…` placeholder by every seam
test; both executors' module docs still say it was never given a mechanism. `D-137`'s title
is still true word for word: nothing here produces or consumes one, and KWB mints a
`ContentIdentity` for itself, not a citation for a peer. `AGT-007.assessment` and
`AGT-017.assessment` still cite `KnowledgeReferenceId` as their gap and are still right to.
KWB's `D-005` and `D-013` are about the work ledger, its `D-007` and `D-012` about XVPE;
each names Nomos, none touches the knowledge crossing, and `D-002` is the only KWB record
that does.

### The decision, per half

**Acquisition is not built, and the blocker is now one specific fact rather than a general
absence.** What this side has is enough to call a binary and read its lines: the process
launching `nomos-platform-xvpe`'s `xvpe_launcher.rs` adapts, the pattern `OD-RULES-010`
decided for reading an external tool's output as a fact, and a landing shape —
`KnowledgeContextItem` in `nomos-contracts`, which `OD-CONTRACTS-005` passed for exactly
"an external knowledge system across the process boundary", with `source:
Option<KnowledgeReferenceId>` and a `provenance` string. What it cannot do is the one thing
the field is for: fill a `Vec<KnowledgeReferenceId>`. No answer carries a claim's identity,
and Nomos may not compute one — `D-137` made the identifier a `Named_Identity` and not a
`Digest_Identity` precisely because "Nomos does not compute this value from bytes it
holds", and re-deriving `kwb-model`'s layout here or naming a `kwb-` crate to do it is the
crossing `ARC-ECOSYSTEM-001`'s amendment says nothing permits.

What could be built and is refused: an adapter over `kwb-mcp <store> search <words>` that
lands each line as a `KnowledgeContextItem { source: None, … }`. Every such item is
`Is_Unresolved()` by construction — `AGT-010`'s own "shall not be treated as normative" —
it is parsed off a line format KWB's band-0 charter says is not authoritative, and it
carries a claim's text across the boundary without the identity that would let anything
re-resolve it: a copy, not a citation, which is the bare assertion `ARC-ECOSYSTEM-001` keeps
off every crossing. Refused, not deferred.

So for acquisition one trigger is not enough. A host that answers a query is necessary and
was not the whole condition; the identity has to leave the process in an answer. **The
KWB-side fact that fires the next revisit for acquisition: a `kwb-mcp` answer that renders a
claim's `ContentIdentity` beside its text, or a tool whose argument is one.** Whether the
shape of that answer is then promised to a peer — `kwb-contracts` admitting something, or a
KWB record saying a line format is the projection of a decided vocabulary — is the second
half of the same fact and the amendment that lands the first will have to measure it.

**Selection is not built, and neither trigger reaches it.** It presupposes acquired items,
so acquisition's blocker is its first. It has one of its own: what a task would select
*with*. KWB's one current-world query over claims is `Claims_Matching` — every word of the
query must appear, unranked, and `kwb-retrieval` says "Deliberately not ranked" for its own
recorded reason. The ranking vocabulary this side has, `KnowledgeSourceRole`'s declaration
order for `AGT-012`, is a role KWB does not emit; a claim there has no standing of its own,
and the one attribute an answer does carry, an assertion's scope, is a name on a `cited`
line. A selector built now would choose the words from `TaskEnvelope.goal` — a caller's
guess wearing the system's name, the thing the requirement rules out — and rank the result
by nothing. **The KWB-side fact that fires a revisit for selection: acquisition's fact,
plus a query that is not all-words text match** — ranked retrieval, which KWB's `D-008` and
`D-012` defer until a stored vector exists, or a structured question keyed by a concept
identity.

**Promotion stays refused, on a stronger ground than version 1 had.** Version 1's reason
stands: nothing produces a `knowledge_context` entry, so nothing exists to promote, and
`ARC-ECOSYSTEM-001` forbids normative authority leaving Nomos. The measurement adds a second.
KWB's only write door reachable from outside is `kwb admit --says`, and `admission.rs` fixes
what it records about the reader: a person, by protocol and by name. A Nomos process driving
that verb would publish a machine's generalization under a person's provenance, which is a
false lineage — and `ARC-ECOSYSTEM-001`'s third crossing admits a generalization only
carrying the runs it came from as provenance, with a second source before it may become a
rule. No host fact fires a revisit here; a query-answering binary is irrelevant to a write.
**What would: the governed crossing itself existing** — `P11-ECOSYSTEM-UPWARD` drew it and
`ARC-HARNESS-001`'s harness, which would produce the run observations it carries, is not
built — **and a KWB reading protocol for a machine reader that carries run provenance**,
which `kwb-extract`'s extraction lineage seam is shaped for and which KWB has deliberately
left without a provider.

### What this amendment does not do

No code changes here, for the same reason as version 1 and with the reason now narrower: the
crate version 1 declined to author would today be able to call a real binary and unable to
produce the one value its field holds.

It does not amend `D-137`. Its title is still true of this workspace, and its condition —
carry, never compute — is what the acquisition decision above turns on.

It does not amend `AGT-007.assessment` or `AGT-017.assessment`; both stay `Partial` and
their gap citation is still the accurate one.

It does not restate KWB's design, and it does not ask KWB for anything. It names, for each
half, the KWB-side fact that would be observable from here, so that the next revisit is a
measurement and not a conversation.

## Status

Accepted, version 2. Knowledge-context acquisition is blocked on one KWB-side fact rather
than on a general absence: at `4eeb04a` a KWB host answers a query, `kwb-model`'s identity
renders as the stable 64-character string `KnowledgeReferenceId` can carry, and no answer of
either host renders a claim's identity or accepts one — so nothing can be acquired that is a
citation rather than a copy. Selection is blocked behind acquisition and, independently, on
KWB offering a query that is not unranked all-words text match. Promotion stays refused:
nothing exists to promote, and KWB's only write door records a person as the reader.
Version 1's second trigger fired in substance and not in wording — the citation-identity
shape is decided in KWB's `D-002` and `ContentIdentity::Render`, and by that same decision
will not appear in `kwb-contracts`. Revisiting acquisition is conditioned on a `kwb-mcp`
answer that carries a claim's `ContentIdentity`; selection on that and a ranked or
structured query; promotion on the governed upward crossing existing and a machine reading
protocol on the KWB side — each an observable fact, not a timer.
