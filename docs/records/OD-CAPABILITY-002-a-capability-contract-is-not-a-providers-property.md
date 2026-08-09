---
id: OD-CAPABILITY-002
type: decision
title: A capability contract is not a provider's property, and a contract earns a crate when a second party exists
status: closed
version: 1
authority: canonical-normative-record
tags:
  - capability
  - layering
relations:
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-STORE-001
    type: relates-to
---

# A capability contract is not a provider's property, and a contract earns a crate when a second party exists

## Question

`nomos.cap.syntax.items` was declared by `nomos-lang-rust` and offered by both it and
`nomos-lang-rust-scan`.

A `CapabilityContract` is the agreed meaning of a question and the ceiling on what any answer
may claim. An agreement is not the property of one party to it. Where does the contract live,
and does that need a crate?

## What Was Actually Wrong

Three things, and only the first is the one the item was named for.

**The ceiling belonged to a party.** The ceiling bounds what *either* provider may claim.
While it lived in `nomos-lang-rust`, that crate could raise or lower what its peer was
permitted to promise, in a file the peer cannot open — two providers of one capability sit at
the same band and `tests/contract` forbids the edge between them, deliberately, so that the
second answer cannot be derived from the first.

**The agreement was held by retyping.** `nomos-lang-rust-scan` agreed with its peer by
spelling four constants again: the capability id, the contract version, the schema, and the
payload format. Its own doc comment said so and called it the design. Three of those four are
not the design — they are the terms of the agreement, and two parties who agree because
somebody retyped a string agree until somebody retypes it differently. Nothing would have
noticed.

**The invariant was held by the wrong thing.** What actually stopped the scanner from
declaring its own terms was `Registry::Declare` refusing a second contract for one capability.
That is the registry compensating for the layering rather than the layering being right, and
the difference shows the moment a composition declares the scanner's contract first.

## The Answer

`nomos-cap-syntax`, at band 23: above `nomos-capability`, which resolves contracts, and below
both providers, which offer against one.

**What is there** is what the parties agreed — the capability's identity, the contract
version, the ceiling, the summary that says what the question means, and the schema every
answer is stamped with. A schema is the shape of an answer rather than a claim about its
accuracy, and two providers writing different shapes would force every consumer to know which
one answered, which is the thing resolving through a registry exists to avoid.

**What is not there** is anything a provider claims for itself. A `ProviderId` is a provider's
own name and a `Guarantee` is its own claim — bounded by the ceiling below and checked against
its output by its own tests. A contract that also stated what each provider promises would be
grading their work for them, which is the defect the ceiling exists to prevent, one level up.

## The Criterion: When A Contract Earns A Crate

**When more than one party names it.**

A capability with a single provider is not wrongly filed for living beside that provider.
`nomos.cap.module.surface` is declared and offered by the same rollup in `tests/integration`,
and moving it out would buy a crate and no property — there is no second party for the first
one to overrule.

The moment a second provider exists, ownership by one party stops being untidy and becomes
false: the first can change the ceiling, the version or the schema its peer is bound by, and
the peer cannot see the file. Contention is the criterion, not principle. That is deliberately
the same shape as `OD-STORE-001`'s — a thing earns its place when something must *behave*
differently, not when a diagram would look tidier — and it is the part of this record worth
carrying forward.

It also bounds the growth. Crates appear per *contended* capability, which is a much smaller
number than per capability, and each one is created in response to a second provider rather
than in anticipation of one.

## What Holds It

**`Test_A_Capability_Id_Should_Be_Written_In_One_Crate`**, in `tests/contract`, is the
general form: no `nomos.cap.…` id may appear in more than one workspace member's `src`. It is
not written against this capability — it would have caught the original arrangement, and it
will catch the next one. Scoped to `src` because a declaration is library code; a test may
name any capability it likes, and `tests/integration` asserts over
`nomos.cap.syntax.items of alpha/one.rs` without being a party to anything.

**The band table** places 23 below both providers, and the existing downward rule does the
rest. Setting the home level with the providers fails it, which is what makes the entry
load-bearing rather than decorative.

Cargo turns out to hold half of this on its own: a contract crate that depends on a provider
that depends on it is a cycle, and cargo refuses to build it at all. That is worth knowing
because it means the band rule is guarding the case cargo cannot see — a sideways edge, or a
contract crate that reaches a provider naming it only by string.

**`Test_Both_Providers_Should_Offer_Against_A_Contract_Neither_Declares`**, in
`tests/integration`, is where all three crates are visible at once, which is the only place
they can be: the contract cannot name a provider and neither provider can name its peer. It
also asserts the ceiling is not equal to either provider's guarantee, because a ceiling that
restates the incumbent's claim is a ceiling that has to be raised whenever somebody improves
something — and one that silently forbids a better second provider.

## What Was Considered And Rejected

**`nomos-contracts`, the protocol crate.** A capability contract is an agreement, and that
crate is where agreements peers reimplement live. Rejected on two counts: the
`CapabilityContract` type is `nomos-capability`'s, so band 0 cannot hold a value of it without
a dependency `Test_Contracts_Should_Depend_On_The_Allowlist_And_Nothing_Else` exists to
forbid; and a capability is a statement about this product's analysis rather than about what a
fact *is*.

**`nomos-capability` itself.** The registry must not know which capabilities exist — it
resolves whatever is declared, and a substrate crate naming `nomos.cap.syntax.items` is
upward knowledge. This was not hypothetical: `nomos-capability`'s own test fixtures spelled
the real capability id, and they now use `nomos.cap.test.items`, which is what a registry test
should have been using all along.

**Leaving it and adding a comment.** The scanner already had the comment. It named the
problem, named the item that would fix it, and was correct in every particular — and the
coupling it described was still a retyped string.

**Moving the payload codec too.** Each provider encodes the shared schema independently, and
that is not the same defect: two implementations of one shape can disagree and be caught,
which is the point of having two providers. The schema *id* is the agreement and has moved;
the encoders have not. Whether two independent encoders of one schema is a seam worth closing
is a separate question, and closing it by construction would remove a disagreement this
workspace deliberately wants observable.

## Status

Closed by P8-CONTRACT-HOME. Three controls were confirmed red: the peer's capability id
retyped into the scanner (the workspace-wide id test), the contract's band set level with its
providers (the downward rule), and a ceiling set to the parser's own guarantee (the
composition's ceiling assertion). A fourth was attempted — the contract crate depending on a
provider — and cargo refused to build it, which is recorded above rather than counted.
