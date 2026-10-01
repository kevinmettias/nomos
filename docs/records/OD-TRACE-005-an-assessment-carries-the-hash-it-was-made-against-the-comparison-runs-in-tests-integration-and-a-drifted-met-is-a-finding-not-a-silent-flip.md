---
id: OD-TRACE-005
type: decision
title: An assessment carries the hash it was made against, the comparison runs in tests/integration, and a drifted Met is a finding, not a silent flip
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - traceability
  - requirements
  - corpus
  - completeness
  - verification
relations:
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-TRACE-002
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
---

# An assessment carries the hash it was made against, the comparison runs in tests/integration, and a drifted Met is a finding, not a silent flip

## Question

Six entries are committed under `tests/contract/requirements`. Every one of them names a
verdict about a requirement's text — `EVID-001`, `CAP-002` and `CAP-003` as `Met`,
`CHK-003` as `Met` by way of `OD-CONTRACTS-002`, and `WORK-LEDGER-001` and `WORK-LEDGER-005`
as `Diverges`. All six were written by reading the v14 corpus once. Nothing has read it
since, on their behalf, and nothing in the committed shape would notice if the text they
were read against changed tomorrow.

`OD-TRACE-001` already named this cost and took it deliberately for one direction: the guard
cannot see code drift out of satisfying a requirement while the site stays valid, because
that is meaning and this workspace has no type for it. A changed requirement is not that
case. The requirement's text is content the store already hashes — `normative_statements`
carries a `canonical_hash` column
(`crates/spec/nomos-spec-store/src/schema.rs`), and
`nomos-spec-ingest` already recomputes it and reports a divergence when the recomputed hash
disagrees with the recorded one
(`crates/spec/nomos-spec-ingest/src/phases/statements.rs`). Comparing an assessed hash
against a current one needs no reading of either text, so it is not the excluded case.

What was actually blocking it, per this item's own framing, is not the comparison's
mechanics but its address. `OD-TRACE-002` already answered where it cannot live and gave a
reason stronger than scheduling: `nomos-contract-tests` is the crate that counts corpus
gates, so a corpus-reading test placed inside it is invisible to both derivations that keep
`corpus_gates.rs` honest at once, which is `OD-GATE-001`'s defect rebuilt inside the file
written to prevent it. That leaves three things undecided, and this record answers them: the
address itself, what an assessment must carry for the comparison to have an input, and what
a mismatch does to the verdict already on file.

## The Decision

### 1. The comparison runs in `tests/integration`, corpus-gated and declared

`OD-TRACE-002` already named this address as "the obvious home" without adopting it; this
record adopts it. A requirement-drift check is a corpus-backed test suite under
`tests/integration`, reading `NOMOS_V14_CORPUS` the way `tests/integration/tests/determinism`
and `tests/integration/tests/analysis_slice` already do, and it is declared as a row in
`corpus_gates.rs` from that side. Both scanners that keep the gate table honest reach it
there: `Files_Naming_A_Corpus` walks every workspace member's `tests/` directory and
`tests/integration` is not the member `Corpus_Gates` skips — only `nomos-contract-tests` is,
and for the stated reason that it is the crate doing the counting. So the declaration is
visible to both derivations, which is exactly the property `OD-TRACE-002` measured
`nomos-contract-tests` as unable to offer.

`tests/contract/tests/requirement_trace/` keeps doing what it already does: read the
committed entries, and check what is checkable without the corpus — that a named site still
resolves, that a named record is registered, that every `Diverges` and `NotBinding` names
one, that the assessed set has not shrunk. None of that moves. The corpus-backed half is
additive, lives beside it in a different crate, and depends on nothing this record changes
about the reader's grammar except the one field the next section adds.

### 2. Two states, and neither of them is `pass`

`OD-GATE-001`'s finding was that a test unable to look prints `ok` and is indistinguishable
from one that looked at everything and agreed. A requirement-drift run has two ways of being
unable to look, and both get a name that is not `pass`, `ok`, or silence — a report state the
run prints about an entry, never a word an assessment file's `verdict:` line may hold:

- **`Unexamined`** — the run's own report when `NOMOS_V14_CORPUS` is not set. Nothing was
  compared, the run says so as its result rather than returning quietly, and it is reported
  the way `corpus_gates.rs` already reports its own hole: a number somebody can read, not an
  absence nobody measured.
- **`Unhashed`** — an entry's report when the corpus *is* mounted but the entry carries no
  hash to compare. This is not folded into `Unexamined`, because the two causes are not the
  same fact: one says nobody could look this run, the other says this particular entry gives
  the tool nothing to look *at* even when it can. Collapsing them would make a corpus-mounted
  run over an unhashed entry read exactly like a corpus-absent run, which is the same
  same-shape-from-outside defect `OD-TRACE-001` opened by measuring that met and unmet have
  the same shape from outside.

An entry with a hash, compared against a mounted corpus, resolves to `Confirmed` (the hashes
agree) or `Drifted` (they do not). Both are informational outputs of the run, not something
this record makes an assessment file allowed to say about itself — `Verdict` stays exactly
`Met`, `Diverges`, `NotBinding` per `OD-TRACE-002`, and `Unassessed` stays refused as a
written word for the same reason it already is.

### 3. An assessment carries the canonical hash of the text it was made against

