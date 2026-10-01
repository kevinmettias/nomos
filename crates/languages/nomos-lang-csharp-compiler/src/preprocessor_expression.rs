//! A pre-processing expression -- the condition of an `#if` or `#elif` -- evaluated against a
//! definition set.
//!
//! The grammar is the C# specification's, whole: `true`, `false`, a conditional symbol, `!`,
//! `==`, `!=`, `&&`, `||` and parentheses, binding in that order from tightest, with an optional
//! single-line comment after. Nothing else is a pre-processing expression, so anything else --
//! a number, a delimited comment, an unbalanced parenthesis -- is refused rather than read as
//! something close.

use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token
{
    Symbol(String),
    True,
    False,
    Not,
    And,
    Or,
    Equal,
    NotEqual,
    Open,
    Close,
}

/// What a symbol is known to be at the point an expression is evaluated.
pub(crate) struct Definitions<'a>
{
    /// Every symbol defined here.
    pub(crate) defined: &'a BTreeSet<String>,
    /// Every symbol a `#define` or `#undef` in an unevaluated branch may have changed, whose
    /// value here is therefore not known.
    pub(crate) uncertain: &'a BTreeSet<String>,
}

/// The expression's value, or `None` when `text` is not a pre-processing expression or its value
/// turns on a symbol whose definition is not known.
pub(crate) fn Evaluate(text: &str, definitions: &Definitions<'_>) -> Option<bool>
{
    let tokens = Tokens(Without_Trailing_Comment(text))?;
    let mut parser = Parser { tokens, at: 0, definitions, unknown: false };
    let value = parser.Or()?;
    if parser.at != parser.tokens.len() || parser.unknown
    {
        return None;
    }

    return Some(value);
}

/// `text` up to a `//` comment, which the grammar allows after an expression.
fn Without_Trailing_Comment(text: &str) -> &str
{
    return text.split_once("//").map_or(text, |(expression, _)| return expression);
}

fn Tokens(text: &str) -> Option<Vec<Token>>
{
    let characters: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0usize;
    while let Some(current) = characters.get(at).copied()
    {
        let next = characters.get(at.saturating_add(1)).copied().unwrap_or('\0');
        let (token, width) = match (current, next)
        {
            (character, _) if character.is_whitespace() => (None, 1),
            ('&', '&') => (Some(Token::And), 2),
            ('|', '|') => (Some(Token::Or), 2),
            ('=', '=') => (Some(Token::Equal), 2),
            ('!', '=') => (Some(Token::NotEqual), 2),
            ('!', _) => (Some(Token::Not), 1),
            ('(', _) => (Some(Token::Open), 1),
            (')', _) => (Some(Token::Close), 1),
            (character, _) if character.is_alphabetic() || character == '_' => Word(&characters, at),
            _ => return None,
        };
        tokens.extend(token);
        at = at.saturating_add(width);
    }

    return Some(tokens);
}

/// The identifier or keyword starting at `at`, and how many characters it spans.
fn Word(characters: &[char], at: usize) -> (Option<Token>, usize)
{
    let word: String = characters.iter().skip(at).take_while(|character| return character.is_alphanumeric() || **character == '_').collect();
    let width = word.chars().count();
    let token = match word.as_str()
    {
        "true" => Token::True,
        "false" => Token::False,
        _ => Token::Symbol(word),
    };

    return (Some(token), width);
}

struct Parser<'definitions, 'symbols>
{
    tokens: Vec<Token>,
    at: usize,
    definitions: &'definitions Definitions<'symbols>,
    /// Set when a symbol whose definition is not known was read.
    unknown: bool,
}

impl Parser<'_, '_>
{
    fn Or(&mut self) -> Option<bool>
    {
        let mut value = self.And()?;
        while self.Take(&Token::Or)
        {
            let right = self.And()?;
            value = value || right;
        }
        return Some(value);
    }

    fn And(&mut self) -> Option<bool>
    {
        let mut value = self.Equality()?;
        while self.Take(&Token::And)
        {
            let right = self.Equality()?;
            value = value && right;
        }
        return Some(value);
    }

    fn Equality(&mut self) -> Option<bool>
    {
        let mut value = self.Unary()?;
        loop
        {
            if self.Take(&Token::Equal)
            {
                value = value == self.Unary()?;
            }
            else if self.Take(&Token::NotEqual)
            {
                value = value != self.Unary()?;
            }
            else
            {
                return Some(value);
            }
        }
    }

    fn Unary(&mut self) -> Option<bool>
    {
        if self.Take(&Token::Not)
        {
            return self.Unary().map(|value| return !value);
        }
        return self.Primary();
    }

    fn Primary(&mut self) -> Option<bool>
    {
        let token = self.tokens.get(self.at).cloned()?;
        self.at = self.at.saturating_add(1);
        return match token
        {
            Token::True => Some(true),
            Token::False => Some(false),
            Token::Symbol(name) =>
            {
                self.unknown |= self.definitions.uncertain.contains(&name);
                Some(self.definitions.defined.contains(&name))
            }
            Token::Open =>
            {
                let value = self.Or()?;
                self.Take(&Token::Close).then_some(value)
            }
            _ => None,
        };
    }

    /// Consumes the next token if it is `wanted`.
    fn Take(&mut self, wanted: &Token) -> bool
    {
        if self.tokens.get(self.at) == Some(wanted)
        {
            self.at = self.at.saturating_add(1);
            return true;
        }
        return false;
    }
}

#[cfg(test)]
mod tests;
