---
id: ARC-CONFORMANCE-003
type: decision
title: Nomos is the ecosystem's shared rule layer, and a corpus is absorbed by reading its declarations rather than by copying its rules
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - conformance
  - ecosystem
  - standards
  - rules
  - roadmap
relations:
  - target: ARC-CONFORMANCE-001
    type: relates-to
  - target: ARC-CONFORMANCE-002
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-034
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-CAPABILITY-012
    type: relates-to
  - target: OD-PLATFORM-003
    type: relates-to
  - target: OD-HOST-013
    type: relates-to
---

# Nomos is the ecosystem's shared rule layer, and a corpus is absorbed by reading its declarations rather than by copying its rules

## Question

Three rule corpora exist to be absorbed: `code-standards/docs/standards` (1,133 rule
documents), `code-standards/rules` (320 Go check binaries over 108 shared engines), and
`xvpe/docs/arch/standards` (261 rule documents). The owner's constraint is that this is
**not a direct migration** — the corpora are one person's preferences, and Nomos must stay
customizable enough that another repository declares its own style guideline and is judged
against that.

`ARC-CONFORMANCE-002` already decided the relationship for one of the three: a sibling's
declared corpus is a fact Nomos *reads*, through a declared per-repository policy, and Nomos
does not write itself into what enforces it. That record wrote no code. `OD-RULES-011`
decided the mechanism for a rule's *parameters*: a fact a repository declares, not a constant
the rule compiles with. Six policy families were built on it.

What is undecided is the shape of the work: what actually gets ported, in what order, and —
since `P26-RULES-MORE-CODE-STANDARDS-3` ("Exhaust current-input code-standards imports") is
**done** — what the next increment is now that the well of rules Nomos can judge *with its
current inputs* is drawn dry.

## What Was Measured

**The ecosystem already has the three layers, and one of them has an incumbent.** Declarations
live in each repository (`code-standards/docs/standards`, `xvpe/docs/arch/standards`, each with
its own front-matter schema). Engines are shared: `xvpe/tools/toolkit/shared_checks.go`
delegates **thirty-six** checks — `TOOL_NAMING_CLARITY`, `TOOL_FUNCTION_SIZE`,
`TOOL_SUPPRESSION_JUSTIFIED`, `TOOL_PATH_ATTRIBUTE` and the rest — to code-standards' shared
Go binary, and those are the same check names `nomos-rules` has been porting under
`P26-RULES-*`. Enforcement is per-repository policy and waivers: `xvpe/suppressions.json` holds
1,045 waivers keyed on `(check, path, message)`; `kwb/suppressions.json` holds 30, naming
checks **Nomos defines**, and `kwb/standards.json` declares its own `naming`, `words` and
`telemetry` blocks. So Nomos is already the engine layer for a second product while
code-standards is the engine layer for xvpe. **Nomos is the Rust-native successor to that
role, and the migration is a succession rather than an import.**

**The three systems independently arrived at the same design.** `xvpe/tools/toolkit/
standards_frontmatter.go:1-8` states it in its own words: *"A document is a rule because it
SAYS `kind: rule`, not because a regex found bold text in it; it is enforced by the tools it
NAMES, not by whatever a blockquote happened to backtick… Every fact a tool needs is declared.
Nothing is inferred, so nothing can be inferred wrong."* That is `OD-RULES-011`, written by the
corpus author in another repository about another mechanism. And code-standards separates 320
thin `main.go` declarations from 108 shared engines, joined by an `// Enforces: <rule-id>.md`
header — the same declaration/engine split `DeclaredTextRule` describes.

**The Go corpus is 88% text, and the 12% that is not already reaches structural facts through a
language kernel.** Measured across all 320 `main.go` files and the 108 engine directories
beneath `*/shared/*`: **zero** of the 108 engines import `go/ast`, `go/parser`, `go/token` or
`go/types` directly. 95 of them use `strings.` and 11 import `regexp`; 13 reach a structural
fact, and every one of those does it by importing a *language kernel* —
`nomos.dev/language-kernels/programming/golang`, `markup/razorlang`, `data/yamllang` and the
rest, **23 kernels across 7 language families** (`programming`, `scripting`, `markup`, `data`,
`schema`, `shell`). Eleven engines import no kernel and use no `strings.` either. 103 engines
sit under `general/` and 5 under `meta/shared/`; **none** sits under `language-specific/`, which
exists and holds thin declarations rather than engines. No engine basename repeats, and 35 of
the 320 checks declare a repair. The consequence is twofold: the dominant port outcome is *a
declared text rule and a serializer* rather than a hand-written Rust function per check, and the
minority that needs structure needs it from a language provider — which is the seam
`nomos.cap.syntax.items` and `crates/languages/` already are here, so that minority is a relay
rather than a reimplementation. The critical-path gap is the narrowness of `DeclaredTextRule`,
not the size of the corpus.

