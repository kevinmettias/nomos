---
id: OD-GATE-024
type: decision
title: A finding a rule addresses by sub-item cannot be named by a person in a declared policy
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - policy
relations:
  - target: OD-GATE-017
    type: relates-to
  - target: OD-GATE-023
    type: relates-to
---

# A finding a rule addresses by sub-item cannot be named by a person in a declared policy

## Question

`P40-GATE-POLICY-AUTHORING-3` gave the four gate policies a declared source. Entries in it
name a **path**, because `SubjectId` is a digest nobody can write by hand, and
`nomos_model::Subject_Of_Path` turns that path into the identity a run matches on.

That works for a finding a rule addresses by the file. It does not work for one addressed by
something inside the file, and the failure is silent: an author writes a suppression for
`completeness-mirror`, gets no error and no warning, and the finding keeps blocking with
nothing saying why the entry did not take.

## What Was Measured

**Matching is digest equality, and nothing in it can see a name.** Both
`Suppression::Matches` and `BaselineDebt::Matches` are
`self.rule == finding.rule && self.subject == finding.subject`
(`policy/suppression_disposition/suppression.rs:36`, `policy/baseline_debt.rs:49`). A
declared entry resolves its `path` to a `SubjectId` once, up front, and the comparison is
between two digests from then on.

**The two sides digest different preimages, and one of them folds case.**
`Subject_Of_Path` is `SubjectId::From_Digest(Content_Digest(Normalize_Path(path).as_bytes()))`,
and `Normalize_Path` unifies separators and **lowercases**. A name-addressed finding computes
`SubjectId::From_Digest(Content_Digest(qualified.as_bytes()))` over the raw bytes — eight
sites across `checks/naming/` do exactly this. So for a finding whose subject is `OrderBook`,
a declared entry naming `OrderBook` resolves to the digest of `orderbook` and cannot match it.

That is the part worth a record rather than a bug report: **an author who types the name
exactly right still misses.** This is not a resolver that needs a wider lookup; it is two
identity derivations that were never meant to meet.

**`subject_name` already exists, on every finding, and is exactly the preimage.**
`Finding::subject_name` is documented as "the human-authored name the identity was derived
from ... the preimage of `subject`, so the two cannot disagree without the digest being
computed from something else," and it is there because "a digest alone cannot be read." The
addressable name the `done_when` asks for is not a field that has to be added.

**It is already heterogeneous, which is a feature here rather than an obstacle.** Sampled
across the rule set, `subject_name` is written three ways: a bare path
(`crosslang/comparison.rs`), a `path:line` (`concurrency_text.rs`, `constant_scope.rs`), and
a qualified name (`checks/naming/*`). A person naming a finding does not have to learn which,
because the run prints the one the rule used.

## The Decision

**They can, and the mechanism is `subject_name` — a declared entry names a subject the way
the run prints it, and the resolver compares names rather than re-deriving a digest.**

1. **A policy entry's subject key is an authored name, not a path.** `path` stays as its
   spelling wherever the entry names a file, because there the subject's name *is* the path;
   it stops being the only spelling. Nothing unauthorable enters the file — a digest is still
   never written by hand, which is the property the original design exists to protect.

2. **The comparison is against `Finding::subject_name`, not against a digest the entry
   resolved to.** This is the substantive change, and the measurement above is why: resolving
   through `Subject_Of_Path` re-derives an identity under path normalization that a
   name-addressed finding never used, so no amount of care by the author can make the two
   meet. Comparing the authored string to the finding's own preimage removes the second
   derivation entirely.

3. **The trade this makes, stated rather than discovered later:** a name comparison is exact,
   so a file entry must spell the path as the run prints it, where `Subject_Of_Path` would
   previously have forgiven a case or separator difference. That forgiveness is not free to
   keep — it is the same folding that makes case-bearing names unmatchable — and the report
   already prints the exact string, so the author is not being asked to guess. Where two
   files on a case-insensitive host differ only in case, `Normalize_Path`'s own reasoning
   still applies to *territory and fact identity* and is untouched by this; what changes is
   only how a person's declared policy entry finds a finding.

4. **An entry that matches nothing is reported, and this holds whichever way the rest is
   decided.** A declared suppression, baseline debt or calibration that matched no finding in
   a run is named in that run's output. It is not an error — a policy legitimately outlives
   the finding it was written for, and a repository whose debt was paid should not fail its
   own gate — but it is never silent again, which is the whole of the defect this record was
   filed for.

## What This Record Does Not Do

It does not add a digest field to the declared file. That would reintroduce exactly the
unauthorable identity the design exists to avoid, and the item that filed this said so first.

It does not change `Normalize_Path`, `Subject_Of_Path`, or how territory and fact identity
are computed. Those answer a different question — whether two paths are one file — and the
reasoning in `Normalize_Path`'s own doc about a lost edit is untouched.

It does not implement any of this. Making the entry name-keyed, moving the comparison onto
`subject_name`, and reporting unmatched entries are a follow-up item's own territory, reaching
`gate_policy_file.rs`, the three `Matches` implementations and the run's reporting — none of
which this record holds.

It does not exhaustively enumerate every rule's `subject_name` shape. Three were sampled and
found to differ; the decision does not depend on the census being complete, because comparing
to whatever preimage the rule used is what makes the shape not matter.

## Status

Accepted. A sub-item finding is nameable, by the `subject_name` the run already prints and
the finding already carries. The blocker was never that findings lack a readable name — it
was that a declared entry's path was re-derived into a digest under a normalization the
finding never used, so an exactly-correct name still could not match. The comparison moves to
the name; the path spelling survives for file-addressed entries; and an entry that matches
nothing is reported rather than silently ignored, which holds regardless of the rest.
