# API — nomos-cap-complexity

## Public Surface

What the parties agree on — the capability's identity, the contract version, the ceiling, the
summary, and the schema every answer is stamped with — plus the payload shape, the one
descriptor every answer carries, and the canonical codec. The same division
`nomos-cap-controlflow` draws between its `contract` and its payload module.

## Contract

This is the first capability of the metric family `OD-ROADMAP-004` placed on the near-term
tier, and the first payload in this workspace that carries its own reading rules. A number
alone does not say whether two of them may be added, which way is worse, or what a missing one
means, and `MET-006` makes each of those a property of the metric rather than of whoever reads
it:

> "Every metric descriptor shall define unit, subject kinds, aggregation operator, weighting,
> normalization, missing-data behavior, directionality, baseline requirements,
> uncertainty/statistical treatment, and snapshot comparability."

`MetricDescriptor` has one field for each, and every payload carries all ten, so a reader
holding a value holds the terms it may be read under. `MET-007` is the reading rule that makes
the descriptor worth carrying:

> "A client may not aggregate or blend a metric beyond the descriptor’s declared semantics.
> Non-aggregable metrics must remain at their native granularity or use an explicitly named
> derived metric."

This metric declares `Aggregation::NotAggregable`: the sum of two functions' complexities is
not the complexity of anything, and a file's or a crate's "total complexity" is exactly the
blend `MET-007` forbids. A reader wanting a per-file figure names a derived metric of its own.

`OD-CAPABILITY-002`'s criterion puts the contract here, below both parties and named by
neither: `nomos-lang-rust-complexity` offers against it and `nomos-rules` reads it.
