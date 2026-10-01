//! A policy made ready to judge lines, paths, messages and identities.

use crate::matcher::{Fold, Line};
use crate::party_policy::{LetterCase, PartyPolicy, Pattern};
use crate::refusal::{IDENTITY_RULE, Place, Refusal, Shown};

/// One rule with its words already split into characters, folded where case is ignored.
struct Compiled
{
    id: String,
    why: String,
    test: Test,
}

enum Test
{
    Phrases { phrases: Vec<Vec<Vec<char>>>, boundary: crate::party_policy::Boundary, ignore_case: bool },
    Tickets { keys: Vec<Vec<char>> },
}

/// A [`PartyPolicy`] ready to judge.
pub struct Judge<'policy>
{
    policy: &'policy PartyPolicy,
    rules: Vec<Compiled>,
}

impl<'policy> Judge<'policy>
{
    /// Prepares every rule once.
    #[must_use]
    pub fn New(policy: &'policy PartyPolicy) -> Self
    {
        let rules = policy
            .rules
            .iter()
            .map(|rule| {
                let test = match &rule.pattern
                {
                    Pattern::Phrases { phrases, boundary, letter_case } =>
                    {
                        let ignore_case = *letter_case == LetterCase::Ignored;
                        let phrases = phrases
                            .iter()
                            .map(|words| {
                                return words
                                    .iter()
                                    .map(|word| return word.chars().map(|character| return if ignore_case { Fold(character) } else { character }).collect())
                                    .collect();
                            })
                            .collect();
                        Test::Phrases { phrases, boundary: *boundary, ignore_case }
                    },
                    Pattern::Tickets { keys } => Test::Tickets { keys: keys.iter().map(|key| return key.chars().collect()).collect() },
                };
                return Compiled { id: rule.id.clone(), why: rule.why.clone(), test };
            })
            .collect();

        return Self { policy, rules };
    }

    /// The party's label.
    #[must_use]
    pub fn Party(&self) -> &str
    {
        return &self.policy.party;
    }

    /// Every rule a line of text breaks, each reported once.
    #[must_use]
    pub fn Text(&self, place: &Place, text: &str) -> Vec<Refusal>
    {
        let line = Line::Of(text);
        return self
            .rules
            .iter()
            .filter(|rule| {
                return match &rule.test
                {
                    Test::Phrases { phrases, boundary, ignore_case } => phrases.iter().any(|words| return line.Has_Phrase(words, *boundary, *ignore_case)),
                    Test::Tickets { keys } => keys.iter().any(|key| return line.Has_Ticket(key)),
                };
            })
            .map(|rule| return Refusal { rule: rule.id.clone(), place: place.clone(), text: Shown(text), why: rule.why.clone() })
            .collect();
    }

    /// A refusal if `email` is at one of the party's domains.
    #[must_use]
    pub fn Identity(&self, place: &Place, email: &str) -> Option<Refusal>
    {
        let address = email.trim().to_lowercase();
        let is_party = self.policy.identity_domains.iter().any(|domain| {
            return address.ends_with(&format!("@{}", domain.trim().trim_start_matches('@').to_lowercase()));
        });
        return is_party.then(|| {
            return Refusal {
                rule: IDENTITY_RULE.to_owned(),
                place: place.clone(),
                text: email.to_owned(),
                why: format!("a commit recorded under a {} address reads as work done for {}", self.policy.party, self.policy.party),
            };
        });
    }

    /// The refusals no exception covers. An identity refusal is never covered, and an exception
    /// names one rule and one exact path.
    #[must_use]
    pub fn Unexcepted(&self, refusals: Vec<Refusal>) -> Vec<Refusal>
    {
        return refusals
            .into_iter()
            .filter(|refusal| {
                let Some(path) = refusal.place.Exception_Path() else { return true };
                return !self.policy.exceptions.iter().any(|exception| return exception.rule == refusal.rule && exception.path == path);
            })
            .collect();
    }

    /// Whether a repository is one of the party's own, by any remote URL or its root path.
    #[must_use]
    pub fn Is_Own_Repository(&self, remote_urls: &[String], root: &str) -> bool
    {
        let own = &self.policy.own_repositories;
        let by_remote = remote_urls.iter().any(|url| {
            let url = url.to_lowercase();
            return own.remotes.iter().any(|prefix| return !prefix.is_empty() && url.starts_with(&prefix.to_lowercase()));
        });
        let root = Comparable_Path(root);
        let by_path = own.paths.iter().any(|prefix| return !prefix.is_empty() && root.starts_with(&Comparable_Path(prefix)));
        return by_remote || by_path;
    }
}

fn Comparable_Path(path: &str) -> String
{
    let mut comparable = path.replace('\\', "/").to_lowercase();
    if !comparable.ends_with('/')
    {
        comparable.push('/');
    }
    return comparable;
}
