//! What became of one rule against one store.

use crate::Violation;
/// What a rule concluded.
///
/// `Satisfied` carries what it looked at. A rule that examined nothing and concluded
/// nothing is wrong is not evidence, and without the count the two are indistinguishable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleOutcome
{
    Satisfied
    {
        checked: u32,
    },
    Violated(Vec<Violation>),
    Errored(String),
}
