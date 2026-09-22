//! A root, held.
//!
//! # What is here and what is deliberately not
//!
//! The state, the request loop and the two private steps a judgment needs. What is not here
//! is any judgment: [`Run_Judgment`] hands the held workspace, store and cache to
//! `nomos_check_orchestration::Run_Reassessing` and carries back what it returns, unchanged
//! and unreduced. That is the whole of this crate's claim to agree with a cold invocation --
//! not that two paths were checked against each other, but that there is one path.

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::{CheckOutcome, RuleReassessmentCache, RunContext, Run_Reassessing};
use nomos_composer_std::{FileSystem, ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_rules::SourceFile;
use nomos_workspace::{Change, Workspace};
use std::path::{Path, PathBuf};

use crate::build_variant::Host_Variant;
use crate::sources::Walked_Sources;
use crate::{ResidencyCost, ResidentAnswer, ResidentRefusal, ResidentRequest, StopReport, TreeReading};

/// One root and everything a second request over it should not have to pay for again.
///
/// # Why nothing is derived
///
/// Neither `Debug` nor `Default`, for `nomos-lsp`'s own reasons one layer over: a
/// [`Workspace`] implements neither, and a fact store has no meaningful empty value distinct
/// from the one [`Self::Start`] builds. A root is not optional either, so a `Default` would
/// have to invent one.
pub struct Resident
{
    root: PathBuf,
    workspace: Option<Workspace>,
    store: MemoryFactStore,
    reassessment: RuleReassessmentCache,
    answered: usize,
    stopped: bool,
}

impl Resident
{
    /// A resident over `root`, holding nothing yet.
    ///
    /// Refuses a root that cannot be read as a directory now, rather than letting the first
    /// request discover it: a service that accepted a residency it can never answer is a
    /// service whose failure arrives at whichever client asks first. The check is one level
    /// through the platform port, which is all `OD-PLATFORM-002`'s own floor offers and all
    /// this question needs -- whether the root is a directory at all, not what is under it.
    ///
    /// # Errors
    ///
    /// [`ResidentRefusal::RootIsNotWalkable`] when `root` is not a directory this process
    /// can read. No residency begins, so there is nothing for the caller to stop.
    pub fn Start(root: &Path) -> Result<Self, ResidentRefusal>
    {
        Refuse_An_Unreadable_Root(root)?;

        return Ok(Self {
            root: root.to_path_buf(),
            workspace: None,
            store: MemoryFactStore::New(),
            reassessment: RuleReassessmentCache::New(),
            answered: 0,
            stopped: false,
        });
    }

    /// The root this residency is over.
    #[must_use]
    pub fn Root(&self) -> &Path
    {
        return &self.root;
    }

    /// How many requests this resident has answered.
    #[must_use]
    pub const fn Answered(&self) -> usize
    {
        return self.answered;
    }

    /// Answers one request, or refuses it.
    ///
    /// `&mut self` is the concurrency decision rather than an implementation detail, and it
    /// is the weaker half of it: a resident is not `Send` either, so two requests to one
    /// resident cannot be concurrent even with a lock around it. The crate doc carries what
    /// was measured, what in `nomos-analysis` causes it, and the shape a concurrent caller
    /// has instead.
    ///
    /// # Errors
    ///
    /// [`ResidentRefusal::AlreadyStopped`] when a client already ended this residency --
    /// every request after that one, including a second [`ResidentRequest::Stop`].
    /// [`ResidentRefusal::RootIsNotWalkable`] when [`ResidentRequest::Judge`] or
    /// [`ResidentRequest::Observe`] can no longer read the root, rather than either of them
    /// being answered from what the resident is still holding.
    pub fn Answer(&mut self, request: ResidentRequest) -> Result<ResidentAnswer, ResidentRefusal>
    {
        if self.stopped
        {
            return Err(ResidentRefusal::AlreadyStopped);
        }

        let answer = match request
        {
            ResidentRequest::Judge => self.Judged()?,
            ResidentRequest::Observe => ResidentAnswer::Observed { reading: self.Read_Tree()? },
            ResidentRequest::Forget => self.Forgotten(),
            ResidentRequest::Stop => self.Ended(),
        };
        self.answered = self.answered.saturating_add(1);

        return Ok(answer);
    }

    /// The root judged through everything this resident holds.
    ///
    /// The reading is taken before the run and the cost after it, because the run is what
    /// moves both: ingesting the walk advances the held workspace past the state the reading
    /// compares against, and materializing is what the cost counts.
    fn Judged(&mut self) -> Result<ResidentAnswer, ResidentRefusal>
    {
        let sources = self.Walked()?;
        let reading = Reading_Of(self.workspace.as_ref(), &sources);
        let before = self.store.Materializations();

        let outcome = Run_Judgment(self, &sources);

        let cost = ResidencyCost {
            materializations: self.store.Materializations().saturating_sub(before),
            live_facts: self.store.Live(),
        };

        return Ok(ResidentAnswer::Judged { outcome, reading, cost });
    }

    /// The tree read and compared, judging nothing.
    ///
    /// Compared against what the resident holds, which a bare observation does not advance --
    /// so two observations with no judgment between them report the same moved set, because
    /// what they are both measured against is the last state that was actually ingested.
    fn Read_Tree(&self) -> Result<TreeReading, ResidentRefusal>
    {
        let sources = self.Walked()?;

        return Ok(Reading_Of(self.workspace.as_ref(), &sources));
    }

    /// Everything held, dropped.
    fn Forgotten(&mut self) -> ResidentAnswer
    {
        let released_facts = self.store.Live();

        self.Release();

        return ResidentAnswer::Forgotten { released_facts };
    }

    /// The residency, ended.
    fn Ended(&mut self) -> ResidentAnswer
    {
        let report = StopReport { answered: self.answered, released_facts: self.store.Live() };

        self.Release();
        self.stopped = true;

        return ResidentAnswer::Stopped(report);
    }

    /// The three held values, replaced by what [`Self::Start`] built.
    ///
    /// Replaced rather than emptied in place, so that a forgotten resident and a
    /// just-started one are the same thing and `OD-HOST-002`'s question -- is any fact lost
    /// that a canonical service could not give back -- has one answer instead of two.
    fn Release(&mut self)
    {
        self.workspace = None;
        self.store = MemoryFactStore::New();
        self.reassessment = RuleReassessmentCache::New();
    }

    /// The sources under this residency's root, or the refusal that the root is gone.
    fn Walked(&self) -> Result<Vec<SourceFile>, ResidentRefusal>
    {
        return Walked_Sources(&self.root)
            .ok_or_else(|| return ResidentRefusal::RootIsNotWalkable { root: self.root.clone() });
    }
}

/// The outcome `nomos_check_orchestration::Run_Reassessing` reaches over `sources`, through
/// the workspace, fact store and reassessment cache `resident` holds between requests.
///
/// A free function rather than a method because the three held values are borrowed mutably
/// and at once; taking `resident` whole is what lets the call name each field exactly once.
fn Run_Judgment(resident: &mut Resident, sources: &[SourceFile]) -> CheckOutcome
{
    return Run_Reassessing(
        sources,
        RunContext {
            variant: Host_Variant(),
            root: &resident.root,
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            workspace: &mut resident.workspace,
            store: &mut resident.store,
        },
        &[],
        &mut resident.reassessment,
    );
}

/// What a walk read, and which of it differs from what `workspace` already carries.
fn Reading_Of(workspace: Option<&Workspace>, sources: &[SourceFile]) -> TreeReading
{
    let changed = sources
        .iter()
        .filter(|source| return Moved_Under(workspace, source))
        .map(|source| return source.path.clone())
        .collect();

    return TreeReading { watched: sources.len(), changed };
}

/// Whether `source`'s content differs from the digest `workspace` holds for that path.
///
/// The comparison is the workspace's own on both sides: `Content_Of` for what is held and
/// `nomos_workspace::Change::Content_Digest` for what was just walked, which is the same
/// value `nomos-check-orchestration` will file the source under a moment later. A digest
/// computed here instead would be a second answer to a question the workspace already owns,
/// and the two could disagree about what "unchanged" means.
///
/// A path the workspace does not hold has moved, which is every path on a first request.
fn Moved_Under(workspace: Option<&Workspace>, source: &SourceFile) -> bool
{
    let Some(held) = workspace.and_then(|held| return held.Content_Of(&source.path))
    else
    {
        return true;
    };
    let walked = Change::Present { path: source.path.clone(), content: source.text.clone() };

    return walked.Content_Digest() != Some(held);
}

/// Refuses a root that is not a directory this process can read.
///
/// The port's error is tested rather than carried. Every way it can fail -- the path is a
/// file, it does not exist, the process may not read it -- has the same remedy and the same
/// refusal, and a refusal naming the root is what a client acts on; a second sentence
/// describing which of the three it was would be text nothing branches on.
fn Refuse_An_Unreadable_Root(root: &Path) -> Result<(), ResidentRefusal>
{
    if FILE_SYSTEM.Read_Directory(root).is_err()
    {
        return Err(ResidentRefusal::RootIsNotWalkable { root: root.to_path_buf() });
    }

    return Ok(());
}

#[cfg(test)]
mod tests;
