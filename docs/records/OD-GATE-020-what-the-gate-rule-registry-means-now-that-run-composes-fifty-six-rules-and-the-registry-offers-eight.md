---
id: OD-GATE-020
type: decision
title: What nomos-gate-orchestration's RuleRegistry means now that Run composes 56 rules and the registry offers 8
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - gate
  - rules
  - registry
  - duplication
relations:
  - target: OD-GATE-011
    type: relates-to
  - target: OD-RULES-004
    type: relates-to
  - target: OD-RULES-005
    type: relates-to
  - target: OD-RULES-006
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
  - target: OD-RULES-003
    type: relates-to
  - target: OD-RULES-008
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: OD-CAPABILITY-010
    type: relates-to
---

# What nomos-gate-orchestration's RuleRegistry means now that Run composes 56 rules and the registry offers 8

## Question

`crates/orchestration/nomos-gate-orchestration/src/composition.rs`'s own module doc states the
registry's charter without qualification: "Registers this workspace's eight shipped rules and
hands back the registry... The eight are exactly what `nomos-check-orchestration::run::Run`
calls." A prior cluster of corrections (tracked on the work ledger under the id prefix
`OD-GATE-019-REGISTRY-COHERENCE-*`, never itself registered as a governing record) closed this
same gap twice already, at four-versus-eight and again at five-versus-eight, each time citing
the same principle stated in the file's own doc: "Honest requires whole... a plan smaller than
the run it describes is worse than no plan."

Measured directly against this workspace at the current revision: `nomos_check_orchestration::
run_context::RULE_COUNT` is `56` (`run_context.rs:61`), and the private `rules` array it bounds
composes exactly that many `Check_*` functions into `Run` (`run_context.rs:526-579`).
`nomos_gate_orchestration::composition::Registered()` still makes exactly eight
`registry.Offer(RuleOffer {...})` calls (`composition.rs:56-64`), guarded by a test named
`Test_Registered_Should_Offer_All_Eight_Shipped_Rules`. The doc's claim of parity is false by
48, the same shape the prior two corrections closed, now an order of magnitude larger.

The open question is not whether to correct the count — the prior corrections already answer
that by precedent — but what `Registered()` *is*, structurally, now that the population has
grown past the size that made "add the next `Offer_*` function by hand" a proportionate
response, and whether the mechanism the eight already use (a `RuleOffer{contract_record,
contract_record_version}` pair, with a `contract_record_version: 0` sentinel for a rule whose
authority is prose rather than a versioned decision) still generalizes at this scale.

## What Was Measured

