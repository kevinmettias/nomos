//! Turning a repository-declared policy capability's own `Materialize_Workspace` into a
//! fact the store can answer `facts.Require` with.
//!
//! Unlike [`super::dependency_materialization`]'s three providers, a policy capability like
//! `nomos.cap.naming.policy` has no subprocess and no per-file source list a rule needs
//! handed back: `nomos_rules::Resolve_Case` already reads it directly, by its own fixed
//! empty-path subject, from whichever real sources `Run` already walked. So this module's
//! own materialization step writes at most one fact into the store and reports how many
//! landed -- `1` once `store.Materialize` accepted it, `0` for any reason it did not, the
//! identical shape [`super::dependency_materialization::Materialize_Syntax`] already
//! returns a count by for the identical reason: a caller that does not need the answer is
//! free to ignore it, and one that does (a future caller, or a test) is not left
//! re-deriving it from the store's own side effects.
//!
//! # One submodule per provider
//!
//! Every `Materialize_*` below wraps exactly one `nomos_repo_policy` (or
//! `nomos_cap_requirement_trace`) provider, and each sits in a module named after that
//! provider. Reading `standards.json` five times is five responsibilities that happen to
//! share a spelling, not one responsibility -- the shared `Materialize` prefix named a
//! boundary this module had not drawn, and this is that boundary.

mod architecture;
mod goals;
mod limits;
mod naming;
mod requirement_trace;
mod scripting;
mod test_material;
mod words;

pub use architecture::Materialize_Architecture;
pub use goals::Materialize_Goals_Policy;
pub use limits::Materialize_Limits_Policy;
pub use naming::Materialize_Naming_Policy;
pub use requirement_trace::Materialize_Requirement_Trace;
pub use scripting::Materialize_Scripting_Policy;
pub use test_material::Materialize_Test_Material_Policy;
pub use words::Materialize_Words_Policy;