**The xvpe corpus carries the field vocabulary the reader needs, and a working oracle already
exists.** `standards_frontmatter.go:13-41` declares `KIND_{RULE,INDEX,REFERENCE}`,
`SEVERITY_{MUST,MUST_NOT,SHOULD,SHOULD_NOT,MAY}`, `GATE_{BLOCKING,ADVISORY,UNREACHABLE,REVIEW}`,
`REVIEW_ENFORCER` and the `code-standards:` pointer scheme; `frontmatter_yaml.go` is a
deliberately tiny, strict YAML subset that turns an unparseable line into an issue rather than a
silent misparse. Separately, `tools/checks/docs/check-gate-truthful` already implements
`ARC-CONFORMANCE-002`'s verdict machinery: for every rule document it derives a `WiringIndex`
(BLOCKING / ADVISORY / UNREACHABLE) **from the real gate roots** — `.github/workflows/*.yml`
and `tools/check.sh` — by recursively following every tool those invoke through every
orchestrator, and reports three sync failures: an empty `enforced_by`, a `review`-enforced rule
whose `gate` is not `review`, and a declared `gate` disagreeing with the computed category. The
reader and its four verdict routes therefore have a working reference implementation to port
against rather than a design to invent.

**The recorded census reproduces exactly, and what was believed about the corpora's join was
wrong.** Re-parsed live from all 375 documents rather than taken from `ARC-CONFORMANCE-002`'s
text: 375 documents, 261 rules, severity `{MUST 161, MUST NOT 5, SHOULD 83, MAY 11, SHOULD
NOT 1}`, gate `{review 224, blocking 31, unreachable 6}`, and `enforced_by` naming Nomos **zero
times**. Two corrections. First, `canonical:` is declared in `rule.schema.json:47` and read by
`StandardsFrontmatter.Canonical_Relative_Path`, and its own description records that *"four of
these once pointed at files that did not exist"* — but **zero of the 375 documents carry one
today**, so the field that would join the two corpora exists in the schema and is empty in the
data. Second, `xvpe/tools/archscan` is a scan-harness library and not an architecture-model
builder; the model-from-source tools are `tools/source/{module-map, impact-of,
tool-architecture-map}`.

**Nomos's own architecture substrate is the *declared* one, and the observed one is deferred by
decision.** `nomos-cap-architecture` carries `ArchitecturePayload` with the general queries
`Is_Permitted(Depending, Depended)`, `Is_Excepted`, `Component_Of` and `Doors_Into`, read from
`nomos-architecture.json`, and it is populated in production composition and judged by three
rules. There is no observed architecture fact (`OD-RULES-024` says it does not exist), no
feature topology, and no code-level cohesion or ownership fact: `ARC-ROADMAP-001` sets Nomos
Core's near-term boundary as the headless enforcement loop and **not** architecture and feature
intelligence, and `OD-CAPABILITY-018` states that feature topology cannot be carried until
something declares what a feature is. `nomos-cap-goals-policy` is the closest thing, and it is
declared goals-to-subsystems, not observed features. So the corpus's 63-file
`architecture/shared/dependencies` engine — which takes its import graph **injected**
(`consumption.go:45`: `imports func(kernel.SourceFile) ([]wrapping.Import, …)`) and compares it
against declared tiers, permitted edges and seams — has a substrate here, but that substrate is
**declaration**, and saying otherwise would overstate what exists.

**A preference axis costs ten paths across six areas, measured on a live item.** The ready
ledger item `P125-A-RULE-THAT-NEEDS-A-METRIC-FACT` reserves `crates/capabilities/
nomos-cap-complexity`, `crates/languages/nomos-lang-rust-complexity`, `crates/rules/
nomos-rules`, `crates/orchestration/nomos-check-orchestration`, `crates/orchestration/
nomos-gate-orchestration/src/composition.rs`, `Cargo.toml`, `Cargo.lock`, `README.md`,
`nomos-architecture.json` and `tests/contract/surface/nomos-rules.txt` to add **one**. Six of the
seven capability families are wired from declarations this way today (`nomos-repo-policy`:
naming, limits, scripting, words, goals, test-material), with the architecture declaration
beside them and `nomos.cap.dependency.policy` arriving instead through a tool relay
(`nomos-lang-rust-deny`, per `OD-RULES-010`). `suppressions.json` —
Nomos's own waiver file — has **no Rust reader in this workspace at all**; it appears only in
comments and is consumed by an external tool.

