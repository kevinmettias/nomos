//! Whether a named program is on this host's search path.

/// The three answers looking for a program can have.
///
/// Three rather than two, because "it is not there" and "I could not look" are different
/// things to tell somebody adopting a tool. A host with no `PATH` at all is not a host
/// missing `cargo`, and reporting it as one would name a tool the person should install
/// when what they actually have is a process started without an environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProgramSearch
{
    /// A file of that name sits in one of the search path's directories.
    Found,
    /// The search path was read and no directory on it holds that name.
    NotOnPath,
    /// There was no search path to look along, so nothing was established either way.
    PathUnreadable,
}
