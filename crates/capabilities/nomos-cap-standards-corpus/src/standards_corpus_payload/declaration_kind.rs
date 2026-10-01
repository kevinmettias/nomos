//! What a standards document says it is.

/// What a standards document says it is — read from its own declaration and never guessed
/// from its shape.
///
/// The corpus this capability reads exists because the format it replaced made tools guess:
/// a rule was "a bold span ending in a severity tag", which read `**Exception [MUST]:**` as a
/// rule named Exception and let a roll-up index invent rules that existed in no document.
/// [`Self::Rule`] is therefore reachable only from a declaration that says `kind: rule`.
///
/// [`Self::Undeclared`] is the fourth case and the one a census has to carry rather than drop:
/// a document that contributes no readable declaration. That covers two documents the corpus's
/// own schema treats alike and this workspace must not: one that wrote nothing, which the
/// schema makes no demand of, and one whose declaration could not be read at all — an unclosed
/// fence, say — which is a document with a defect. Neither is a rule and neither is a
/// [`super::DeclarationKind`] the reader recognised; the difference between them is carried in
/// [`super::DeclarationIssue`], which the second has and the first does not. Collapsing the two
/// cases into the enum instead would lose the defect, and dropping the row would lose the
/// document from the population the census counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeclarationKind
{
    /// The document declares `kind: rule` and is a rule because it says so.
    Rule,
    /// The document declares `kind: index`.
    Index,
    /// The document declares `kind: reference`.
    Reference,
    /// The document carries no declaration at all, and is therefore no kind of rule.
    Undeclared,
}

impl DeclarationKind
{
    /// The kind as the document declared it, or `undeclared` where it declared none.
    ///
    /// `undeclared` is this workspace's own word and not the corpus's: it names the absence of
    /// a declaration, which no document can write.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Rule => "rule",
            Self::Index => "index",
            Self::Reference => "reference",
            Self::Undeclared => "undeclared",
        };
    }

    /// The kind a document declares by writing `declared`, or `None` where the corpus's own
    /// vocabulary has no such kind.
    ///
    /// `None` is a refusal rather than [`Self::Undeclared`], and the distinction is the whole
    /// reason both exist: a document that wrote a kind this workspace cannot read has a
    /// declaration that did not parse, while a document that wrote nothing has no declaration
    /// to parse. Filing the first as the second would silently drop it from the population.
    ///
    /// This is the reader for a *document's own front matter*, and it deliberately does not
    /// accept [`Self::Undeclared`]'s spelling: no document can write the absence of a
    /// declaration. [`Self::From_Label`] is the reader for this crate's own encoding, whose
    /// vocabulary is one word larger for exactly that reason.
    #[must_use]
    pub fn Declared_By(declared: &str) -> Option<Self>
    {
        return match declared.trim()
        {
            "rule" => Some(Self::Rule),
            "index" => Some(Self::Index),
            "reference" => Some(Self::Reference),
            _ => None,
        };
    }

    /// The kind [`Self::Label`] wrote, which is this crate's own encoding's vocabulary rather
    /// than a corpus's.
    ///
    /// The inverse of [`Self::Label`] and not a second spelling of [`Self::Declared_By`]: the
    /// two agree on the three kinds a document can declare and differ on the fourth, which
    /// exists only here. A payload that carried `undeclared` for a document is round-tripping
    /// this workspace's own record of an absence, and reading that record back with the
    /// corpus's vocabulary would refuse a payload this crate encoded.
    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label.trim()
        {
            "rule" => Some(Self::Rule),
            "index" => Some(Self::Index),
            "reference" => Some(Self::Reference),
            "undeclared" => Some(Self::Undeclared),
            _ => None,
        };
    }

    /// Whether a document of this kind is a rule.
    #[must_use]
    pub const fn Is_Rule(self) -> bool
    {
        return matches!(self, Self::Rule);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Every variant, so a test that walks the vocabulary cannot miss one added later.
    const EVERY: [DeclarationKind; 4] = [
        DeclarationKind::Rule,
        DeclarationKind::Index,
        DeclarationKind::Reference,
        DeclarationKind::Undeclared,
    ];

    /// Every kind a document can *write*. [`DeclarationKind::Undeclared`] is deliberately
    /// absent: it names the absence of a declaration, so no document can declare it, and a
    /// round trip over the corpus's vocabulary would be asserting what the type exists to deny.
    const EVERY_DECLARABLE: [DeclarationKind; 3] = [
        DeclarationKind::Rule,
        DeclarationKind::Index,
        DeclarationKind::Reference,
    ];

    #[test]
    fn Test_Every_Kind_Should_Round_Trip_Through_Its_Own_Label()
    {
        for kind in EVERY
        {
            assert_eq!(DeclarationKind::From_Label(kind.Label()), Some(kind), "{kind:?}");
        }
    }

    #[test]
    fn Test_Every_Declarable_Kind_Should_Round_Trip_Through_The_Corpus_Vocabulary()
    {
        for kind in EVERY_DECLARABLE
        {
            assert_eq!(DeclarationKind::Declared_By(kind.Label()), Some(kind), "{kind:?}");
        }
    }

    /// The two readers differ on exactly the one variant that is this workspace's word rather
    /// than a document's, which is the whole reason they are two functions.
    #[test]
    fn Test_Undeclared_Should_Be_Readable_From_The_Encoding_And_Not_From_A_Document()
    {
        assert_eq!(
            DeclarationKind::From_Label(DeclarationKind::Undeclared.Label()),
            Some(DeclarationKind::Undeclared)
        );
        assert_eq!(DeclarationKind::Declared_By(DeclarationKind::Undeclared.Label()), None);
    }

    #[test]
    fn Test_Only_The_Rule_Kind_Should_Be_A_Rule()
    {
        let rules: Vec<DeclarationKind> = EVERY.into_iter().filter(|kind| return kind.Is_Rule()).collect();

        assert_eq!(rules, vec![DeclarationKind::Rule], "a document is a rule because it declares kind: rule");
    }

    #[test]
    fn Test_An_Unknown_Kind_Should_Be_Refused_Rather_Than_Treated_As_Undeclared()
    {
        assert_eq!(DeclarationKind::Declared_By("standard"), None);
        assert_eq!(DeclarationKind::Declared_By(""), None, "an empty `kind:` wrote something this workspace cannot read");
    }
}
