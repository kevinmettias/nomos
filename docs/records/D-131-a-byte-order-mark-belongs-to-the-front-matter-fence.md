---
id: D-131
type: decision
title: A byte order mark belongs to the front matter fence
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - preservation
  - ingest
relations:
  - target: ARC-SPECDB-001
    type: affects
  - target: D-129
    type: affects
---

# A byte order mark belongs to the front matter fence

## Decision

`Segment` recognises an optional leading byte order mark as part of a document's opening
front matter fence. Where front matter follows, the mark leaves with it and reaches no
block. Where none follows, the mark is kept in the text exactly as found.

## Why It Had To Be Settled First

1637 of the 2187 authored v14 documents carry a mark, including all six decision records
and every artifact under `01_authoring/artifacts`. `str::trim` does not remove one —
U+FEFF is not whitespace under Unicode — so the first line of a marked document did not
compare equal to `---` and its front matter was segmented as prose. I4 ingests exactly
those documents through `Ingest_Source_Document`, which segments raw file text.

Every block hash downstream depends on the answer, and the P1 gate rests on reproducing
v14's hashes. Guessing would have moved them.

## The Evidence

Both v14 readers that ship compile the same pattern:

```python
FRONT_RE = re.compile(r'^\ufeff?---\n(.*?)\n---\n', re.S)
```

`build_spec.py::split_markdown_front_matter` and
`validate_bundle.py::markdown_document_records` each read with `encoding='utf-8'`, which
leaves the mark in the string, and each match with that pattern, which consumes it.

Run over the archives, the pattern matches all 2187 authored documents — 1637 marked, 550
not — with no exception in either direction, and no body retains a mark. The negative case
is what makes it conclusive: `markdown_document_records` walks the whole authoring tree and
reports `missing YAML front matter` on any document it cannot match. Had the mark not been
tolerated, v14 would have reported 1637 such errors. It validated clean.

## What The Archives Do Not Settle

`source-block-lineage.yaml` records 2533 blocks drawn from exactly ten documents, the
domain volumes, and **none of the ten carries a mark**. No recorded block hash corroborates
any of this, and the segmenter that produced those hashes does not ship with the corpus.

So the authority here is the two readers' agreement, not the manifest. This is recorded
rather than resolved by preference, and it is the reason the decision is stated as a
decision instead of as a reproduction.

The second branch — a marked document with no front matter — occurs zero times in the
corpus and is therefore settled by faithfulness to what v14's reader did rather than by
evidence: no fence matched, so it dropped nothing. Keeping the mark is also the only
choice consistent with a preservation ledger, because the sole bytes discarded are a
delimiter's.

## Consequences

No existing hash moves. The ten manifest-covered volumes carry no mark, so the change is a
no-op on every value the P1 gate checks, and that gate stays green.

`Parse_Record` already stripped a mark before requiring `---`, which is the same rule
arrived at from the record reader's side; the two now agree by construction rather than by
coincidence.
