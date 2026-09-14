---
id: OD-SPEC-004
type: decision
title: The filler blocklist misses the wording that hollowed 44 restored members
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - preservation
  - regression
  - filler
relations:
  - target: ARC-SPECDB-001
    type: affects
  - target: OD-SPEC-002
    type: affects
---

# The filler blocklist misses the wording that hollowed 44 restored members

Numbered 004 rather than 003. `OD-SPEC-003` was authored and reverted on `dev` as a second
home for what `D-132` settled; reusing the identifier would make one number name two
subjects in the same history.

## Question

`FILLER_PATTERNS` is seeded from v14's own `validate_bundle.py` blocklist and is what
`NSV-PRESERVE-004` is specified to run. The cross-revision regression report measured what
that blocklist sees in v15.0, and it sees a little more than half of it.

## The Measurement

v15.0 hollows its sections with two sentences. The blocklist carries one of them:

| Wording | Sections | Documents | Declared |
| --- | --- | --- | --- |
| `This section preserves the reference or explanatory material for …` | 187 | 29 | yes |
| `This section preserves the governed reference or explanatory material for …` | 196 | **132** | **no** |

One inserted adjective. `Is_Filler` is substring containment, so `governed` between
`the` and `reference` is enough to miss every one of them.

The consequences are exact, and two of them are the plan's own numbers:

- **The plan's 132 is this template.** The plan states that 132 v15 files contain
  placeholder filler and that ~102 are pure stubs. 132 is the number of documents standing
  on the undeclared wording, and the stub count reproduces only when it is counted —
  neither figure is reachable from the declared blocklist, which matches 99 documents.
- **44 restored members are hollowed by it alone**: all 16 appendix-D members, all 7
  appendix-H sections, the 11 E.1 headless-inventory sections, the 6 F.1 IDE profiles and
  the 4 end-to-end scenarios. Their headings survive in v15.0 and their bodies are the
  form letter. A preservation run whose only filler test is the blocklist passes every one
  of them, and `NSV-PRESERVE-004` reports clean over the sections that lost the most.
- The declared blocklist is not idle and its own figures stand: 405 blocks across 99
  documents, which is what P3-OVERLAY measured and what it measures correctly.

The regression report finds the undeclared template by repetition rather than by pattern: a
body whose text, its own section title elided, is carried by three or more sections is a
form letter, and the report names the template and how many sections stand on it. Over the
ten v14.36 domain volumes that test fires on 13 sections out of 482, so it is not a test
that calls ordinary prose filler.

## Why It Matters

A blocklist matches sentences someone wrote down. The v15.0 generator wrote a sentence
nobody had written down, and it is not a defeat by adversary — it is one word of ordinary
editorial drift. `NSV-PRESERVE-004` is specified as a blocklist seeded from
`filler_patterns`, so as specified it inherits this hole: the rule would run, report
nothing, and the run would be `pass`. That is the exact shape of the failure this whole
build exists to prevent — the absence of a finding is indistinguishable from the absence of
the problem.

It also bears on `NSV-PRESERVE-005`, the narrative coverage floor. The floor would have
caught this where the blocklist did not, because D.0 falling from five thousand characters
to two hundred is a coverage collapse whatever the replacement prose says. The two rules
are not redundant, and this is the case that shows which one is load-bearing.

## What Would Close It

A decision on what the ledger's filler test is:

1. **A repetition census with the blocklist retained as the naming layer.** Repetition
   answers *is this filler*, which no pattern list can keep up with; the blocklist answers
   *which known pattern*, which a lineage row needs in order to say why a block was judged.
   Recommended.
2. The blocklist alone, with the governed wording added. Closes this instance and not the
   class: the next generator writes the next sentence.
3. The blocklist alone, unchanged, with `NSV-PRESERVE-005` accepted as the rule that
   catches hollowing. Coherent, but it means `NSV-PRESERVE-004` is documentation of known
   patterns rather than a preservation guarantee, and it should say so.

Adding the governed pattern to `FILLER_PATTERNS` is deliberately not done here. It is
P3-OVERLAY's territory, it would move that item's measured figures from 99 documents to
206, and doing it silently would leave the class unaddressed while making it look handled.

## Resolution

