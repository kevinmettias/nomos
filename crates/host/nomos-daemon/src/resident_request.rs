//! What a client can ask a resident.

/// The canonical operations a resident answers.
///
/// Four, and none of them is a judgment this crate reaches. [`Self::Judge`] is
/// `nomos_check_orchestration::Run_Reassessing` over the held state, [`Self::Observe`] reads
/// the tree without judging it, [`Self::Forget`] drops everything held, and [`Self::Stop`]
/// ends the residency. A fifth belongs here only when a client has a question the seam can
/// already answer and this vocabulary cannot carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResidentRequest
{
    /// Judge the root through the held workspace, fact store and reassessment cache.
    Judge,
    /// Read the tree and report what moved, judging nothing.
    Observe,
    /// Drop everything held, so the next request pays what a first request pays.
    Forget,
    /// End the residency. Every later request is refused.
    Stop,
}

impl ResidentRequest
{
    /// What this request is called, for a caller rendering one.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Judge => "judge",
            Self::Observe => "observe",
            Self::Forget => "forget",
            Self::Stop => "stop",
        };
    }

    /// Every request a client can make.
    ///
    /// Mirrored by `Test_Every_Request_Should_Be_Matched_Exhaustively`, an exhaustive match
    /// over every variant with no wildcard arm, in this file. It fails to compile, not merely
    /// to pass, if a variant is added here without being added there.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[Self::Judge, Self::Observe, Self::Forget, Self::Stop];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Where [`ResidentRequest::Stop`] sits in [`ResidentRequest::All`], counted from zero --
    /// its ordinal.
    ///
    /// The match below stays a match rather than becoming an index lookup: the property its
    /// doc comment claims is that a variant added to the enum and not to this arm list fails
    /// the file to *compile*, and a lookup would only fail to pass. So the arm names the
    /// position it answers instead of spelling a bare ordinal.
    const STOP_POSITION: usize = 3;

    /// The variants [`ResidentRequest::All`] lists, asserted beside the position check below
    /// so that a duplicate plus a dropped variant -- which leaves the length alone -- still
    /// fails.
    const REQUESTS_IN_ALL: usize = 4;

    /// [`ResidentRequest::All`]'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to [`ResidentRequest`] without a
    /// matching arm added here fails this file to *compile*, not merely to pass -- and the
    /// position each arm answers is where `All` actually lists it, so a variant that exists,
    /// has an arm and was never added to `All` fails too.
    #[test]
    fn Test_Every_Request_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal_Of(request: ResidentRequest) -> usize
        {
            return match request
            {
                ResidentRequest::Judge => 0,
                ResidentRequest::Observe => 1,
                ResidentRequest::Forget => 2,
                ResidentRequest::Stop => STOP_POSITION,
            };
        }

        assert_eq!(ResidentRequest::All().len(), REQUESTS_IN_ALL, "{:?}", ResidentRequest::All());
        for (index, request) in ResidentRequest::All().iter().enumerate()
        {
            assert_eq!(Ordinal_Of(*request), index, "{request:?} is listed at {index} and matched elsewhere");
        }
    }

    #[test]
    fn Test_Every_Request_Should_Carry_Its_Own_Label()
    {
        use std::collections::BTreeSet;

        let labels: BTreeSet<&str> = ResidentRequest::All().iter().map(|request| return request.Label()).collect();

        assert_eq!(labels.len(), REQUESTS_IN_ALL, "two requests share a label: {labels:?}");
    }
}
