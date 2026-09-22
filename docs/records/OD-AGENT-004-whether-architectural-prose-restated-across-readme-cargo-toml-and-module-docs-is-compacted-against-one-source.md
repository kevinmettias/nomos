---
id: OD-AGENT-004
type: decision
title: A restated fact goes stale exactly where nothing checks it, so the rule is route-what-is-checked-elsewhere rather than write-less-prose
status: accepted
version: 3
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
  - target: OD-ROADMAP-005
    type: relates-to
  - target: OD-PACKAGE-004
    type: relates-to
  - target: OD-PACKAGE-017
    type: relates-to
  - target: OD-PROJECT-001
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

**Both are closed** by
`P97-TWO-FURTHER-COPIES-OF-A-PRINTED-VOCABULARY-STAND-OUTSIDE-THE-POPULATION-THAT-WAS-MEASURED`,
filed as one item because they are one record edit — a record is reserved once, and the ledger
refused the second item that tried to amend this one, which is the mechanism working.

The README's five enumerations split on reachability rather than on preference, which is the
condition's own shape:

- **`--kind` and `--origin` are compared**, by
  `Test_The_Readmes_Listed_Kinds_Should_Be_Every_Kind_An_Item_Can_Declare` and
  `Test_The_Readmes_Listed_Origins_Should_Be_Every_Origin_An_Item_Can_Declare` in
  `tests/contract/tests/boundaries/readme.rs`, against `nomos_ledger`'s own enums, which that
  suite already depends on. The third copy — the same five kinds restated in prose two
  paragraphs below the synopsis — was removed rather than given a second comparison.
- **The `--state` list and `request submit --kind` route.** Neither can be compared from
  outside `nomos-cli`: `Listing_Label` and `Refusal_Label` are private to a binary crate that
  exports no library, and `nomos-spec-model` is not a dependency of the contract suite.
  Restating either set in a test would be the second authority this record refuses, so the
  synopsis names the flag and points at the command that prints its values.

A removal is not self-sustaining the way a comparison is — nothing stops a later author pasting
a routed list back, and it would read as an improvement — so
`Test_The_Readme_Should_Not_Relist_A_Vocabulary_It_Routes_To` refuses exactly that.

The `spec/parsing.rs` copy became a floor rather than a list:
`Test_Usage_Text_Should_Name_Some_Command_At_All` asserts the usage text names some verb, which
is the one claim `spec::tests`' comparison cannot make, because that comparison reads the text
to find its own subjects and a text listing nothing would leave it passing having compared
nothing. Its name no longer collides with the stronger test's.

This section is kept current rather than left as the count taken when the debt was opened.

### What would decide this differently

Version 1's three triggers stand, the third now having fired once. A fourth is added: **a
group whose exit-code comparison passes while the text a user is shown is wrong**, which
would mean the comparison is reading something other than that text.

## Amendment, Version 3: The Prose Around The Checked Table Is Compared Where The Declaration Holds The Fact, And The `Owns` Column Is Watched By Nothing

`OD-ROADMAP-005` decision 6 supersedes version 1's decline of generated README tables and
compacted manifest commentary. What it authorizes is narrower than what the review asked for
and it says so in its own words: **a fact `nomos-architecture.json` holds stops being restated
in prose that nothing compares.** That is checking rather than generating, and the override is
explicit that the reason is this record's own measurement rather than a compromise struck
around it — "The measurement that decline rested on stays true and is the reason the increment
is a projection or a check rather than a rewrite."

So version 1's finding is not weakened here. It is the thing being built on, and it still
reads the same way: one port change on 2026-09-05, fifteen stale restatements the next day,
every one of them in unchecked module prose and none in `README.md` or `Cargo.toml`. The two
artifacts the review called bloated are the two that did not go stale, because something
compares them, and generating the tables would delete that comparison's subject. What was
missing was never a generator. It was the rest of the file: the sentences *around* the table
state facts the same declaration holds, and nothing read them.

