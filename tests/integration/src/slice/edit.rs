//! Changing the workspace, and invalidating exactly what the change reached.
//!
//! An edit that says what the workspace already said invalidates nothing, and that is not
//! the same as an invalidation pass that ran and reached nothing.

use super::*;

impl Slice
{
    /// The corpus is rewritten to match, because it is the reading of the tree the providers
    /// actually parse. A silent no-op here would make an invalidation test assert that
    /// changing nothing invalidates nothing.
    pub(super) fn Assert_The_Corpus_Holds(corpus: &mut Corpus, path: &str, content: &str)
    {
        assert!(
            corpus.Rewrite(path, content),
            "`{path}` is not in the corpus, so this edit changed the workspace and nothing \
             the providers read"
        );
    }

    /// A file changed, so the cause is file-granular. The engine broadens it to whatever each
    /// affected provider can actually deliver, and records having done so — the rollup will be
    /// broadened to Project.
    pub(super) fn Invalidate_One_Subject(&mut self, path: &str) -> InvalidationReport
    {
        return self.store.Invalidate(
            &nomos_analysis::GenerationCause::SubjectChanged {
                subject: Subject_Of_Path(path),
                granularity: IncrementalGranularity::File,
            },
            self.generation,
        );
    }

    /// The whole landing as one change set, applied through the one door.
    pub(super) fn Land(&mut self, landing: &[(&str, &str)]) -> Applied
    {
        let mut checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
        for (path, content) in landing
        {
            checkout = checkout.Present(*path, *content);
        }

        return self
            .workspace
            .Apply(&checkout)
            .unwrap_or_else(|error| panic!("the checkout was refused: {error}"));
    }

    /// Every path in the landing has to be one the providers read.
    pub(super) fn Assert_The_Corpus_Holds_Each(corpus: &mut Corpus, landing: &[(&str, &str)])
    {
        for (path, content) in landing
        {
            assert!(
                corpus.Rewrite(path, content),
                "`{path}` is not in the corpus, so this checkout changed the workspace and \
                 nothing the providers read"
            );
        }
    }

    /// Read off what the workspace said rather than recomputed against it. A second
    /// computation of the same thing is a second answer waiting to disagree.
    pub(super) fn Differing_Members(applied: &Applied) -> BTreeSet<SubjectId>
    {
        return applied
            .Effects()
            .iter()
            .filter(|effect| return effect.Altered())
            .map(|effect| return Subject_Of_Path(effect.Path()))
            .collect();
    }

    /// A checkout replaces the workspace state wholesale, so the store is told which members
    /// are not the same in both.
    pub(super) fn Invalidate_The_Whole_Tree(
        &mut self,
        from: SnapshotId,
        to: SnapshotId,
        differing: BTreeSet<SubjectId>,
    ) -> InvalidationReport
    {
        return self.store.Invalidate(
            &nomos_analysis::GenerationCause::SnapshotReplaced {
                from,
                to,
                differing,
            },
            self.generation,
        );
    }
}
