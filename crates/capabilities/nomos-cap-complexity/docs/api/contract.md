# API — contract

## Contract

`CAPABILITY` is named for what a caller gets — a function's complexity, in the metric family —
rather than for how it is obtained. `SCHEMA` is versioned apart from the contract, the way
every capability here versions them, so a change to the wire shape and a change to what is
promised are separate events.

## Returns

The ceiling is `FactVariant::SemanticallyResolved`: a count taken over a resolved program, with
every macro expanded, would see every branch a function really has. `OD-ROADMAP-004` names the
level a producer reaches as the level its fact carries — `Syntactic` for a count read from text,
`SemanticallyResolved` for one read from a resolved model — and the ceiling states the true tier
rather than the first provider's, the same choice `nomos.cap.controlflow.reachability`'s ceiling
makes for the same reason.

Both `Assurance`s are `Sound` at the ceiling, and `IncrementalGranularity::File`, because a
function body lives in one file and there is no coarser unit a change could force a
re-derivation across.