### What was measured

2026-09-21 at `18db19c5`, against `README.md`, `nomos-architecture.json` and every assertion
under `tests/contract/tests/boundaries/` that opens either. The declaration holds five keys —
`components`, `members`, `permits`, `exceptions` and `authorities` — and each prose sentence
was put to `OD-PACKAGE-004` version 3's first two questions: name the declared source as a
path and a field, then read the comparison rather than its description.

| What the prose states | The declared fact, measured | Compared before | Compared now |
|---|---|---|---|
| crates are ordered into twelve named zones | `components`, 12 of them | yes, since earlier the same day | unchanged |
| the declaration names a set of zones, the zones each may depend on, and a short named list of the same-zone edges a real crate needs | `components`, `permits`, `exceptions`; 49 excepted edges, none of them crossing a component | no | yes |
| six crates, spelled in backticks in the paragraphs around the two tables | `members`; all six placed | no | yes |
| four rows below are marked `[repo tooling]`, and the four crates it names | the marks live in `Owns`; `Repo Tooling` holds three members and these are four | no | no, deliberately |
| the specification system sits beside the kernel rather than above it, so nothing in the product may name it | `permits`; `Specification` is reachable from `Host` and `Verification` and from nothing else | no | no, deliberately |
| `nomos-rules` declares `ZONES`, `Permits` and `SAME_ZONE_EDGES` | false — no such item exists anywhere under `crates/` | no | no; the correction is another item's |
| `tests/contract/tests/boundaries.rs` compares the tables against the `nomos-rules` declared `ZONES` | false in both halves — that path does not exist, and the comparison is `tests/contract/tests/boundaries/readme.rs` against `nomos-architecture.json` | no | no; an item is on the board for it |

### What is compared now

Two assertions were added to `tests/contract/tests/boundaries/readme.rs`, beside the
zone-count assertion that landed there earlier the same day. Both compare a sentence against
the declaration, both read the prose rather than a table row, and neither was believed until
it had been made to fail in a scratch worktree with the tree restored byte-identically
afterwards.

- **`Test_Every_Crate_The_Readmes_Prose_Names_Should_Be_One_The_Declaration_Places`.** Every
  backticked `nomos-*` token in the prose is a crate `members` places. Made to fail by
  renaming one of them to a crate this workspace does not have, which turned it red naming
  that crate; the rows stayed green throughout, because a crate named in a sentence is in no
  row. One direction, and that is a decision: the other — every declared member appears in the
  prose — is the *table's* claim, held by `Assert_Every_Member_Is_Listed`, and a paragraph is
  not an enumeration.
- **`Test_The_Excepted_Pairs_Should_Be_The_Same_Zone_Edges_The_Prose_Calls_Them`.** Two
  disagreements, because it has two subjects. Appending a `Host`-to-`Substrate` pair to
  `exceptions` turned it red on the declaration. Rewriting the sentence's own words turned it
  red on the missing subject, which is the half that matters: an assertion pinning the
  declaration without opening the file it is about would be the shape version 3 of
  `OD-PACKAGE-004` names in `transport_registry.rs`, cited by a check and read by none.

Four controls stand beside them and each one fails when the guard it names is removed, which
is the only evidence that any of them discriminates:
`Test_A_Prose_Naming_A_Crate_That_Does_Not_Exist_Should_Read_As_Naming_It`,
`Test_A_Crate_Named_Only_In_A_Table_Row_Should_Not_Be_Read_As_Prose`,
`Test_A_Fenced_Blocks_Body_Should_Not_Be_Read_As_Prose` and
`Test_An_Excepted_Pair_Naming_A_Package_No_Component_Places_Should_Be_Found`.

