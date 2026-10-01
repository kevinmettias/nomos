//! The policy a user keeps about a party, and how it is read.
//!
//! `OD-POLICY-002` decision 2 is the vocabulary. Every field is read strictly: an unknown field
//! is refused rather than ignored, because a misspelt `"phrases"` that parsed as an empty rule
//! would silently switch that rule off, and a guard that goes quiet is the failure the record
//! exists to prevent.

use serde::Deserialize;

/// Where a phrase must begin and end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary
{
    /// Anywhere, including inside a longer token.
    Anywhere,
    /// Neither side touches a letter or digit.
    Word,
    /// Only the start is bounded, so a prefix catches the names built on it.
    WordStart,
}

/// Whether a phrase's letter case matters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetterCase
{
    /// `Northwind` matches `NORTHWIND` and `northwind`.
    Ignored,
    /// For an acronym that is an ordinary word in lower case.
    Exact,
}

/// What one rule looks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pattern
{
    /// Phrases, each held as its words; separators between words are not significant.
    Phrases
    {
        /// Every phrase, as its words.
        phrases: Vec<Vec<String>>,
        /// Where each phrase must begin and end.
        boundary: Boundary,
        /// Whether letter case matters.
        letter_case: LetterCase,
    },
    /// Ticket keys: `NW` matches `NW-12` and longer, case-sensitive and bounded both sides.
    Tickets
    {
        /// The keys.
        keys: Vec<String>,
    },
}

/// One rule: what it looks for, and why finding it is a refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyRule
{
    /// The rule's id, as a refusal and an exception name it.
    pub id: String,
    /// Why a match is refused, in the policy author's words.
    pub why: String,
    /// What it looks for.
    pub pattern: Pattern,
}

/// A vetted false positive: one rule, one exact repository-relative path (or `(message)`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exception
{
    /// The rule it excepts.
    pub rule: String,
    /// The exact path, or [`MESSAGE_PLACE`].
    pub path: String,
    /// Why it is not a claim.
    pub reason: String,
}

/// The place an exception names to cover a commit message.
pub const MESSAGE_PLACE: &str = "(message)";

/// The party's own repositories, where the guard stands aside.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnRepositories
{
    /// Remote URL prefixes, compared without regard to letter case.
    pub remotes: Vec<String>,
    /// Local path prefixes, compared with `/` for `\` and without regard to letter case.
    pub paths: Vec<String>,
}

/// Everything the user knows about one party and keeps out of every repository.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyPolicy
{
    /// The party's label, used only in refusals.
    pub party: String,
    /// E-mail domains that are the party's identities.
    pub identity_domains: Vec<String>,
    /// What content is refused.
    pub rules: Vec<PartyRule>,
    /// Where the guard stands aside.
    pub own_repositories: OwnRepositories,
    /// Vetted false positives. An exception without a reason is dropped when read.
    pub exceptions: Vec<Exception>,
}

/// Why a policy could not be used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyError
{
    /// The text is not the policy's JSON shape.
    Malformed(String),
    /// It parsed, but says something the guard cannot act on.
    Invalid(String),
}

