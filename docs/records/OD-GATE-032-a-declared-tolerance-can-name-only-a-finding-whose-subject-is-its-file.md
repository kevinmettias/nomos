---
id: OD-GATE-032
type: decision
title: A declared tolerance can name only a finding whose subject is its file
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - gate
  - policy
  - addressing
relations:
  - target: OD-GATE-024
    type: relates-to
  - target: OD-GATE-030
    type: relates-to
  - target: OD-ANALYSIS-011
    type: affects
  - target: OD-MODEL-001
    type: relates-to
---

# A declared tolerance can name only a finding whose subject is its file

## Question

`P109` gave the four gate policies a declared source in `nomos-gate.json`, and a later P109
item made the exceeded-baseline report name the scope its author actually wrote. Both reach a
finding through `nomos_model::Subject_Of_Path`: a declared entry names a **path**, and the run
computes the identity that path folds to. A suppression reaches its finding the same way.

That addresses a finding whose subject *is* its file. It does not address one whose subject is
something inside the file, and the failure is silent in both directions: the entry matches
nothing, the finding is not tolerated, and until recently nothing said so.

`gate_policy_file.rs`'s own module doc states this limit under "What a path-authored entry does
not reach", names the shape of an answer — sub-item findings carrying an addressable name a
person can write — and then defers it explicitly: "That is a question about `Finding` rather
than about this file, and it belongs to whichever item takes it up." This record is that
decision.

## What Was Measured

**The population is sixteen construction sites in fourteen files, and that is the unit the
item's own figure disagrees about.** Every site is
`SubjectId::From_Digest(Content_Digest(qualified.as_bytes()))`, measured at `5e08ea73` and
unchanged at `6b9a4102`, the commit that boarded this item. Thirteen of the sixteen sit under
`checks/naming/`, across eleven files; three sit outside it, in `checks/function_shape.rs`,
`checks/guarantee_exerciser.rs` and `checks/mirror/verdict.rs`. The item records "fourteen rule
construction sites", and fourteen is exactly the *file* count, so the two figures differ by the
unit counted rather than by a mistake in either. This record states its own count and its own
unit, because a number without a unit is the thing that made the two look like a disagreement.
A later reader re-measuring will get a different number again — this is a census of a moving
tree — and what is stable is the shape rather than the count.

**`qualified` is not one string, and no author can predict it.** Three files from one family:

- `checks/naming/violations.rs` builds `format!("{path}::{}", item.qualified_name)`;
- `checks/naming/abbreviations.rs` builds `format!("{path}::{}::{name}", item.qualified_name)`;
- `checks/naming/file_names.rs` builds `format!("{path}::{}", item.qualified_name)`.

Each is assembled at the site, over the raw bytes, with no shared constructor and no
normalization. So the string an entry would have to name is private to the rule that built it —
a person cannot write it from the file they are looking at, and a policy that guessed would be
wrong in a way the run could not detect.

**`subject_name` is the bare name, so it is not the composite and cannot address it.**
`item.qualified_name.clone()` in `violations.rs`, `name.to_owned()` in `abbreviations.rs`,
`type_name.to_owned()` in `file_names.rs`. `subject` is a digest over the composite while
`subject_name` is one member of it, so the two are related by containment rather than by
equality. This is the fact `OD-GATE-024` did not have when it first decided: its remedy compared
an authored string against `subject_name`, and over this family that comparison would either
fail to match at all or — were the composite ever collapsed to the member — match every symbol
sharing a display name at once. That is broadening, not addressing.

**The naming family's derivation is a fourth derivation `OD-ANALYSIS-011` does not name.**
`tests/contract/tests/boundaries/findings.rs` permits exactly three: `subject` is
`Content_Digest(subject_name)`, or `Subject_Of_Path(location)`, or `Subject_Of_Path` of the file
part of a `path:line` location. A naming-family finding satisfies none of them. The guard does
not catch it because its fixture is two fact-free rules and never exercises this family, and the
guard's own doc anticipates exactly this: "That is not automatically wrong -- it is a fourth
derivation nobody has decided on, and `OD-ANALYSIS-011` is where the decision would have to be
amended before the rule lands."

**This paragraph is what was true on 2026-09-15 and is no longer true of either file.** The
amendment below says which commits changed it and why the counts above are left as they were
measured. The quotation in particular no longer resolves at the file it cites.

