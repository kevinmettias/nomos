---
id: OD-PROJECT-002
type: decision
title: A required projection is a re-render obligation, so the required set is declared and not inferred
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - projection
  - freshness
  - gate
  - publication
relations:
  - target: OD-PROJECT-001
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
  - target: OD-SPEC-005
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
---

# A required projection is a re-render obligation, so the required set is declared and not inferred

## Question

`freshness --require` decides which projections must be present and current, and the gate has
named exactly one since the flag existed. Nothing records why one, which profiles could join
it, or what adding one costs. So the required set looked like an oversight, and every reader
who has looked at it has proposed widening it.

Two facts were missing, and both were measured rather than argued.

## What Was Measured

Every one of the eighteen shipped profiles was rendered against a store holding only this
repository's governing records — the store any checkout assembles, and the only store a runner
has, because the three corpora live outside this repository and CI has none of them.

**Four render. Fourteen refuse.**

| | profiles |
|---|---|
| render without a corpus | `diagram-set`, `domain-specification`, `html-site`, `traceability-matrix` |
| refuse | the other fourteen |

The refusal is `ProjectError::Empty` and it is correct: an empty projection over a store
nothing was read into is not a projection of an empty specification. A profile that cannot be
rebuilt on the tree CI checks out must not be required on it, or the step is red for a
variable that is not set rather than for a file that drifted.

## Rebuildability Is Not A Property Of A Profile

The second fact is the one that makes this a record rather than a list.

Which content kinds a seeded store can answer for **is** static, and is now declared as
`SEEDED_BY_RECORDS` in `crates/spec/nomos-spec-project/tests/projections.rs`: documents,
headings, blocks, rows, nodes, relations and lineage. Seeding writes no suites, no normative
statements and no omissions.

The declaration sits in the test file rather than on the crate's public surface, and that is
decided rather than left. Nothing reads it — the gate reads `--require` and the renderer reads
sections — so a public method would be API with no consumer, which this repository refuses
wherever it decides that only what something uses should exist. It was first written as
`Content::Seeded_By_Records` and that version moved the public surface snapshot of a crate
whose snapshot the authoring item had not reserved; making it private instead made it dead code
under `-D warnings`. Both signals point the same way, and what the declaration is *for* is the
comparison, so it lives with the comparison.

That declaration was wrong when first written — it omitted `Rows` — and the test comparing it
against a really-seeded store in both directions is what said so. Governing records carry
markdown tables and `Write_Record` writes their rows. The comparison runs in both directions
deliberately: one direction alone would let the list shrink to nothing or grow to all ten and
still pass.

But reaching only those kinds is **necessary and not sufficient**, and the gap is where the
first attempt at this decision went wrong. A section can reach a seeded kind and filter it on a
node kind only a corpus has:

| profile | reaches | refuses because |
|---|---|---|
| `feature-design` | nodes, relations | filters nodes on kind `concept` |
| `release-specification` | nodes, relations | filters nodes on kind `milestone` |

A governing record is kind `decision` or `architecture` and nothing else. So whether a filter
selects anything is data-dependent and cannot be read off a profile at all.

The consequence is stated as a rule, because the next person to widen the set will meet it:

> Requirability is established by rendering against a seeded store, never by inspecting the
> profile. A static screen over content kinds is a screen and not an answer.

`Reaches_Only_Seeded_Content` is that screen and is named so it cannot be read as the answer —
its first name was `Rebuildable_From_Records`, and the rename is the finding.
`Test_Every_Required_Profile_Should_Render_Over_A_Seeded_Store` is the authority, and
`Test_Rendering_Over_A_Seeded_Store_Should_Imply_Reaching_Only_Seeded_Content` holds the
asymmetry by pinning both the set that renders and the two witnesses that pass the screen and
refuse anyway. An empty witness set would mean the screen had silently become the answer.

## The Decision

**`domain-specification` joins `diagram-set` in the required set. `html-site` and
`traceability-matrix` rebuild here and are deliberately not required.**

`domain-specification` is added because it is the record set as one readable document — an
inventory of documents and headings, and then the prose of every governing record — and its
absence is the whole of the gap between an agent routing through `README.md` and an agent
opening a current projection of what this repository specifies about itself.