Two of the four were rewritten after failing to discriminate, and that is worth recording
because both failures had the remedy's shape. The fence control originally put a command
synopsis between two delimiters and passed with its guard deleted, since two fences are six
backticks and the spans after a balanced pair never move; what moves is the reading *inside*
the block, which returns the gaps between spans instead of the spans. The unplaced-package
control originally carried a pair with one unplaced end, which a plain equality catches
anyway; only a pair with *both* ends unplaced compares an absence against an absence and reads
as agreement.

### What is deliberately left alone, and why

**The `Owns` column is watched by nothing, and `OD-PACKAGE-004` version 3 decided that
deliberately.** 72 rows, 216 cells, 33,905 characters in the third column and not one of them
empty, measured here at `18db19c5`. That record rejected a check over the column on a stated
ground rather than deferring one: comparing it needs a declared source outside `README.md`,
which would be either the same prose in a second file — the second authority `OD-AGENT-001`
and this record refuse — or a shorter derived fact that would not be that column. It rejected
a non-emptiness check in the same breath, because a check that cannot tell a true description
from a placeholder must not be presented as one that watches the column.

Nothing this increment adds reads it. `Prose_Of` drops every table row before any assertion
looks, and `Test_A_Crate_Named_Only_In_A_Table_Row_Should_Not_Be_Read_As_Prose` goes red if
that stops being true. What is given up is real and is stated there rather than softened here:
an `Owns` cell that becomes false will not be reported by anything.

Two assertions reach that column incidentally and neither is a check over it, both re-measured
here rather than taken from the record that named them.
`Test_Band_Zero_Should_Be_Described_In_One_Place` requires `README.md` to contain
`OD-CONTRACTS-001`, and the file's single occurrence of that identifier — still exactly one —
sits inside the `nomos-contracts` row's `Owns` cell, so a mechanism that rewrote the column
would be told it had described band zero in the wrong number of places rather than that it had
deleted thirty-three thousand characters.
`Test_The_Transport_Should_Name_No_Repo_Tooling_Handler` quotes the four `[repo tooling]` marks
in its own module doc and in its failure message and never opens `README.md` at all: the file
names that document twice, in prose, and opens `crates/host/nomos-api-transport/src` and
`tests/contract/surface/nomos-api.txt`. Cited by a check, read by none.

**Which of the two tables a row belongs in.** `OD-PACKAGE-017` measured the listing as two
tables separated by a blank line, two lines of a person's prose and a repeated header, and
found the split has no declared source: `members` is a flat map from crate to zone, and
`nomos-spec-orchestration` is a `Specification` crate in the first table while the six
`nomos-spec-*` crates in the second are `Specification` as well. So the row set stays compared
as the union of both tables, which is what `Zone_Row`'s reading has always done, and which
table a row belongs in is asserted nowhere. Naming that is the honest outcome, exactly as the
`Owns` column is.

**The `[repo tooling]` paragraph.** The only counterpart for "marked" is in the free column,
and the declaration is not a counterpart for it: `Repo Tooling` holds three members —
`nomos-ledger`, `nomos-surface-provenance` and `nomos-work-orchestration` — while four rows
carry the mark, `nomos-spec-orchestration` being marked and zoned `Specification`. That is not
a defect in either. The mark says which product a crate serves, and `ARC-ECOSYSTEM-001` is
already the record that a shared zone does not decide product ownership. Comparing the sentence
against the marks would compare two hand-authored copies inside one file, neither of them an
authority, while making the free column watched.

**"The specification system sits beside the kernel rather than above it ... nothing in the
product may name it."** `permits` holds the fact this sentence is about, and the sentence is
true of it today: `Specification` appears in the permits of `Host` and `Verification` and
nowhere else. It is left uncompared because "the kernel" and "the product" are not terms the
declaration has and do not line up with its components — the analysis kernel's own crates sit
in `Substrate`, `Capability Contract`, `Provider` and `Rules` at once. A comparison would have
to author that grouping in the test, which is the second authority this record refuses, and
`OD-ROADMAP-005` states in as many words that its override "does not grant a zone permission
by fiat".

