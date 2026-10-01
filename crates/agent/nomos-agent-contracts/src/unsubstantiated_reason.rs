//! Why a portion of a work result could not be substantiated.

/// Why the producing executor could not substantiate one portion of a
/// [`WorkResult`](crate::WorkResult).
///
/// A named reason rather than a bare absence, the shape `nomos_model::UnknownReason` already
/// holds one crate over: a caller cannot choose a reply to an absence that does not say why,
/// and "this dispatch's mechanism cannot ground the portion" calls for a different reply from
/// "the answer's schema did not carry it" — the first is a limit of the producer and the
/// second a limit of the answer.
///
/// There is one variant today because exactly one reason has been measured. A second reason
/// is a second variant rather than a reinterpretation of this one, and
/// `Test_Describe_Should_Name_A_Reason_For_Every_Variant` enumerates every variant so adding
/// one is a compile error there rather than a variant nothing covers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnsubstantiatedReason
{
    /// The producing dispatch's own mechanism gives this portion no grounding.
    ///
    /// This is the bare-prompt dispatch `OD-EXECUTOR-008` measured: an isolated empty
    /// directory, no tool granted, one turn, so the model has seen no real file and computed
    /// no real digest by the time it answers. A portion the model could not have observed is
    /// one no schema can make honest, which is why `OD-EXECUTOR-011` declares the limit
    /// instead of asking the model to fill it in.
    ProducerCannotGround,
}

impl UnsubstantiatedReason
{
    /// A one-line description naming why the portion could not be substantiated.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::ProducerCannotGround =>
            {
                "the producing dispatch has no grounding for this portion, having seen no \
                 real file and computed no real digest"
                    .to_owned()
            }
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Long enough that a description names a reason rather than restating the variant.
    const MINIMUM_USEFUL_DESCRIPTION_LENGTH: usize = 20;

    /// Every variant, so a second reason is a compile error here rather than a variant
    /// nothing covers. This is the discipline `UnknownReason`'s own test holds.
    #[test]
    fn Test_Describe_Should_Name_A_Reason_For_Every_Variant()
    {
        let reasons = [UnsubstantiatedReason::ProducerCannotGround];

        for reason in &reasons
        {
            assert!(
                reason.Describe().len() > MINIMUM_USEFUL_DESCRIPTION_LENGTH,
                "{} is too terse to act on",
                reason.Describe()
            );
        }
    }
}
