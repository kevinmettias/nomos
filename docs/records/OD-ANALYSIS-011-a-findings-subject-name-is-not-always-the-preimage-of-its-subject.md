---
id: OD-ANALYSIS-011
type: decision
title: A finding's subject name is not always the preimage of its subject
status: accepted
version: 2
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

It does not add a mechanical guard over the permitted derivations. Version 1 of this record
said one would be buildable by asserting that every finding's `subject` is either
`Subject_Of_Path` of one of its locations or the digest of its `subject_name`. **That sentence
was wrong**, and the amendment below replaces it: measured against the real functions, it is
false for 22 of the 63 sites censused above. A guard is still buildable over the three
derivations the amendment names is buildable, and one now exists: the test at
`tests/contract/tests/boundaries/findings.rs`, filed and landed separately from this record
rather than claimed here.

## Amendment: The Guard Described Here Would Have Refused A Third Of The Rule Set

Version 1 closed one trap and set another in the same breath. Its "What This Record Does Not
Do" section described a mechanical guard — `subject` is either `Subject_Of_Path` of one of the
finding's locations, or the digest of its `subject_name` — and a later session read that
sentence as a specification, which is exactly what a canonical record is for. Measured
2026-09-12 against the real `nomos_model::Subject_Of_Path`, `nomos_model::Normalize_Path` and
`nomos_model::Content_Digest`, **both arms fail** for the largest single shape in the census
above.

For a finding built by `checks/error_text.rs`'s `Finding_At`:

| field | what the rule sets |
|---|---|
| `subject` | `Subject_Of_Path(source.path)` — the file, with no line |
| `subject_name` | `format!("{}:{line_number}", source.path)` |
| `locations` | `vec![format!("{}:{line_number}", source.path)]` |

`Normalize_Path` unifies separators, drops `.` and empty segments and lowercases. It does not
strip a trailing `:line`, and nothing else does either — so `…/error_text.rs:238` normalizes to
itself, and against a subject digested from `…/error_text.rs`:

- `subject == Subject_Of_Path(one of its locations)` — **false**
- `subject == digest(subject_name)` — **false**
- `subject == Subject_Of_Path(that location's file part)` — **true**

**22 of the 63 construction sites are that shape**: 14 writing `subject_name` as the formatted
path and line directly, 8 writing it as a `location` variable of the same shape, all 22 paired
with `subject: source.subject`. They span 17 files — `borrowed_container`, `closure_bounds`,
`concurrency_text`, `constant_scope`, `domain_type_alias`, `enum_shape`, `error_text`,
`facade`, `flakiness_text`, `formatting`, `go_text`, `lifetime_discipline`, `placement`,
`procedural_macro`, `rust_text`, `scalar_range` and `security_text`.

### There are three permitted derivations, not two

`subject` is one of:

1. **`Subject_Of_Path` of the *file part* of one of the finding's locations**, the `:line`
   suffix stripped. This is the 22 sites above, and it is what the coarse subject is *for*:
   `Suppression` and `BaselineDebt` match on it and must outlive the revision they were
   written in, which is the argument this record already makes for not making `subject`
   line-sensitive.
2. **`Subject_Of_Path` of a location that is already a bare path** — case 1 with nothing to
   strip, which is the 16 sites whose `subject_name` is `source.path` and the 4 that name a
   package.
3. **The digest of the qualified name `subject_name` carries** — the naming family, where the
   relationship version 1 documented does hold.

Nothing about the decision changes. The rules were right in version 1 and are right now; what
was wrong is a sentence describing a guard over them. That is the same class of defect this
record was written to correct — prose asserting a derivation that does not hold — which is why
the correction belongs here rather than in the guard's own file.

### Why that guard would not have reported this

It would have passed. `nomos gate run` over this whole tree fires **8 of about 70 rules** —
`completeness-mirror`, `function-naming-convention`, `abbreviations`, `single-letter-names`,
`data-names-stay-lower-snake`, `lint-diagnostics`, `dependency-policy` and
`unread-reaches-finding` — and not one of their findings carries a line-suffixed location.
Narrowing to `tests/corpus` fires the same set. **No committed corpus exercises any of the 22
sites**, because this repository conforms to those 17 files' rules, and a rule that finds
nothing constructs nothing to check.

So the guard as version 1 described it had both failure modes at once: vacuous today, and red
the first time any of those 17 files' rules actually fires. That is the part worth carrying
forward — a claim about what the rules *construct* cannot be validated by an orchestrated run
over a clean tree, because a rule that finds nothing constructs nothing to check.

What the landed guard does instead is not a bigger corpus. The test at
`tests/contract/tests/boundaries/findings.rs` calls two **fact-free** rules directly —
`Check_No_Orphan_Modules`, which attributes to the file, and `Check_No_Trailing_Whitespace`,
which names a line — over a three-file fixture written to produce both shapes. Both
derivations are then exercised by construction rather than by hoping a corpus happens to
contain them, and a second test fails if the fixture ever stops producing either shape. Around
fifty rules expose that same `Check_*(sources: &[SourceFile])` form, so this route is open to
any future claim about what the rule set constructs — which is the cheaper half of what this
amendment has to teach.

## Status

Accepted, version 2. `subject` is a stable identity and `subject_name` is a readable label, at
deliberately different granularities, and `Finding`'s documentation said otherwise. The rules
are unchanged because they were never wrong; the contract is corrected because it was. Amended
once: version 1's own description of a mechanical guard over "the two permitted derivations"
was false for 22 of the 63 sites it had just censused, and the amendment above names the three
derivations that hold, together with the measured reason a guard written to the old sentence
would have passed vacuously rather than reporting the mismatch.
