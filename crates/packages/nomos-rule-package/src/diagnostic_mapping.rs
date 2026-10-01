//! Mapping an external tool's own diagnostic code onto a Nomos rule.

use nomos_contracts::{ProviderId, RuleId};

/// One entry of `ARCH-002`'s "external diagnostic mappings".
///
/// No shipped rule has one yet — `OD-PACKAGE-008`'s four-rule measurement found this
/// field's real instance count at zero, one of five fields that field-by-field
/// measurement could not fill from an existing case. The shape below is transcribed
/// directly from what the field's own name asks for: which external provider's
/// diagnostic, spelled in that provider's own vocabulary, this rule's finding
/// corresponds to — the identical relationship `nomos_contracts::RuleId` already
/// resolves between a Nomos rule and its governing record, one step further out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticMapping
{
    /// The external tool whose own diagnostic vocabulary this mapping is stated in.
    pub external_tool: ProviderId,
    /// The diagnostic code as that tool spells it, such as `"clippy::needless_return"`.
    pub external_code: String,
    /// The Nomos rule this external diagnostic corresponds to.
    pub maps_to: RuleId,
}

impl DiagnosticMapping
{
    /// Constructs a diagnostic mapping.
    #[must_use]
    pub const fn New(external_tool: ProviderId, external_code: String, maps_to: RuleId) -> Self
    {
        return Self { external_tool, external_code, maps_to };
    }
}