**Matching is digest equality in both policies, so this is not a baseline-only gap.**
`Suppression::Is_Applicable_To` and `BaselineDebt::Is_Applicable_To` are both `rule` and
`subject` compared for equality — `OD-GATE-024` cites these same two implementations under
their former name, `Matches`. Nothing in either can see a name, and the suppression one says so
in its own doc: matching "invents no addressing scheme of its own". A later reader who fixes one
of these and believes the other followed has fixed half of this. The naming family is also what
a repository adopting on existing code reaches for first, so the configuration language is
weakest at exactly the point adoption depends on it.

**An entry that matches nothing is now reported rather than silent.** `GateRunResult` carries
`unmatched_policy`, and its own doc records that `OD-GATE-024` was filed because that silence
was the whole of the defect. That clause is independent of how addressing is spelled, and it is
what makes the third acceptance case below observable rather than merely intended.

## Decision

**A declared entry addresses its finding through a tagged selector with exactly one addressing
mode, and a rule that subjects its findings to something inside the file declares an address a
person can write.**

The measurement exposes two candidate answers, and they are not equally good.

The first is to give these findings an addressable name a person can write, and have the
selector resolve that name to the identity the finding already carries. This is the answer the
module doc named, and it is the one taken here.

The second is to let a declared entry name a subject some other way — a raw `SubjectId` digest,
or the `subject_name` string that `OD-GATE-024`'s first decision reached for. The module doc
already refuses the first, because a file naming a raw digest is the unauthorable identity that
module exists to compute rather than accept; and the second is already retracted, because
`subject_name` is not reliably the preimage of the subject beside it. Neither is reopened here.

1. **The selector is one tagged value, not two optional fields.** A declared subject is either a
   path or a qualified name, and the two cannot both be present. Two independent optional fields
   would admit a declaration that means two things at once, with a precedence nobody decided and
   only `deny_unknown_fields` standing between it and a run. One tag makes the invalid
   combination unrepresentable rather than refused, which is the stronger of the two answers.

2. **The path selector keeps today's semantics exactly.** It resolves through `Subject_Of_Path`,
   byte for byte as it does now, including the folding that makes two spellings one subject.
   Every entry that works today keeps working, and this record changes nothing about it.

3. **The qualified-name selector names the address a rule declares, and a rule that declares
   none cannot be addressed this way.** The naming sites stop assembling a composite privately
   and declare the address on the finding, so the string a policy entry writes is a string the
   run can print back. That is what makes the entry authorable rather than guessable, and it is
   the change to `Finding` that the module doc named.

4. **Matching stays on `subject`.** The selector resolves to a `SubjectId` and the comparison is
   unchanged, so the run still matches on identity while the address remains authoring and
   display material. This preserves the separation the baseline work has been keeping —
   canonical identity is not presentation identity — and it is why a name-addressed entry cannot
   silently broaden: it resolves to one composite digest, never to a display name that several
   symbols share.

5. **A selector that resolves to nothing is refused rather than reported unmatched.** The
   `unmatched_policy` line already reports an entry that reached no finding. A selector naming
   an address no rule in the run can produce is a different event and must read differently: it
   is a declaration the run could not interpret, and reporting it as "matched nothing" would
   tell an author their symbol was fixed when in fact they misspelled it.

6. **The naming family's derivation becomes a named one, and `OD-ANALYSIS-011` is amended to carry
   it.** *Discharged -- see the amendment below.* The boundary guard permits three derivations
   today and this family satisfies none, which is a defect in the record set rather than in the
   rules. The fourth is the digest of the address the rule declares, and it is added to the
   permitted set with the guard extended to exercise it — a guard whose fixture cannot produce the
   shape it exists for has already been written once in this repository and is not worth writing
   again.

## What This Does Not Decide

**It does not decide baseline capacity.** `OD-GATE-030` owns how much a tolerance may cover, and
it is untouched by which findings a tolerance can name. The two questions meet at the same
declared file and are otherwise independent.

**It does not re-open `OD-GATE-024`'s retracted decisions.** Comparing an authored string
against `subject_name` stays retracted, for the reason that record measured. The selector here
resolves a name to a digest rather than comparing strings, which is the substantive difference
between the two designs and the reason this one survives the fact that killed the other.

**It does not change `Normalize_Path`, `Subject_Of_Path`, or how territory and fact identity are
computed.** Those answer whether two paths are one file, which is `OD-MODEL-001`'s question and
not this one.

**It does not put a digest in the declared file.** A selector names an address a person can read
and the run computes the identity, which is the property the original design exists to keep.

