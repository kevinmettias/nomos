---
id: OD-AGENT-004
type: decision
title: A restated fact goes stale exactly where nothing checks it, so the rule is route-what-is-checked-elsewhere rather than write-less-prose
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - agent
  - documentation
  - architecture
  - staleness
relations:
  - target: OD-AGENT-001
    type: relates-to
  - target: OD-GATE-011
    type: relates-to
  - target: OD-HOST-001
    type: relates-to
  - target: OD-PLATFORM-002
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
---

# A restated fact goes stale exactly where nothing checks it, so the rule is route-what-is-checked-elsewhere rather than write-less-prose

## Question

An eighth-round external architecture review named documentation volume as a form of bloat:
`Cargo.toml` carrying architectural essays per member, `README.md` carrying crate and zone
tables, module roots carrying chronological narratives of which item added which increment,
and the same architectural fact therefore standing in a governing record, a README table, a
manifest comment, a module doc and a commit message at once. It recommended compacting the
prose and generating the dependency and package tables from one machine-readable declaration.

`OD-AGENT-001` already decided this shape for agent instruction files: an unchecked copy of a
checked file is a defect, and an instruction file routes to authority rather than restating it.
Whether that principle extends past `AGENTS.md` to manifest comments and module-root prose was
undecided.

## What Was Measured

**The volumes the review cites are accurate.** `Cargo.toml` is 30,191 bytes, of which 276 of
502 lines are comments -- 55 percent prose. `README.md` is 41,559 bytes. Module roots do carry
per-item narratives.

**But volume did not predict staleness, and a natural experiment settled it.** On 2026-09-05,
`OD-PLATFORM-002` gave `nomos_platform::FileSystem` a `Read_Directory` operation
(`P41-PLATFORM-DIRECTORY-ENUMERATION-3`, `093a0e4e`). The next day, the claim that no such
operation exists was still standing at **fifteen sites across eight crates and one test crate**,
plus `OD-HOST-001`, `OD-HOST-002` and `OD-LEDGER-025`.

**Every one of those fifteen was in a module doc comment. None was in `README.md`. None was in
`Cargo.toml`.**

That is the finding. The two artifacts the review names as bloated are the two that stayed
correct, and they stayed correct for a reason that is already in this repository:
`tests/contract/tests/boundaries/readme.rs` and `manifest_bands.rs` assert the README's tables
and the manifest's band assignments against the real workspace, both directions. The prose that
went stale was the prose nothing checks.

**The failure had a propagating form, which volume also does not explain.**
`nomos-surface-provenance::discovery` did not restate the claim independently -- it quoted the
sentence out of another module as its own stated authority. A false claim spread by citation.
Two further sites enumerated the port's operations by name and were wrong about the port's
shape rather than about one operation, having also never learned about `Remove_File`.

**The cost was paid outside the repository.** An external reviewer, reasoning carefully from
committed documentation, read one of the fifteen and recommended against adding directory
enumeration to a port that had gained it the day before. `OD-GATE-011` names this defect class;
this is the first instance where its cost is legible as a wrong answer given to a real reader
rather than as maintenance burden.

## The Decision

**`OD-AGENT-001`'s rule extends, and it extends on the checked/unchecked axis rather than the
volume axis: prose must route to an authority for any fact that a mechanical check already
holds, and may state a fact freely where it is the authority.**

Concretely, in a module doc comment:

- **A fact another artifact checks is routed to, never restated.** Band membership, crate
  ownership, a port's operation set, a record's decision. The doc names the authority; it does
  not reproduce its content.
- **A fact this module is the authority for is stated fully, and length is not a defect.** Why
  this type has these variants, what a function refuses and why, what a decision cost here.
  `nomos-cli::check::sources::Walked_Sources` is the worked example: it holds the real reason a
  recursive walk is not the port's one-level primitive, and it was the one site in the whole
  population that was correct, because it was the authority rather than a copy of one.
- **Quoting another module's prose as your own stated authority is refused outright.** It
  creates a citation edge with no mechanical backing, which is how a false claim propagated
  here. Route to the record, or state your own reason.

**The review's two specific recommendations are declined, on this record's own evidence.**
Compacting `Cargo.toml` and generating the README tables would spend effort on the two
artifacts that demonstrably did not fail, and generating the README tables would remove the
contract tests' subject -- the tables are checked *because* they are authored, and a generated
table asserts nothing about a hand-maintained fourth copy the way `manifest_bands.rs` does
today.

**No new gate step is added by this record.** A rule that detects a restated-rather-than-routed
fact would need to know which facts have mechanical authorities elsewhere, which is not
derivable from text. This record governs how the prose is written; it does not claim a checker
exists for it.

