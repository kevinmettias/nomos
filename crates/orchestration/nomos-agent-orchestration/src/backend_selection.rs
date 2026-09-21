//! What a caller asks a dispatch to run, and how that becomes a backend.

use nomos_model_package::ModelExecutionProfile;

use crate::{BackendAbsence, DeclaredTarget, DispatchConfig, ProfileResolution, Resolve_Profile};

/// A dispatch's request: what the caller declared, and what it may run against.
///
/// The declared set travels in the request rather than being read inside, because
/// `OD-PACKAGE-016` decision 2 decided the resolver resolves against a caller-supplied
/// sequence and discovers nothing. [`crate::Declared_Targets`] is what a composition root
/// supplies today; a host that had a manifest would supply that instead, and nothing here
/// would change.
pub struct BackendSelection<'declared>
{
    /// What the caller declared it wants, which the resolution answers.
    pub profile: &'declared ModelExecutionProfile,
    /// A family label a person named, attempted ahead of the resolution and allowed to fail.
    ///
    /// A label rather than a [`Backend`], so that naming one costs no call site the right to
    /// construct a dispatch target. A flag parser validates the spelling it accepts and hands
    /// the text on; which target that text names is decided here, against the declared set,
    /// exactly as it is when nobody names anything.
    pub preferred: Option<&'declared str>,
    /// The targets this dispatch may reach.
    pub declared: &'declared [DeclaredTarget],
}

/// The backend and effort a dispatch will run with, or why it reached none.
///
/// The order is the whole contract. A named preference is attempted first, because a person
/// who typed `--executor claude-code` declared something the profile did not; it is honoured
/// only if the declared set really offers it, and fails rather than quietly resolving to
/// something else. With no preference, the resolution answers, which is what keeps a
/// no-flags dispatch from returning a backend that nothing chose.
///
/// # Errors
///
/// Returns [`BackendAbsence`] naming which of the two happened.
pub fn Selected_Dispatch(selection: &BackendSelection<'_>) -> Result<DispatchConfig, BackendAbsence>
{
    if let Some(preferred) = selection.preferred
    {
        let Some(target) = selection.declared.iter().find(|target| return target.backend.Label() == preferred)
        else
        {
            return Err(BackendAbsence::PreferenceNotDeclared { preferred: preferred.to_owned() });
        };

        return Ok(DispatchConfig { effort: selection.profile.effort, backend: target.backend });
    }

    return match Resolve_Profile(selection.profile, selection.declared)
    {
        ProfileResolution::Resolved { config, .. } => Ok(config),
        ProfileResolution::Unresolved { selector, absence } => Err(BackendAbsence::Unresolved { selector, absence }),
    };
}