Option 1: a repetition census with the blocklist retained as the naming layer.
`NSV-PRESERVE-004` (`crates/spec/nomos-spec-validate/src/rule/no_undeclared_filler_template.rs`)
groups `source_blocks` rows of kind `prose` by exact text — a heading is its own block and
excluding it *is* the "section title elided" this record already specified, not a second
mechanism for it — and treats a group carried by `SHARED_BY` (3) or more sections the same
way `archaeology::Shared_Templates` already treats one for the cross-revision regression
report: a template. A template `Is_Filler` names is accounted for; one it does not is the
violation. The blocklist keeps its job — naming which pattern a template matched, for a
lineage row that has to say why — and stops being the only test standing between an
ordinary editorial sentence and a hollowed corpus walking past it clean, which is what
option 2 would have left true for the next generator's next sentence and what option 3
would have conceded outright.

This does not add the governed wording to `FILLER_PATTERNS`. That remains P3-OVERLAY's
territory, unchanged by this resolution: the repetition census finds an undeclared template
by what it *is*, not by widening the list of sentences it is checked against, so the two
items still measure different things and neither substitutes for the other.

## Amendment (P103-REPETITION-IS-NOT-SUFFICIENT-EVIDENCE-OF-FILLER-AND-OD-SPEC-004-SAYS-IT-IS): repetition is necessary and not sufficient

Version 3. The resolution above stands for the corpus it was taken over, and this amendment
extends it rather than correcting it. What changed is not that the census was wrong but that
it has now been run over two corpora it was never calibrated against, and over both of them
repetition alone selects things that are not filler.

### What was measured, and how to re-run it

2026-09-14, at a clean tree, with both corpus variables exported:

| Corpus | Test | Subjects | Violations |
| --- | --- | --- | --- |
| Ten v14.19 domain volumes | `crates/spec/nomos-spec-ingest/tests/restoration.rs` | 2,988 | 16 |
| The sibling suites and both plans | `crates/spec/nomos-spec-ingest/tests/siblings.rs` | 18,873 | 103 |

Both are `NSV-PRESERVE-004` reporting through the preservation run, and both are reproduced
by `cargo test --no-fail-fast -p nomos-spec-ingest --test restoration --test siblings` with
`NOMOS_V14_CORPUS` and `NOMOS_SPEC_ARCHIVES` both set. Set only the first and the sibling
suite reports nine passed having read nothing, because `Ecosystem()` returns early without
the second. That is the hazard `AGENTS.md` names, and it is why the 103 went unseen until
now.

**The 13-of-482 calibration above was not re-run, and 16 is not a delta against it.** That
figure was taken over the ten v14.36 domain volumes by the cross-revision regression report.
The corpus on this machine is v14.19 -- 51 occurrences of that version string and none of
v14.36 -- so the two numbers are two measurements of two editions by two code paths, and
nothing here says the original was wrong. It says it cannot be checked from here.

### One: the length floor

**A body below a length floor is not eligible to be a template at all, and the floor is 36
characters of the normalized block text.**

The floor is derived from the gap between three measured populations, not chosen to bound a
count:

| Population | What it is | Length |
| --- | --- | --- |
| 102 of the 103 sibling violations | connectives, labels, a horizontal rule, a single arrow | 1 to **30** |
| nothing observed | | 31 to 42 |
| 16 domain violations, and the 103rd sibling one | edition line, suite title, abstracts, a status line | **43** and up |
| The nine `FILLER_PATTERNS` entries | what this repository declares filler *is* | **50** and up |

The longest thing that collided by accident is 30 characters. The shortest contentful body
repeated by design is 43. Between them the distribution is empty in both corpora, and the
floor is a number in that gap.

Two boundaries fix it from opposite sides. It must exceed 30, or the collisions this
amendment exists to stop stay in. It must not exceed 43, because the edition line is
contentful text that a corpus ought to have to *declare*, and a floor above it would drop
that line for being short -- the right answer for the wrong reason, and one that stops being
right the first time a suite repeats a short line it meant to. So a floor at 50, which would
look natural because it matches the declared vocabulary, is refused for that reason.

Within the band the two errors are not symmetric, and the record already says why: a floor
set too low reports a fragment somebody files an item about, and a floor set too high hides
a hollowing, which is the failure this whole record was written about -- the absence of a
finding being indistinguishable from the absence of the problem. So the floor is taken from
the lower half of the band. 36 clears every observed collision by six characters and leaves
seven before the shortest body that must survive.

