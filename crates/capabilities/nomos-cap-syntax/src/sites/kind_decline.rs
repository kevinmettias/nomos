//! A kind a provider declines for a file, and why.

/// A declined kind and the reason the provider gives.
///
/// A provider declines a kind only because the construct does not exist in its language, and it
/// says so -- the corpus's C# kernel declines the labeled jump because C# cannot name a loop. The
/// reason is the difference between a decision and a gap (`OD-CAPABILITY-019`, sixth decision),
/// so the reader refuses a decline that gives none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindDecline
{
    pub kind: String,
    pub reason: String,
}
