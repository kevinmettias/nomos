//! What the process exits with.

/// What the process exits with.
///
/// A contract, not an implementation detail. Agents branch on these rather than parsing
/// output, so they are documented here and covered by tests. The distinction that earns
/// its own code is [`ExitCode::ClaimUnavailable`]: an agent that is told the item is
/// taken should try another one, and an agent that is told the ledger is broken should
/// stop and get a human — collapsing those into "non-zero" makes the first case
/// indistinguishable from the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The operation succeeded.
    Ok = 0,
    /// The ledger is invalid, or an operation was refused on its merits.
    ValidationError = 1,
    /// The command line was wrong.
    Usage = 2,
    /// Somebody else holds it. Retryable.
    ClaimUnavailable = 3,
    /// A conflict a human has to resolve.
    Conflict = 4,
    /// The ledger or its lock could not be used at all.
    StoreError = 5,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// `work`'s exit codes as the shell sees them. The claim codes are what an agent retries
    /// on, so their values are named here and pinned below, rather than left as bare literals
    /// inside the assertions that read them.
    const USAGE_CODE: i32 = 2;
    const CLAIM_UNAVAILABLE_CODE: i32 = 3;
    const CONFLICT_CODE: i32 = 4;
    const STORE_ERROR_CODE: i32 = 5;

    /// The exit codes are a contract agents branch on, so their values are pinned.
    ///
    /// Named to avoid a false address: this file's `exit_code` unit is shared, by bare file
    /// stem, with five sibling `exit_code.rs` files under `check/`, `gate/`, `agent/`,
    /// `request/` and `spec/` — the coverage rule keys a Rust unit by file name alone, not by
    /// path. A name ending `..._For_Every_Exit_Code` tokenizes into `every_exit_code`, which
    /// is `gate::exit_code::Every_Exit_Code`'s own already-covered address and is LONGER than
    /// `value` — so the longest-match rule silently attributed this test to that function
    /// instead, and `Value` stayed unaddressed. `Discriminant` carries no such collision.
    #[test]
    fn Test_Value_Should_Return_The_Variants_Own_Discriminant()
    {
        assert_eq!(ExitCode::Ok.Value(), 0);
        assert_eq!(ExitCode::ValidationError.Value(), 1);
        assert_eq!(ExitCode::Usage.Value(), USAGE_CODE);
        assert_eq!(ExitCode::ClaimUnavailable.Value(), CLAIM_UNAVAILABLE_CODE);
        assert_eq!(ExitCode::Conflict.Value(), CONFLICT_CODE);
        assert_eq!(ExitCode::StoreError.Value(), STORE_ERROR_CODE);
    }
}
