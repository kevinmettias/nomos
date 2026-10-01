//! The closed vocabulary of constructs a declaration may name.
//!
//! Its own file rather than a variant list inside [`super::DeclaredTextRule`]: this is the
//! type that carries `OD-RULES-034`'s load-bearing refusal, and the refusal is a property of
//! this type rather than of the declaration that names one of its members.
//!
//! # Why a declaration names a detector and never contains one
//!
//! `OD-RULES-034` censused the forty-four detectors of the archetype and found they are not
//! one shape: a substring test, a matcher that returns the text it matched, a parser for a
//! declared numeric range inside a documentation comment, and an integer-width computation.
//! A form general enough to hold the last two is a programming language, and one holding
//! only the first would let a declaration be *written* for the other three and silently fail
//! to *mean* them. So the vocabulary is an enum of names, it holds no pattern, no predicate
//! and no callable a declaration could supply, and a rule needing a detector it does not
//! name is a Rust change -- adding a variant here, beside the reading it stands for.
//!
//! That refusal is not stated in prose and then trusted: it is what this type is. There is
//! no constructor taking a `fn`, no field holding a string, and no variant carrying data, so
//! a declaration that contained a detector would not compile.
//!
//! # Membership
//!
//! One entry per reading a declared rule actually names, and no entry with no caller. The
//! record left membership to be "read off the forty-four rules when one is built" rather
//! than guessed at in advance, and a variant nothing declares would be exactly the field
//! measured against nothing that its own method test forbids.

/// One construct a declared rule judges, named from the readings this crate already has.
///
/// Every member is a `fn(&str) -> bool` over the comment-stripped, string-masked prefix of
/// one line, which is the one shape `checks::rust_text`'s shared engine drives. The line's
/// own text is all any of them sees: no block state, no scope, no neighbouring file.
// The shared `Attribute` postfix is what the three members have in common today, not
// redundancy: `OD-RULES-034` censused the archetype's forty-four detectors and four of the
// ones it quotes read no attribute at all -- a substring test, a matcher that returns its
// match, a documentation-comment range parser and an integer-width computation. A member
// that names one of those has to be distinguishable from one that reads an attribute, so
// the postfix is the distinction this vocabulary will need rather than a word repeated.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclaredDetector
{
    /// `#[allow(...)]` or `#![allow(...)]`.
    AllowAttribute,
    /// `#[inline(always)]`.
    InlineAlwaysAttribute,
    /// A bare `#[ignore]` with no `= "reason"` value of its own.
    BareIgnoreAttribute,
}

impl DeclaredDetector
{
    /// The reading this member names, as the one predicate shape the engine drives.
    ///
    /// The resolution is the interpreter's, which is the whole of the difference between
    /// naming a detector and containing one: a declaration holds the member, and only this
    /// function knows what code it stands for.
    pub(crate) fn As_Predicate(self) -> fn(&str) -> bool
    {
        return match self
        {
            Self::AllowAttribute => crate::checks::Has_Allow_Attribute,
            Self::InlineAlwaysAttribute => crate::checks::Has_Inline_Always_Attribute,
            Self::BareIgnoreAttribute => crate::checks::Has_Bare_Ignore_Attribute,
        };
    }

    /// Whether this reading's judgment moves with a repository-declared value.
    ///
    /// `false` for every member today, and measured rather than assumed --
    /// [`super::DeclaredParameter`] records the count and the observation that would change
    /// it. It is a method rather than a constant because the answer belongs to the member,
    /// and the day one member takes a parameter the others must keep saying they do not.
    pub(crate) const fn Is_Taking_A_Parameter(self) -> bool
    {
        return match self
        {
            Self::AllowAttribute | Self::InlineAlwaysAttribute | Self::BareIgnoreAttribute => false,
        };
    }
}
