---
id: OD-LEDGER-035
type: decision
title: Whether actual impact should be recorded against declared territory, or territory being declared not enforced makes the drift unmeasured rather than owed
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - territory
  - traceability
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
---

# Whether actual impact should be recorded against declared territory, or territory being declared not enforced makes the drift unmeasured rather than owed

## Question

An external design review (`nomos_spec_and_work_ledger_arch.txt`) recommends recording a
task's actual touched paths against its declared expected impact at completion, flagging
expansion beyond declared scope, to catch undeclared scope creep and improve future
planning. `LedgerItem::territory` declares expected paths and patterns; `OD-LEDGER-001`
already states territory is *declared, not enforced* — nothing stops a write into a
claimed path. So a finished item's actual diff can exceed its declared territory today,
and nothing records that it did. `P13-IMPACT-ACTUAL-QUESTION` asks whether that gap is a
real, currently-occurring cost or a theoretical one, by checking real commits rather than
arguing from the schema alone.

## What Was Measured

`e7e7d9a` (`P13-CAPABILITY-008-THIRD-PROVIDER-LANDED`) declared one path as territory:

```
docs/records/OD-CAPABILITY-008-...-already-closes-the-gap.md
```

`git show --stat --format="" e7e7d9a` reports four files actually changed:

```
...already-closes-the-gap.md                          |  86 +++++++++++---
spec/domain-specification.md                           | 123 ++++++++++++++++-----
spec/domain-specification.md.nomos-projection.json     |  54 +++++++--
work/ledger.json                                       |  54 +++++++++
4 files changed, 263 insertions(+), 54 deletions(-)
```

Two files outside declared territory, every time a record-writing item finishes: the
rendered `domain-specification.md` and its `.nomos-projection.json` sidecar. `work/ledger.json`
is not a territory violation — every `work finish` writes the board itself, expected by
construction.

**This is not drift; it is a named, deliberate exception.** `OD-GATE-005` decision 2
already ruled that the derived diagram (and, by the same reasoning, the derived
domain-specification projection) is reserved by no item, specifically because reserving it
in every record-writer's territory "would serialize the whole board on a file nobody can
conflict over." So the one reproducible category of actual-impact exceeding declared
territory that this record could find is a category `OD-GATE-005` already excluded from
territory on purpose, not an oversight `OD-LEDGER-001`'s "not enforced" was warning about.

**What a naive version of the reviewed recommendation would do.** A mechanical check
comparing a commit's actual files against declared `Territory` and flagging any excess
would flag `spec/domain-specification.md` and its sidecar on essentially every
record-writing commit — a large fraction of this board's completed items, since most
`Correction` and `Decision`-kind items touch `docs/records/`. Built without first
exempting exactly the outputs `OD-GATE-005` already named as nobody's territory, the
check's first measured behavior would be a false positive on the majority of its subjects,
which is worse than the silence it would replace: a warning that fires on nearly everything
teaches its reader to ignore it.

**What was not found.** No commit sampled shows a *non-projection* file touched outside its
item's declared territory. This does not establish the absence generally — only this one
commit and the surrounding pattern were checked, and `OD-LEDGER-001`'s point stands: nothing
mechanical would catch it if it happened. The finding is that the one concrete, reproducible
instance of the gap this record could locate is already accounted for by a different,
already-written record, not that the underlying risk `OD-LEDGER-001` names is closed.

## Decision

**Declined**, as currently scoped. No actual-impact field or `work finish`-time
git-diff-versus-territory comparison is built now. Building it before the projection
exception exists as a named allowlist would make the first version of the check actively
misleading — flagging the normal, sanctioned case as the notable one — and no incident of
a genuine, non-projection out-of-territory write has been found to justify the cost of
building that allowlist first.

**What would reverse this.** A real incident of a non-projection file being touched outside
an item's declared territory, undetected until it caused an actual coordination failure —
a peer's claim colliding with work nobody could see coming because it was never declared.
Or, independently, someone building a `work finish`-time git-diff check for an unrelated
reason (catching an accidentally-broad commit before it lands, for instance) — at which
point excluding the `OD-GATE-005` projection outputs from its comparison is a small
addition to a mechanism that already exists, rather than a reason to build the mechanism
itself.

## Status

Accepted, drawn by `P13-IMPACT-ACTUAL-QUESTION` against one sampled commit rather than a
survey of the whole board — the sample is small by construction, and the trigger above is
written so a future reader does not need a larger survey to reopen this if a real incident
appears.
