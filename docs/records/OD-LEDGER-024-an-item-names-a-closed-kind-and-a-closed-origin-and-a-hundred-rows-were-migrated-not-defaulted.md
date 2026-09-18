---
id: OD-LEDGER-024
type: decision
title: An item names a closed kind and a closed origin, and a hundred rows were migrated, not defaulted
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - ledger
  - work-ledger
  - schema
relations:
  - target: OD-LEDGER-002
    type: relates-to
  - target: OD-LEDGER-006
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
  - target: OD-LEDGER-019
    type: relates-to
  - target: OD-LEDGER-022
    type: relates-to
---

# An item names a closed kind and a closed origin, and a hundred rows were migrated, not defaulted

## Question

`LedgerItem` carried fourteen fields — id, title, why, done_when, territory, state,
depends_on, blocked, claim, verification, verified, abandoned, displaced, declined — and
none of them said what *kind* of work a row was, or where it came from. `work add` refused
exactly one thing, an item that reserves nothing, and accepted any prose, from anybody, as
any kind of item.

That was a reasonable board for a person to write by hand. It stops being one the moment
work is proposed by the same kind of thing that executes it: a design in progress for this
repository has an agent observe something, propose an item, and the system classify that
proposal before it enters the graph. The classification step has nowhere to attach — no
`kind` to classify into, no `origin` distinguishing a proposal from a requirement — so the
guard against a session inventing its own next subject would have to be a convention, and
`OD-LEDGER-006` already records that this ledger cannot enforce a convention.

It also left the board unreadable in the one direction that matters for ordering. `state`
says where an item is; nothing said what it is. This item's own `why`, quoting the item
that named this defect, put it in four words: "a validation obligation, an architectural
decision, a correction and a cleanup are indistinguishable rows." Any priority over them had
to be recovered from prose by whoever was reading.

This is `P11-ITEM-DISCRIMINANT`, itself a reissue of `P11-ITEM-KIND`, which was declined
only because `P11-ITEM-KIND` and `P11-ADD-NOT-DURABLE` had both reserved
`docs/records/OD-LEDGER-021` — an identifier collision, not a disagreement about the
defect. The subject and the argument this record closes are unchanged from that first
opening.

## What Changed

Two new fields on `LedgerItem`, both in
`crates/substrate/nomos-ledger/src/item.rs`, both new submodules of `item`:

- `kind: ItemKind` (`crates/substrate/nomos-ledger/src/item/kind.rs`) — a closed,
  five-variant enum: `Capability`, `Decision`, `Validation`, `Correction`, `Cleanup`.
- `origin: ItemOrigin` (`crates/substrate/nomos-ledger/src/item/origin.rs`) — a closed,
  two-variant enum: `Required`, `Proposed`.

Both follow `ItemState`'s existing shape: plain Rust enums, serialized by serde as bare
strings. **Neither carries `#[serde(default)]`.** Every other optional field this ledger has
added since its first commit — `displaced`, `declined`, `VerificationRecord::revision` — was
given one, on the stated reasoning that "every item written before this field existed has
none, and that is a fact about those items rather than something to backfill"
(`LedgerItem::abandoned`'s own doc comment). `kind` and `origin` are the opposite case, on
this record's own `done_when`: "a field empty on a hundred rows and set on the next is a
field nothing can be asked about." A defaulted field would have let the two new keys arrive
on new rows only, indistinguishable in the file from a row nobody got round to. Refusing to
load a row missing either — the same mechanism `#[serde(deny_unknown_fields)]` already uses
for an unrecognized *key* — is what forced every existing row to be migrated in the same
commit that added the field, rather than left to happen eventually. That migration is
recorded below.

`SCHEMA_VERSION` rises `4 -> 5` in this same commit, per `OD-LEDGER-008`'s ordering and the
hazard this repository has already paid for once (`ea3c167`, which split a schema bump from
its field and left `dev` unusable): a build older than this one that meets a ledger row
missing `kind` or `origin` refuses the whole file, via `deny_unknown_fields` reached through
a missing-field error rather than an unrecognized one, and the refusal names both version
numbers.

An unrecognized *value* for either field is refused the same way an unrecognized *key*
already is: serde's default behaviour for a plain enum is to reject a string matching none
of its variants, so `"kind": "Feature"` fails to parse rather than being silently accepted.
`Test_An_Unrecognized_Kind_Or_Origin_Should_Be_Refused`
(`crates/substrate/nomos-ledger/src/item/tests.rs`) asserts it directly, alongside a
round-trip over every declared variant of both types.

### Why these five kinds

Not a free-form tag list — this item's own `done_when` refuses that shape by name: "this is
a discriminant, and a list of strings is the prose it was supposed to replace." Four of the
five are a direct transcription of the item's own `why`: **Validation** (a check, gate step,
snapshot or assertion, added or repaired), **Decision** (a seam, boundary, ownership rule or
policy settled and recorded), **Correction** (something wrong — a defect, a gap, a stale
statement — fixed), **Cleanup** (a rename, split, decomposition or named constant, reshaping
working code without changing what it does).