## What Would Decide It Differently

- **A second staleness population found in `README.md` or `Cargo.toml`.** Would falsify this
  record's central measurement and reopen the volume argument.
- **A mechanical way to detect a restated fact.** Would turn this from a writing rule into a
  checkable one, and is the increment worth wanting.
- **A checked artifact going stale anyway**, which would mean the checks are narrower than the
  facts they appear to cover. *(Fired. See the amendment below.)*

## Amendment, Version 2: The Third Trigger Fired, On A Printed Help Text

The third trigger above — a checked artifact going stale anyway, because the checks are
narrower than the facts they appear to cover — fired on 2026-09-13, on an artifact version
1's measured population did not contain.

`crates/host/nomos-cli/src/check/parsing.rs`'s `USAGE`, the text `nomos check` prints,
carried two declared lists inside one string constant:

| the list | its authority | compared against it |
|---|---|---|
| the exit codes | `check::ExitCode::All()` | yes — `Test_All_Should_Match_The_Documented_Exit_Codes`, both directions, refusing to pass on an empty parse |
| the rules | `nomos_rules::DESCRIPTORS` | nothing |

The sentence directly above the second list promised the command "runs every rule over the
tree". The list named one: `completeness-mirror`. `DESCRIPTORS` held **70 entries when the
defect was filed at `3c728d31`, and 71 four commits later when it was corrected** — so the
list was wrong by 69, and then by 70, having gone wronger while the item to fix it sat on
the board. That is the property a hand-written enumeration has and a route does not.

Nothing in this repository could have caught it. `Declared_Universes`
(`tests/contract/src/universes.rs`) recognises a universe as `pub const NAME: &[...]` or as
`Type::All()`, and `USAGE` is `pub(super) const USAGE: &str` — so an enumeration written as
prose *inside* a string constant is structurally outside every completeness mechanism there
is. `DESCRIPTORS` itself was never in doubt: it is `Standing::Mirrored` in
`tests/contract/tests/completeness_universes/table.rs`, held by
`Test_Every_Composed_Rule_Should_Have_A_Descriptor`. The copy of it in a sentence was.

### Why this confirms version 1 rather than reopening it

This does not fall on the volume axis. `USAGE` is thirteen lines. It is a *checked*
artifact, and the half that went stale is the half nothing compared — which is version 1's
own finding holding at a finer grain than version 1 measured it: **the unit that is checked
or not is the fact, not the file.** A file can be half-checked and read as checked, and the
checked half is what makes the unchecked half look safe.

### The rule extends to the text a command prints

Version 1's population was module doc comments, `README.md` and `Cargo.toml`. Help text is a
fourth artifact and takes the same route-what-is-checked-elsewhere rule, stated as a
condition a later author applies rather than a judgement they remake:

> A help text may enumerate a compiled vocabulary only where a test compares that
> enumeration against the vocabulary's own authority, in both directions, and refuses to
> pass on an empty parse. Where no such test exists, the help text routes to the verb that
> prints the vocabulary rather than naming any of it.

The empty-parse guard belongs to the condition rather than to one implementation of it. A
test that locates its list by splitting prose on a heading compares nothing at all once that
heading is renamed, and reports the same as a test that compared everything — the defect
this record is about, wearing the remedy's costume.

**`check` routes.** Its `USAGE` now says `nomos gate plan --root <path>` names every rule,
which that verb already prints out of `DESCRIPTORS` itself.
`Test_The_Usage_Text_Should_Route_To_The_Rule_Set_Rather_Than_Name_Any_Of_It` asserts both
halves — that the route is there, and that no identifier in `DESCRIPTORS` occurs anywhere in
`USAGE` — so pasting the list back in is refused rather than merely discouraged.
Hand-pasting seventy-one names was rejected for the obvious reason: it recreates the same
unguarded copy one composed rule later.

### What is compared after this amendment, and what is not

Eight groups on this binary print an exit-code list, each with its own `ExitCode` enum:
`agent`, `check`, `correct`, `gate`, `request`, `spec`, `work` and `workflow`. **Two
compared it against the enum; six now do**, by
`Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With`. The six were
checked variant by variant against their own prose before the tests were written and all six
were already correct — they were unguarded rather than wrong, which is worth saying rather
than inflating. Each new test was confirmed able to fail by injecting a wrong code into that
group's own prose and observing red, and the routing test by pasting a real rule identifier
back into `USAGE`; none of the seven was assumed to work.

The same help text restates four further compiled vocabularies that **nothing compares
against anything**, measured across every `nomos-cli` test that reads a usage text:

