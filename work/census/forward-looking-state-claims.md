# Forward-looking implementation-state claims: a census

Run 2026-09-14 against `dev` at `24464f2c`, for
`P99-FORWARD-LOOKING-STATE-CLAIMS-ARE-CENSUSED-BEFORE-ANY-RULE-IS-DESIGNED-FOR-THEM`. The item
exists because three stale claims were found incidentally in one session, and three instances
found by accident is evidence of a pattern rather than a measurement of one. This counts the
population before anything is designed for it.

## What was searched, and where

The eleven phrases the item names, case-insensitively: `not yet`, `nothing calls`,
`follow-up increment`, `once one exists`, `will wire`, `future provider`, `still unbuilt`,
`not composed`, `not reachable`, `not CLI-wired`, `no caller`.

Two populations, matching the item's own scope:

- **Module docs under `crates/`** — lines beginning `//!` only. A `//` comment inside a
  function is not what a reader is routed to as navigational authority, which is the defect
  `OD-AGENT-001` names one layer down.
- **Every line of `docs/records/`.**

| population | raw phrase hits | scope searched |
|---|---|---|
| all comments and code under `crates/` | 86 | (not the subject; measured only to show what the narrowing removed) |
| module docs under `crates/` | **17** | the subject |
| `docs/records/` | **88** | the subject |
| **total in scope** | **105** | |

The narrowing from 86 to 17 in `crates/` is most of the difference between "a phrase appears"
and "a reader is told something about the state of the workspace".

## Buckets

The item's four, applied to all 105:

| bucket | crates module docs | `docs/records/` | total |
|---|---|---|---|
| 1. still true | 10 | 43 | **53** |
| 2. superseded by implementation | **3** | **0** | **3** |
| 3. intentionally deferred | 2 | 4 | **6** |
| 4. historical context, dated or scoped | 2 | 41 | **43** |
| | 17 | 88 | 105 |

**How each bucket was assigned, because the buckets were not assigned the same way and saying
which is which is the point.**

Bucket 2 is the only one determined by checking the claim against the code it names. Every hit
in every section that states or supports a decision — `The Decision`, `Status`,
`What This Record Does Not Do` and their spelling variants, 29 record hits in all, plus all 17
crates hits — was read and verified individually. The three below are what survived that.

Buckets 1, 3 and 4 on the record side were assigned by a stated, reproducible rule rather than
by judging 88 sentences one at a time: a hit in `Question`, `What Was Measured`, `Current
Position`, `The Finding`, a titled amendment section, the title or the front matter is bucket
4; of the rest, one whose sentence states a decision not to build (`does not decide`,
`declined`, `left to`, `its own item`, `not built here`, `deliberately`, `is a question for`)
is bucket 3, and the remainder is bucket 1. The 17 crates hits were assigned by reading each,
and are listed individually in the appendix.

The 1-versus-3 line is thin for record prose and nothing here depends on where it falls. What
the census is for is the size of bucket 2, and that number was verified rather than inferred.

Bucket 4 is large because of one structural fact about records: a record's `## Question` and
`## What Was Measured` sections **are** dated context by construction — they state what
prompted a decision — and a record that is amended gains a new section rather than having its
old one rewritten. 13 of the 88 record hits are in `What Was Measured`, 4 in `Question`, and 11
more are in amendment sections naming the version that added them. That is not stale prose; it
is the record format working.

## The superseded bucket, in full

Three, all in `nomos-gate-orchestration`. Each was verified against the code it names rather
than inferred from the phrasing.

### 1. `crates/orchestration/nomos-gate-orchestration/src/lib.rs:189`

> It is not CLI-wired: no flag or config file reaches it from a `nomos gate` invocation, and
> `nomos-cli`'s own gate-command parser still refuses a `compare` subcommand outright.

False on both halves. `crates/host/nomos-cli/src/gate/parsing.rs:163` returns
`Invocation::Compare`, and `--against`, `--from` and `--to` are in that file's own recognized
flag list at line 10. `P101-GATE-COMPARE-IS-REACHABLE-ONLY-FROM-A-TERMINAL` and
`P101-THE-HEADLESS-SURFACES-STILL-STOP-AT-THREE-GATE-VERBS` are both Done.

### 2. `crates/orchestration/nomos-gate-orchestration/src/lib.rs:13`

> Most of that is still unbuilt.

Said of `WF-001`'s list — "required phases, thresholds, coverage, unsupported-analysis policy,
waivers, approvals, and blocking behavior" — and contradicted by this same module doc further
down, which enumerates the increments that built coverage (`OD-GATE-016`, ninth increment), the
policy file (`P40-GATE-POLICY-AUTHORING-3`, tenth) and phases and approvals
(`P40-GATE-PHASES-APPROVALS-5`, eleventh). A reader meets the sentence as the crate's present
state; the evidence beneath it is scoped to "at this crate's own start" and the sentence is not.

### 3. `crates/orchestration/nomos-gate-orchestration/src/composition.rs:30`

> This is not yet a shared derivation: the rows below are still authored, because a rule's
> contract citation is knowledge no export carries.

The premise is now false. `nomos_rules::RuleDescriptor` carries `contract_record` and
`contract_record_version`, `DESCRIPTORS` is public, and `OD-RULES-027` is what put them there —
the same record this file's own neighbouring paragraphs cite for the parity it already derives.
The rows may still be worth authoring by hand for other reasons, but not for the one given.

