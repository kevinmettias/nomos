---
id: OD-GATE-032
type: decision
title: A declared tolerance can name only a finding whose subject is its file
status: accepted
version: 1
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
`SubjectId::From_Digest(Content_Digest(qualified.as_bytes()))`, measured at `35464ec8` and
unchanged at `2bf7df52`, the commit that boarded this item. Thirteen of the sixteen sit under
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

6. **The naming family's derivation becomes a named one, and `OD-ANALYSIS-011` is amended to
   carry it.** The boundary guard permits three derivations today and this family satisfies
   none, which is a defect in the record set rather than in the rules. The fourth is the digest
   of the address the rule declares, and it is added to the permitted set with the guard
   extended to exercise it — a guard whose fixture cannot produce the shape it exists for has
   already been written once in this repository and is not worth writing again.

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
