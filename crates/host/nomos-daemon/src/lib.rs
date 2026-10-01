//! Zone: Host. The process that outlives an invocation.
//!
//! Every `nomos check` and every `nomos gate run` builds a `nomos_workspace::Workspace`, a
//! `nomos_analysis::MemoryFactStore` and a `nomos_check_orchestration::RuleReassessmentCache`
//! from nothing, judges once, and exits. The incremental machinery underneath all three is
//! real -- every non-syntax family proves its fact current before filing it, and a rule whose
//! declared families did not move is not re-judged -- and no shell invocation can reach any of
//! it, because nothing survives long enough to have a second call. [`Resident`] is the thing
//! that survives: one root, held, answering requests.
//!
//! # What it holds, and why that is a cache rather than privileged state
//!
//! Exactly the three values `nomos_check_orchestration::Run_Reassessing` takes as
//! caller-supplied parameters -- the workspace, the fact store and the reassessment cache --
//! plus the root it was started against and how many requests it has answered. `OD-HOST-002`'s
//! test is whether any fact would be lost that no other client could get back by calling the
//! same canonical service, and the answer is no in the strongest available form: dropping all
//! three is a request a client can make ([`ResidentRequest::Forget`]), and a resident that has
//! dropped them answers what a resident that never held them answers. Nothing here is a second
//! copy of the system's state, because the seam recomputes each of the three from the tree.
//!
//! The two lifecycle values are not domain facts at all. How many requests a service has
//! answered and whether it has been asked to stop describe the service's own life, which is
//! what `OD-HOST-002` separates out as view state: no canonical service could hold them,
//! because no other client would need them to answer the question this one answers.
//!
//! # What it watches, and how
//!
//! **What:** every file under its root whose extension the shared walk recognizes -- a
//! registered language package's own extensions plus `check-script-discipline`'s script
//! extensions, which is the same set `nomos-cli::check` walks and is why a judgment from here
//! and a judgment from a cold invocation are over the same population.
//!
//! **How:** by re-walking at request time and comparing each walked file's content against
//! the digest the held workspace already carries for that path -- `nomos_workspace::Change`'s
//! own `Content_Digest` against `nomos_workspace::Workspace::Content_Of`, so the comparison is
//! the workspace's own and not a second digest kept beside it. There is no filesystem watcher
//! and no background thread: a resident notices a change when it is asked, never before.
//! `nomos-platform` has no watch port, and a poll at request time cannot miss a change that
//! happened before the request, which is the only change the answer to that request has to
//! reflect. What it costs is that a client wanting to be *told* has to ask.
//!
//! **What it does not watch:** everything else under the root. Those are not served from the
//! cache either, and the reason is not this crate's: every provider that reads for itself --
//! `cargo metadata`, `cargo clippy`, `cargo deny`, the repository's own policy files, the
//! committed requirement registry -- runs again on every request, and
//! `nomos-check-orchestration`'s currency check compares the answer it produced against the
//! fact the store is serving rather than against a remembered input. A moved input produces a
//! different fact, the write happens, and every rule declaring that family is re-judged. So a
//! resident serves nothing staler than a cold invocation would; what it cannot do is *state*
//! that an unwatched input did not move, and it does not claim to.
//!
//! **A change it cannot see at all** is one that takes the root away. A root that stops being
//! readable is refused ([`ResidentRefusal::RootIsNotWalkable`]) rather than answered from the
//! findings the resident is still holding, which is the one case where a cache could have been
//! passed off as an observation and is not.
//!
//! # Concurrency: a residency is confined to one thread, and requests to it serialize
//!
//! Decided on a measurement rather than assumed, and the measurement is the compiler's. A
//! [`Resident`] is not `Send`, and the reason belongs to the store and not to this crate:
//! `nomos_analysis::MemoryFactStore` holds its dependency-propagation strategy as a
//! `Box<dyn DependencyPropagation<FactSlot>>` with no `Send` bound, so a store cannot cross a
//! thread boundary and neither can anything holding one. This was established by writing the
//! shared-behind-a-lock arrangement and being refused: `Arc<Mutex<Resident>>` does not
//! compile, and the error names that box.
//!
//! So two requests to one resident do not merely fail to interleave because [`Resident::Answer`]
//! takes `&mut self` -- they cannot be concurrent at all, and the sequence each one sees is the
//! state the one before it left. A caller that wants concurrency starts a resident per thread.
//! Those share the root safely, because a resident reads the tree and writes nothing to it,
//! anywhere, ever: there is no shared mutable state between two residencies and each keeps its
//! own cache. What crosses a thread boundary is the answer, not the resident. All three are
//! asserted in [`resident`]'s own tests rather than argued here.
//!
//! # Why no transport ships with this
//!
//! A resident is a library, the same shape `nomos-api` is and for a sharper reason. What this
//! workspace serves over a wire is `nomos-api-transport`'s, by `OD-HOST-013`, and a Host may
//! not name another Host -- so a wire protocol added here would be both a second serving
//! surface and a forbidden edge. A transport host composes a [`Resident`] the way `nomos-mcp`
//! composes a dispatch; nothing in this crate decides how a client reaches it.

mod build_variant;
mod resident;
mod residency_cost;
mod resident_answer;
mod resident_refusal;
mod resident_request;
mod sources;
mod stop_report;
mod tree_reading;

pub use residency_cost::ResidencyCost;
pub use resident::Resident;
pub use resident_answer::ResidentAnswer;
pub use resident_refusal::ResidentRefusal;
pub use resident_request::ResidentRequest;
pub use stop_report::StopReport;
pub use tree_reading::TreeReading;