**None of the three is corrected here.** All three live in `nomos-gate-orchestration`, which
this item does not reserve and which a peer holds at the time of writing. They are boarded as
`P99-THREE-GATE-MODULE-DOC-CLAIMS-DESCRIBE-A-CRATE-THAT-HAS-SINCE-BEEN-BUILT`, which reserves
the two files and follows the `67d66b8e` pattern: the original restraint is preserved and what
satisfied its trigger is stated beside it, because a decision held and then released on evidence
is a stronger precedent than one still waiting.

## The three that prompted this item are absent from the superseded bucket, correctly

`gate_command.rs`'s "compare is absent", `nomos-api`'s citation of it, and `nomos-api`'s
transport promise were each corrected before this census ran. `crates/host/nomos-api/src/lib.rs:154`
still matches the phrase search and reads, in full, "`nomos-api-transport` is the real second
caller this section once said a follow-up increment would wire" — the corrected text, quoting
the claim it replaced. That is bucket 4, and it is the pattern the corrections above are asked
to follow.

## One near-miss, checked and left alone

`OD-HOST-002:122` reads "No `Connector` type exists anywhere in this workspace's code." A crate
called `nomos-connector-coderabbit` has existed since `OD-CAPABILITY-002`, so the sentence
invites being read as false. Grepped for `pub struct Connector`, `pub trait Connector` and `pub
enum Connector`: there is no such type, and the sentence is exactly true as written. It is
recorded here because the next reader will have the same doubt, and because it is the shape a
mechanical check would get wrong in the other direction — a rule matching "connector" against
the crate list would flag a true sentence.

## Is the superseded bucket mechanically checkable?

**Not on this evidence, and the census is what says so.**

All three share a shape: a module doc asserting a *negative* about a named identifier or CLI
verb — no flag reaches it, the parser still refuses it, no export carries it. That is more
structure than "a sentence containing `not yet`", and it is the part a rule could match. What a
rule cannot do is the rest: deciding whether "no flag reaches it" is true requires resolving
which flag, in which parser, and evaluating a claim about reachability that no fact in this
workspace establishes. A rule matching the shape alone would report all 105 hits and be wrong
about 102 of them, which is a rule that fires on prose rather than on state.

Three instances in one crate, with one cause between them — that crate was built in eleven
increments and its module doc was extended rather than re-read — is a population worth naming
and not yet a population worth designing against, the same criterion `OD-GATE-023` already
applies. The useful result of this census is the bound: not the 174 a naive grep suggests, not
the 105 in scope, but **3**.

## What this census does not do

It designs no rule and composes none. It corrects nothing itself — the three corrections are
boarded with the territory they need, because this item reserves only this file.

It does not re-classify a record's `Question` or `What Was Measured` section as stale merely
because later work happened. `P99`'s own `done_when` forbids that, and the record format
depends on it: a record whose measurement was rewritten every time the workspace moved would
stop being evidence of anything.

## Appendix: every module-doc hit under `crates/`

| location | bucket | text |
|---|---|---|
| `crates/contracts/nomos-contracts/src/workflow_step.rs:15` | deferred | "engine, once one exists, would have to honor" — no workflow engine exists; `OD-WORKFLOW-003` |
| `crates/host/nomos-api/src/lib.rs:154` | historical | the corrected text, quoting the claim it replaced |
| `crates/languages/nomos-lang-rust-compiler/src/check.rs:12` | still true | not offered in `Declare_*_Capability`; verified against `composition.rs` |
| `crates/languages/nomos-lang-rust-compiler/src/nested_lock_check.rs:11` | still true | same provider, same verification |
| `crates/languages/nomos-lang-rust-deny/src/lib.rs:22` | deferred | `advisories` is named as another provider's question |
| `crates/orchestration/nomos-check-orchestration/src/package_conformance.rs:29` | still true | verified: only `lib.rs` re-exports `Check_Package_Conformance`; no host calls it |
| `crates/orchestration/nomos-check-orchestration/src/package_conformance.rs:31` | still true | same verification |
| `crates/orchestration/nomos-check-orchestration/tests/incremental_equivalence.rs:21` | still true | describes the test's own fixture |
| `crates/orchestration/nomos-check-orchestration/tests/incremental_equivalence.rs:23` | still true | same |
| `crates/orchestration/nomos-gate-orchestration/src/composition.rs:30` | **superseded** | see 3 above |
| `crates/orchestration/nomos-gate-orchestration/src/gate_compare.rs:6` | historical | past tense about what `gate_command.rs` said before its own correction |
| `crates/orchestration/nomos-gate-orchestration/src/lib.rs:13` | **superseded** | see 2 above |
| `crates/orchestration/nomos-gate-orchestration/src/lib.rs:128` | still true | about `command.rules`/`command.scope` ordering within one function |
| `crates/orchestration/nomos-gate-orchestration/src/lib.rs:189` | **superseded** | see 1 above |
| `crates/platform/nomos-platform/src/lib.rs:34` | still true | a stated port-admission principle, not a state claim |
| `crates/spec/nomos-spec-store/src/edit/staged_edit.rs:1` | still true | describes a stage in this type's own lifecycle |
| `crates/spec/nomos-spec-store/src/table/rows.rs:6` | still true | "no caller has to subtract one" is a property of the API, not a state claim |
