//! One workspace member's declared component.

/// One `package belongs to component` statement from a repository's own declaration.
///
/// Both halves are the repository's own words. `component` is not drawn from any set this
/// crate knows: it is one of the names [`super::ArchitecturePayload::components`] declares,
/// and a declaration naming a component it never declared is refused rather than guessed at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Membership
{
    pub package: String,
    pub component: String,
}