**The pair does not meet `OD-GATE-011`'s own legitimate-exception test.** `OD-GATE-011` names
"two independent encodings of one decision" as a defect class and states the narrow condition
under which a second encoding is not that defect: both sides derive independently from one
named external authority outside either artifact, the reason is stated in a doc comment or
record at the site, and a test pins the two sides together rather than a check's silent
exclusion. `Registered()`'s list does not derive from `Run`'s list, or from any third, named
authority both sides read — it is a second, independently hand-typed enumeration that is
*supposed* to equal the first, which is exactly the shape the class describes ("each is
independently readable as the authoritative answer to the same question"). `Test_Registered_
Should_Offer_All_Eight_Shipped_Rules` pins `Registered()` against a literal, hand-written
expectation of eight named ids — the same "a count agrees with itself" risk `composition.rs`'s
own doc already names and tries to avoid by asserting rule ids rather than a bare count — but a
hand-written expectation checked against a hand-written registration is still two hand-written
artifacts, not one authority and one derived projection. The pair fails the test's first
condition regardless of how carefully the second registration is worded.

This is a narrower finding than "this is one of `OD-GATE-011`'s five closed instances,"
because that record's own definition also requires "neither artifact says the other exists" —
and `composition.rs` names `Run` explicitly and claims parity with it. This is not a hidden
duplicate nobody thought to compare; it is a known, declared pair whose own stated invariant
has already gone false twice, silently, between the moments anyone checked it by hand.

**`nomos-check-orchestration` exposes no authority `Registered()` could derive from even if it
tried.** `run_context::RULE_COUNT` and the private `rules` array are not `pub`; `lib.rs`'s
export list (`Run`, `RunContext`, `CheckCommand`, `Claim`/`Claim_Of`/`CheckOutcome`/`Examined`,
and its own separate `composition::Registered`) carries no accessor for the composed rule-id
set. A shared-derivation fix — the shape that *would* satisfy `OD-GATE-011`'s exception test —
does not exist to switch to today; it would need a new, deliberate export, not a rewire of
something already there.

**The `contract_record_version: 0` sentinel already generalizes past its first instance.**
`Offer_Naming_Convention`'s own doc names `contract_record: "README.md", contract_record_
version: 0` as a marker for "no versioned record", not "version zero of one," and states
plainly that `OD-RULES-005` and `OD-RULES-006` already declined to extend `RuleOffer`'s shape
without a second real case forcing it, calling a record-less rule needing its own
representation "that second case, left for whoever next needs more than a sentinel here to say
so." Measured now: of the eight offered rules, seven cite a real, versioned record through a
`*_CONTRACT_RECORD` constant (`CONTRACT_RECORD` for `D-134`, `DEPENDENCY_CONTRACT_RECORD` for
`OD-RULES-003` — shared by both `DEPENDENCY_DIRECTION` and `DEPENDENCY_COMPLETENESS`,
`LINT_CONTRACT_RECORD` and `DEPENDENCY_POLICY_CONTRACT_RECORD` for `OD-RULES-010`,
`CROSS_LANGUAGE_CONTRACT_RECORD` for `OD-CAPABILITY-010`, `UNREAD_REACHES_FINDING_CONTRACT_
RECORD` for `OD-RULES-008`) — six constants across seven offers — and exactly one,
`NAMING_CONVENTION`, uses the sentinel. Of the 48 rules `Run` composes and `Registered()` does
not yet offer, a direct search (`grep -rl` for an `OD-RULES-`/`OD-CAPABILITY-`/`D-`-shaped
citation across `crates/rules/nomos-rules/src/checks*`) finds roughly thirty module files
naming some record in prose without yet carrying a formal constant — more of the same shape
`Check_Dependency_Policy` and `Check_Lint_Diagnostics` were in before their own corrections gave
them constants — and the remainder cite no record at all, the `Check_Naming_Convention` shape.
Both populations exist at real, multi-instance scale now, not as a hypothetical this record
would be deciding in the abstract.

**`OD-RULES-009` leaned on this pair's size as disconfirming evidence, on a premise this
measurement contradicts.** That record's fifth amendment states "the list `Run` and
`gate-orchestration`'s `RuleRegistry` maintain has not grown by thirty-one entries; it has
grown by one." Measured today, `Run`'s list grew from eight (that amendment's own count) to
fifty-six, and `Registered()`'s stayed at eight — the two diverged by forty-eight, not by one.
This record does not re-decide `OD-RULES-009`'s question; `P33-RULES-009-SIXTH-ROUND-RECHECK`
is that item, and this measurement is data for it, not a substitute for it.

## Decision

**`Registered()`'s charter is the honest whole, unchanged from what its own doc already
claims — parity with `Run` — and the two-field `RuleOffer{contract_record, contract_record_
version}` shape need not grow to sustain it at this scale.** The `contract_record_version: 0`
sentinel `Offer_Naming_Convention` introduced provisionally is accepted as the general pattern
for a record-less rule, not a one-off awaiting its own type: a `String` already carries a
versioned record id (`"OD-RULES-003"`) or a prose authority (`"README.md"`, or whatever a
rule's own module doc already names) without ambiguity, and `contract_record_version: 0`
already means "no version" unambiguously as a sentinel rather than a real version zero. Forty-
eight more instances of the same two shapes `Offer_Naming_Convention` already covers is exactly
the second-and-onward case `OD-RULES-005`/`OD-RULES-006` asked for before extending `RuleOffer`,
and the shape those forty-eight instances take is the same shape the ninth rule already proved
out, not a new one. No new field, enum, or type is added to `RuleOffer` by this decision.

**Closing the gap by hand, rule by rule, the way the prior two corrections did, is not
declined — but it does not, by itself, bring the pair into `OD-GATE-011`'s legitimate-exception
shape, and a later item should not claim that it does.** A correction that adds forty-eight more
`Offer_*` functions and raises a hand-written test's expected count from eight to fifty-six
would make `Registered()` honestly whole again, exactly as the prior two corrections did — but
it reproduces the same not-yet-a-legitimate-exception pair at a larger size, one more manual
synchronization a person has to remember to repeat at rule fifty-seven. That is an acceptable
near-term shape, consistent with this workspace's standing discipline of composing by hand until
a real trigger forces otherwise (`OD-HOST-004`), and is explicitly not foreclosed here.

**The fix that would satisfy `OD-GATE-011`'s own exception test does not exist to adopt today,
and building it is named rather than done by this record.** `nomos-check-orchestration` would
need to export its composed rule-id set — a `pub fn` beside `Run`, not a rewire of
`RULE_COUNT` or the private `rules` array themselves — for `nomos-gate-orchestration` to derive
`Registered()`'s test (or `Registered()` itself) from, turning "two hand-typed lists, checked
against each other by eye at correction time" into "one hand-typed list and one comparison
against it, checked by `cargo test` every time." Whether that export is worth building now, or
waits for a further trigger the way `OD-RULES-009`'s planner does, is left to the item that
proposes it, informed by this record rather than decided here.

## What This Record Does Not Decide

It does not add any of the forty-eight offers, does not write the export `nomos-check-
orchestration` would need for a shared-derivation fix, and does not choose between "close the
gap by hand again" and "build the export first" — both remain open, real options for a
follow-on item, and this record's job was to establish that the second option is what would
actually satisfy `OD-GATE-011`, not merely another instance of the first at a bigger number. It
does not re-decide `OD-RULES-009`; `P33-RULES-009-SIXTH-ROUND-RECHECK` carries that question,
informed by the divergence this record measured. It does not audit which of the forty-eight
rules cite a real record versus need the sentinel individually — that per-rule classification is
a correction item's own territory, sized by how much of it a single item can honestly close.

## Status

Accepted. `Registered()`'s charter stays the honest whole; `RuleOffer`'s two-field shape,
including the `contract_record_version: 0` sentinel, is confirmed sufficient at real,
multi-instance scale and gains no new field. The gap itself — eight offered against
fifty-six composed — remains open, tracked as future correction-item territory this record
names but does not build. Revisit if a shared-derivation export is built and this record's
"what would satisfy `OD-GATE-011`" analysis needs checking against it, or if the record-less
population turns out not to fit the two shapes measured here once audited rule by rule.

The revisit condition named above has since fired, twice and in that order.

`P35-GATE-020-COMPOSED-RULES-EXPORT` built the export this record named:
`nomos_check_orchestration::Composed_Rules`, a `pub fn` beside `Run` reading the same array
literal `Run` executes, exactly the shape described here and not a rewire of `RULE_COUNT` or
the private table. That turned "two hand-typed lists, checked against each other by eye at
correction time" into "one hand-typed list and one comparison against it", which is what this
record predicted it would buy.

`P52-COMPOSED-RULES-BECOME-DECLARATIONS-3` then went further than this record anticipated, and
the difference is the part the analysis above did not reach. This record identified the
remaining obstacle correctly -- "a rule's contract citation is knowledge no export carries", so
`OFFERINGS` had to stay authored even once the comparison existed. What it did not consider is
that the citation could move onto the descriptor already carrying the rule's identity and its
capability requirements. It did: `nomos_rules::RuleDescriptor` gained `contract_record` and
`contract_record_version`, `OFFERINGS` was deleted, and `Registered` derives from
`nomos_rules::DESCRIPTORS`. `composition.rs` fell from 323 lines to 177, and all seventy-six of
its rule-identifier imports became unused, which is what a table being removed rather than
relocated looks like from the outside.

**The analysis held up, and the conclusion about scale is the thing that aged.** What this
record said would satisfy `OD-GATE-011` -- one authority named outside both artifacts, each
side deriving from it rather than restating it -- is what was built, and the two parity tests
now compare a derivation against its source. What it also said was that closing the gap by hand
was "an acceptable near-term shape... one more manual synchronization a person has to remember
to repeat at rule fifty-seven". That stopped being hypothetical: it was repeated by hand up to
fifty-six, and at fifty-seven it was not remembered. `P47-RULES-ORPHAN-MODULES` composed a rule
without its row, `nomos gate plan` described a smaller gate than `nomos gate run` performed,
and the parity test this record's own export made possible is what caught it within hours. The
near-term shape was acceptable exactly as long as this record said it would be, and the number
it named as the limit is the number at which it failed.

**Nothing above is withdrawn.** `RuleOffer`'s two-field shape gained no field, the
`contract_record_version: 0` sentinel remains the general pattern for a record-less rule --
`RuleDescriptor::Cites_A_Versioned_Record` is now where it is read, so no caller compares
against zero -- and the per-rule classification this record left to a correction item came out
as predicted: seven rules citing a versioned governing record, one citing `README.md`, every
other rule citing the ported standard, and no third shape found.
