---
id: OD-PLATFORM-004
type: decision
title: A local patch override is a development convenience and never the governing form of the XVPE crossing
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - platform
  - dependencies
  - determinism
relations:
  - target: D-130
    type: relates-to
  - target: OD-PLATFORM-003
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# A local patch override is a development convenience and never the governing form of the XVPE crossing

## Question

`D-130`'s surviving clause requires the XVPE crossing be adopted by git reference and commit,
and the tree now takes that form: sixteen dependencies across eight workspace members name one
repository at one revision, and `Cargo.lock` records it.

Pinning removed something real. An edit in the sibling checkout no longer reaches a build here
without a commit, a push and a revision bump across eight manifests. An uncommitted Cargo
`[patch]` override restores that loop by redirecting those crates back to the checkout. (It was
first placed at `.cargo/config.toml`, which Cargo discovers by itself. The amendment at the end
of this record moves activation from that to an explicit request, for reasons measured after
this section was written.)

The override is useful. It is also the exact mechanism the pinning was adopted to remove,
pointed the other way. What authority does it have, and how does a reader know one is active?

## Decision

**The pinned dependency is the only governing form. An override carries no authority.**

Four clauses, and the third is the one that matters.

**It may redirect a crate only to the same crate.** An override that changes which crates exist,
or points a name at a different body of code, is not a convenience; it is a different workspace
wearing this one's manifests.

**It is never committed.** `.gitignore` excludes it, so a scoped `git add` cannot sweep it up.
That is mechanical rather than a matter of care, because this tree is shared with live sessions
and an untracked file in it is one command away from being published as though it were the
decided form.

**A result measured under an active override has not been measured.** This is the whole reason
the record exists. A build with the override active is a build against whatever the sibling
checkout is sitting at, which is precisely the ambient state `D-130` refuses — so a green run,
a benchmark, a corpus count or a gate result taken under it is evidence about a tree nobody
else has, including CI. Before any claim rests on a number, move the file aside and rebuild.
This is the same failure shape this repository already writes records about: an exit code of
zero that answered a different question from the one asked.

**It explains itself where it sits.** The file opens with what it is and what it is not,
because the reader most likely to be misled by it is the one who did not create it.

**`Cargo.lock` is restored before any commit that carries it.** This clause is not a
precaution; it is a measured consequence, and it is the sharpest edge on the whole mechanism.

## What The Override Does To `Cargo.lock`

Cargo records a patch in the lock file. Measured directly on 2026-09-14: with the override
active, a single `cargo metadata` call rewrote the tracked `Cargo.lock`, deleting the
`source = "git+https://github.com/kevinmettias/xvpe.git?rev=..."` line from **all twelve**
`xvpe-` packages. Nothing else in the file moved and no command reported anything.

So the lock, which is the artifact that makes the adopted revision a recorded fact rather than
an ambient one, is turned back into the un-pinned form by the mere act of building. The result
is a tracked file, so `.gitignore` cannot protect it the way it protects the override itself,
and a scoped `git add Cargo.lock` — the exact discipline `AGENTS.md` asks for everywhere else
— is what publishes the damage.

Two consequences follow, and the second is the one that outlives this record.

**Before committing, park the override and restore the lock.** Moving the file aside and
running `git checkout -- Cargo.lock` returns all twelve sources. Doing this is also what makes
the commit's own verification honest, since a predicate run under an active override is a
predicate run against a tree nobody else has.

**A convention is not sufficient here and a guard is owed.** Every other clause in this record
fails safely: a wrong override breaks a local build and the person who wrote it finds out. This
one fails silently, in the committed state, in the direction of the thing `D-130` exists to
prevent. The mechanical half — an assertion that every `xvpe-` package in `Cargo.lock` carries
a git source — does not exist yet and is not written here, because this record's territory is
the status of the override rather than the guard over the lock.

## How To Tell Which Form Is In Effect

Read the `source` of any `xvpe-` package. It is unambiguous in both directions, and checked
both ways before being written here:

```
cargo metadata --format-version 1 | python -c "import json,sys; print([p['source'] for p in json.load(sys.stdin)['packages'] if p['name']=='xvpe-primitives'])"
```

A `git+https://...?rev=...` string is the pinned form — what the repository publishes and what
CI builds. `null` means an override is redirecting that crate to a local path, and no result
from that build is evidence about the published tree.

## Why This Is Not Just A Convention

A convention would be enough if the override announced itself. It does not. It changes nothing
a reader sees: the manifests still name a git revision, `Cargo.lock` still records it, and the
build still succeeds. The divergence is visible only in resolved metadata, which nobody reads
by habit.

It is worse in a shared tree. `OD-LEDGER-001` already records that territory is declared rather
than enforced, and the same asymmetry applies here one level down: a file one session drops in
silently changes what every other session in this working tree builds against, with nothing in
any of their outputs saying so.

## What This Does Not Decide

Whether the loop the override restores should be paid for some better way — a vendored
checkout, a workspace-level source replacement, a scripted revision bump — is open. This record
decides the status of the mechanism actually in use, not that it is the best one available.

It also does not reopen `D-130`. The pinned form is unchanged and unweakened, and nothing here
licenses a path dependency in a manifest.

## Amendment: Activation Becomes A Request Rather Than A Default

Nothing this record decides changes. Local substitution is still permitted, it still carries no
authority, and the pinned dependency is still the only governing form. What changes is when it
takes effect.

**The defect was the filename.** `.cargo/config.toml` is a path Cargo discovers on its own, so
the substitution applied to every command anyone ran anywhere in this tree — including a
concurrent session's, which had not asked for sibling development and had no way to know it was
getting it. The clauses above are all written as though a person decides when the override is
active. With that filename, nobody did.

The consequence was not theoretical. Measured twice on 2026-09-14, an ordinary command stripped
the git source line from all twelve `xvpe-` packages, and at one point this working tree stood at
zero of twelve pinned with the lock guard failing. The guard did its job; it is a backstop, and
a backstop catching something routinely is evidence about the thing in front of it.

**The reproducible form is what a build now falls into.** The override lives at
`.cargo/xvpe-local.toml`, a name Cargo does not look for, and is requested by naming it:

```
cargo --config .cargo/xvpe-local.toml check --workspace
```

Checked both directions on 2026-09-14. An ordinary `cargo metadata` and an ordinary
`cargo check` leave `Cargo.lock` byte-identical with all twelve sources pinned; the same commands
under `--config` resolve `xvpe-primitives` to a local path. Sibling development is now something
asked for rather than something arrived in.

**What did not improve.** An opt-in build still rewrites `Cargo.lock`, deleting the source line
from all twelve packages exactly as before — measured, not assumed. That is a property of Cargo's
patching and no filename fixes it. `git checkout -- Cargo.lock` afterwards, and the guard in
`tests/contract/tests/boundaries/lock_pinning.rs` remains as defense in depth rather than as the
primary control. The difference this amendment buys is narrow and worth stating plainly: the
damage can no longer happen to somebody who did not ask for it.

**A build resolved this way is development-resolved, not reproducible**, and that term is now the
one the override's own header uses. It resolves against whatever the sibling checkout is sitting
at, which no peer and no CI can reproduce, so the third clause above applies to it unchanged: a
result measured that way has not been measured.

`.gitignore` excludes the whole `/.cargo/` directory rather than one filename. Ignoring only the
override's own name would leave `config.toml` free to be recreated, which is the shape this
amendment exists to remove.

Checked 2026-09-14 against the override's own header, `.gitignore`'s exclusion of the whole `/.cargo/` directory, and the lock-pinning guard this amendment leaves in place.
