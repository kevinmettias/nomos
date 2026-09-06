//! Why a requirement could not be met.

use nomos_contracts::Applicability;
use nomos_contracts::ProviderId;
use nomos_contracts::ContractVersion;
/// Why a requirement could not be met.
///
/// Separate from [`Applicability`] on purpose. `Applicability` is the vocabulary a run
/// reports in and is deliberately small; this is the actionable detail behind one of its
/// values. One relation, two projections — the alternative is growing `Applicability` a
/// variant per diagnosis until nothing can match on it exhaustively.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unmet
{
    /// Nobody declared this capability. Distinct from having no provider, because the
    /// remedy is authoring a contract rather than installing something.
    Undeclared,
    /// Declared, and nothing offers it.
    NoProvider,
    /// Offered at a contract version this caller cannot read.
    VersionMismatch
    {
        offered: ContractVersion,
    },
    /// Offered, and no offer reaches the required guarantee.
    BelowRequirement
    {
        closest: ProviderId,
    },
}

impl Unmet
{
    /// How a run reports this.
    ///
    /// Every arm is [`Applicability::MissingCapability`] and none is
    /// [`Applicability::NotApplicable`] — that distinction is the entire point. "No
    /// provider offers what this rule needs" is coverage debt; "this rule does not bind
    /// this subject" is a judgement about the subject, and the registry is in no
    /// position to make it. Collapsing the two is how a rule that could not run reads
    /// like a rule that did not need to.
    #[must_use]
    pub const fn Applicability(&self) -> Applicability
    {
        return match *self
        {
            Self::Undeclared
            | Self::NoProvider
            | Self::VersionMismatch { .. }
            | Self::BelowRequirement { .. } => Applicability::MissingCapability,
        };
    }

    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Undeclared => "no capability contract declares this".to_owned(),
            Self::NoProvider => "the contract is declared and no provider offers it".to_owned(),
            Self::VersionMismatch { offered } => {
                format!("offered at contract version {offered}, which this caller cannot read")
            }
            Self::BelowRequirement { closest } => {
                format!("{closest} is the closest offer and does not reach the required guarantee")
            }
        };
    }

    /// What a caller can do about this, alongside [`Self::Describe`]'s account of what is
    /// wrong.
    ///
    /// `P54-AN-UNMET-CAPABILITY-CARRIES-NO-REMEDY`: the Go predecessor's `kernel/toolspec`
    /// package states this split in its own words -- "the dotnet runtime is not on PATH" is
    /// what is wrong, "install dotnet 10 then run dotnet build" is what to do, and a message
    /// that merges them tends to be neither. `Remedy` is that second half, structured rather
    /// than prose so a caller renders both without composing a sentence of its own.
    #[must_use]
    pub fn Remedy(&self) -> Remedy
    {
        return match self
        {
            Self::Undeclared => Remedy::DeclareTheContract,
            Self::NoProvider => Remedy::RegisterAProvider,
            Self::VersionMismatch { offered } => Remedy::AgreeOnAVersion { offered: *offered },
            Self::BelowRequirement { closest } => Remedy::StrengthenTheClosestOffer { closest: closest.clone() },
        };
    }
}

/// What closes the gap a matching [`Unmet`] variant names -- the only half of the pair a
/// caller can act on, structured so it survives being rendered more than one way rather than
/// fixed as one sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Remedy
{
    /// Author a capability contract naming this capability, then declare it against a
    /// [`crate::Registry`].
    DeclareTheContract,
    /// The contract is declared; register a provider offer against it.
    RegisterAProvider,
    /// Offered at a version this caller cannot read; either side moving to a version the
    /// other can read closes the gap.
    AgreeOnAVersion
    {
        offered: ContractVersion,
    },
    /// The closest offer does not reach the required guarantee; strengthen it, or register
    /// a new offer that does.
    StrengthenTheClosestOffer
    {
        closest: ProviderId,
    },
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Applicability_Should_Report_Every_Variant_As_A_Missing_Capability()
    {
        assert_eq!(Unmet::Undeclared.Applicability(), Applicability::MissingCapability);
        assert_eq!(Unmet::NoProvider.Applicability(), Applicability::MissingCapability);
        assert_eq!(
            Unmet::VersionMismatch { offered: ContractVersion::New(2, 0) }.Applicability(),
            Applicability::MissingCapability
        );
        assert_eq!(
            Unmet::BelowRequirement { closest: ProviderId::New("nomos.test.closest") }.Applicability(),
            Applicability::MissingCapability
        );
    }

    #[test]
    fn Test_Describe_Should_Name_What_Is_Missing()
    {
        assert!(Unmet::Undeclared.Describe().contains("no capability contract"));
        assert!(Unmet::NoProvider.Describe().contains("no provider"));
        assert!(
            Unmet::VersionMismatch { offered: ContractVersion::New(2, 0) }
                .Describe()
                .contains("cannot read")
        );
        assert!(
            Unmet::BelowRequirement { closest: ProviderId::New("nomos.test.closest") }
                .Describe()
                .contains("nomos.test.closest")
        );
    }

    #[test]
    fn Test_Remedy_Should_Name_What_Closes_Each_Gap()
    {
        assert_eq!(Unmet::Undeclared.Remedy(), Remedy::DeclareTheContract);
        assert_eq!(Unmet::NoProvider.Remedy(), Remedy::RegisterAProvider);
        assert_eq!(
            Unmet::VersionMismatch { offered: ContractVersion::New(2, 0) }.Remedy(),
            Remedy::AgreeOnAVersion { offered: ContractVersion::New(2, 0) }
        );
        assert_eq!(
            Unmet::BelowRequirement { closest: ProviderId::New("nomos.test.closest") }.Remedy(),
            Remedy::StrengthenTheClosestOffer { closest: ProviderId::New("nomos.test.closest") }
        );
    }
}
