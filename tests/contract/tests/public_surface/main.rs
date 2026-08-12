//! Each crate's public surface, checked in and compared.
//!
//! The fifth anti-leak assertion, and the one that was never built. Three are in
//! `boundaries.rs`, module reachability shipped, and the transport check has no transports
//! to check. This is the remaining one: a type that leaks out of a crate's surface was a
//! review question, and the other four exist precisely because review questions are the
//! ones answered wrong quietly.
//!
//! # How it fails
//!
//! Widening a surface fails this suite until the snapshot beside it is updated. That is the
//! point: `git diff` then shows the new export as a line somebody added to a committed
//! file, in the commit that added it, rather than as nothing at all.
//!
//! To update, run the suite with `NOMOS_SURFACE_BLESS` set to the crate whose snapshot you
//! meant to change — `NOMOS_SURFACE_BLESS=nomos-rules`, or several names separated by
//! commas. It rewrites those snapshots, leaves every other one alone byte for byte, and
//! then *fails*, so a machine with the variable left set cannot report green — a bless that
//! passes is a check that anybody's environment can switch off.
//!
//! # Why it makes you name the crate
//!
//! Because it once did not, and this tree is worked by several sessions at a time. Blessing
//! used to rewrite every snapshot, and rewrite it from the *working* tree: a snapshot
//! carrying somebody else's deliberate, uncommitted change was not reverted to `HEAD`, it
//! was overwritten with whatever their half-finished crate happened to export — and the
//! result still passed this suite afterwards. That happened, between two live claims, and
//! the only reason the work survived is that both authors happened to look. Nothing in the
//! mechanism looked. It is a territory violation committed by a tool rather than by an
//! author, and no ownership rule can express it, because the tool is a test that everybody
//! else also has to run.
//!
//! So the widest action is no longer the easiest one to reach for: there is no spelling that
//! means "all of them", only the list of names you are willing to write down.
//!
//! A value that names no crate — `NOMOS_SURFACE_BLESS=1`, which is what "set to anything"
//! used to look like — is refused, loudly, with the names it could have used. Refusing is
//! not politeness: rewriting nothing while appearing to have been honoured is the same
//! defect as rewriting everything, wearing the other sign. A name that matches no workspace
//! member is refused for the same reason. [`bless`] holds that machinery and the two tests
//! that hold it to its word.
//!
//! # Where the snapshots come from
//!
//! `nomos_contract_tests::Public_Surface`, which reads the crate's own source. Not
//! `cargo public-api`: that tool is on no CI runner here and would need a nested `cargo`
//! invocation, which blocks on the target-directory lock held by the `cargo test` that
//! called it. `OD-GATE-002` records the decision and what it gives up.
//!
//! [`bless`]: crate::bless


mod bless;
mod common;
mod scanner;
mod snapshots;
