//! Why the guard could not judge at all, as distinct from what it refused.

/// A question the guard could not get answered. At a transition this refuses the transition,
/// because a guard that cannot ask is not a guard that found nothing (`OD-POLICY-002`
/// decision 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardError
{
    /// Git could not be run, or answered with a failure.
    Git(String),
}

impl std::fmt::Display for GuardError
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return match self
        {
            Self::Git(detail) => write!(formatter, "git could not be asked: {detail}"),
        };
    }
}
