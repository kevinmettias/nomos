---
id: OD-SPEC-004
type: decision
title: The filler blocklist misses the wording that hollowed 44 restored members
status: accepted
version: 2
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

## Status

Accepted. `NSV-PRESERVE-004` joined `DECLARED_RULES`
(`crates/spec/nomos-spec-validate/src/run.rs`) and `Registered()`
(`crates/spec/nomos-spec-validate/src/preserve.rs`) beside the other four preservation
rules, so the class this record measured — a hollowing pattern nobody had written down yet
— now fails a validation run rather than passing one silently.
