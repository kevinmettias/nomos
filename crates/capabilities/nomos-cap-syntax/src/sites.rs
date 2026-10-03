//! `nomos.cap.syntax.sites`: every projection a rule reads from a syntax tree, as a kind one
//! family carries -- `OD-CAPABILITY-019`.
//!
//! # Why one family and not one capability per projection
//!
//! A projection is a located construct a rule judges: a jump that names a loop, a call nested in
//! another, an assertion that a value is present. The corpus this workspace absorbs holds one
//! per shared engine, and most of them are extracted from a parse tree. Each one built as a
//! capability of its own would cost what the metric family cost -- two crates and some twenty
//! wiring files -- so `OD-CAPABILITY-019` made a projection a *kind*: declared once here, in the
//! agreement every language provider already names, and carried by this one family. The family
//! is paid for once; a kind after that edits this crate, the providers that offer it and the
//! rules that read it.
//!
//! # The grammar
//!
//! A payload is UTF-8 text, a record per line, fields separated by one tab and records ended by
//! `\n` and never by `\r\n`, for the reason `nomos.syntax.items.v2` gives.
//!
//! ```text
//! payload  := stance* site*
//! stance   := "offers" TAB kind LF
//!           | "declines" TAB kind TAB reason LF
//! site     := "site" TAB kind TAB line (TAB field)* LF
//! field    := name "=" value
//! line     := the 1-based line the construct starts on
//! reason   := escaped, and never empty
//! ```
//!
//! **A stance is what the provider says about a kind for this file.** It offers the kind, so the
//! file's records of it are every one the provider found -- an offered kind with no records is a
//! clean file; or it declines the kind with the reason the construct does not exist in its
//! language. A kind the payload names in neither is a gap: nobody said, and a rule reads that as
//! `Applicability::MissingCapability` rather than as a pass. A payload names a kind in at most one
//! stance, and may name none -- the empty payload is every declared kind unanswered. That is why it
//! is read rather than refused, where an `items` payload without its header is refused: an empty
//! `items` payload would read as a file declaring nothing, which is a pass, and an empty sites
//! payload reads as a gap, which never is.
//!
//! **A site is one located construct of an offered kind**, in source order, carrying exactly the
//! fields its kind declares, each once and in any order. A value is written by the type its field
//! declares: text escaped as [`crate::Escape_Text`] escapes it, an integer in decimal, a truth
//! value as `true` or `false`, and a list of text as each item escaped, with `;` written `\;`,
//! and followed by `;` -- so the empty list and the list of one empty item are two spellings.
//!
//! # One reader and one writer, both here
//!
//! `nomos.syntax.items.v2` keeps its writers with its providers, because two independent writers
//! a third party can read are the evidence two providers are interchangeable. A kind is different:
//! its fields are declared once, in this crate, and a provider that encoded them itself would be a
//! second statement of the fields -- the restatement `OD-CAPABILITY-019`'s third decision forbids.
//! So [`Render_Sites_Payload`] writes what a provider hands it in the declaration's order, and
//! [`Parse_Sites_Payload`] judges it; a provider proves its output by decoding it through the
//! reader in its own tests, exactly as the items providers do.

mod kind_decline;
mod kind_stance;
mod labeled_jump;
mod parse;
mod render;
mod site_field;
mod site_kind;
mod site_record;
mod site_value;
mod site_value_type;
mod sites_contract;
mod sites_payload;
mod sites_refusal;
mod sites_refusal_kind;
#[cfg(test)]
mod tests;

pub use kind_decline::KindDecline;
pub use kind_stance::KindStance;
pub use labeled_jump::{LABELED_JUMP, LabeledJump};
pub use parse::Parse_Sites_Payload;
pub use render::Render_Sites_Payload;
pub use site_field::SiteField;
pub use site_kind::{SITE_KINDS, Site_Kind, SiteKind};
pub use site_record::SiteRecord;
pub use site_value::SiteValue;
pub use site_value_type::SiteValueType;
pub use sites_contract::{
    SITES_CAPABILITY, SITES_CONTRACT_VERSION, SITES_SCHEMA, Sites_Capability, Sites_Capability_Contract, Sites_Ceiling,
    Sites_Payload_Schema,
};
pub use sites_payload::SitesPayload;
pub use sites_refusal::SitesRefusal;
pub use sites_refusal_kind::SitesRefusalKind;
