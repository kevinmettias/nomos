//! What one revision lost against the one before it.

use core::fmt::Write as _;
use crate::fate::Fate;

/// How many losses a summary line names before it falls back to the count alone.
const NAMED_IN_A_SUMMARY: usize = 3;

/// How much of the widest undeclared text the filler line quotes. Enough to recognise the
/// boilerplate on sight, short enough that the line stays one line.
const EXCERPT_CHARACTERS: usize = 72;

use crate::tally::Tally;
use crate::restored::Restored;
use crate::filler_census::FillerCensus;
use crate::member_fate::MemberFate;
use crate::document_fate::DocumentFate;
#[derive(Clone, Debug, Default)]
pub struct RegressionReport
{
    pub from: String,
    pub to: String,
    pub documents: DocumentFate,
    pub members: Vec<MemberFate>,
    pub filler: FillerCensus,
}

impl RegressionReport
{
    #[must_use]
    pub fn In(&self, family: Restored) -> Vec<&MemberFate>
    {
        return self
            .members
            .iter()
            .filter(|member| return member.family == family)
            .collect();
    }

    #[must_use]
    pub fn Tally(&self, family: Restored) -> Tally
    {
        let mut tally = Tally::default();

        for member in self.In(family)
        {
            let counter = match member.fate
            {
                Fate::Preserved { .. } => &mut tally.preserved,
                Fate::Hollowed { .. } => &mut tally.hollowed,
                Fate::Mentioned { .. } => &mut tally.mentioned,
                Fate::Gone => &mut tally.gone,
            };
            *counter = counter.saturating_add(1);
        }

        return tally;
    }

    #[must_use]
    pub fn Named(&self, name: &str) -> Option<&MemberFate>
    {
        return self
            .members
            .iter()
            .find(|member| return member.name == name || member.id == name);
    }

    #[must_use]
    pub fn Summary(&self) -> String
    {
        let documents = self.Documents_Line();
        let mut lines = vec![format!("{} -> {}", self.from, self.to), documents];

        for family in Restored::All()
        {
            if let Some(line) = self.Family_Line(*family)
            {
                lines.push(line);
            }
        }

        let filler = self.Filler_Line();
        lines.push(filler);

        return lines.join("\n");
    }

    /// What moved between the two revisions, at the level of whole documents.
    fn Documents_Line(&self) -> String
    {
        return format!(
            "  documents: {} appeared, {} disappeared, {} changed in place, {} relocated",
            self.documents.appeared.len(),
            self.documents.disappeared.len(),
            self.documents.changed.len(),
            self.documents.relocated.len()
        );
    }

    /// One family's tallies, or nothing at all when the revision carried no member of it.
    ///
    /// A family with a zero total is left out rather than printed as four zeroes, because a
    /// summary that lists every family the build knows about buries the one that moved.
    fn Family_Line(&self, family: Restored) -> Option<String>
    {
        let tally = self.Tally(family);
        if tally.Total() == 0
        {
            return None;
        }

        let named = self.Named_Losses(family);
        let mut line = format!(
            "  {}: {} preserved, {} hollowed, {} mentioned, {} gone",
            family.Label(),
            tally.preserved,
            tally.hollowed,
            tally.mentioned,
            tally.gone
        );
        if !named.is_empty()
        {
            let _ = write!(line, " ({})", named.join(", "));
        }

        return Some(line);
    }

    /// Up to three members of a family that did not survive.
    ///
    /// Naming a few is what makes a tally actionable; naming all of them would make the
    /// summary the report it is supposed to introduce.
    fn Named_Losses(&self, family: Restored) -> Vec<&str>
    {
        return self
            .In(family)
            .iter()
            .filter(|member| return !matches!(member.fate, Fate::Preserved { .. }))
            .take(NAMED_IN_A_SUMMARY)
            .map(|member| return member.name.as_str())
            .collect();
    }

    /// What the blocklist matched, and the widest thing standing on filler that it did not.
    fn Filler_Line(&self) -> String
    {
        let mut filler = format!(
            "  filler: {} documents the blocklist matches, {} carrying nothing but filler",
            self.filler.declared.len(),
            self.filler.stubs.len()
        );
        if let Some(widest) = self.filler.Widest_Undeclared()
        {
            let _ = write!(
                filler,
                "\n  undeclared: {} sections across {} documents stand on \"{}\"",
                widest.sections,
                widest.documents.len(),
                widest.text.chars().take(EXCERPT_CHARACTERS).collect::<String>()
            );
        }

        return filler;
    }
}
