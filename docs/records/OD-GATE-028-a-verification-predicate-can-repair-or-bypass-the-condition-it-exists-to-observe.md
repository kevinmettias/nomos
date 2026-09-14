---
id: OD-GATE-028
type: decision
title: A verification predicate can repair or bypass the condition it exists to observe, and is not established until it has been seen to fail
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - gate
  - enforcement
  - determinism
relations:
  - target: OD-GATE-001
    type: relates-to
  - target: OD-GATE-027
    type: relates-to
  - target: OD-PLATFORM-004
    type: relates-to
---

# A verification predicate can repair or bypass the condition it exists to observe, and is not established until it has been seen to fail

## Question

`OD-GATE-001` decided that a skipped test reports `ok`, and this workspace has built a good deal
of machinery on the principle behind it: a check that verified nothing must not report what a
check that verified everything reports. Every instance of that principle so far has concerned the
*subject* — an empty corpus, a rule that judged no file, a universe with no members.

Three defects found in one session share a different cause, and it is not an empty subject. In
each, the subject was present and the violation was real, and the verification mechanism
destroyed or stepped around the violation on its way to looking.

## The Three Instances

Evidence, not illustration. Each was measured, and each reported success.

**A warm target directory answered for a build that never happened.** `cargo check --workspace
--all-targets`, run to establish that this workspace builds against a pinned XVPE revision,
exited zero in twenty seconds having compiled no `xvpe-` crate at all: a concurrent session had
already warmed the shared target directory. The identical command with an isolated
`CARGO_TARGET_DIR` compiled ten of them from the pinned revision by name. The exit code was
evidence that nothing failed to typecheck against artifacts somebody else produced, which is not
the claim it was being read as.

**An ambient configuration kept a violation alive, and was the only reason a guard could fail.**
A guard over `Cargo.lock` was landed, and appeared to work: the lock was un-pinned, the guard went
red. It went red because an auto-discovered `.cargo/config.toml` kept a Cargo patch applied
throughout the test run. When `OD-PLATFORM-004`'s amendment made that substitution opt-in — a
strictly better arrangement, for unrelated reasons — the guard stopped being able to fail at all.
A correct change to something else silently disarmed it.

**Test setup repaired the violation before the test body could see it.** The same guard read
`Cargo.lock` from the working tree. Cargo re-resolves and re-pins that file on any invocation,
including the `cargo test` that runs the guard. Measured: an opt-in build left the tree at zero of
twelve packages pinned, and one ordinary `cargo metadata` call restored all twelve. By the time
the test opened the file, the defect had been undone underneath it.

## The Common Cause

**The verification mechanism modified or bypassed the condition it existed to observe.**

Not bad input. Not an empty subject. In all three the violation was really there, and the act of
checking is what removed it — by reusing work done under other conditions, by depending on an
ambient state that was itself the violation, or by running a tool that repairs the artifact under
test as a side effect of starting.

## Why This Earns A Record

Because of an asymmetry that makes it uniquely expensive: **a predicate in this state cannot
report the problem itself.** An ordinary broken check goes red and somebody investigates. One of
these is green, stays green, and is green *for the wrong reason*, so every signal available says
the thing it guards is fine. Two of the three above were found by checking a `done_when` clause
by hand; the third by reading the build log of a command that had already exited zero. None was
found by anything failing, and none could have been.

It is also self-concealing in a way `OD-GATE-001`'s family is not. Vacuity over an empty subject
is at least visible to anyone who asks how many subjects there were — which is why the vacuity
tracking in the preservation run exists. Here the count is right, the subject is right, and the
answer is still meaningless.

## The Obligation

**A predicate is not established until it has been observed to fail on a real violation.**
Passing on a tree that already satisfies it is not evidence that it can fail; it is consistent
with a predicate that cannot.

Two consequences, both practised in the commits this record was written from.

**Separate the judging from the gathering.** Where the judgment can be a function over content —
lock text, manifest text, a report's bytes — it can be handed a violating input directly and
required to reject it, with no build and no repository state involved. That negative control
answers the question independently of whatever the environment did.

