//! What a [`super::NomosResolvedChangeContext`] was resolved for -- `AGT-007`'s own "a
//! symbol, selected scope, or task description", each transcribed as the existing type
//! that already names that concept in this workspace.

use nomos_contracts::SubjectId;
use nomos_scope_verification::Territory;
use serde::{Deserialize, Serialize};

/// What a [`super::NomosResolvedChangeContext`] was resolved for -- `AGT-007`'s own "a
/// symbol, selected scope, or task description", each transcribed as the existing
/// type that already names that concept in this workspace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeContextSubject
{
    Symbol(SubjectId),
    Scope(Territory),
    TaskDescription(String),
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::Digest128;

    /// The byte the one symbol identity in this test is filled with. It is a seed and
    /// not a value the test reads back, so any other byte would do; naming it keeps it
    /// from reading as a decision about which symbol this is.
    const SYMBOL_SEED: u8 = 5;

    #[test]
    fn Test_A_Subject_May_Be_A_Symbol_A_Scope_Or_A_Task_Description()
    {
        let symbol = ChangeContextSubject::Symbol(SubjectId::From_Digest(Digest128::From_Bytes([SYMBOL_SEED; Digest128::BYTE_LENGTH])));
        let scope = ChangeContextSubject::Scope(Territory::Of_Files(["crates/agent"]));
        let task = ChangeContextSubject::TaskDescription("add a test".to_owned());

        assert_ne!(symbol, scope);
        assert_ne!(scope, task);
    }
}
