---
name: nomos-spec-change
description: Change specification content safely - read the store, stage an edit, read the preview, commit it, then prove nothing was lost and no projection went stale. Use when editing a governing record or any specification document, when adding a store table, or when a projection or freshness check has gone red.
---

# Changing specification content

`AGENTS.md` is the contract; this is the procedure. The store is the authority, and
`docs/records/ARC-SPECDB-001-the-specification-is-a-database.md` is why.

The reason this is a procedure rather than a command is the failure it guards against. The
previous revision of this specification destroyed 282 table rows, all 6 code blocks and 132
sections of narrative, and the mechanism that would have caught it was present and never
ran. Every step below exists because something was lost while a green run said otherwise.

## 1. Read what is actually there

Two reads, and they answer different questions.

- `nomos spec record --id <node-id>` prints the bytes the store was given.
- `nomos spec markdown --id <node-id>` renders the record back out of the store's own rows.

Read the second one before editing. It is the round trip: if it does not agree with the
first, the store did not hold everything the file said, and your edit will be authored on
top of a loss rather than the cause of one.

Content goes to stdout and everything about it to stderr, so a redirect captures exactly
what the store holds and nothing else.

## 2. Stage, preview, then commit

`nomos spec commit` refuses an edit it has not previewed, and prints the preview it did.
That refusal is the feature; do not work around it.

```
nomos spec preview --id <node-id> --from <file> [--rename <path>]
nomos spec commit  --id <node-id> --from <file> [--rename <path>]
```

Read the preview rather than scanning it. It separates the things that look alike:

- **blocks** changed, added, removed — the count that would have caught the 282 rows;
- **identity** changes — a heading or an id moving is not the same edit as its prose moving;
- **relations** added and removed, which are edges in the graph this system exists to keep
  honest, so a wrong one is worse than a missing one;
- **statements** — normative movement, meaning a requirement changed force;
- **wording moved** and **changes nothing**, which are the two answers that should make you
  stop and check you edited the file you meant to.

An edit refused is exit 9, and a source that is absent is exit 6. Absence is reported as an
absence, never as a shorter answer — `nomos spec sources` says what a store actually holds,
and a corpus that is not on this machine is the usual explanation for a record that is
suddenly not found.

## 3. Prove nothing was lost

The preservation rules are not a command. They are the run in the validation crate, driven
by `crates/spec/nomos-spec-validate/tests/preservation_holds.rs`, and the numbers that
matter come from the real corpus.

Two ways that run lies if you let it:

**A rule that judged nothing reports the same as a rule that judged everything.** The run
tracks which rules were vacuous for exactly this reason. A summary is not evidence unless
you have looked at whether the subjects were there.

**The corpus lives outside this repository and CI has none of it.** A preservation test
that cannot find its corpus returns early and prints `ok`. So a green pull request is not
evidence that a corpus-backed claim was checked; it is evidence that nothing contradicted
it on a machine that could not look.

Set the corpus variable and re-run locally before believing a preservation result.

## 4. Prove no projection went stale

`nomos spec render --profile <id> --into <directory>` writes a body and a sidecar stamping
what produced it and from what. `nomos spec freshness --into <directory>` compares the two
back and keeps two failures apart that share exit code 8: the store moved under an
unchanged body (**stale**), and somebody typed into the body (**edited**).

A body with no sidecar beside it is a failure rather than a skip. Deleting the sidecar is
otherwise how an edit stops being caught, and a check that teaches that trick is worse than
no check.

Never hand-edit a rendered body. If the text is wrong, the store is wrong.

### Render from the record set your commit will publish

Not from your working tree. `OD-GATE-005` is why, and it is the step whose absence left the
gate red for eleven commits.

The store is assembled from `crates/spec/nomos-spec-store/records/` **on disk**, so a binary
built in the shared tree holds whatever every live session has lying in it. Measured on
2026-08-10: 50 records in the working tree against 49 at `HEAD`, the difference being one
peer's unlanded file. Render there and you commit their unpublished work into an artifact CI
rebuilds without it, and the gate fails on *your* commit.

The rule is exact in both directions:

> The tree you render in holds exactly the registered records your commit will publish — no
> more, and no fewer.

