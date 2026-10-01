# API — nomos-cap-csharp-semantics

## Public Surface

What the parties agree on — the capability's identity, the contract version, the ceiling, the
summary, and the schema every answer is stamped with — plus the payload shape and its canonical
codec, the division `nomos-cap-controlflow` draws between its contract and its payload.

## Contract

A C# `#if`/`#elif`/`#else` chain holds every branch's declarations in one file, and which of them
the compiler reads depends on the preprocessor symbols the build defines: a configuration's
`DEBUG` or `RELEASE`, a target framework's `NET8_0` and `NET8_0_OR_GREATER`, a project's own
`DefineConstants`, and the file's own `#define` and `#undef`. Nothing in the file says which
build it is compiled for, so no reading of the file alone can answer. `nomos-lang-csharp`'s
syntax provider says so in its own guarantee and declines to read inside these regions for that
reason.

This capability answers exactly that question for one named build: for every branch of every
chain in one file, whether that build compiles it. Its answer is what would let the syntax
reading know which declarations exist in a given build, which is a consequence of this contract
and not something it does.

The crate name is broader than its one capability because the next C# questions that need a
compilation's semantics — an effective accessibility, a resolved symbol, which partial
declaration completes which type — are the same party's to answer. Each will be its own
capability when a rule asks for it; `OD-ANALYSIS-007` version 2 decided a compiler-backed
contract lives under `crates/capabilities`, apart from its provider, so that `nomos-rules` can
read it at all.
