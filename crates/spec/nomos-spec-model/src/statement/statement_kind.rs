//! Which of the five things a normative statement is.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatementKind
{
    Requirement,
    Story,
    Acceptance,
    Criterion,
    Concept,
}