**The paths the prose names.** 17 path-shaped tokens, of which 9 deliberately do not exist:
two are the territory-normalization examples, and `README.projection.md` and
`spec/architecture.md` are documents the same paragraph says are not committed here. An
existence check would need a hand-authored exclusion list, which is the mechanism whose
failure this record's own history is about.

**"`tests/contract` asserts against that one declaration rather than a second copy of its
own."** True, and not mechanically separable: the suite names `nomos-architecture.json` in ten
places, every one of them a message or a doc, and no reading tells a second parser from a
sentence that names the file.

**"neither is among the four that render without one."** Its authority is
`nomos-spec-project`'s profile set, which this suite does not depend on. Reaching it is a
manifest change and a `Cargo.lock` change rather than a check, and that is a design question
for whichever item wants it.

### Two sentences are false, and the correction is not in this increment

`README.md` was held by a live claim for the whole of the increment, so both are recorded here
as measurements rather than repaired:

- The Layout paragraph says `nomos-rules` declares `ZONES`, `Permits` and `SAME_ZONE_EDGES`.
  No such item exists anywhere under `crates/`; `OD-RULES-003`'s third prerequisite moved the
  declaration out of every crate into `nomos-architecture.json` at the repository root, with
  `nomos-cap-architecture` carrying the contract and `nomos-repo-policy` reading the file. This
  is named in `P123-A-CRATE-DOC-AND-ITS-RECORD-DISAGREE-ABOUT-WHICH-OVERRIDE-LICENSED-THE-WORKFLOW-RUNTIME-2`'s
  own `done_when` and is that item's to correct.
- The closing section says `tests/contract/tests/boundaries.rs` compares the tables against the
  `nomos-rules` declared `ZONES`. Both halves are false: that path has not existed since the
  file became a directory, and the comparison is `tests/contract/tests/boundaries/readme.rs`
  against `nomos-architecture.json`. No item named it, so one was authored for it.

Both are version 1's finding once more and at a finer grain than version 2 measured it. The
prose that went stale is the prose nothing compares, and it went stale inside the file this
record's own measurement called correct — because that file is correct *where it is checked*,
which is four sentences of it and not the document.

### What would decide this differently

Version 1's three triggers and version 2's fourth stand. A fifth is added: **a sentence stating
a fact the declaration holds, added to `README.md` and left uncompared while a comparison for
it was available.** That is the failure this amendment is the remedy for, and it recurs by
writing rather than by anything breaking, so nothing will report it.

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
population it measured could not see. Those two are closed in turn by
`P97-TWO-FURTHER-COPIES-OF-A-PRINTED-VOCABULARY-STAND-OUTSIDE-THE-POPULATION-THAT-WAS-MEASURED`,
which found that this record's version 1 conclusion about `README.md` holds at a finer grain
than it was measured: that file is correct where it is checked, and its unchecked
vocabularies had gone stale like any other unchecked prose. No version bump for either: the
decision is untouched and the amendment's own coverage section is what moved, which is that
section saying what is true rather than what was true when the debt was counted.

Amended to version 3 by
`P126-A-FACT-THE-DECLARATION-HOLDS-IS-RESTATED-IN-README-PROSE-THAT-NOTHING-COMPARES`, on
`OD-ROADMAP-005` decision 6 superseding version 1's decline of generated README tables. The
version bump is because the decline moves, not because the measurement does: what the override
authorizes is that a fact `nomos-architecture.json` holds stops being restated in prose that
nothing compares, and it authorizes it *on this record's own evidence* — the checked tables are
exactly the artifacts that did not go stale, which is why the increment checks rather than
generates. Two comparisons were added over the prose around the zone tables and seven further
sentences were measured and deliberately left uncompared, each with its reason. The table's
`Owns` column is not among what moved and is watched by nothing, which `OD-PACKAGE-004`
version 3 decided deliberately and on a stated ground rather than leaving owed.
