//! The labeled jump: the family's first kind.

use super::{SiteField, SiteKind, SiteRecord, SiteValue, SitesPayload, SiteValueType};
use std::collections::BTreeMap;

/// The field naming the jump's own keyword.
const KEYWORD: &str = "keyword";
/// The field naming the label as the language spells it.
const LABEL: &str = "label";
/// The field quoting the jump as a message spells it back.
const TEXT: &str = "text";
/// The field holding the line the label is written on.
const LABEL_LINE: &str = "label_line";
/// The field saying whether deleting the label would land in the same place.
const HAS_SAME_TARGET_UNLABELED: &str = "has_same_target_unlabeled";
/// The field saying whether the jump stands inside a construct that catches a bare `break`.
const IS_INSIDE_BREAK_CAPTURE: &str = "is_inside_break_capture";

/// The labeled jump, as the corpus's `labeledloop` engine projects it at code-standards
/// `f0d820729`: a `break` or `continue` that names, by its label, a loop enclosing it.
///
/// Only a jump whose label resolves to a *loop* is a site. A jump out of a labeled block abandons
/// no iteration, and a jump whose label resolves to nothing (out of a closure, which does not
/// compile) locates nothing.
pub const LABELED_JUMP: SiteKind = SiteKind {
    name: "labeled-jump",
    construct: "a `break` or `continue` that names, by its label, a loop enclosing it",
    fields: &[
        SiteField { name: KEYWORD, value: SiteValueType::Text },
        SiteField { name: LABEL, value: SiteValueType::Text },
        SiteField { name: TEXT, value: SiteValueType::Text },
        SiteField { name: LABEL_LINE, value: SiteValueType::Integer },
        SiteField { name: HAS_SAME_TARGET_UNLABELED, value: SiteValueType::Truth },
        SiteField { name: IS_INSIDE_BREAK_CAPTURE, value: SiteValueType::Truth },
    ],
};

/// One labeled jump, read from or written to its record.
///
/// The typed view of [`LABELED_JUMP`]'s declaration, kept beside it so the field names are spelled
/// once: a provider builds this and calls [`LabeledJump::Record`], and a rule reads it back with
/// [`LabeledJump::All_In`], and neither names a field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabeledJump
{
    /// The line the jump is written on.
    pub line: usize,
    /// The language's own word for the jump -- `break`, `continue`. Carried for a message and
    /// never switched on.
    pub keyword: String,
    /// The label as the language spells it, apostrophe and all.
    pub label: String,
    /// The jump as a message quotes it back.
    pub text: String,
    /// The line the label is written on -- the label's identity, because two functions in one
    /// file may each label a loop with one name.
    pub label_line: usize,
    /// Whether deleting the label would land in the same place: a fact about the language's
    /// binding rule for a bare jump, which only a provider for that language can state.
    pub has_same_target_unlabeled: bool,
    /// Whether the jump stands inside a construct that catches a bare `break` and is not the
    /// target loop -- a Go `switch` or `select`. Always false in a language with no such
    /// construct.
    pub is_inside_break_capture: bool,
}

impl LabeledJump
{
    /// This jump as a record of [`LABELED_JUMP`].
    #[must_use]
    pub fn Record(&self) -> SiteRecord
    {
        let values = BTreeMap::from([
            (KEYWORD.to_owned(), SiteValue::Text(self.keyword.clone())),
            (LABEL.to_owned(), SiteValue::Text(self.label.clone())),
            (TEXT.to_owned(), SiteValue::Text(self.text.clone())),
            // A source with more lines than an `i64` counts does not exist; saturating states
            // that rather than inventing a failure the caller would have to handle.
            (LABEL_LINE.to_owned(), SiteValue::Integer(i64::try_from(self.label_line).unwrap_or(i64::MAX))),
            (HAS_SAME_TARGET_UNLABELED.to_owned(), SiteValue::Truth(self.has_same_target_unlabeled)),
            (IS_INSIDE_BREAK_CAPTURE.to_owned(), SiteValue::Truth(self.is_inside_break_capture)),
        ]);

        return SiteRecord { kind: LABELED_JUMP.name.to_owned(), line: self.line, values };
    }