Two things this deliberately is not. It is not the observation that 102 of 103 violations
are 40 characters or fewer; that is a consequence of the band and not evidence for any
particular number in it. And it is not composed with the declaration below -- the floor
decides eligibility before repetition is counted, and the declaration decides admissibility
after. Whoever builds them should be able to build either one first.

Because the floor is derived rather than stipulated, it is re-derivable: when the corpora
move, re-measure the longest accidental collision and the shortest deliberate repetition and
check that 36 still lies between them.

### Two: declared publication text

**A corpus may declare the text its own layout repeats, the declaration is corpus-side data,
and it names structural roles rather than strings.**

Sixteen of the violations are one phenomenon. Four are suite-wide front-matter carried by ten
sections across ten documents -- the edition line, the suite title, the volume-ownership
sentence, the Word navigation note. Eleven are volume abstracts at three sections across two
documents, which is the cover line, the `## Document purpose` section and the suite index
entry: three canonical places, so a ten-volume suite crosses `SHARED_BY` by construction and
would do so however well it was written. None of the sixteen is prose that says a section
exists without saying what it says.

**Extending `FILLER_PATTERNS` is refused.** `Get_Filler_Pattern` returns *which* pattern
matched so that a lineage row can name why a block was judged filler instead of asserting it,
and that list is documented as prose that says a section exists without saying what it says.
Putting a volume abstract in it would make a lineage row state something false about a block
that does say what it says. The blocklist is a vocabulary of filler; an abstract is not
filler, and the fix for a rule calling it filler is not to agree with the rule.

**A second compiled-in list beside it is refused for the same reason this record already
gave once.** A list in this repository's source is this repository's opinion about another
corpus's layout, and it goes stale exactly the way the blocklist went stale when one
adjective changed. The corpus knows what its own format prescribes; this crate does not.

So the declaration is corpus-side, and it declares *roles*, not text. A corpus says that a
canonical block is intentionally projected into named structural positions -- cover, document
purpose, suite index entry -- and the expected multiplicity follows from the declaration. A
block appearing in the places its declaration names is admitted and is not filler. A fourth,
undeclared occurrence of the same block is still a violation, and so is a repetition nothing
declared at all. That is the difference between a declaration and a suppression list, and it
is the whole reason this is not simply a wider blocklist: a suppression list answers *ignore
this string*, and a declaration answers *this block belongs in these three places*, which is
falsifiable.

### Three: the parity note is a third case, and this record does not decide it

One violation is neither of the above: a parity note repeated ten times inside a single
volume, once beneath each of ten capability tables, at ten sections across one document.

It is not short -- it is past the floor by any reading. It is also not the projection case,
because it never leaves its document, and cover-and-purpose-and-index has nothing to say
about a block that appears ten times in one file. Admitting it as structural repetition
because it resembles the sixteen would move the false positive rather than remove it, which
is the mistake this amendment is trying not to make twice.

What would decide it is why the ten exist. If the ten tables are ten projections of one
capability set, the note is one block with ten roles and question two already answers it. If
they are ten independent statements that happen to share wording, then it is the case
`NSV-PRESERVE-004` was built for and red is the correct answer. Nobody has established
which, so this record admits it under neither heading and leaves the test red.

### What this amendment does not do

It builds no mechanism and pins no violation count into any test. Two items follow from it,
one per defect, because the two defects need different mechanisms and the item that bundled
them could not land. Until those land, `restoration.rs` and `siblings.rs` stay red. Quieting
them with an accepted count would be precisely the suppression `NSV-PRESERVE-004` exists to
make visible, and this record would have argued itself into the position it was written to
refuse.

## Status

Accepted. `NSV-PRESERVE-004` joined `DECLARED_RULES`
(`crates/spec/nomos-spec-validate/src/run.rs`) and `Registered()`
(`crates/spec/nomos-spec-validate/src/preserve.rs`) beside the other four preservation
rules, so the class this record measured — a hollowing pattern nobody had written down yet
— now fails a validation run rather than passing one silently.

Version 3 keeps that acceptance and narrows what repetition alone is allowed to
conclude. Two mechanisms follow from the amendment above and neither is built here, so
`crates/spec/nomos-spec-ingest/tests/restoration.rs` and
`crates/spec/nomos-spec-ingest/tests/siblings.rs` are red at this commit and are meant
to be. They go green when the floor and the declaration land, not when a count is
written into them.
