---
id: OD-RELEASE-001
type: decision
title: Main and dev mean different things, and the distance between them is a decision, not an accident
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - repository
  - release
  - agents
relations:
  - target: OD-AGENT-002
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
---

# Main and dev mean different things, and the distance between them is a decision, not an accident

## Question

Every commit in this repository lands on `dev`, and `dev` reaches `main` only by a merge.
No merge has happened since the Phase-0 state in August, and the distance since is now the
entire system: every crate, every governing record, every gate check this repository has.
The gate already runs on `dev`, so the risk is not that a broken branch is unwatched — it is
that `main` no longer means anything a reader could rely on, and the first merge across that
distance is itself a significant, untested event with no stated criterion for when it should
happen or who is entitled to make it happen.

That absence is what this record closes. Not by merging anything — see *What This Does Not
Do* — but by stating what `main` is for, what makes a `dev` commit eligible to reach it, and
who decides.

## What main is for

`main` is a published milestone, not a mirror of `dev`. `dev` is where this repository's own
continuous, largely agent-driven development happens — the ledger, the gate, and every
record in this session's own history describe a tree that changes commit by commit, often
several times an hour, across concurrent sessions. `main` is the answer to a different
question: what state of this repository is a person willing to stand behind as a coherent,
gate-clean whole, at a moment they chose. A clone of `main` should never need `dev`'s own
commit log to explain what state it is in.

## What makes a dev commit eligible

Two conditions, both mechanical:

1. **The gate is green at that exact commit**, checked in a clean worktree or a fresh clone —
   never against the shared working tree, for the identical reason `docs/records/
   OD-GATE-005-a-derived-projection-is-owned-by-nobody-and-is-rendered-from-the-record-set-
   its-commit-publishes.md` gives for a spec render: a tree that can hold a peer's unlanded
   work answers a question about that peer, not about the commit being considered. `.github/
   workflows/gate.yml` is the gate this criterion means — the full pipeline, not the narrower
   predicate any one ledger item happens to run.
2. **The board holds no claim whose territory the merge would touch.** A merge that lands
   while an item is mid-claim on a file the merge also carries is the same "widen another
   holder's territory" hazard `AGENTS.md` already names for a single commit, at the scale of
   every file `dev` and `main` disagree on at once.

Green at the tip of `dev` is necessary and not sufficient by itself: the gate can go green
again on the next commit after a real regression, and a merge is a decision about the
commit chosen, not a standing subscription to whatever `dev`'s tip happens to be when
somebody looks.

## Who decides

**A person decides to merge `dev` into `main`.** An agent may verify the two conditions
above and prepare the merge — confirm the gate is green in a clean checkout, confirm no
claim overlaps it, stage the fast-forward or merge commit for review — but authorizing the
push to `main` itself is not delegated. This is the same category of action this
workspace's own agent-facing guidance already treats differently from an ordinary commit to
`dev`: it is hard to reverse once anyone has pulled it, it is visible to and relied on by
whoever treats `main` as the citable state of this repository, and unlike a `dev` commit,
there is no `git revert` on `dev` that would undo what a reader already did with a stale
`main`. Every other decision this repository's records make is answerable by an agent
because the ledger and the gate can check it mechanically after the fact; whether `main`
should move is answerable only by whether a person is willing to stand behind it, which is
not a mechanical question.

## What This Does Not Do

It does not merge `dev` into `main`. The distance measured in the *Question* section above
is exactly as large after this record as before it; closing that distance is a person's own
next action, informed by this record's criteria, not an action this record or the item that
produced it takes on their behalf.

It does not introduce a release process beyond the one question it answers — no version
numbering scheme, no changelog format, no cadence. Those are real questions this repository
will eventually need answered, and each is its own decision when a real case asks it, per
`OD-ROADMAP-001`'s own distinction between retiring a wait and retiring engineering
judgment: this record answers the one question that was blocking the *first* merge, not
every question a mature release process would eventually have.

It does not change how `dev` itself is developed. Every ledger verb, every gate step, every
concurrent-session convention this repository already runs stays exactly as it is; this
record is entirely about the one door between `dev` and a branch nothing currently reads.

## Status

Accepted. `main` is a published milestone; a `dev` commit becomes eligible only through a
clean-checkout green gate and a claim-free merge, and only a person authorizes the push.
