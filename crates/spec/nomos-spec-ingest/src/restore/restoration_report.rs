use super::{Member, Restored};
use core::fmt::Write as _;

/// How many members a family line names before it falls back to the count alone.
const NAMED_IN_A_SUMMARY: usize = 3;

#[derive(Clone, Debug, Default)]
pub struct RestorationReport
{
    /// Every member, named. Counts are queries over this.
    pub members: Vec<Member>,
    /// Names more than one restored member claims, so none of them takes it.
    ///
    /// The corpus really does define `Capability` twice — once as a canonical domain model
    /// and once as a glossary term — and they are two nodes. Handing the bare name to
    /// whichever volume was read first would make resolution depend on directory order and
    /// answer confidently with one of two right answers.
    pub ambiguous_names: Vec<String>,
    /// Aliases something outside the restoration already owned, per alias rather than
    /// counted. Left where they are: a restoration may not repoint another authority's
    /// name.
    pub contested_aliases: Vec<String>,
}

impl RestorationReport
{
    #[must_use]
    pub fn In(&self, family: Restored) -> Vec<&Member>
    {
        return self
            .members
            .iter()
            .filter(|member| return member.family == family)
            .collect();
    }

    #[must_use]
    pub fn Named(&self, name: &str) -> Option<&Member>
    {
        return self
            .members
            .iter()
            .find(|member| return member.name == name || member.id == name);
    }

    /// Names families and their members, never a bare total.
    #[must_use]
    pub fn Summary(&self) -> String
    {
        let mut lines = Vec::new();
        for family in Restored::All()
        {
            lines.push(self.Family_Line(*family));
        }

        return lines.join("\n");
    }

    /// One family's count, and up to three of the members behind it.
    ///
    /// Naming a few is what makes a count checkable against the volume by eye; naming all
    /// of them would make the summary the report it is supposed to introduce.
    fn Family_Line(&self, family: Restored) -> String
    {
        let members = self.In(family);
        let named: Vec<&str> = members
            .iter()
            .take(NAMED_IN_A_SUMMARY)
            .map(|member| return member.name.as_str())
            .collect();
        let mut line = format!("{}: {} restored", family.Label(), members.len());
        if !named.is_empty()
        {
            let _ = write!(
                line,
                " ({}{})",
                named.join(", "),
                if members.len() > named.len() { ", …" } else { "" }
            );
        }

        return line;
    }
}
