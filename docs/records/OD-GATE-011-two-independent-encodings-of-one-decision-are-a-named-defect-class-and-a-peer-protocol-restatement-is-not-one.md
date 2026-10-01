---
id: OD-GATE-011
type: decision
title: Two independent encodings of one decision are a named defect class, and a peer-protocol restatement is not one
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - duplication
  - determinism
relations:
  - target: OD-GATE-007
    type: relates-to
  - target: OD-GATE-013
    type: relates-to
---

# Two independent encodings of one decision are a named defect class, and a peer-protocol restatement is not one

## Question

Five closed defects in this repository share one shape, and none of the five was found by a
check.

`DECLARED_RULES`' doc comment cited `Test_A_Rule_Nobody_Declared_Should_Fail_The_Run`, which
reconciles a synthetic fixture against input it invents. `Test_The_Registry_Should_Match_The_
Manifest`, in `nomos-spec-validate/tests/preservation_holds.rs`, had validated the real
registry against the real manifest all along — a second, correct answer to the same question
the table's comment already claimed to answer, cited by nothing (P10-MIRROR-DISAGREEMENT).

The README's band table, `nomos-contracts`' crate-root doc comment, and `Cargo.toml`'s
workspace-members comment each described what belongs in band 0, in independently chosen
wording. The widest of the three sat in a manifest comment nothing parses, so the
disagreement had no mechanical authority to resolve it against, and nothing constrained what a
tenth or eleventh module could argue its way into (P10-BAND0-ADMISSION).

`work audit` asked `ExclusionLedger::Conflicts` which live claims overlap an item's territory.
`work list` asked a different function, `Claim_Refusal`, for the same question. They had
already come apart in both directions before anyone noticed: measured on the live board,
`audit` called forty-four of fifty-four lines blocked by a claim that had already finished, or
would never contend for that territory again (P10-AUDIT-STATE).

Three call sites — the ledger addressing a claim, the composition root, and the integration
corpus — each reduced a repository path to a `SubjectId` by hand, with a spelling rule that
was byte-for-byte identical in all three copies (P10-SUBJECT-HOME).

`Cargo.toml`'s `[workspace.lints.clippy]` table stated `clippy::all` and `clippy::pedantic` at
`warn`. `.github/workflows/gate.yml`'s `Lint` step ran the same command with `-- -D warnings`
appended, which promoted every one of those `warn`-level findings to a hard CI failure without
saying so anywhere the table's reader would see it (P11-LINT-AUTHORITY-3, `OD-GATE-007`).

Different files, different languages of expression — a doc comment against a test citation, a
manifest comment against a crate-root comment, one function against a sibling function, three
copy-pasted call sites, a `Cargo.toml` table against a workflow flag. What is identical across
all five is the failure mode: two artifacts each independently answer one question, nothing
compares them, and the pair agrees for as long as nobody edits either — so there is no
symptom, and no warning, until the edit that breaks the pair looks complete, because its
author changed the one authority they knew about. `AGENTS.md` already states the resolution
once a disagreement is found — "when two authorities disagree, the mechanical one wins and the
disagreement is a defect worth an item" — but nothing finds it. Each of the five above was
found by a person reading two files side by side, not by a run.

## Decision

**Two artifacts encode one decision when each is independently readable as the authoritative
answer to the same question, such that editing one without editing the other changes the
answer a reader gets, and neither artifact says the other exists.** That is narrower than
"the same value appears twice" — a value repeated for two different questions is not this
defect — and it is verified against the five instances above rather than against a definition
invented for this record: a check, or a reviewer, that does not name all five when applied to
this repository's history is measuring some other shape.

**A second encoding is not this defect when both sides derive independently from one stated
external authority, rather than from each other, and a test pins the pair against each other
so drift between the two is still caught.** `PackageKind`'s Rust variant names and its
serialized `Label` constants (`crates/contracts/nomos-contracts/src/package.rs`) are the
worked example: both are transcribed from the package-taxonomy sentence of volume 03 of the
corpus, in the order that sentence names them, because "a peer reimplementing this enum in
another language reads the label and never sees this Rust, so a label is a protocol commitment
rather than a local identifier" — the enum's own doc comment says so, naming the corpus source
at the site rather than leaving a reader to infer why two spellings exist for one taxonomy.
`Test_Every_Kind_Should_Carry_The_Label_The_Corpus_Names` pins the two encodings against each
other, so the pair drifting apart is still a failing test even though "matches one internal
source of truth" does not apply to either half. Three conditions distinguish this class from
the defect above, and all three must hold: the derivation is from a named authority outside
either artifact, the reason is stated in a doc comment or record at the site rather than left
implicit, and a test — not a check's silent exclusion — holds the two sides together.

Two artifacts that fail to meet all three and still disagree are not a legitimate exception in
waiting; they are an unfound instance of the class above.

## What This Costs

**No check exists yet.** This record commits to the definition, the validating corpus, and the
legitimate-exception criterion — the three things the class needed before anything could be
built against it — and not to the implementation. Every instance named above was found by a
person, and that stays true until a later item builds a check against this record's
definition. `OD-GATE-013` accepted the identical shape of cost for a narrower class — a
checker gap named and bounded by record rather than closed by code — for the same reason: the
definition has to be right, and validated against real instances, before a check built against
a wrong definition either misses the five above or flags `PackageKind` as the sixth.

**The definition is stated as a question-and-answer shape, not as a syntactic pattern, and
that is itself a risk.** A future check applying it has to decide, per candidate pair, whether
the two sides are "independently readable as the authoritative answer to the same question" —
a judgment call this record makes five times by citation and does not reduce to a grep. A
check that instead flags every textual repetition would be noise (most of this repository's
five instances were textually dissimilar), and a check that requires byte-identical wording
would have caught none of them.

## Consequences

None. This record adds a name and a corpus for a defect class this repository has already
paid to find and fix five times; it changes no code and reserves no additional territory. A
later item that builds a check against this definition points at it rather than re-deriving
the shape from the five commits again.

## What Holds It

Nothing mechanical, today. The five instances above were each closed by ordinary review — a
person reading two artifacts and noticing they answered the same question differently — and
that is what still holds the boundary this record names, exactly as `OD-GATE-013`'s equivalent
section says for its narrower class. A sixth instance is found the same way until a check is
built.

## What This Record Does Not Decide

It does not build the check. Where such a check would live — a new rule in `nomos-rules`
beside `Check_Completeness_Mirrors`, a standalone binary, or something else — and how it would
be validated to still find all five instances without flagging `PackageKind`, is a later
item's territory, not this one's. It does not re-open or re-decide any of the five closed
items cited as corpus; their own records and tests, where they have them, stand as written.
