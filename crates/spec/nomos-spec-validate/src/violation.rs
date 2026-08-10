//! One row that offends a rule.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation
{
    pub subject: String,
    pub detail: String,
}