**It does not implement any of this.** The selector type, the address on `Finding`, the sixteen
construction sites, the two matching implementations, the refusal path and the
`OD-ANALYSIS-011` amendment are a follow-up item's own territory, none of which this record
holds.

## Amendment: Decision 6 Is Discharged, And This Record's Account Of The Guard Is Dated

Two commits satisfied things this record decided and neither came back to it, so for eleven days
its measurement read as current and its Decision read as outstanding. Re-measured at `cd615f61` on
2026-09-26:

| what this record says | where | what is true now |
|---|---|---|
| the derivation is one `OD-ANALYSIS-011` does not name | above | that record is version 3 and its second amendment names it |
| `findings.rs` permits exactly three | above | it permits four, and its own doc says "one of the four derivations the record permits" |
| a naming-family finding satisfies none of them | above | it satisfies the first arm, `Names_Its_Address`, added for it |
| the guard permits three derivations today | Decision 6 | four: `Names_Its_Address`, the digest of `subject_name`, `Subject_Of_Path` of a location, and `Subject_Of_Path` of a `path:line` location's file part |

**What discharged it.** `ecbcfac0`, on 2026-09-21 and six days after this record landed, added
`Finding::address` -- the string a person would write to address a finding, published rather than
assembled privately, which is the gap this record's own measurement identified -- and added the
guard's fourth arm over it. `ed0394aa`, on 2026-09-26, amended `OD-ANALYSIS-011` to version 3 to
carry the derivation, which is what Decision 6 asked for in those words.

**The quotation is dated, not repaired.** This record quotes the guard's doc saying a subject
matching none of the arms is "a fourth derivation nobody has decided on". `ed0394aa` changed that
sentence to say *fifth*, for a reason that is the same reason this amendment exists: a fourth
derivation that has been decided cannot serve as the example of one that has not.
`tests/contract/tests/boundaries/findings.rs` line 39 now reads "a fifth derivation nobody has
decided on", so the quoted string resolves nowhere at the cited file. It is kept as the text as it
stood on 2026-09-15, because it is the evidence that the guard's own author had anticipated this
family before anybody decided it, and that is the observation that made Decision 6 cheap to reach.

**Why the counts above are left wrong.** They are dated evidence and they are the reason the
decision was taken. Editing "three" to "four" in the measurement would leave a record asserting
that the guard always permitted four and that Decision 6 therefore decided nothing -- which is
how a reader would then be unable to reconstruct why any of this work was done.
`OD-ANALYSIS-011` keeps its own version 1 sentence the same way, wrong, with a pointer to the
amendment that replaced it.

### What is still owed, which is most of this record

Only Decision 6 is discharged. Decisions 1 through 5 -- the tagged selector, its two addressing
modes, matching that stays on `subject`, and the refusal of a selector resolving to nothing -- are
**unbuilt**. Measured at `cd615f61`: `Suppression::subject` and `BaselineDebt::subject` are both a
plain `SubjectId`, no tagged subject selector type exists anywhere under `crates/`, and no
declared entry can name a qualified name yet. `Finding::address` makes such an entry *writable*
and nothing yet reads one.

So this record is not satisfied. What changed is that the one prerequisite it named for itself --
a derivation nobody had decided on -- is decided, and the record set no longer disagrees with the
guard about how many there are.

## Status

Accepted. A declared tolerance addresses its finding through a tagged selector with exactly one
addressing mode: a path, which keeps today's semantics exactly, or a qualified name, which names
an address the rule declares rather than a composite the rule assembles privately. Matching
stays on `subject`, so canonical identity and authoring address remain separate and a
name-addressed entry cannot broaden across symbols sharing a display name. A selector that
resolves to nothing is refused rather than reported as an entry that matched nothing. The naming
family's fourth derivation is named and `OD-ANALYSIS-011` is amended to carry it. `OD-GATE-030`'s
baseline capacity question and `OD-GATE-024`'s retracted decisions are both untouched, and no
mechanism is built here.

Amended once, and the amendment is about this record's own dating rather than about its decision.
The naming family's fourth derivation *is* named now: `ecbcfac0` built the mechanism and
`ed0394aa` amended `OD-ANALYSIS-011` to carry it, so the sentence above promising that amendment
is discharged and the measurement above describes the guard as it stood on 2026-09-15. Decisions 1
through 5 remain unbuilt -- `Suppression::subject` is still a plain `SubjectId` and no tagged
selector exists -- so this record is still owed almost all of itself.
