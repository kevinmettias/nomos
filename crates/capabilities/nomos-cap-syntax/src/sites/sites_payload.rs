//! A decoded `nomos.syntax.sites.v1` payload.

use super::{KindDecline, KindStance, SiteRecord};

/// One file's sites: which kinds its provider offers, which it declines and why, and every
/// record of an offered kind in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SitesPayload
{
    pub offered: Vec<String>,
    pub declined: Vec<KindDecline>,
    pub records: Vec<SiteRecord>,
}

impl SitesPayload
{
    /// What this payload says about `kind`.
    #[must_use]
    pub fn Stance(&self, kind: &str) -> KindStance<'_>
    {
        if self.offered.iter().any(|offered| return offered == kind)
        {
            return KindStance::Offered;
        }

        return self
            .declined
            .iter()
            .find(|decline| return decline.kind == kind)
            .map_or(KindStance::Unanswered, |decline| return KindStance::Declined(decline.reason.as_str()));
    }

    /// Every record of `kind`, in source order.
    pub fn Records_Of<'payload>(&'payload self, kind: &'payload str) -> impl Iterator<Item = &'payload SiteRecord>
    {
        return self.records.iter().filter(move |record| return record.kind == kind);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::collections::BTreeMap;

    fn Payload() -> SitesPayload
    {
        return SitesPayload {
            offered: vec!["offered-kind".to_owned()],
            declined: vec![KindDecline { kind: "declined-kind".to_owned(), reason: "the language has no such thing".to_owned() }],
            records: vec![
                SiteRecord { kind: "offered-kind".to_owned(), line: 1, values: BTreeMap::new() },
                SiteRecord { kind: "other-kind".to_owned(), line: 2, values: BTreeMap::new() },
            ],
        };
    }

    #[test]
    fn Test_Stance_Should_Tell_Offered_Declined_And_Unanswered_Apart()
    {
        let payload = Payload();

        assert_eq!(payload.Stance("offered-kind"), KindStance::Offered);
        assert_eq!(payload.Stance("declined-kind"), KindStance::Declined("the language has no such thing"));
        assert_eq!(payload.Stance("never-named"), KindStance::Unanswered);
    }

    #[test]
    fn Test_Records_Of_Should_Yield_Only_That_Kinds_Records()
    {
        let payload = Payload();

        let lines: Vec<usize> = payload.Records_Of("offered-kind").map(|record| return record.line).collect();

        assert_eq!(lines, vec![1]);
    }
}
