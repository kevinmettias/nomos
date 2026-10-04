# Literal safety in automatic whitespace corrections

The raw-line formatting rule can report trailing bytes inside a multiline literal.
They are data there. Before the release safety correction, the real pipeline staged
removing three spaces from a Rust raw string while labeling that edit mechanically
safe and behavior preserving. The new commit/stage regression reproduced this
before the guard was added.

Candidate construction now refuses files containing a quote, apostrophe or
backtick delimiter, and paths outside the currently supported Rust, Go and
C-sharp extensions. The refusal states that literal safety is unproven. No edit
is staged or written for that candidate. The same constructor serves individual
and wave correction paths. This guard is deliberately conservative: a single-line
string, lifetime apostrophe or quoted comment can also prevent automatic trimming.
It does not pretend to distinguish those cases lexically.

```powershell
cargo test --locked --offline -p nomos-correction-orchestration --no-fail-fast
```

The multiline regression invokes the real individual correction pipeline with
commit disabled and enabled. Both must refuse and leave the exact file bytes
intact. The direct guard regression exercises Rust ordinary/raw/byte strings,
Go backticks, C-sharp quoted text, unsupported C, and supported literal-free
positive controls. Existing batching and line-ending regressions still apply.

Sound provider-owned literal spans remain necessary before automatic trimming
can safely operate on eligible lines in files that also contain literals. The
formatting finding remains a raw-line policy observation; this change neither
turns it into a syntax-aware finding nor certifies every such finding as a useful
engineering violation. Compilation alone would not have caught the changed
literal value. No complete 0.1 release claim follows from this guard.