**A waiver is already a declaration in this workspace; what is missing is the file the
ecosystem actually wrote.** `nomos-gate-orchestration`'s `policy/suppression_disposition/` holds
a complete disposition mechanism — `SuppressionPolicy`, `Suppression`, `Status_At(now)` and a
`Lapsed` set that separates a tolerance that ran out from a suppression that was never written,
*"the difference between a gate reporting a regression and a gate reporting that a decision
somebody made has come due."* Its declared source is **`nomos-gate.json`** at the run's root
(`policy/gate_policy_file.rs`), and **this repository has no such file**, so every real run here
resolves the empty default and the mechanism is reachable only from unit tests. Meanwhile the
waivers the ecosystem already wrote are 1,045 in `xvpe/suppressions.json` and 30 in `kwb/`'s, in
a shape `standards.json` itself declares (`data_format_contracts` requires `schema: number` and
`waivers: array`). So M5 is not "build waivers" — it is **joining a built mechanism to a
declared corpus that already exists**, which is M1's shape one layer up.

And `gate_policy_file.rs` records, in its own words, the exact limit M5 must not inherit: a
waiver there *"suppresses and baselines findings a rule addresses by file, and silently matches
nothing for the rest"* — an author who writes an entry for `completeness-mirror` **"gets no error
and no effect."** A waiver that silently matches nothing is the same defect as a rule that
silently judges nothing, and it is the reason G3's sub-goal is stated as an *equality* against
the gate's real wiring rather than as a count of declared rules.

**A new declaration does not go in `standards.json`, and that is already settled convention.**
`nomos-cap-test-material-policy` reads `nomos-test-material.json` and not `standards.json`,
because *"another tool decodes [`standards.json`] with unknown fields disallowed"* — the
convention `nomos-architecture.json` already set. So the corpus this plan makes locatable gets
its own declared file, the way the architecture declaration and the test-material policy did. A
new block added to `standards.json` would be refused by a tool outside this workspace, and the
refusal would arrive in somebody else's repository. This is the same reason `G1`'s first
sub-goal is written as *"located by a declaration"* rather than *"located by a config key"*: the
declaration's file is a decision M1 makes, and it is a new one rather than a key.

**The language-server question is closed, and Nomos already takes the compiler route.**
`OD-CAPABILITY-012` decided a language server may be a provider in principle but is not
buildable today, on two grounds: `nomos_platform` has no port for a long-lived, bidirectionally
communicating process, and `Assurance` has no variant for a best-effort index-backed answer when
`nomos-lang-rust-compiler` already reaches `Assurance::Sound` for the same class of question
through a direct compiler API.

**Nomos is an application over xvpe, and may adopt from it freely at the one pinned revision.**
`OD-PLATFORM-003` retires `AGT-006`'s no-dependency clause for the xvpe crossing — *"XVPE is the
engine. This workspace is an application over it"* — and `AGT-006` is assessed `Diverges`,
governed by that record. The guard was **deleted, not widened**:
`Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` does not resolve — it and
`PLATFORM_ADAPTER` are gone from `tests/contract/tests/boundaries/graph.rs`, deleted rather than
widened, because adding nine names to an allow-list
"would have left a rule that still *read* as a boundary while enforcing nothing." So **no
allowlist limits which xvpe crates may be adopted**; what binds is that every adoption comes in
at the single revision declared once in `[workspace.dependencies]` and stays pinned in
`Cargo.lock`. Nine of the ten crates declared there are consumed by at least one member
manifest; `xvpe-clock` alone is not, because `OD-ROADMAP-005` brought `Timestamp` back into
`nomos-platform`.

**The direction of travel has already run this way twice, which is why adopting from xvpe is
reclaiming rather than importing.** The agent dispatch this workspace had built **moved down**
into `xvpe-agent-backend-claude-code` and `xvpe-agent-backend-ollama` over a capability contract
in `xvpe-agent-execution`, and the whole LSP server moved down into
`xvpe-language-server-backend-lsp` over `xvpe-diagnostics` (`OD-HOST-013`) — in both cases
because *"a general capability sitting up here was unreachable by everything down there."* The
kwb crossing is untouched, and this record does not reach it:
`Test_No_Crate_May_Name_The_Sibling_Knowledge_Workbench` still fails the build on a `kwb-`
prefix, with an empty `KNOWLEDGE_ADAPTER`.

