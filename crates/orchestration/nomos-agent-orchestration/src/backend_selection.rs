//! What a caller asks a dispatch to run, and how that becomes a port.

use nomos_agent_contracts::DeclaredTarget;
use nomos_model_package::ModelExecutionProfile;

use crate::{BackendAbsence, DispatchConfig, ProfileResolution, Resolve_Profile};

/// A dispatch's request: what the caller declared, and what it may run against.
///
/// The declared set travels in the request rather than being read inside, because
/// `OD-PACKAGE-016` decision 2 decided the resolver resolves against a caller-supplied
/// sequence and discovers nothing. Since `OD-ROADMAP-005` decision 2 each element also
/// carries the port that answers it, so this crate no longer needs -- and no longer has -- a
/// `Declared_Targets` of its own naming the two backends this build happens to ship. A
/// composition root offers each adapter's own declaration; a host holding a real manifest
/// would offer what it read, and nothing here would change.
pub struct BackendSelection<'port>
{
    /// What the caller declared it wants, which the resolution answers.
    pub profile: &'port ModelExecutionProfile,
    /// A family label a person named, attempted ahead of the resolution and allowed to fail.
    ///
    /// A label rather than a port, so that naming one costs no call site the right to
    /// construct a dispatch target. A flag parser validates the spelling it accepts and hands
    /// the text on; which target that text names is decided here, against the declared set,
    /// exactly as it is when nobody names anything.
    pub preferred: Option<&'port str>,
    /// The targets this dispatch may reach.
    pub declared: &'port [DeclaredTarget<'port>],
}

/// The port, family and effort a dispatch will run with, or why it reached none.
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
pub fn Selected_Dispatch<'port>(
    selection: &BackendSelection<'port>,
) -> Result<DispatchConfig<'port>, BackendAbsence>
{
    if let Some(preferred) = selection.preferred
    {
        let Some(target) = selection.declared.iter().find(|target| return target.family == preferred)
        else
        {
            return Err(BackendAbsence::PreferenceNotDeclared { preferred: preferred.to_owned() });
        };

        return Ok(DispatchConfig {
            effort: selection.profile.effort,
            family: target.family.as_str(),
            port: target.port,
        });
    }

    return match Resolve_Profile(selection.profile, selection.declared)
    {
        ProfileResolution::Resolved { config, .. } => Ok(config),
        ProfileResolution::Unresolved { selector, absence } => Err(BackendAbsence::Unresolved { selector, absence }),
    };
}