"Everything is committed" fails it twice over. It is satisfied by a tree carrying a peer's
uncommitted record, and it is *not* satisfied by the tree you are in when you add a record of
your own, which must be on disk or the projection is stale the instant you commit.

Construct that tree rather than waiting for one:

```
git worktree add --detach <scratch>/render HEAD
cp docs/records/<your-record>.md          <scratch>/render/docs/records/
cp crates/spec/nomos-spec-store/records/<ID>.record <scratch>/render/crates/spec/nomos-spec-store/records/
cd <scratch>/render
CARGO_TARGET_DIR=<scratch>/render-target cargo build -q --bin nomos
<scratch>/render-target/debug/nomos spec render    --profile diagram-set           --into .
<scratch>/render-target/debug/nomos spec render    --profile domain-specification  --into .
<scratch>/render-target/debug/nomos spec freshness --into . --require diagram-set --require domain-specification   # must exit 0
```

**Both required profiles, every time.** The gate requires `diagram-set` and
`domain-specification`, so rendering one and not the other leaves a stale output and the gate
fails on your commit for the half you skipped. `freshness` with both `--require` flags is what
tells you before you push; run it with the same pair the gate uses, which
`.github/workflows/gate.yml` holds.

Then copy **all four** halves back — each body and the sidecar beside it:
`diagrams/relations.mmd`, `diagrams/relations.mmd.nomos-projection.json`,
`spec/domain-specification.md` and `spec/domain-specification.md.nomos-projection.json` — and
commit them with the record that moved them. Build in the worktree: a binary built in the
shared tree has the wrong records compiled into it, which is the whole point.

Two things follow that surprise people:

- **`spec freshness` in your working tree is not the verdict.** It reports stale over a
  correct file whenever a peer holds an unlanded record, so exit 8 there is not evidence you
  rendered wrongly. The clean worktree and CI are the trees where the question is well posed.
  Do not plan on waiting for the shared tree to go quiet: polled every minute for an hour on
  2026-08-10, while seven commits landed from three sessions, it never once did.
- **`work finish` does not run this check**, deliberately, for the same reason. Adding a
  governing record is one of the few changes whose obligation the ledger cannot enforce for
  you.
- **If your item's own predicate is a projection check, run `finish` from the worktree**, with
  `NOMOS_WORK_DIR` pointing at the real `work/` so the transition lands on the shared board.
  The worktree is byte-for-byte what CI checks out; the shared tree would answer about a record
  set nobody will publish. Do not wait for the shared tree instead — a session that lapses
  leaves its unlanded records stranded there, and no one else can clear them.

Every edit to a record body changes the store, so **render after the last word is written**, not
before. A record you touch again is a diagram you render again.

### The diagram is nobody's territory, and you may re-render it

`diagrams/relations.mmd` is reserved by no item and re-rendering it is not a territory
violation. It is a derived output: two authors cannot disagree about its contents, only about
which record set it was taken over, and the later render subsumes the earlier one. Reserving
it in every record writer's territory was refused because it would serialize the whole board
on a file nobody can conflict over — `OD-GATE-005`, decisions 1 and 2.

So do not reserve it, and do not leave it stale because you did not.

## 5. If you are adding a store table, budget the bundle

A new table is a bundle change, because the bundle is the authority committed to git and a
table that does not travel in it is content a rebuilt store cannot hold. The table list in
the store has a test comparing it against the schema both ways, so that half is loud. The
other half is in `crates/spec/nomos-spec-bundle` and none of it is reachable from the crate
you changed: the column coverage list, an exporter, an importer, both driver lists, and a
round-trip fixture that must insert at least one row per table.

Expect the crate you edited to stay green through all of it.

## 6. A record of this repository is not governing until it is registered

Writing the document under `docs/records/` does not make it govern anything. The
registration file under `crates/spec/nomos-spec-store/records/` does, and the two are
compared against each other both ways — so a record that stops being registered fails a
test rather than quietly leaving the store.

Reserve both in your item's territory before you start.

Reserve those two and no more. The diagram your record moves is deliberately not a third —
see step 4 — and adding it would refill a register `OD-LEDGER-011` spent two items emptying.
