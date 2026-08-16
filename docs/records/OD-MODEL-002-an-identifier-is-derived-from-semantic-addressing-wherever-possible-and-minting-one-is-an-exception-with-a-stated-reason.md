---
id: OD-MODEL-002
type: decision
title: An identifier is derived from semantic addressing wherever possible, and minting one is an exception with a stated reason
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - model
  - identity
  - contracts
  - reproducibility
relations:
  - target: OD-MODEL-001
    type: relates-to
  - target: D-137
    type: relates-to
  - target: ARC-SPECDB-002
    type: relates-to
---

# An identifier is derived from semantic addressing wherever possible, and minting one is an exception with a stated reason

## Question

`crates/contracts/nomos-contracts/src/identity.rs` decides what an identifier looks like:
every identity in Nomos is one of two shapes, a content digest or a human-authored string
with a stable spelling, and neither is ever a path and a line. That module doc is settled
and this record does not restate it, let alone amend it.

It does not settle what goes into a digest, or what a human-authored spelling is composed
from, and those two constructions fail differently. A digest taken over the semantic
address of the thing it names is reproducible: the same subject, addressed the same way on
a different machine or a different run over the same corpus state, produces the same
bytes. A digest taken over whatever the constructing code happened to be holding at the
time is not — it has the right shape, it compiles, it passes every type check the newtype
gives it, and it produces a different value the next time the same subject is addressed.
That is the join-with-no-key failure `identity.rs`'s own module doc names as the
prototype's defining mistake, recurring under a `Digest128` that looks like a fix for it.

Nothing in the crate distinguishes the two constructions. There is no rule that an
identifier is derived where derivation is possible, no statement of what counts as a
semantic address, and no obligation on a caller that mints an opaque one instead to say why
derivation was unavailable for that subject.

`P10-CANONICITY-DECISION` is not this question, and `ARC-SPECDB-002` is the record that
answers what it actually asked: whether an object's *content* is canonical as a structured
record or as a document, decided by whether a schema for it could be written before the
object existed. That is a question about where a thing's bytes live. This record is about
a different axis entirely — once a thing's bytes exist, on either side of that line, what
bytes may go into the identifier computed over it. The two questions share no evidence and
settling one says nothing about the other.

## The Rule

> An identifier is **derived** wherever a semantic address exists for the thing it names.
> Its bytes are computed from that address, and computing them again over the same address
> produces the same identifier, on any machine, on any run, from any clone of the same
> corpus state.
>
> **Minting** an identifier — assigning it a value not computed from the thing's own
> semantic address — is the exception, not an equally available second choice. A caller
> that mints one owes, at the site of the mint, a stated reason that the subject was not
> derivable: what about it resists a semantic address, not merely that deriving one was
> inconvenient at the time.

The obligation is a sentence at the construction site, not a new field on the identity
type. `identity.rs` names two shapes and this record does not add a third; it constrains
how a caller of either shape is allowed to arrive at a value. A `Digest_Identity` computed
by hashing a semantic address is derived; one computed by hashing anything else is a mint
wearing a digest's clothes and owes the same reason a mint of a `Named_Identity` does.

## What Semantic Addressing Means

**Included.** A semantic address is bytes drawn from what the subject *is*, not from when,
where, or by what process it was addressed. Concretely, in this workspace, a derived digest
is built from: a normalized, repository-relative path and nothing else
(`nomos_model::Subject_Of_Path`, over `Normalize_Path`'s output); a declaration's language,
fully qualified name, signature, structural shape hash and declaring repository
(`CompositeIdentity::Entity_Id`, `crates/kernel/nomos-model/src/identity/composite_identity.rs`);
or a workspace state's build variant, resolved configuration digest, and the sorted content
digests of its members (`WorkspaceSnapshot::Id`, over `WorkspaceSnapshot::Encode`). Each of
these is a function of the corpus and nothing else — run it twice over an unchanged corpus
and the bytes fed to the hash do not move.

**Excluded**, because each one differs between two machines or two runs over the identical
corpus state, and including any of them turns a derived digest into a mint that still
carries a digest's shape:

- **machine-local inputs** — a hostname, a process id, an absolute filesystem path outside
  the corpus root, an environment variable, a user or account identifier, anything read
  from the executing machine rather than the subject;
- **run-local inputs** — a wall-clock timestamp, a randomly generated value (a `UUID v4` or
  any other RNG output), an in-memory address, a sequence number issued by the running
  process rather than recovered from the subject's own content.

`WorkspaceSnapshot::Encode` already states the general form of this exclusion for one case:
its own doc comment says the generation a state was reached in is deliberately absent from
the encoded bytes, because including it would make one file edited and edited back into a
third, distinct state. `GenerationId` (`identity.rs`'s own `generation_id` module) is not
one of the two identity shapes for the same reason — it counts a workspace's history and
is never itself hashed into an identifier. That exclusion, so far implicit at one call
site, is the general rule this record names: an identifier answers "which thing", never
"when was it looked at" or "on what machine".

**For a `Named_Identity`, the same test in its own vocabulary.** A spelling is derived when
it is composed from the subject's own domain vocabulary — a capability's namespaced name, a
schema's declared name, an operation's canonical name, a rule's identifier as its author
assigned it in the rule's own definition. It is minted when the value was assigned by an
authority other than the subject's own semantic address: chosen by a person with no naming
rule behind the choice, or handed over whole by an external system Nomos does not compute
from bytes it holds. `D-137`'s `KnowledgeReferenceId` is the accepted instance of the
latter, and it already states the reason this record asks every mint site to state:
"Nomos does not compute this value from bytes it holds, so wrapping it as one of Nomos's
own content digests would claim a verification Nomos cannot perform." That sentence is the
shape the obligation takes; this record generalizes the obligation, not the sentence.

## Scope

The rule covers every entity family whose identity is or will be a `Digest128` or a
`Named_Identity` in `identity.rs`'s sense — the identity module states the two shapes this
record binds, and is cited rather than repeated here. Concretely, at minimum: work items,
requirements, decisions, runs, packages, projections, and external artifacts, alongside the
families `identity.rs` already names — analysis subjects, pinned workspace states,
canonical entities under a snapshot, build variants, and resolved configurations. Deciding
the question once per crossing, instead of once here, is how it would end up answered
differently for each of these; that is the failure this record exists to close off before
a second family reopens it.

It does not extend to `GenerationId`, which is not one of the two shapes to begin with, and
it does not relitigate `KnowledgeReferenceId`, whose mint is already justified by `D-137`
in exactly the form this record asks for generally.

## What This Does Not Decide

Not which substrate is canonical for an object's content — `ARC-SPECDB-002` answers that,
on a different axis, and this record leaves it untouched.

Not a third identity shape, and not a change to any accessor or derive on `Digest128`,
`Digest_Identity`, or `Named_Identity`. `identity.rs` is unchanged by this record.

Not a retroactive audit of every current mint site. `RunId` and `PackageId` are declared in
`identity.rs` and constructed nowhere in the workspace today; this record binds whoever
constructs one next, and does not by itself require anyone to revisit a site that predates
it.

## Status

Accepted. It states the rule, what counts as a semantic address on both identity shapes,
and the scope of entity families it binds. It decides no particular caller's construction
site; the obligation lands on each one as it is written.
