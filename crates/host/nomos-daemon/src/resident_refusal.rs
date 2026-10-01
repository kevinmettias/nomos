//! What a resident refuses rather than answering.

use std::path::PathBuf;

/// Why a request was not answered.
///
/// Two, and both are refusals a cache could have hidden. A root that stopped being readable
/// is the case the crate doc names: the resident is still holding a whole judgment of that
/// tree, and handing it back would be presenting a cache as an observation. An already-stopped
/// resident is the other: it has released what it held, so an answer from it would either be
/// wrong or would quietly restart a residency the client ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidentRefusal
{
    /// The root is not a directory this process can read, now -- whether or not it was one
    /// when the residency began.
    RootIsNotWalkable
    {
        /// The root that was asked for.
        root: PathBuf,
    },
    /// A client already asked this resident to stop.
    AlreadyStopped,
}

impl std::fmt::Display for ResidentRefusal
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return match self
        {
            Self::RootIsNotWalkable { root } => write!(
                formatter,
                "{} is not a directory this process can read, so nothing was observed and \
                 nothing was answered from what the resident still holds",
                root.display()
            ),
            Self::AlreadyStopped => write!(formatter, "this resident was already asked to stop"),
        };
    }
}

impl std::error::Error for ResidentRefusal {}