    /// The labeled jump `record` holds, or `None` when it is another kind's record or lacks a
    /// field this kind declares.
    #[must_use]
    pub fn From_Record(record: &SiteRecord) -> Option<Self>
    {
        if record.kind != LABELED_JUMP.name
        {
            return None;
        }

        return Some(Self {
            line: record.line,
            keyword: record.Text(KEYWORD)?.to_owned(),
            label: record.Text(LABEL)?.to_owned(),
            text: record.Text(TEXT)?.to_owned(),
            label_line: usize::try_from(record.Integer(LABEL_LINE)?).ok()?,
            has_same_target_unlabeled: record.Truth(HAS_SAME_TARGET_UNLABELED)?,
            is_inside_break_capture: record.Truth(IS_INSIDE_BREAK_CAPTURE)?,
        });
    }

    /// Every labeled jump `payload` records, in source order -- or `None` when any record of this
    /// kind cannot be read as one, because a list silently missing the record that failed is the
    /// shape of a file judged on part of what it holds.
    ///
    /// [`crate::Parse_Sites_Payload`] has already refused a record lacking a declared field, so
    /// over a decoded payload this is `Some`; the `None` is for one assembled by hand.
    #[must_use]
    pub fn All_In(payload: &SitesPayload) -> Option<Vec<Self>>
    {
        return payload.Records_Of(LABELED_JUMP.name).map(Self::From_Record).collect();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The line the fixture jump is written on.
    const JUMP_LINE: usize = 5;
    /// The line the fixture jump's label is written on.
    const LABEL_LINE_OF_FIXTURE: usize = 2;

    fn Jump() -> LabeledJump
    {
        return LabeledJump {
            line: JUMP_LINE,
            keyword: "break".to_owned(),
            label: "'outer".to_owned(),
            text: "break 'outer".to_owned(),
            label_line: LABEL_LINE_OF_FIXTURE,
            has_same_target_unlabeled: false,
            is_inside_break_capture: false,
        };
    }

    #[test]
    fn Test_Record_Should_Carry_Exactly_The_Declared_Fields()
    {
        let record = Jump().Record();

        let carried: Vec<&str> = record.values.keys().map(String::as_str).collect();
        let mut declared: Vec<&str> = LABELED_JUMP.fields.iter().map(|field| return field.name).collect();
        declared.sort_unstable();
        assert_eq!(carried, declared);
        for field in LABELED_JUMP.fields
        {
            assert_eq!(record.values.get(field.name).map(SiteValue::Type), Some(field.value), "{}", field.name);
        }
    }

    #[test]
    fn Test_From_Record_Should_Read_Back_What_Record_Wrote()
    {
        assert_eq!(LabeledJump::From_Record(&Jump().Record()), Some(Jump()));
    }

    #[test]
    fn Test_From_Record_Should_Refuse_Another_Kind_And_A_Short_Record()
    {
        let mut other = Jump().Record();
        other.kind = "another-kind".to_owned();
        assert_eq!(LabeledJump::From_Record(&other), None);

        let mut short = Jump().Record();
        short.values.remove(LABEL_LINE);
        assert_eq!(LabeledJump::From_Record(&short), None);
    }

    #[test]
    fn Test_All_In_Should_Refuse_A_Payload_With_One_Unreadable_Record()
    {
        let mut short = Jump().Record();
        short.values.remove(KEYWORD);
        let payload = SitesPayload { offered: vec![LABELED_JUMP.name.to_owned()], declined: Vec::new(), records: vec![Jump().Record(), short] };

        assert_eq!(LabeledJump::All_In(&payload), None);

        let whole = SitesPayload { records: vec![Jump().Record()], ..payload };
        assert_eq!(LabeledJump::All_In(&whole), Some(vec![Jump()]));
    }
}
