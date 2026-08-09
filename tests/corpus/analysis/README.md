# The precision corpus

Nine files across three directories, and the whole dependency graph fits on this page.
That is the entire point of it.

`F:/repos/xvpe` answers whether the slice works at scale — several thousand files nobody
wrote for this test. It cannot answer whether a change recomputes *exactly* the right
facts, because stating the right answer requires knowing every edge, and nobody knows
every edge of a corpus that size. A test over xvpe can only count, and a count of two is
satisfied by recomputing the wrong two.

So this corpus exists to be known entirely.

## Shape

| Group | File | Parses | Items | Public |
|---|---|---|---|---|
| `alpha` | `one.rs` | yes | 4 | 2 |
| `alpha` | `two.rs` | yes | 3 | 1 |
| `beta` | `three.rs` | yes | 5 | 3 |
| `beta` | `four.rs` | yes | 2 | 0 |
| `gamma` | `five.rs` | yes | 3 | 0 |
| `gamma` | `broken.rs` | **no** | — | — |

Three groups, six files, one of them unparseable. Every figure above is asserted in
`tests/integration/tests/analysis_slice.rs`, so this table cannot drift from the corpus
without a test failing.

## Why one file does not parse

`gamma/broken.rs` carries a `U+FEFF` byte order mark in the middle of the file — the same
damage the xvpe walk found in seven files, reproduced here at a size somebody can inspect.
It makes `gamma`'s rollup a *degraded* answer rather than a smaller one: the surface fact
for `gamma` records `unreachable = 1`, and a rollup that quietly summed the file it could
read would report `gamma` as a directory declaring nothing unusual.

A corpus in which everything parses cannot demonstrate that.

## Why these are not compiled

Nothing here belongs to a cargo package. They are input text, read by the provider the
same way any other corpus is read, and the workspace never tries to build them. That is
also why `broken.rs` can be broken.
