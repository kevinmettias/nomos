//! What committing an edit would change.

use nomos_spec_model::RecordRelation;

use crate::BlockChange;
use crate::IdentityChange;
use crate::NormativeMovement;
use crate::StagedEdit;

/// What committing an edit would change.
///
/// Only [`StagedEdit::Preview`] can build one, and [`SpecificationStore::Commit_Edit`] takes
/// one. That is the whole mechanism by which the mandatory preview is mandatory.
#[derive(Debug)]
pub struct Preview
{
    pub(crate) staged: StagedEdit,
    pub(crate) blocks: Vec<BlockChange>,
    pub(crate) identity: Vec<IdentityChange>,
    pub(crate) relations_added: Vec<RecordRelation>,
    pub(crate) relations_removed: Vec<RecordRelation>,
    pub(crate) statements: Vec<NormativeMovement>,
}

impl Preview
{
    #[must_use]
    pub fn Node_Id(&self) -> &str
    {
        return &self.staged.claimed.projection.node_id;
    }

    /// The paths before and after, when they differ.
    #[must_use]
    pub fn Rename(&self) -> Option<(&str, &str)>
    {
        let before = self.staged.claimed.projection.path.as_str();
        let after = self.staged.path.as_str();

        return (before != after).then_some((before, after));
    }

    #[must_use]
    pub fn Blocks(&self) -> &[BlockChange]
    {
        return &self.blocks;
    }

    #[must_use]
    pub fn Identity(&self) -> &[IdentityChange]
    {
        return &self.identity;
    }

    #[must_use]
    pub fn Relations_Added(&self) -> &[RecordRelation]
    {
        return &self.relations_added;
    }

    #[must_use]
    pub fn Relations_Removed(&self) -> &[RecordRelation]
    {
        return &self.relations_removed;
    }

    #[must_use]
    pub fn Statements(&self) -> &[NormativeMovement]
    {
        return &self.statements;
    }

    /// The staged bytes, so a caller can write them where the author expects them.
    #[must_use]
    pub fn Markdown(&self) -> &str
    {
        return &self.staged.markdown;
    }

    #[must_use]
    pub fn Path(&self) -> &str
    {
        return &self.staged.path;
    }

    /// The question `D-129` calls mandatory: does this edit move wording that was there?
    ///
    /// Answered from two kinds of evidence, because they are not the same claim. A normative
    /// statement recorded against this record is the strong answer; the blocks are the answer
    /// available for every record, including the ones whose statements no corpus has been
    /// ingested for. A preview that reported only the strong answer would print nothing at
    /// all for this repository's own records, and nothing reads as *no*.
    #[must_use]
    pub fn Wording_Moved(&self) -> bool
    {
        use crate::NormativeOutcome;

        return self.blocks.iter().any(BlockChange::Is_Disturbing_Wording)
            || self.statements.iter().any(|movement| {
                return matches!(
                    movement.outcome,
                    NormativeOutcome::Moved { .. } | NormativeOutcome::Gone { .. }
                );
            });
    }

    /// Whether this edit changes anything at all.
    #[must_use]
    pub fn Changes_Nothing(&self) -> bool
    {
        return self.blocks.is_empty()
            && self.identity.is_empty()
            && self.relations_added.is_empty()
            && self.relations_removed.is_empty()
            && self.Rename().is_none();
    }

    /// The preview an author reads.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        let mut lines = vec![format!("{} at {}", self.Node_Id(), self.Path())];

        self.Describe_Changes(&mut lines);
        if self.Changes_Nothing()
        {
            lines.push("  nothing changes".to_owned());
        }
        lines.push(self.Describe_Normative());

        return lines.join("\n");
    }

    /// One line for each thing the edit changes, in the order an author reads them.
    fn Describe_Changes(&self, lines: &mut Vec<String>)
    {
        if let Some((before, after)) = self.Rename()
        {
            lines.push(format!("  renamed: {before} -> {after}"));
        }
        for change in &self.identity
        {
            lines.push(format!("  {}: {:?} -> {:?}", change.field, change.before, change.after));
        }
        for change in &self.blocks
        {
            lines.push(format!("  {}", change.Describe()));
        }
        for relation in &self.relations_added
        {
            lines.push(format!("  relation added: {} {}", relation.relation, relation.target));
        }
        for relation in &self.relations_removed
        {
            lines.push(format!("  relation removed: {} {}", relation.relation, relation.target));
        }
    }

    /// The mandatory sentence, and what it was derived from.
    fn Describe_Normative(&self) -> String
    {
        let verdict = self.Verdict();

        if self.statements.is_empty()
        {
            return format!(
                "  {verdict} - derived from the blocks, because no normative statement is \
                 recorded against {} in this store",
                self.Node_Id()
            );
        }

        return format!("  {verdict} - {}", self.Statement_Detail());
    }

    /// The sentence `D-129` calls mandatory, in the two words it can take.
    fn Verdict(&self) -> &'static str
    {
        if self.Wording_Moved()
        {
            return "normative wording moved";
        }

        return "no normative wording moved";
    }

    /// Each statement recorded against this record, and what became of it.
    fn Statement_Detail(&self) -> String
    {
        return self
            .statements
            .iter()
            .map(|movement| {
                return format!("{} {}", movement.statement_id, movement.outcome.Describe());
            })
            .collect::<Vec<String>>()
            .join(", ");
    }
}