Going forward, an entry names, beside its verdict and its sites, the canonical hash of the
requirement text the verdict was reached against — the same hash the store already computes
for that text once the corpus is mounted, not a second algorithm invented for this registry.
An entry with a verdict and no hash is the state the six entries are in today, and section 4
is what a reader does with that state; it is not proposed here as a shape a *new* entry may
choose. The reader `OD-TRACE-002` built has no lenient path for a malformed entry — a missing
verdict, a second record line, a site with no symbol are all refused rather than skipped —
and an entry making a claim about text it does not identify belongs in that same refused
set, because a hash field a writer may omit is a hash field nobody can rely on being there.

Writing the field is not this record's act. It is the shape the next item that touches
`tests/contract/tests/requirement_trace/reader.rs` must build toward, exactly as `OD-TRACE-002`
decided the file grain and left the registry itself to the items that would carry it.

### 4. The six committed entries predate the field, and stay `Unhashed` until backfilled

`EVID-001`, `CAP-002`, `CAP-003`, `CHK-003`, `WORK-LEDGER-001` and `WORK-LEDGER-005` were
each written by a hand audit against a mounted corpus, and none of them recorded the hash of
the text they read, because no such field existed to record it into. This record does not
invent one retroactively — a hash nobody actually compared at authoring time would be a
fabricated fact wearing a real one's shape, which is worse than the gap it would appear to
close.

So all six are `Unhashed` under a requirement-drift run from the moment such a run exists,
regardless of what their `verdict:` line says. Their `Met` and `Diverges` calls stand exactly
as authored — this record does not touch the assessment files, which are outside its
territory in any case — but the drift run has nothing to compare them against and must say
so as `Unhashed`, not fold them into `Confirmed` by having nothing to disagree with. Bringing
a given entry current is a re-audit against the mounted corpus that records the hash found
there, the same act `OD-TRACE-001` already calls re-verification and already requires the
corpus to perform. Nothing here obligates that six-entry backfill to happen in one item or on
any schedule; `OD-TRACE-001` chose a floor over a count for the same reason a completion
deadline was refused there.

### 5. A drifted `Met` is a finding for a person, not an automatic flip

The alternative was real and is refused here by name: when a run finds `Drifted`, it does not
rewrite the entry's `verdict:` line to something else and it does not delete the entry.

The reason is the same sentence `OD-TRACE-001` already used to decide the registry's whole
shape: **an assessment is a declared entry committed to this repository, and it is not
derived from the corpus at check time.** A tool that reacts to a hash mismatch by rewriting
`Met` to anything else is deriving committed content from the corpus at check time — the
exact shape that decision excludes, applied to an edit instead of to an initial write. A hash
mismatch also under-determines the right new verdict on its own: the text may have been
reworded without changing what it requires, in which case the honest answer is still `Met`
with an updated hash, or it may have changed the requirement in a way the site no longer
satisfies, in which case the honest answer is `Diverges` or `Unassessed`-by-deletion with a
record explaining which. Telling those apart is reading the two texts and judging whether the
difference matters — the same act `OD-TRACE-001` already calls an audit and already refuses to
let a mechanical guard perform on its own.

So `Drifted` is a finding: the run's output names the entry, the hash it was authored
against, and the hash the corpus holds now, and a person resolves it by re-auditing and
re-committing the entry, the same authoring act that produced it the first time. Until that
happens, the committed `verdict:` continues to read exactly as authored. This is a narrower
promise than "the registry is always current" — it is the promise `OD-TRACE-001` already made
and this record keeps rather than quietly widens: a stale `Met` is wrong about one
requirement, and now that wrongness is a named, surfaced state instead of an invisible one,
which is the entire distance this record moves the registry.

## Why This Is `OD-TRACE-002`'s Constraint Honored, Not Routed Around

`OD-TRACE-002` drew a line this record does not cross: the corpus comparison "lives outside
`nomos-contract-tests`" and, if it exists, "it is declared in `corpus_gates.rs` from there,
where both derivations can see it." This record does not add a test to
`tests/contract/tests/requirement_trace.rs`, does not put a corpus read behind a helper that
would make it invisible to either scanner, and does not ask the guard crate to hold anything
it cannot compute without the corpus. Every corpus-reading act this record describes happens
in `tests/integration`, exactly where `OD-TRACE-002` already pointed and declined to build.
What this record adds beyond that pointer is the vocabulary the pointed-to check needs before
it can be written at all: two non-`pass` states for a run that cannot look, the field an
entry must carry for a comparison to have two hashes to compare, the fate of the six entries
that predate that field, and the refusal to let a mismatch mutate committed content by itself.

## What This Record Does Not Do

- It writes no code. `reader.rs`'s grammar does not gain the `hash:` key here, no
  `tests/integration` suite is created, and `corpus_gates.rs` gains no row. Those are a
  future item's territory, the same relationship `OD-TRACE-001` had to `OD-TRACE-002` and
  `OD-TRACE-002` had to the registry it left unbuilt.
- It does not backfill the six entries or assess a 364th requirement.
- It does not reopen semantic drift. Code drifting out of satisfying a requirement whose text
  has not changed is still outside what any guard here can see, unchanged from `OD-TRACE-001`.
- It does not make `Confirmed` or `Unexamined` or `Unhashed` a written word inside an
  assessment file. All three describe a run's report about entries, never a fourth thing an
  author may type into one — the same separation `OD-TRACE-002` already drew between what the
  registry says and what a comparison, if it existed, would report.

## Status

Closed by `P12-TRACE-DRIFT`. No item has yet built the field, the `tests/integration` suite,
or its `corpus_gates.rs` declaration; this record decides their shape so that whichever item
does inherits it rather than reopening the address question `OD-TRACE-002` already
half-answered — the same relationship `OD-TRACE-001` had to the registry `OD-TRACE-002` and
`P10-LEDGER-CORPUS` still owe.