The price is stated in figures rather than in adjectives, because it is the reason the set stays
short. `spec/domain-specification.md` is **18,401 lines** against `diagrams/relations.mmd`'s
432, and a governing-record commit rewrites it. That churn is accepted here: the file is derived,
its diff is never reviewed for content, and what it buys is the one thing the diagram cannot —
a committed reading of the whole specification that is provably the store's, not a copy somebody
maintained. It is also why a second 18,000-line projection of the same content, which is what
`html-site` would be, is refused below rather than added for symmetry.

The two that are left out are left out for a stated price rather than by omission:

> Every required projection is a re-render obligation on every future record author.

`OD-GATE-005` established that a projection must be rendered from a tree whose registered
records are exactly the ones the commit publishes, and that such a tree is constructed rather
than found. That procedure is per-render. Requiring a fourth and fifth projection would put
five renders in front of every governing record, and the failure mode is not a slower loop —
it is an author who renders three of five, commits, and reddens the gate for everybody behind
them. `html-site` is `domain-specification`'s content in another format, so requiring it buys
a second obligation for no second fact. `traceability-matrix` overlaps `diagram-set` on
relations.

So the required set is a **declared list with a reason per entry**, and not every profile that
happens to render. That is the whole content of this decision, and it is publication policy:
the catalogue says how each projection is built and nothing in it says whether this repository
ships it.

## The List Is Written Twice, Deliberately

`.github/workflows/gate.yml` names the pair, and so does
`Test_Every_Required_Profile_Should_Render_Over_A_Seeded_Store`. That duplication is chosen
rather than overlooked.

The gate step is what CI runs, and a test is what a local `cargo test` can check before a push.
Deriving the test's list from the workflow would put a workflow parser in band 1, and
`nomos_ledger::Derive_Step` — which does exactly that derivation for the lint step — is in a
band `nomos-spec-project` may not depend on. `OD-GATE-005` decision 4 already refused to derive
this particular step into `work finish` for an unrelated and stronger reason.

Two hand-written copies of a two-element list is the cheaper defect, and it is the shape
`OD-SPEC-007` settled: both sides independently authored, compared by a test. The comparison
that matters is not between the two lists but between each entry and a seeded store, and that
is what runs.

## What This Binds

The gate requires `diagram-set` and `domain-specification`, and both are committed as a body
and a sidecar.

Adding to the required set requires a reason recorded here, because the cost lands on authors
who are not in the room when the profile is added.

Every file that tells an author how to re-render names every required profile. `AGENTS.md` and
`.claude/skills/nomos-spec-change/SKILL.md` are those files, and an instruction naming one
profile while the gate requires two is a procedure that produces a red gate for whoever
follows it.

## What This Does Not Do

It does not amend `OD-PROJECT-001`, which decides how projections are built and stamped. This
decides which of them this repository publishes.

It does not make the fourteen corpus-backed profiles defective. They project a whole store and
refuse an incomplete one, which is the contract working.

It does not add a check that the two written copies of the required list agree. Deriving one
from the other is refused above; comparing them would need the same parser in the same band.
The exposure is a two-element list in two files named by this record, and the failing direction
is loud: a profile required by the gate and absent from the test is caught the first time it
stops rendering, on CI.

It does not settle whether a required projection should be re-rendered automatically. That
would be a change to how commits are made rather than to what is required, and `OD-GATE-005`
priced it as a full build in a private target directory per finish.

## Controls

| Weakening | What it produces |
|---|---|
| require every profile that renders | five renders in front of every governing record, and an author who does four |
| require a corpus-backed profile | a gate step red on every runner for a variable that is not set |
| read `Reaches_Only_Seeded_Content` as requirability | `feature-design` required and refusing, which is the error this record was written after making |
| derive the seeded-kind list from the store | a comparison that cannot fail, which is `OD-SPEC-005` |
| check the list in one direction | it shrinks to nothing, or grows to all ten, and still passes |
| leave `AGENTS.md` naming one profile | every following record author renders half and reddens the gate |
| drop the sidecar to silence a comparison | a body with no sidecar is a failure rather than a skip |

## Status

Accepted. Two profiles are required, two more could be and are not, and the reason each way is
recorded. Whether a third should join is a question this record can be argued against rather
than a silence somebody fills.
