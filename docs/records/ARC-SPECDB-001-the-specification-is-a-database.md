---
id: ARC-SPECDB-001
type: architecture
title: The specification is a database with an enforced preservation ledger
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - preservation
relations:
  - target: D-129
    type: affects
---

# The specification is a database with an enforced preservation ledger

## Context

The Nomos specification existed as roughly twenty versioned archives of markdown. The
latest of them, v15.0, is a reorganization of v14.36 that destroyed content: 132 files
hold placeholder prose, 282 markdown table rows and all 6 code blocks are gone, and with
them the entire canonical domain model, all 8 roadmap release definitions, all 8 worked
scenarios and all 54 service descriptions.

The mechanism that would have caught this already existed. v14 shipped a per-block hash
manifest under a policy named `blocks_are_not_silently_dropped_from_source`, and a bundle
validator that implements the heading-disposition check. v15.0 shipped the normalized
views, deleted the narrative, and ran none of it. A publication gate had already been
declared for exactly this case.

The failure was not a missing check. It was that nothing forced the check to run, and a
run that checked nothing was indistinguishable from a run that found nothing wrong.

## Decision

The specification is a database. Content is addressed by hash, identity is a stable
`node_id`, and `uid` is a join surrogate that never leaves the process. Markdown, YAML,
JSON and HTML become generated projections of that store.

Every source document is segmented into blocks, and every block carries both its verbatim
hash and its normalized hash. Every block and every heading must have a recorded
disposition or a recorded omission. An omission carries a reason, a justification and a
decision record, and is itself validated.

A validation run passes only when every declared rule ran to completion. A declared rule
with no implementation, a registered rule the manifest never declared, and a rule that
errored internally all fail the run. The absence of a validator must be indistinguishable
from a failing one.

A satisfied rule records how many subjects it examined, because "0 violations over 0
subjects" and "0 violations over 2533 subjects" print the same and mean opposite things.

## Consequences

Content cannot leave silently. It can still leave — through an omission that names a
decision record — which is what keeps the gate a gate rather than a wall.

The portable authority committed to git is a deterministic JSONL bundle rather than the
database file. A bundle round trip is byte-identical, so the corpus can be reviewed as a
diff.

Generated documentation is never committed and never becomes source. Five independent
barriers enforce that, because one barrier is a convention.

## Alternatives Considered

Keeping markdown as the identity substrate and adding a stricter validator was rejected.
That is what v14 already had, and it did not survive one reorganization, because the
validator was a step somebody had to remember rather than a property of the store.

Storing only the normalized view was rejected. It is what v15.0 did. A normalized view is
never a replacement for either source authority, and the 282 rows lost are what that
sentence costs when it is advice instead of a constraint.

Rebuilding from the archives by hand was rejected. Without a per-block ledger there is no
way to tell restoration from invention.