impl std::fmt::Display for PolicyError
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return match self
        {
            Self::Malformed(detail) => write!(formatter, "the policy is not valid policy JSON: {detail}"),
            Self::Invalid(detail) => write!(formatter, "the policy cannot be acted on: {detail}"),
        };
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyText
{
    party: String,
    #[serde(default)]
    identities: IdentitiesText,
    rules: Vec<RuleText>,
    #[serde(default)]
    own_repositories: OwnRepositoriesText,
    #[serde(default)]
    exceptions: Vec<ExceptionText>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentitiesText
{
    #[serde(default)]
    email_domains: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleText
{
    id: String,
    why: String,
    #[serde(default)]
    phrases: Vec<String>,
    #[serde(default)]
    tickets: Vec<String>,
    #[serde(default)]
    boundary: Option<String>,
    #[serde(default)]
    case: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnRepositoriesText
{
    #[serde(default)]
    remotes: Vec<String>,
    #[serde(default)]
    paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExceptionText
{
    rule: String,
    path: String,
    #[serde(default)]
    reason: String,
}

/// Reads a policy from its JSON text.
///
/// # Errors
///
/// [`PolicyError::Malformed`] for text that is not the policy's shape, including any unknown
/// field; [`PolicyError::Invalid`] for a policy with no party, no rules, a rule with nothing
/// to look for or both kinds at once, a repeated rule id, or an unknown boundary or case.
pub fn Party_Policy_From_Json(text: &str) -> Result<PartyPolicy, PolicyError>
{
    let raw: PolicyText = serde_json::from_str(text).map_err(|error| return PolicyError::Malformed(error.to_string()))?;
    if raw.party.trim().is_empty()
    {
        return Err(PolicyError::Invalid("`party` is empty".to_owned()));
    }
    if raw.rules.is_empty()
    {
        return Err(PolicyError::Invalid("there are no `rules`, so nothing would be judged".to_owned()));
    }
    let mut rules: Vec<PartyRule> = Vec::new();
    for rule in raw.rules
    {
        if rules.iter().any(|seen| return seen.id == rule.id)
        {
            return Err(PolicyError::Invalid(format!("rule `{}` is declared twice", rule.id)));
        }
        rules.push(Rule_From_Text(rule)?);
    }
    let exceptions = raw
        .exceptions
        .into_iter()
        .filter(|exception| return !exception.reason.trim().is_empty())
        .map(|exception| {
            return Exception { rule: exception.rule, path: exception.path, reason: exception.reason };
        })
        .collect();

    return Ok(PartyPolicy {
        party: raw.party,
        identity_domains: raw.identities.email_domains,
        rules,
        own_repositories: OwnRepositories { remotes: raw.own_repositories.remotes, paths: raw.own_repositories.paths },
        exceptions,
    });
}

fn Rule_From_Text(rule: RuleText) -> Result<PartyRule, PolicyError>
{
    let invalid = |detail: &str| return PolicyError::Invalid(format!("rule `{}` {detail}", rule.id));
    if rule.id.trim().is_empty() || rule.why.trim().is_empty()
    {
        return Err(PolicyError::Invalid("every rule needs an `id` and a `why`".to_owned()));
    }
    let pattern = match (rule.phrases.is_empty(), rule.tickets.is_empty())
    {
        (false, true) =>
        {
            let phrases: Vec<Vec<String>> = rule.phrases.iter().map(|phrase| return Phrase_Words(phrase)).collect();
            if phrases.iter().any(Vec::is_empty)
            {
                return Err(invalid("has a phrase with no words in it"));
            }
            Pattern::Phrases {
                phrases,
                boundary: Boundary_From_Text(rule.boundary.as_deref()).ok_or_else(|| return invalid("names an unknown `boundary`"))?,
                letter_case: Letter_Case_From_Text(rule.case.as_deref()).ok_or_else(|| return invalid("names an unknown `case`"))?,
            }
        },
        (true, false) =>
        {
            if rule.tickets.iter().any(|key| return key.is_empty() || !key.chars().all(char::is_alphanumeric))
            {
                return Err(invalid("has a ticket key that is not letters and digits"));
            }
            Pattern::Tickets { keys: rule.tickets.clone() }
        },
        (true, true) => return Err(invalid("has neither `phrases` nor `tickets`, so it would find nothing")),
        (false, false) => return Err(invalid("has both `phrases` and `tickets`; give each its own rule")),
    };

    return Ok(PartyRule { id: rule.id, why: rule.why, pattern });
}

/// A phrase's words: split at whitespace and at the separators a phrase ignores between words.
pub(crate) fn Phrase_Words(phrase: &str) -> Vec<String>
{
    return phrase
        .split(|character: char| return character.is_whitespace() || Is_Separator(character))
        .filter(|word| return !word.is_empty())
        .map(str::to_owned)
        .collect();
}

/// The characters a phrase does not care about between its words.
pub(crate) const fn Is_Separator(character: char) -> bool
{
    return matches!(character, '.' | '_' | '-') || character == ' ';
}

fn Boundary_From_Text(text: Option<&str>) -> Option<Boundary>
{
    return match text.unwrap_or("word")
    {
        "anywhere" => Some(Boundary::Anywhere),
        "word" => Some(Boundary::Word),
        "word-start" => Some(Boundary::WordStart),
        _ => None,
    };
}

fn Letter_Case_From_Text(text: Option<&str>) -> Option<LetterCase>
{
    return match text.unwrap_or("ignored")
    {
        "ignored" => Some(LetterCase::Ignored),
        "exact" => Some(LetterCase::Exact),
        _ => None,
    };
}
