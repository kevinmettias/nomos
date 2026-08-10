//! One fact replacing another, and why.

use nomos_contracts::GenerationId;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Supersession
{
    pub invalidated_at: GenerationId,
    pub cause: String,
}
