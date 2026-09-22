//! What this host establishes about one provider, short of running it.

/// Available, unavailable for a named reason, or not established here.
///
/// The third variant is the point of the type. A provider that runs `cargo clippy` is not
/// shown to work by `cargo` being on the path — the component may not be installed, the
/// toolchain may not carry it, the subcommand may refuse — and the only thing that settles
/// it is running it, which this verb deliberately does not do. Folding that into "available"
/// would hand a person adopting the tool a green line they would later discover was a
/// silent incompleteness, and folding it into "unavailable" would name a tool that is
/// already installed. So it is its own answer, and it carries the reason rather than a
/// placeholder for one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProviderStanding
{
    /// The provider is linked into this binary and launches nothing, so there is nothing
    /// about this host that could stop it answering.
    InThisBinary,
    /// The provider runs a program and no directory on this host's search path holds it.
    /// `tool` is what to install, which is the actionable half.
    ToolMissing
    {
        /// The program that was looked for and not found.
        tool: String,
    },
    /// Nothing was established, and `because` says what was in the way.
    Undetermined
    {
        /// Why this host could not settle it without running something.
        because: String,
    },
    /// The capability is declared and no provider offers it at all, so nothing can answer
    /// it whatever this host carries. Settled by the registry alone, which is why it is the
    /// one standing no probe contributes to.
    NothingOffered,
}

impl ProviderStanding
{
    /// How strong this standing is, so a capability with several offers can report the best
    /// one its offers reach.
    ///
    /// A capability is answerable if *any* of its providers can answer, so the strongest
    /// standing wins rather than the first or the worst. Undetermined outranks a missing
    /// tool because an offer that may work is a better account of the capability than one
    /// that certainly does not, and the report says which provider the standing came from
    /// so the reader is never left guessing which offer was spoken for.
    ///
    /// [`Self::NothingOffered`] ranks lowest and never actually competes: it is the answer
    /// for a capability with no offers, so there is no second standing for it to be ranked
    /// against.
    #[must_use]
    pub(crate) const fn Rank(&self) -> u8
    {
        return match *self
        {
            Self::InThisBinary => LINKED_IN,
            Self::Undetermined { .. } => NOT_SETTLED,
            Self::ToolMissing { .. } => TOOL_ABSENT,
            Self::NothingOffered => NO_OFFER,
        };
    }
}

/// A provider that needs nothing from this host.
const LINKED_IN: u8 = 3;

/// A provider whose standing this host could not settle.
const NOT_SETTLED: u8 = 2;

/// A provider whose program is not here.
const TOOL_ABSENT: u8 = 1;

/// No provider at all.
const NO_OFFER: u8 = 0;