- `work list`'s nine states, against `nomos_ledger::item::State`;
- `work add`'s five kinds and two origins, against `nomos_ledger::item::Kind` and
  `nomos_ledger::item::Origin`;
- `agent`'s six `--effort` spellings, against `nomos_model_package::EffortLevel` — all six
  are exercised against the *parser*, and nothing compares them to the printed list;
- `request`'s three submission kinds and two states.

A fifth, `spec`'s nine command names, is compared — but against a hand-written array in the
test file rather than against a compiled authority, which is a weaker claim than the
condition above asks for.

Those five were named here so the condition read as not-yet-met rather than as satisfied.
**All five now meet it**, closed by
`P97-FIVE-PRINTED-VOCABULARIES-HAVE-NOTHING-COMPARING-THEM-AND-ONE-IS-ALREADY-SHORT`:
`Test_The_Listed_States_Should_Be_Every_Word_The_Listing_Can_Print`,
`Test_The_Listed_Add_Kinds_Should_Be_Every_Kind_An_Item_Can_Declare`,
`Test_The_Listed_Add_Origins_Should_Be_Every_Origin_An_Item_Can_Declare`,
`Test_The_Listed_Submission_Kinds_Should_Be_Every_Kind_A_Submission_Can_Declare`,
`Test_The_Listed_Submission_States_Should_Be_Every_State_A_Submission_Can_Be_In`,
`Test_The_Named_Effort_Levels_Should_Be_Every_Level_The_Command_Line_Takes`, and
`Test_Usage_Text_Should_Name_Every_Command`, which stopped comparing against a hand-written
array and now drives every name through the parser.

**One of the five was wrong, not merely unguarded, and only measuring found it.** `work list`
printed nine states and the listing produces ten. `lapsed` was missing, and `Listed_As` filters
by comparing the requested word against that same function's output — so
`nomos work list --state lapsed` worked and the help text did not say so. That is the second
time in this record's history that writing the guard found the artifact already wrong, the
first being `check`'s rule list, and it is the argument for building the comparison rather than
reading the list and judging it correct.

### Two further copies, found by the same measurement

The five above were measured across every `nomos-cli` **test** that reads a usage text, which
is the population the condition is about. Closing them surfaced two copies of the same
vocabularies that population could not see, and neither meets the condition:

- **`README.md` line 141 prints the `--state` list**, and is short `lapsed` exactly as the help
  text was. This one bears on version 1's central measurement. That measurement found
  `README.md` stayed correct while module prose went stale, because
  `tests/contract/tests/boundaries/readme.rs` asserts its **tables** against the real
  workspace. A pipe-joined vocabulary inside a fenced command line is not a table, nothing
  checks it, and it went stale. So the README is not correct *because it is the README*; it is
  correct where it is checked, which is this record's rule holding rather than an exception to
  it.
- **`crates/host/nomos-cli/src/spec/parsing.rs` holds a third copy of the nine command names**,
  as nine `contains` assertions. It is weaker than the array it sits beside in two ways: one
  direction, and substring rather than equality — renaming the verb to `recordx` leaves it
  green, which is how it was found.

Neither is corrected here; each is its own item. They are recorded so this section keeps saying
what is true rather than what was true when the debt was first counted.

### What would decide this differently

Version 1's three triggers stand, the third now having fired once. A fourth is added: **a
group whose exit-code comparison passes while the text a user is shown is wrong**, which
would mean the comparison is reading something other than that text.

## Status

Accepted. Decided on a natural experiment rather than on a principle: one port change on
2026-09-05, fifteen stale restatements the next day, all fifteen in unchecked module prose and
none in the two artifacts the review called bloated. The remedy is routing what is checked
elsewhere, not writing less.

Amended to version 2 by
`P96-THE-CHECK-VERBS-HELP-NAMES-ONE-RULE-OF-SEVENTY-AND-CLAIMS-IT-RUNS-EVERY-ONE`, on this
record's own third revisit trigger firing: `nomos check`'s help text held two declared lists,
one mirrored and one wrong by 69 of 70 rules, in a string constant no completeness mechanism
can see into. The rule extends to the text a command prints, and is stated there as a
condition — enumerate only where a test compares, otherwise route — with the five printed
vocabularies that do not yet meet it named rather than left implied.

Those five were closed by
`P97-FIVE-PRINTED-VOCABULARIES-HAVE-NOTHING-COMPARING-THEM-AND-ONE-IS-ALREADY-SHORT`, which
found one of them already wrong rather than merely unguarded and two further copies the
population it measured could not see. No version bump: the decision is untouched and the
amendment's own coverage section is what moved, which is that section saying what is true
rather than what was true when the debt was counted.