The fifth, **Capability**, is not in that list, because the four correction-era words have
nowhere to put this ledger's own first half. `OD-LEDGER-002` measured that batches `P1`
through `P8` built crates and commands that did not exist yet — `nomos-spec-model`,
`nomos-spec-store`, `nomos-capability`, `nomos-workspace`, and their gates — and named `P9`
onward "findings from an audit of the tree against what it claims." A defect taxonomy with
nothing for the thing it corrects *against* has no fifth of the board to describe. Rejected
alternatives: folding `Capability` into `Decision` (a new crate is not always a decision
about anything — most of `P1`–`P8` decided nothing that was in question, they built what the
plan already called for); leaving it out and forcing the earliest batches into `Correction`
(false — nothing was wrong when `nomos-spec-model` did not exist, there was simply nothing
there yet).

### What origin distinguishes, and why only two

`origin: ItemOrigin` answers the minimum this item's `done_when` asks for: "work that was
required from work a session proposed." `Required` is work a person specified — directly, or
through the plan `OD-LEDGER-002` says `P1`–`P8` built out. `Proposed` is work a session
opened because it observed the tree disagreeing with what it claims, with nobody having
asked for that exact item by name — again `OD-LEDGER-002`'s own words for `P9` and after.
No third value: the classification step a proposing agent would feed does not need a finer
distinction than "did a person ask for this," and a third value invented now with no
consumer is exactly the kind of speculative surface this ledger's other records
(`OD-LEDGER-017`) have already priced.

### Where the change actually landed, against what the item said

Two files in this item's assigned territory had already been split by the time it was
claimed, matching the shape `OD-LEDGER-022` and `OD-LEDGER-027` each record:
`crates/host/nomos-cli/src/work.rs` is the command dispatcher; the argument parser that
builds a `LedgerItem` from `add`'s flags — where `--kind` and `--origin` had to be read —
lives in its submodule `crates/host/nomos-cli/src/work/parse.rs`, reached by `mod parse;`.
`work.rs` itself gained one line, in `Show`, printing the two new fields for `work show`.

### Mechanically necessary edits outside the item's declared territory

Both new fields are required, with no `#[serde(default)]`. That is a Rust-level
constraint independent of serde: every direct `LedgerItem { .. }` struct literal in this
workspace has to name every field or the crate does not compile, and every hand-written JSON
fixture standing in for a ledger row in a test has to carry both keys or that row fails to
deserialize. Neither consequence is a choice this item made about scope; both are forced by
the field addition `done_when` calls for. The exhaustive set, found by
`grep -rn "LedgerItem {"` and `grep -rln '"done_when":'` across the workspace, and fixed
alongside the declared territory:

- `crates/substrate/nomos-ledger/src/lib.rs` — `ItemKind` and `ItemOrigin` added to the
  crate's re-export list, the same one-line-per-new-public-item shape `OD-LEDGER-022`
  recorded for `RefusalLayer`.
- `crates/substrate/nomos-ledger/tests/exclusion_holds/board.rs`,
  `crates/substrate/nomos-ledger/tests/declining_ends_an_item.rs`,
  `crates/substrate/nomos-ledger/tests/gate_covers_finish/launcher.rs` — each a `LedgerItem { .. }`
  struct-literal fixture helper, given `kind: ItemKind::Correction, origin: ItemOrigin::Proposed`.
- `crates/substrate/nomos-ledger/tests/exclusion_holds/persistence.rs`,
  `crates/substrate/nomos-ledger/tests/exclusion_holds/takeover.rs`,
  `crates/host/nomos-cli/tests/stale_writer_is_refused.rs`,
  `crates/host/nomos-cli/tests/takeover_is_recorded.rs`,
  `crates/host/nomos-cli/tests/abandon_is_readable.rs`,
  `crates/host/nomos-cli/tests/list_tells_the_truth/authored.rs`,
  `crates/host/nomos-cli/tests/add_guarantees_what_it_says.rs` — each a hand-written JSON
  ledger-row fixture, given `"kind":"Correction","origin":"Proposed"` beside `"done_when"`.
- `crates/host/nomos-cli/tests/add_guarantees_what_it_says.rs`'s `Add_Arguments` — the shared
  argument list every test in that file extends to run a real `nomos work add` — given
  `--kind correction --origin proposed`, since both are now required flags.
