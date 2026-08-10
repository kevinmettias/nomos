//! Every way the ledger file itself refuses to be read or written.

/// Why a ledger operation could not be carried out.
#[derive(Debug)]
pub enum LedgerError
{
    /// The ledger file could not be read or written.
    Unreadable
    {
        /// What went wrong.
        cause: String,
    },
    /// The ledger file exists and is not valid.
    Malformed
    {
        /// What is wrong with it.
        cause: String,
    },
    /// The lock could not be taken.
    Locked
    {
        /// What went wrong.
        cause: String,
    },
    /// The ledger's own invariants are violated.
    Invalid
    {
        /// Every violation found, not just the first.
        violations: Vec<String>,
    },
    /// The ledger holds something this build cannot account for.
    ///
    /// Distinct from [`LedgerError::Malformed`] because the two remedies are opposites: a
    /// malformed ledger is repaired, and this one is left alone while the *reader* is
    /// rebuilt. Reporting the second as the first sends an operator to edit a file that is
    /// correct, which is the "two causes wearing one name" shape `OD-LEDGER-009` names,
    /// with the causes swapped.
    Unrecognized
    {
        /// What this build understands.
        understood: u32,
        /// What the file says it is.
        found: u32,
        /// What could not be accounted for, verbatim from the parser.
        cause: String,
    },
}

impl core::fmt::Display for LedgerError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unreadable { cause } => write!(formatter, "ledger could not be read: {cause}"),
            Self::Malformed { cause } => write!(formatter, "ledger is malformed: {cause}"),
            Self::Locked { cause } => write!(formatter, "ledger is locked: {cause}"),
            Self::Invalid { violations } => write!(
                formatter,
                "ledger is invalid:\n  {}",
                violations.join("\n  ")
            ),
            Self::Unrecognized {
                understood,
                found,
                cause,
            } => write!(
                formatter,
                "this build understands ledger schema {understood} and the file is schema \
                 {found}: {cause}. Writing it back would drop what could not be read, so \
                 nothing was written. Rebuild (`cargo build -p nomos-cli`) and retry"
            ),
        };
    }
}

impl std::error::Error for LedgerError {}
