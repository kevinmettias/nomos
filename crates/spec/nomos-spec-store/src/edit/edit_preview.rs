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
///
/// Named `EditPreview` rather than `Preview` because this crate publishes nine subsystems'
/// vocabulary at one flat root, where a bare `Preview` would not say whose it is.
#[derive(Debug)]
pub struct EditPreview
{
    pub(crate) staged: StagedEdit,
    pub(crate) blocks: Vec<BlockChange>,
    pub(crate) identity: Vec<IdentityChange>,
    pub(crate) relations_added: Vec<RecordRelation>,
    pub(crate) relations_removed: Vec<RecordRelation>,
    pub(crate) statements: Vec<NormativeMovement>,
}

impl EditPreview
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
    pub fn Is_Wording_Moved(&self) -> bool
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
    pub fn Has_No_Changes(&self) -> bool
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
        if self.Has_No_Changes()
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
        if self.Is_Wording_Moved()
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::NormativeOutcome;
    use crate::{ClaimedRecord, RecordProjection, StagedEdit};
    use nomos_spec_model::{FrontMatter as RecordFrontMatter, Record};

    #[test]
    fn Test_Node_Id_Should_Name_The_Record_Being_Previewed()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("a.md")).Node_Id(), "D-1");
    }

    #[test]
    fn Test_Rename_Should_Report_The_Two_Paths_Only_When_They_Differ()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("b.md")).Rename(), Some(("a.md", "b.md")));
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("a.md")).Rename(), None);
    }

    #[test]
    fn Test_Blocks_Should_Return_What_The_Edit_Changed()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("a.md")).Blocks().len(), 1);
    }

    #[test]
    fn Test_Identity_Should_Return_What_The_Front_Matter_Changed()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("a.md")).Identity().len(), 1);
    }

    #[test]
    fn Test_Relations_Added_Should_Return_What_The_Edit_Would_Add()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("a.md")).Relations_Added().len(), 1);
    }

    #[test]
    fn Test_Relations_Removed_Should_Return_What_The_Edit_Would_Withdraw()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("a.md")).Relations_Removed().len(), 1);
    }

    #[test]
    fn Test_Statements_Should_Return_What_Became_Of_Each_One()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("a.md")).Statements().len(), 1);
    }

    #[test]
    fn Test_Markdown_Should_Return_The_Staged_Bytes()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("a.md")).Markdown(), "# D-1\n\nNew.\n");
    }

    #[test]
    fn Test_Path_Should_Return_Where_The_Staged_Edit_Would_Live()
    {
        assert_eq!(A_Preview(BeforePath("a.md"), AfterPath("b.md")).Path(), "b.md");
    }

    #[test]
    fn Test_Has_No_Changes_Should_Be_False_When_Anything_Changed()
    {
        assert!(!A_Preview(BeforePath("a.md"), AfterPath("a.md")).Has_No_Changes());
    }

    #[test]
    fn Test_Describe_Should_Report_The_Rename_And_Every_Change()
    {
        let text = A_Preview(BeforePath("a.md"), AfterPath("b.md")).Describe();

        assert!(text.contains("renamed: a.md -> b.md"), "{text}");
        assert!(text.contains("relation added: relates-to D-2"), "{text}");
    }

    #[derive(Clone, Copy)]
    struct BeforePath<'a>(&'a str);
    #[derive(Clone, Copy)]
    struct AfterPath<'a>(&'a str);

    /// A preview carrying one of every kind of change, so each accessor has something real
    /// to return. `before_path` and `after_path` let a caller isolate the rename question.
    fn A_Preview(before_path: BeforePath<'_>, after_path: AfterPath<'_>) -> EditPreview
    {
        let staged = StagedEdit {
            claimed: A_Claimed(NodeId("D-1"), RecordPath(before_path.0), ClaimedText("# D-1\n\nOld.\n")),
            path: after_path.0.to_owned(),
            markdown: "# D-1\n\nNew.\n".to_owned(),
            record: Record {
                front_matter: A_Front_Matter("D-1"),
                body: "# D-1\n\nNew.\n".to_owned(),
            },
        };

        return EditPreview {
            staged,
            blocks: vec![BlockChange::Reworded {
                ordinal: 1,
                before: "Old.".to_owned(),
                after: "New.".to_owned(),
            }],
            identity: vec![IdentityChange {
                field: "status".to_owned(),
                before: "draft".to_owned(),
                after: "accepted".to_owned(),
            }],
            relations_added: vec![RecordRelation {
                target: "D-2".to_owned(),
                relation: "relates-to".to_owned(),
            }],
            relations_removed: vec![RecordRelation {
                target: "D-3".to_owned(),
                relation: "supersedes".to_owned(),
            }],
            statements: vec![NormativeMovement {
                statement_id: "S-1".to_owned(),
                canonical_hash: "sha256:xyz".to_owned(),
                outcome: NormativeOutcome::Held { block: 1 },
            }],
        };
    }

    #[derive(Clone, Copy)]
    struct NodeId<'a>(&'a str);
    #[derive(Clone, Copy)]
    struct RecordPath<'a>(&'a str);
    #[derive(Clone, Copy)]
    struct ClaimedText<'a>(&'a str);

    fn A_Claimed(node_id: NodeId<'_>, path: RecordPath<'_>, markdown: ClaimedText<'_>) -> ClaimedRecord
    {
        return ClaimedRecord {
            projection: RecordProjection {
                node_id: node_id.0.to_owned(),
                path: path.0.to_owned(),
                revision: "v1".to_owned(),
                markdown: markdown.0.to_owned(),
                source_hash: "sha256:same".to_owned(),
                projected_hash: "sha256:same".to_owned(),
            },
            front_matter: A_Front_Matter(node_id.0),
            document_uid: 1,
        };
    }

    fn A_Front_Matter(node_id: &str) -> RecordFrontMatter
    {
        return RecordFrontMatter {
            id: node_id.to_owned(),
            kind: "decision".to_owned(),
            title: "A record".to_owned(),
            status: "accepted".to_owned(),
            authority: "canonical-normative-record".to_owned(),
            version: 1,
            tags: Vec::new(),
            relations: Vec::new(),
        };
    }
}