- `crates/host/nomos-cli/src/work/tests.rs` — in the item's own declared territory as
  `work.rs`'s test submodule — seven `add` invocations that reach `New_Item` (as opposed to
  three that are refused before reaching it, for missing or patterned territory, which stay
  as they were) given `--kind correction --origin proposed`.

None of this is new logic. Every one of the eleven files above already existed to hold a
`LedgerItem` shape or drive `work add`; each gained exactly the two fields the type now
requires, in the value already used as this record's own migration default (see below), and
nothing else about any of them changed.

## Migrating the board

`work/ledger.json` held 200 items when this item was claimed, none carrying `kind` or
`origin`. Hand-classifying 200 rows individually is not a task this item could do well by
authoring judgment for each — `done_when` allows for that directly: "if that's infeasible to
do well by hand for all of them, decide and document a principled default/inference rule."
The rule used, run once as a migration script and not committed as code (it is not consulted
again — the schema, not a script, is what enforces `kind`/`origin` from here on):

1. **Origin** by id-prefix family, against `OD-LEDGER-002`'s own table: `P1`, `P2`, `P3`,
   `P4`, `P7`, `P8` are `Required` (the plan's own batches); `P9`, `P10`, `P11`, `P12`,
   `P13`, and the one `T-` item are `Proposed` (`OD-LEDGER-002`'s own words: "findings from
   an audit of the tree against what it claims").
2. **Kind**, checked against the item's title (lowercased), first match wins:
   - **Cleanup** if the title contains any of: `decomposition`, `decompose`, `named liter`,
     `grouped parameter`, `clippy`, `collapsible`, `taxonomy`, `simplif`, `rename`,
     `file-decomposition`, `test-decomposition`.
   - **Validation** if the title contains any of: `gate`, `predicate`, `determinism`,
     `corpus-trace`, `trace`, `coverage`, `audit`, `lint`, `verif`, `unchecked`,
     `skill-agreement`, `conformance`, `vacuity`, `msrv`, `public api`, `one direction`, or
     the title ends in a reissue suffix `-2`, `-3`, `-4` (a reissue of a validation item is
     itself a validation item — the check is what did not land the first time).
   - **Decision** if the title contains any of: `seam`, `boundary`, `ownership`,
     `authority`, `scope`, `canonicity`, `policy`, `route`, `citation shape`,
     `staging name`, `evidence`, `floor`, `which product`, `undecided`, `no record`,
     `canonical for`.
   - Otherwise, **Capability** for a `Required`-origin item, **Correction** for a
     `Proposed`-origin one.

The rule is deliberately title-only, not `why`-text: the prose in `why` is long enough that
a keyword lands there by coincidence far more often than in a title this ledger already
writes as a one-line defect statement.

Result: 200 items classified, no absence.

| Origin | Kind | Count |
|---|---|---|
| Required | Capability | 30 |
| Required | Decision | 1 |
| Proposed | Cleanup | 4 |
| Proposed | Validation | 31 |
| Proposed | Decision | 16 |
| Proposed | Correction | 118 |

`Correction` dominating the `Proposed` bucket is not a rule artifact to be suspicious of —
it is this ledger's own history read back: past `P8`, the overwhelming majority of what this
repository's sessions have opened is exactly a defect somebody found between what a record
claims and what the tree does, which is what `Correction` names.

The one `Required`/`Decision` row is `P7-STORE` ("`nomos-store`: content-addressed documents
with one write door per authority"), matched on `authority` — accepted rather than forced to
`Capability`, because naming the one-write-door-per-authority rule was itself an
architectural decision the item made while building the crate, not merely following a plan
that had already decided it.

This item's own row (`P11-ITEM-DISCRIMINANT`) falls out of the rule as `Proposed` /
`Correction`, matching neither a title keyword above nor the `-2`/`-3`/`-4` reissue
suffix — its own lineage carries the subject forward through a changed record identifier
rather than a numbered suffix, so the rule does not read it as a reissue. Left as the rule
produced it rather than hand-corrected to `Validation`, which its own content might argue
for: a rule with a silent exception for the row that authored it is not the rule this record
describes, and "the item that named a missing field was itself fixing a gap" is exactly what
`Correction` already means.

## What Holds It

- `Test_A_Field_Added_To_An_Item_Should_Raise_The_Schema_Version`
  (`crates/substrate/nomos-ledger/src/item/tests.rs`) — the field-count assertion moved
  `14 -> 16`.
- `Test_An_Unrecognized_Kind_Or_Origin_Should_Be_Refused`
  (`crates/substrate/nomos-ledger/src/item/tests.rs`) — the closed-set guard, and a
  round-trip over every declared variant of both types.
- `Test_Every_Node_In_A_Ledger_Should_Refuse_An_Undeclared_Key`
  (`crates/substrate/nomos-ledger/tests/exclusion_holds/persistence.rs`) — unaffected in what
  it walks, since both new fields serialize as bare strings rather than objects, but still
  green over a document now carrying them.
- `tests/contract/surface/nomos-ledger.txt`, reblessed from source
  (`NOMOS_SURFACE_BLESS=nomos-ledger`): `ItemKind`, `ItemOrigin`, their variants, and
  `LedgerItem::kind`/`LedgerItem::origin`.
- `nomos work validate` against the migrated `work/ledger.json`, reporting schema 5.
- `cargo test --no-fail-fast -p nomos-ledger -p nomos-cli -p nomos-contract-tests`, skipping
  the two `records_do_not_serialize::snapshot_grain` tests that assert a property of the
  live, currently-open board unrelated to this item's diff — confirmed failing identically
  against an isolated worktree at this item's parent commit, the same mechanism
  `P10-DECLINED-DEPENDENCY-4`, `P11-DISPATCH-SPLIT-3` and `P11-EVIDENCE-IDENTITY` each
  recorded.

## What This Record Does Not Decide

It does not build the classification step that would read a session's proposal and assign
`kind`/`origin` before `work add` accepts it — the design `ARC-HARNESS-001` and this item's
own `why` describe. It makes the two fields exist, typed and closed, for that step to write
into; deciding what that step is remains `P11-NEXT-WORK`'s territory.

It does not add ordering over `kind` or `origin` to `work list` or `work audit` beyond what
`work show` now prints. A reader wanting to prioritize `Decision` work over `Cleanup` work
today still reads `show` per item; building that into a listing is follow-on, not this
record's `done_when`.

It does not revisit the 200 migrated rows' classifications by hand. The rule is documented
above precisely so a disagreement with one row's kind is checkable and correctable later,
by anyone, without re-deciding the rule itself.

## What Was Considered And Rejected

**A free-form `tags: Vec<String>` field instead of a closed `kind`.** Rejected on the
item's own `done_when`, verbatim: "not closed by a free-form tag list. This is a
discriminant, and a list of strings is the prose it was supposed to replace." A tag list
answers the question this ledger already had a shape for — `why`, in prose — where the
missing piece was a single, closed, filterable value.

**`#[serde(default)]` on both new fields, backfilling absent rows to some placeholder
variant.** Rejected because it produces exactly the field `done_when` names as worthless: one
"set correctly" on a hand-authored item and "defaulted, meaning nothing was decided" on the
other hundred and ninety, indistinguishable in the file. The cost was a wider blast radius —
every direct `LedgerItem` construction site and hand-written JSON fixture in this workspace
needed the two keys added regardless, once the type stopped compiling without them — but that
cost falls on the same commit that already had to migrate 200 real rows by hand; it did not
buy back anything a default would have saved.

**A single combined `kind` enum with ten variants, one per `(ItemKind, ItemOrigin)` pair,
instead of two orthogonal fields.** Rejected: the two axes are independent in exactly the way
`origin` says (a person can require a `Cleanup`; a session can propose a `Decision`), and
collapsing them into one enum would either lose that independence or duplicate five variants
across two prefixes for no comparison the flat form does not already give a caller matching
on either field alone.

## Amendment: The Test Named Above Was Renamed

Version 1 named a test under **What Holds It** that no longer exists under that name:
`Test_Every_Object_In_A_Ledger_Should_Refuse_An_Undeclared_Key` does not resolve. It is
`Test_Every_Node_In_A_Ledger_Should_Refuse_An_Undeclared_Key` in
`crates/substrate/nomos-ledger/tests/exclusion_holds/persistence.rs` since `6927e8d0`, the
naming pass that took `object` to `node` across this crate. Only the name moved.

**The assertion still holds**, and the sentence above now names the test that is there. What it
walks is unaffected by either new field for the reason version 1 gives — a unit variant
serializes as a bare string, so `kind` and `origin` are values and not containers, and the walk
probes object nodes — and the test is green over a document carrying them.

It is re-pointed rather than declared because the sentence is a claim about coverage: present
tense, under a heading that exists to name what holds this record today. `OD-SPEC-017` decided a
test name in a record is a live reference while a path citation is dated history, and `D-134`
already ranks a false claim of coverage above an admitted gap. The name version 1 used is quoted
here rather than left in place because a reader arriving at it in the list above would have gone
looking for a test that is not there, and the record republishes into `spec/domain-specification.md`.
