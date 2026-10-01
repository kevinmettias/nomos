//! What a `ModelBackendPackage` establishes by answering one task.

/// What one [`crate::ModelBackend`] dispatch established: a response, and nothing else.
///
/// There is no `denied_tool_uses` here, no `is_error`, no `spend` and no `duration_ms`, and
/// their absence is the point rather than an omission. `OD-EXECUTOR-004` measured a real
/// model backend's mechanism directly and found no tool subsystem to deny into -- the
/// absence of one *is* the whole boundary, not a signal read after the fact -- and no
/// per-call dollar cost, because local inference has no metered charge. A field here for
/// either would be a number nothing measured.
///
/// `response` is free text and is never evidence of what happened. The same adversarial
/// measurement found a model narrating a file write and a shell command it structurally
/// could not perform. A clean exit with something to say is the whole of what happened.
///
/// The guarantee is structural rather than editorial:
/// [`tests::Test_A_Model_Answer_Carries_A_Response_And_Nothing_A_Model_Backend_Cannot_Ground`]
/// binds this type with no rest pattern, so a field added here stops that test compiling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelAnswer
{
    /// What the model said, which is not a report of what it did.
    pub response: String,
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The honesty property `OD-EXECUTOR-005` paid to establish, kept by the type system
    /// rather than by a doc comment: there is no value in this workspace on which a model
    /// backend's answer can be asked for a cost, a duration or a denial list.
    ///
    /// The binding below names every field and admits no `..`, so adding one -- a
    /// `spend: MicroDollars` copied across from [`crate::AgentExecution`] to make the two
    /// ports look alike -- stops this test *compiling*, which is a louder failure than an
    /// assertion going red. Proved by injection: a `spend` field was added to the struct
    /// above and `cargo test -p nomos-agent-contracts` failed with E0027, "pattern does not
    /// mention field `spend`"; the file was then restored byte-identically.
    ///
    /// A count would not do this job. `assert_eq!(fields, 1)` is a number somebody has to
    /// remember to hold, and the compiler is not asked anything by it.
    #[test]
    fn Test_A_Model_Answer_Carries_A_Response_And_Nothing_A_Model_Backend_Cannot_Ground()
    {
        let answer = ModelAnswer { response: "PONG".to_owned() };

        let ModelAnswer { response } = answer;

        assert_eq!(response, "PONG");
    }
}
