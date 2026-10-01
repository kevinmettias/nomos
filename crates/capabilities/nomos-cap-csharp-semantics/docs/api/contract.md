# API — contract

## Contract

`CAPABILITY` is named for what a caller gets — which conditional branches a build compiles —
rather than for how it is obtained. `SCHEMA` is versioned apart from the contract, so a change to
the wire shape and a change to what is promised are separate events.

## Returns

The ceiling is `FactVariant::SemanticallyResolved`: the answer resolves each condition's symbols
against the definition set a real build hands its compiler, which is a fact about the
compilation rather than about the text. Both `Assurance`s are `Sound` at the ceiling — every
branch reported exists and is judged as the compiler judges it, and every branch the file has is
reported — so a provider that establishes both may claim it.

`IncrementalGranularity::File`: one file's answer is computed from that file's text and the
build's definition set. A change to one file re-derives that file's answer alone; a change to
the definition set changes every answer's inputs, and each file is still re-derived on its own.