## The Decision

**Nomos is the ecosystem's shared rule layer, and this record fixes the shape and order of the
work that makes it one — not by copying either corpus, but by making each corpus readable, its
preferences declarable, and its undecidable residue routable.**

**A corpus is absorbed by reading its declarations, not by porting its rules.** `ARC-CONFORMANCE-002`
decided this and it remains the cheapest coverage available: one increment makes all 1,133
code-standards documents and all 261 xvpe documents visible at the cost of one reader, and it is
the purest form of the owner's constraint because nothing is copied — each corpus stays its
author's. It is built first.

**Only the machinery is ported, and every port states its outcome as one of four forms.** A port
is (P1) a tool relay, where a tool already reaches the verdict (`OD-RULES-010`); (P2) a declared
text rule, where the predicate is lexical (`OD-RULES-034`); (P3) a native function over a fact,
where structure is genuinely required; or (P4) a declaration that the rule is model-judged
(`OD-RULES-022`). The measurement above says P2 dominates, and that P3 is a minority case rather
than the default.

**Parameterize before porting.** `OD-RULES-011` governs and the order is not negotiable: a check
ported with a preference welded into its predicate is a check that must be ported again the
first time another repository declares a different style — which is the one outcome the
requirement forbids. `OD-RULES-011`'s own text already records two shipped Go rules that drifted
from the vocabulary their own rule id named for exactly this reason.

**The four forms are reached in this order, and the milestones in the ledger carry it.** M1, a
distributed corpus is readable. M2, a preference axis is one declaration. M3, a lexical rule is a
declaration. M4, a tool is a declaration. M5, a waiver and a repair are declarations, and a waiver that matches nothing is
never a silent pass. M6, what cannot be decided is routed rather than faked. M7, breadth. The order is
coverage per unit of work: M1 first because it is already decided and covers the most, M2 before
M3 because porting before parameterizing is the defect above, and M6 last because routing an
undecidable rule honestly is worth more once the decidable ones are decided.

**Adopt, do not rebuild, from the engine this workspace is an application over.** The finding
pass names the substrate, and every crate it names exists at the pinned revision:
`xvpe-diagnostics` (the canonical finding and severity shape, already consumed by `nomos-lsp`),
`xvpe-collections-text` (`Trie`, `radix_tree`, `SuffixArray`, `AhoCorasick`, `WaveletTree` —
the structures a vocabulary rule needs), `xvpe-algorithms-graph`, `xvpe-algorithms-sequence`,
`xvpe-workspace-architecture` (a deterministic, fingerprinted, model-free architectural index),
the injected persistence stack (`xvpe-file-system`, `-record-log`, `-event-journal`,
`-content-store`, `-corpus-ledger`), the reading substrates (`xvpe-corpus-text`,
`-document-model`, `-corpus-front`), and at foundations level `xvpe-command-line`,
`xvpe-content-identity`, `xvpe-evidence` and `xvpe-json-text`.

**One thing is deliberately absent from that list, and its absence is the size of M1.** No xvpe
crate parses markdown front matter or reads a declared standards corpus — a fact verified by
opening the candidates, not by their names, because **two of those names invite the opposite
reading**:

- `xvpe-corpus-front` is a corpus *port*: its modules are `corpus_front_strategy.rs`,
  `corpus_units.rs`, `library_formats.rs`, `unit_identities.rs`, `unit_read.rs` and a
  `strategies/` directory — what a corpus is made of, and the reader over a unit's *content*.
  Nothing in it looks at a delimiter fence or a YAML key.
- `xvpe-corpus-judgement` reads closest of all and is the subtlest near-miss: it is *"how a claim
  taken out of a corpus stands to this repository"* — a book chapter that proposes what the
  workspace already has is its `repo-is-ahead` verdict. That is the **corpus-to-repository**
  direction, mined from prose, and it is a closed verdict set over *claims*. M1 needs the
  **repository-to-corpus** direction: a document that declares `kind: rule` is a rule, and the
  question is which of the repository's files it reaches. Sharing the word "corpus" is not
  sharing the question.

