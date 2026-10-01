# API — contract

## Contract

`CAPABILITY` is named for what a caller gets — whether a control-flow path forward from a
fact-read failure reaches a `Finding` — rather than for how it is obtained, the same reason
`nomos.cap.dependency.edges` is named for its answer rather than for Cargo.

## Returns

[`FactVariant::SemanticallyResolved`] because `OD-RULES-008` already settled what a *sound*
answer to this question needs: every call an `Err` arm reaches, including into helper
functions elsewhere in the crate, resolved to confirm it actually constructs or propagates
a `Finding` rather than merely being named as though it does. Pattern-matching the arm's own
tail expression — what any provider offering below this ceiling does — cannot see past a
call site, which is exactly the gap a sound provider closes. The ceiling states the
capability's true tier; it is not moved down to match what the first provider offering
against it actually delivers, the same choice `nomos.cap.dependency.edges`' own ceiling
makes for the same reason.

[`IncrementalGranularity::File`]: this capability's canonical subject is a control-flow edge
inside one function body, and a function body lives in one file — there is no coarser unit
a change here could force a re-derivation across, the same granularity
`nomos.cap.syntax.items` already claims for the same reason.

Both `Assurance`s are `Sound`, not pinned to what any current provider achieves: this
ceiling leaves room for a provider whose soundness and completeness are both established,
the same way `nomos.cap.dependency.edges`' ceiling leaves room for one that also resolves
generated dependency declarations.
