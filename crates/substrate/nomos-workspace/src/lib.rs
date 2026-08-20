//! Band 18 — what the workspace currently is.
//!
//! Above the document store, below everything that consumes a snapshot. It reads no
//! files: content arrives through a change set, which is what keeps "what the workspace
//! is" a function of what was submitted rather than of what happened to be on a disk when
//! somebody looked.
//!
//! # One door
//!
//! A correction, an IDE save, a git checkout, an agent's edit and a code generator's
//! output are five sources and one entry point: [`WorkspaceChangeSet`]. There is no
//! second mutating method on [`Workspace`] — no `Insert`, no `Remove`, no
//! `Set_Contents` — because a second path would be a second answer to what the workspace
//! currently is, and the two would disagree on the day it mattered.
//!
//! What the door produces is one [`Applied`] outcome, and it has two arms rather than a
//! generation and a `bool`. [`Applied::Advanced`] means the workspace is now something
//! else. [`Applied::Unchanged`] means every change in the set said what the workspace
//! already said — a checkout that landed where you already were, an editor saving a file
//! it did not modify — and advancing the generation for that would invalidate every fact
//! in the store to arrive back at the same answer.
//!
//! # Identity is the state, not the history
//!
//! [`WorkspaceSnapshot::Id`] is a digest of the members, the build variant and the
//! configuration. It is deliberately **not** a digest of the generation. Edit a file and
//! edit it back: the generation has advanced twice and the snapshot identity is what it
//! was, because the workspace *is* what it was. A snapshot keyed on history would call
//! those two states different and recompute a corpus to reach the answer it already had.
//!
//! # Portable
//!
//! A snapshot's bytes name paths relative to the workspace root and content by digest.
//! Nothing in them is a fact about the machine that produced them — no absolute path, no
//! drive letter, no timestamp — so a process with no access to the originating tree can
//! decode one and answer every question it answers. `tests/portable.rs` asserts this over
//! `F:/repos/xvpe` by looking for the corpus root in the bytes, because the failure this
//! prevents is a snapshot that quietly records where it was taken.

#![forbid(unsafe_code)]

// A change's parts sit under `change`, an effect's kind under `effect`, and the change
// set and refusals a workspace raises under `workspace`. Flat, this level was thirteen
// files whose grouping was spelled only in their name prefixes.
mod applied;
mod change;
mod determinism;
mod effect;
mod member;
mod snapshot;
mod variant;
mod workspace;

pub use applied::Applied;
pub use change::{Change, ChangeSet, ChangeSource};
pub use determinism::SnapshotSerialization;
pub use effect::{Effect, EffectKind};
pub use member::Member;
pub use snapshot::{SNAPSHOT_SCHEMA, WorkspaceSnapshot};
pub use variant::BuildVariant;
pub use workspace::{Workspace, WorkspaceChangeSet, WorkspaceError};