Its dependency block is what makes this trustworthy rather than merely lexical: `xvpe-corpus-judgement` declares `[dependencies]` **empty**, with the recorded reason that *"the verdict set, the judgement and the evidence are value types over `std`, and the census reads Rust source as text — it does not parse it, and it does not know what a corpus, a chunk or a model is."* Neither crate carries a front-matter reader, so M1 builds one. What they do change is the shape M1's reader returns: a corpus is a **port** with units and identities, not a directory the reader walks itself, and a document has an `entity_id` and a `revision` (`xvpe-document-model`) rather than a path.

**An adopted crate is not a member of this workspace's declaration, and adds no row to it.**
`nomos-architecture.json` names twelve components and this workspace's own crates; an external
crate is outside its `members` map, so `Is_Permitted` is never asked about it. What governs
where it may be reached from is the zone of the crate that adopts it, and those permits already
allow it: `Rules` permits `Protocol`, `Substrate` and `Capability Contract`, and `Provider`
permits those same two plus `Capability Contract`. The crossing's own rule is unchanged and
still binding: one revision, declared once in `[workspace.dependencies]`, pinned in
`Cargo.lock`.

## Goals, Sub-Goals and Milestones

Three goals, and every sub-goal below is a property the workspace either has or does not —
not a task.

**G1 — Coverage without copying.** A repository's declared standards are judged where they are
declared, and nothing is copied into this workspace to make that possible. Sub-goals: a corpus
is located by a declaration rather than by a path compiled into a reader; a document is a rule
because it says so, never because a pattern recognised its shape; a declaration that will not
parse is an issue rather than a silent omission from the population; and the severity and gate
a document declares survive into the verdict unchanged.

**G2 — A preference is a declaration, and adding one is cheap.** No rule compiles a preference,
and no axis costs a bespoke path through the workspace to add. Sub-goals: every configurable
axis resolves through one shared read step; a waiver is a declaration rather than a marker in
the code it suppresses; a repair is a declaration, and a repaired finding is never reported as
a pass; and the cost of a new axis is measured against a stated bound rather than assumed.

**G3 — A verdict the repository can be held to.** What Nomos says about a rule is the same
thing the repository's own gate says, and what neither can decide is routed rather than
reported clean. Sub-goals: a declared `gate` is compared against the gate's real wiring; a rule
that cannot be decided mechanically is *representable* as such; an empty population is reported
rather than silently clean; and a rule nothing enforces is named rather than merely counted.

| Milestone | Serves | What it makes true |
|---|---|---|
| M1 | G1 | A distributed corpus is readable — one reader makes all 1,133 code-standards documents and all 261 xvpe documents visible. |
| M2 | G2 | A preference axis is one declaration: the new-axis cost falls from the ten paths `P125` reserves to a bounded one. |
| M3 | G1, G2 | A lexical rule is a declaration: `DeclaredTextRule` carries the detectors, parameters, citation and text surface the corpus's 95 text engines need. |
| M4 | G1, G2 | A tool is a declaration: a new `ToolProvider` costs a declaration rather than a crate. |
| M5 | G2, G3 | A waiver is a declaration: the 1,075 waivers the ecosystem already wrote reach the `SuppressionPolicy` that already exists, a waiver matching nothing is an error rather than a silent no-op, and a repaired finding is never reported as a pass. |
| M6 | G3 | What cannot be decided is routed: `Judgment::ModelJudged` is reachable and `Applicability::AgentRequired` is produced rather than assumed. |
| M7 | G1 | Breadth: the engines are ported and the language front ends multiply beyond the three that exist. |

`work/ledger.json` carries the tasks and this record carries none. Each milestone's own tasks
are authored as it opens, because a task's territory is grepped from the code it changes and
cannot be forecast from the milestone above it; what the board holds when this record lands is
the M1 item, `P149-A-DECLARED-STANDARDS-CORPUS-IS-READABLE-WHERE-IT-IS-DECLARED`, which is the
first increment this record decides.

That item reserves fifteen paths and not one of them is a design decision this record leaves
open: a new `nomos-cap-standards-corpus` contract crate in the `Capability Contract` zone, the
reader as a seventh provider in `nomos-repo-policy`, the `Declare` call and
`DECLARED_CAPABILITY_COUNT` in `nomos-check-orchestration`, the rule and its `DESCRIPTORS` row in
`nomos-rules`, the `OFFERINGS` row in `nomos-gate-orchestration`, and the four surfaces a new
crate and a new rule redden (`composed_offers.rs`'s declared uncomposed-offer list,
`bundled_contract_half.rs`, `completeness_universes/table.rs`, and the blessed snapshots).
`standards.json` is deliberately **not** among them, for the reason above. The count is stated
here because it is a measurement and not a forecast — it was grepped from the registration set
the last new capability contract actually touched (`e12a4c32`), not derived from the milestone's
description.