**Treat the environment as part of the subject.** Ask what the predicate's own execution does to
what it measures, and what it inherits from a tree it does not control: a shared target
directory, an auto-discovered configuration, a tool that rewrites its inputs, a cache, a lock, a
previously generated artifact. Where that relationship is load-bearing, the predicate says so and
arranges to be immune to it — an isolated target directory, the committed blob rather than the
working file, an explicit opt-in rather than an ambient default.

`OD-GATE-027` records a circumstance that sharpens this: while remote gate execution is
unavailable, the one observer with no relationship to the tree under test is absent, and two of
the three instances above would have been caught there for free.

## What This Does Not Do

**It does not build an abstraction.** There is no trait, no harness and no generalized
"environment-independent predicate" type, deliberately. Three instances in one session is a
population worth naming and not yet a population worth designing against — the same criterion
`OD-CAPABILITY-002` uses for when a contract earns a crate, and `OD-STORE-001` for when a thing
earns its place: something must have to behave differently, not merely be tidier. What the three
share today is a reading habit, and a record is the right shape for a reading habit.

**It is expected to recur, and naming where is the point of writing it down now.** Cache
validation and incremental analysis both answer from work done under prior conditions. Generated
artifacts and projections are rewritten by the tools that check them — the freshness check already
keeps *stale* and *edited* apart for a closely related reason. Correction staging mutates a tree
and then judges it. Replay asserts that a rerun reaches the same place, which is the whole class
in miniature. When the second or third of those arrives with the same shape, the population may
be worth an abstraction, and this record is what a later reader should be holding when deciding
that.

## Amendment: A Full Population Can Make A Narrowed Consumer's Dependence Unobservable

A fourth instance arrived, and it belongs to this record's class by the same test: the
verification passed, stayed passing, and was passing for a reason unrelated to the property it
was believed to establish.

**What happened.** `NESTING_DEPTH` declared `RequiredFact::LimitsPolicy`. The hand-written guard
deciding whether to materialize that family named five rules and not it. Every test was green,
including tests that ran `NESTING_DEPTH` and judged its findings — because those runs selected
everything, and five sibling rules that *were* named in the guard forced the limits-policy fact
into existence anyway. The rule got its fact, from a demand nobody had recorded it as having.

Narrowed, it did not. Selected without those siblings, `Check_Nesting_Depth` fell back to
`MAX_NESTING_DEPTH`'s built-in default instead of the limit the repository configured, reported
no `MissingCapability` and produced no finding. Narrowing is not an exotic path: it is what
`RuleSelector` and `gate run --rule` give an end user.

**Why this is the same class rather than a neighbouring one.** The other three instances had the
environment repair or supply the condition. Here a *sibling consumer* supplied it. In both shapes
something outside the subject satisfies the subject's precondition, the check cannot tell the
difference, and the green result is about a configuration the narrowed caller never runs in. A
full-suite pass was not evidence about selective execution, and nothing said so.

### The Heuristic

For a dependency several consumers share, three populations rather than one:

1. **The full population** — what a default run does, and what every existing test already covers.
2. **The smallest consumer population** — one consumer alone, which is the only way its own
   declared dependence is actually exercised rather than inherited from a neighbour.
3. **A neighbouring population that does not require the dependency** — without which a
   provider that supplies everything unconditionally passes the first two while making narrowing
   buy nothing.

The second is what was missing. The third is what stops the fix from overshooting.

**Where the shape recurs.** Anywhere a shared precondition has several consumers and one of them
can be selected alone: gate phases, workflow steps whose bodies share a composition, provider
fallbacks where one consumer's requirement pulls in a provider another was relying on, and
correction validation reusing a check run somebody else's selection produced.

**No framework is needed and none is added.** This is the ordinary shape of a negative control,
stated so the next reader recognises the masking rather than rediscovering it — the same reason
this record declines a general abstraction for the three instances above it.

Checked 2026-09-14.
