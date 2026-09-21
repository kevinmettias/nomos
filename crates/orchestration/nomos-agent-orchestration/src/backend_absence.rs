//! Why a dispatch selected no backend, so nothing ran.

use nomos_model_package::ModelSelector;

use crate::ProfileAbsence;

/// The two ways a dispatch can reach no backend at all.
///
/// Distinct from [`crate::AgentDispatchOutcome::Unavailable`], which is a backend that *was*
/// selected and then could not be started or did not answer. Nothing was selected here, so
/// there is no backend to report anything about, and the two must not read alike: one says
/// the thing we chose did not work, the other says we chose nothing.
///
/// Measured rather than free text in both arms, so a caller can branch on it and a test can
/// assert it -- the same discipline [`ProfileAbsence`] already holds to for the resolver's
/// own half.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendAbsence
{
    /// The profile did not resolve against the declared set, and this is the selector it
    /// stated and the absence the resolver measured.
    Unresolved
    {
        /// The selector the profile stated.
        selector: ModelSelector,
        /// What the resolver measured about it.
        absence: ProfileAbsence,
    },
    /// A person named a backend and the declared set does not offer it.
    ///
    /// A preference is attempted first and may fail, which is the whole difference between a
    /// preference and a resolution. It deliberately does not fall back to whatever the
    /// profile would have resolved to on its own: `nomos_capability::Selection::Over` does
    /// fall back for a provider preference, and doing the same here would run a different
    /// backend than the one a person typed after `--executor`, which is not a thing to
    /// discover from the output.
    PreferenceNotDeclared
    {
        /// The family label that was named, as the caller spelled it.
        preferred: String,
    },
}
