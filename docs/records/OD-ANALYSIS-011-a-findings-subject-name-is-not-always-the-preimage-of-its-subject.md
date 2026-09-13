---
id: OD-ANALYSIS-011
type: decision
title: A finding's subject name is not always the preimage of its subject
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - analysis
  - rules
  - gate
relations:
  - target: OD-GATE-024
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
---

# A finding's subject name is not always the preimage of its subject

## Question

`Finding::subject` documents itself as "Derived from [`Finding::subject_name`], never from a
path", and `Finding::subject_name` as "the preimage of `subject`, so the two cannot disagree
without the digest being computed from something else."

They disagree, across most of the rule set. `OD-GATE-024` was written on that contract and
retracted when implementing it proved it false, so the question is no longer whether the two
disagree but which of them is wrong: the rules, or the contract.

## What Was Measured

Censused every `Finding` construction under `crates/rules/nomos-rules/src/checks/` — 63
sites. Of those, **42 set `subject: source.subject`**, the digest of the *file*, while
`subject_name` names something else:

| `subject_name` written as | sites |
|---|---|
| `source.path.clone()` | 16 |
| `format!("{}:{line_number}", source.path)` | 14 |
| `location.clone()` | 8 |
| a package or external id | 4 |

The naming family is the other shape: it sets
`subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes()))` over the very name
`subject_name` carries, and there the documented relationship does hold.

**Even the sixteen that look like they hold do not.** A walked source's subject is
`nomos_model::Subject_Of_Path(relative)` (`nomos-cli`'s own `Read_Source`), which digests the
path *after* `Normalize_Path` unifies separators and lowercases it. `subject_name` is the raw
relative path. The two agree only for a path that is already normalized — so the relationship
is "digest of a normalized form of it", not "preimage".

**The `path:line` shape cannot be made to hold without breaking something real.** Making
`subject` the digest of `"file.rs:12"` would give a finding a new identity every time a line
above it moves. `Suppression` and `BaselineDebt` match by `subject` precisely so that a
tolerated finding stays tolerated across revisions; a line-sensitive subject would expire
every entry on the next unrelated edit. The coarse subject is load-bearing.

**And coarsening the name is not available either.** Reporting `src/lib.rs` instead of
`src/lib.rs:12` for fourteen rules would take from a reader the one thing that lets them find
what the rule is talking about.

## The Decision

**The contract is wrong, not the rules. `subject` and `subject_name` are deliberately at
different granularities, and the record of that is this.**

- **`subject` is the stable identity a finding is attributed to across revisions.** For most
  rules that is the *file* (`Subject_Of_Path`); for the naming family it is the *qualified
  name*. What makes it right in both cases is that it survives edits that did not change what
  the finding is about — which is exactly what `Suppression` and `BaselineDebt` need, since
  they match on it and are meant to outlive the revision they were written in.
- **`subject_name` is the human-readable label for what was found**, and is frequently finer
  than the subject: a line within the file, a function within the module. It is not in general
  the digest's preimage, and requiring it to be would force a choice between an identity that
  expires on every edit and a report that cannot say where to look.
- **Neither is derivable from the other in general.** A caller that needs identity uses
  `subject`; a caller that needs something to show a person uses `subject_name`; a caller that
  needs to *match what a person wrote* has neither today, which is precisely why
  `OD-GATE-024` could not be implemented and why that record is retracted rather than pending.

`Finding`'s own documentation is corrected to say this, because the claim it makes now is the
reason a whole decision was written on a false premise. The two sentences that assert the
derivation are replaced by what the two fields actually are.

## What This Unblocks, And What It Does Not

It unblocks nothing by itself, and that is worth stating plainly. `OD-GATE-024`'s question —
how a person names a sub-item finding in a declared policy — is *still open*, and this record
makes clear why it is harder than it looked: there is no existing field that is both
authorable by a person and a stable match key. A future answer has to either add one, or
accept matching on something coarser than the finding.

What it does close is the trap. The next session to read `Finding::subject_name` will not be
told it is the preimage of `subject`, will not build a matcher on that, and will not have to
re-derive by experiment what this one derived by implementing and reverting.

## What This Record Does Not Do

It does not change any rule. All 63 construction sites are correct under the corrected
contract, which is the point of correcting the contract rather than the rules.

It does not change `Subject_Of_Path`, `Normalize_Path` or how territory identity is computed.

It does not add a mechanical guard over the two permitted derivations. One would be buildable
— assert that every finding's `subject` is either `Subject_Of_Path` of one of its locations or
the digest of its `subject_name` — and it reaches the rule crate and the contract test suite,
neither of which this record holds. It is filed separately rather than claimed here.

## Status

Accepted. `subject` is a stable identity and `subject_name` is a readable label, at
deliberately different granularities, and `Finding`'s documentation said otherwise. The rules
are unchanged because they were never wrong; the contract is corrected because it was.
