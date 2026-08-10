//! What a rollup is computed against: who may answer, how good the answer has to be, and the build its facts are filed under.

use crate::provider::FactContext;
use nomos_capability::Requirement;
use nomos_capability::Registry;
/// What a rollup is computed against: who may answer, how good the answer has to be, and
/// the build its facts are filed under.
///
/// The three travel together through every step of a rollup and none of them is useful
/// without the others — a registry with no floor admits everything, and a floor with no
/// build has nothing to file the result under.
#[derive(Clone, Copy)]
pub struct Against<'a>
{
    /// Who may answer for a member.
    pub registry: &'a Registry,
    /// How good an answer has to be to count.
    pub need: &'a Requirement,
    /// The build every fact read and written here is filed under.
    pub context: FactContext,
}