**And the zone is what this workspace enforces, while `tiers` is what another tool reads.**
`standards.json`'s `tiers` array is **not** a retired census. `code-standards` decodes it —
`kernel/config/limits/architecture_manifest.go` builds `limits.Dependency_Tiers()` from it and
projects it into an `architecture.Manifest`, with a test whose own comment calls the layer order
"the assertion that matters most in the file" — and the `standards.json` that tool reads is
**this repository's**. So `tiers` has a real reader. It just has no reader *here*: nothing in
this workspace asserts `tiers` against the workspace, and it has drifted badly — it declares
**48 crate paths over 29 tiers** where `nomos-architecture.json`'s `members` map declares **76
members**, and **26 crate directories on disk appear in no tier at all**, eleven of them
capability crates, `nomos-cap-review-finding` among them.

That is this plan's own subject occurring inside its first item. A worker registering a new
crate updates the declaration their own gate checks (`nomos-architecture.json`'s `members` map,
the `README.md` row beside it) and never sees `tiers`, because nothing here reads it — and the
omission is invisible in this repository and wrong in another repository's projection of this
one. A grep scoped to this workspace finds no reader of a field that has one, and only reading
the file from the other side finds it.

M1's item therefore does **not** reserve `standards.json`, and that is a decision rather than an
omission: a row nothing in this workspace asserts is not a clause a `done_when` can carry
honestly. A claimant would add it, no test would check it, and the item would finish green
having made nothing true — which is the defect class this whole plan exists to remove. The drift
is stated here as a finding with its own consequence, and it belongs to `G3` — a declared
architecture compared against the reality it claims to enumerate — rather than to the reader M1
builds. `nomos-architecture.json`'s `members` map and the `README.md` row beside it are
different, and both are enforced: `bands.rs` reads the declaration through the one provider that
parses it and never a second way, `graph.rs` asserts every member declares a component and that
dependencies run strictly downward, and `readme.rs` asserts the README lists every member. A
crate absent from the declaration fails the first two; a row absent from the README fails the
third. That is why those two are in the item's territory and `standards.json` is not.

## What This Does Not Do

**It builds nothing.** It is the plan's home and the campaign's authority; each milestone and
each task is its own ledger item with its own territory, `done_when` and predicate, and no code
moves here. A goal above is a property to reach, never a thing to claim.

**It does not reopen `ARC-ROADMAP-001`, `OD-RULES-024` or `OD-CAPABILITY-018`.** Observed
architecture and feature topology remain deferred, and the architecture rules this plan absorbs
are judged against the **declaration** — `Is_Permitted`, `Is_Excepted`, `Component_Of`,
`Doors_Into` — not against an observed graph. A milestone that found it needed an observed fact
would owe a new record first, not a widened territory.

**It does not re-decide whether a language server may be a provider.** `OD-CAPABILITY-012` holds:
not today, and on two named grounds. It also does not decide which linter speaks for a language —
that is a port's own argument, made in the crate doc, as `P128-GO-HAS-A-PARSER-AND-A-MANIFEST-
READER-AND-NO-TOOL-SPEAKS-FOR-IT` already requires.

**It does not decide what a model-judged rule's verdict is**, only that an undecidable question
must be *representable* and routed (`Applicability::AgentRequired`) rather than reported clean.
`OD-RULES-022`'s `Judgment` field and `P99-A-RULE-THAT-JUDGED-AN-EMPTY-POPULATION-2`'s
empty-population report are its prerequisites and neither is this record's to build. (The
unsuffixed `P99` identifier is the declined first attempt, not a live item.)

**It does not decide corpus selection or precedence.** Which corpus a repository points at, and
what happens when two are declared, is the per-repository policy's own business and stays
declared rather than compiled.

## Status

Accepted. The campaign's milestones are M1 through M7 as above; the ledger carries their tasks
and this record is the authority each of them cites rather than restates. `P26-RULES-MORE-CODE-
STANDARDS-3`'s completion — the exhaustion of what the current inputs can judge — is the event
that made this the next question, and it is why M1 rather than a further import slice is the
first increment. `P149-A-DECLARED-STANDARDS-CORPUS-IS-READABLE-WHERE-IT-IS-DECLARED` is on the
board and Ready, reserving the fifteen paths above, and it is the only item this record authors.
M2 through M7 are dated by nothing here: each opens when the one before it closes, and its
territory is grepped then.
