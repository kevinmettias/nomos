//! One pass over a file's lines, judging every conditional branch as the compiler does.

use crate::lexer::LineLexer;
use crate::preprocessor_expression::{Definitions, Evaluate};
use crate::ParseFailure;
use nomos_cap_csharp_semantics::{Branch, BranchState, ConditionalRegion, DefinitionEffect, FileDefinition};
use std::collections::BTreeSet;

/// A chain opened by an `#if` whose `#endif` has not been read yet.
struct OpenChain
{
    /// The state of the branch the chain sits in, which bounds every branch of the chain.
    enclosing: BranchState,
    /// Whether a branch of this chain has been compiled, so every later one is skipped.
    taken: bool,
    /// Whether a branch of this chain could not be evaluated, so no later one can be known.
    uncertain: bool,
    /// Whether `#else` has been read, after which only `#endif` may follow.
    seen_else: bool,
    /// Where in the walk's regions the chain's current branch is.
    region: usize,
}

/// The walk's state between lines.
pub(crate) struct Walk
{
    lexer: LineLexer,
    defined: BTreeSet<String>,
    uncertain: BTreeSet<String>,
    chains: Vec<OpenChain>,
    pub(crate) regions: Vec<ConditionalRegion>,
    pub(crate) definitions: Vec<FileDefinition>,
}

impl Walk
{
    pub(crate) fn New(symbols: &BTreeSet<String>) -> Self
    {
        return Walk {
            lexer: LineLexer::default(),
            defined: symbols.clone(),
            uncertain: BTreeSet::new(),
            chains: Vec::new(),
            regions: Vec::new(),
            definitions: Vec::new(),
        };
    }

    /// Reads one line, `number` being one-based.
    pub(crate) fn Step(&mut self, number: usize, line: &str) -> Result<(), ParseFailure>
    {
        let context = self.Context();
        let at_directive_position = context != BranchState::Compiled || self.lexer.Is_At_Code();
        if at_directive_position
        {
            if let Some((name, argument)) = Directive_Of(line)
            {
                return self.Directive(number, &name, argument);
            }
        }
        if context == BranchState::Compiled
        {
            self.lexer.Read_Line(line);
        }

        return Ok(());
    }

    /// Refuses a file that ends with a chain still open.
    pub(crate) fn Finish(&self) -> Result<(), ParseFailure>
    {
        let Some(open) = self.chains.first()
        else
        {
            return Ok(());
        };
        let line = self.regions.get(open.region).map_or(0, |region| return region.line);

        return Err(Failure(line, "an #if with no #endif before the end of the file"));
    }

    /// The state of the branch the next line sits in: compiled at the top of the file.
    fn Context(&self) -> BranchState
    {
        return self.chains.last().and_then(|chain| return self.regions.get(chain.region)).map_or(BranchState::Compiled, |region| return region.state);
    }

    fn Directive(&mut self, number: usize, name: &str, argument: &str) -> Result<(), ParseFailure>
    {
        return match name
        {
            "if" => {
                self.Open_Chain(number, argument);
                Ok(())
            }
            "elif" => self.Next_Branch(number, Branch::Elif, argument),
            "else" => self.Next_Branch(number, Branch::Else, ""),
            "endif" => self.Close_Chain(number),
            "define" => self.Definition(number, DefinitionEffect::Define, argument),
            "undef" => self.Definition(number, DefinitionEffect::Undefine, argument),
            // `#region`, `#pragma`, `#line`, `#nullable`, `#error`, `#warning` and the rest
            // neither open a branch nor change a symbol.
            _ => Ok(()),
        };
    }

    fn Open_Chain(&mut self, number: usize, argument: &str)
    {
        let enclosing = self.Context();
        let state = match enclosing
        {
            BranchState::Compiled => self.Evaluated(argument),
            other => other,
        };
        let region = self.Push_Region(Branch::If, number, argument, state);

        self.chains.push(OpenChain {
            enclosing,
            taken: state == BranchState::Compiled,
            uncertain: state == BranchState::Unevaluated,
            seen_else: false,
            region,
        });
    }

    fn Next_Branch(&mut self, number: usize, branch: Branch, argument: &str) -> Result<(), ParseFailure>
    {
        let label = branch.Label();
        let Some(chain) = self.chains.last()
        else
        {
            return Err(Failure(number, &format!("an #{label} with no #if open")));
        };
        if chain.seen_else
        {
            return Err(Failure(number, &format!("an #{label} after its chain's #else")));
        }

        let state = match (chain.enclosing, chain.uncertain, chain.taken)
        {
            (BranchState::Compiled, true, _) => BranchState::Unevaluated,
            (BranchState::Compiled, false, true) => BranchState::Skipped,
            (BranchState::Compiled, false, false) if branch == Branch::Else => BranchState::Compiled,
            (BranchState::Compiled, false, false) => self.Evaluated(argument),
            (other, _, _) => other,
        };
        let previous = chain.region;
        self.Close_Region(previous, number);
        let region = self.Push_Region(branch, number, argument, state);

        if let Some(chain) = self.chains.last_mut()
        {
            chain.region = region;
            chain.taken |= state == BranchState::Compiled;
            chain.uncertain |= state == BranchState::Unevaluated && chain.enclosing == BranchState::Compiled;
            chain.seen_else |= branch == Branch::Else;
        }
        return Ok(());
    }

    fn Close_Chain(&mut self, number: usize) -> Result<(), ParseFailure>
    {
        let Some(chain) = self.chains.pop()
        else
        {
            return Err(Failure(number, "an #endif with no #if open"));
        };

        self.Close_Region(chain.region, number);
        return Ok(());
    }

    /// A `#define` or `#undef`: applied where compiled, left uncertain where its branch is
    /// unevaluated, and ignored where skipped -- the compiler never reads it there.
    fn Definition(&mut self, number: usize, effect: DefinitionEffect, argument: &str) -> Result<(), ParseFailure>
    {
        let symbol = Symbol_Of(argument).ok_or_else(|| return Failure(number, &format!("#{} names no symbol", effect.Label())))?;
        match self.Context()
        {
            BranchState::Skipped => {}
            BranchState::Unevaluated =>
            {
                self.uncertain.insert(symbol);
            }
            BranchState::Compiled =>
            {
                if self.lexer.seen_token
                {
                    return Err(Failure(number, &format!("#{} after the first token in the file", effect.Label())));
                }
                self.Apply(number, effect, symbol);
            }
        }

        return Ok(());
    }

    fn Apply(&mut self, number: usize, effect: DefinitionEffect, symbol: String)
    {
        match effect
        {
            DefinitionEffect::Define => self.defined.insert(symbol.clone()),
            DefinitionEffect::Undefine => self.defined.remove(&symbol),
        };
        self.definitions.push(FileDefinition { line: number, symbol, effect });
    }

    fn Evaluated(&self, argument: &str) -> BranchState
    {
        let definitions = Definitions { defined: &self.defined, uncertain: &self.uncertain };
        return match Evaluate(argument, &definitions)
        {
            Some(true) => BranchState::Compiled,
            Some(false) => BranchState::Skipped,
            None => BranchState::Unevaluated,
        };
    }

    fn Push_Region(&mut self, branch: Branch, number: usize, argument: &str, state: BranchState) -> usize
    {
        self.regions.push(ConditionalRegion { branch, line: number, end_line: number, condition: Condition_Text(argument), state });
        return self.regions.len().saturating_sub(1);
    }

    fn Close_Region(&mut self, region: usize, number: usize)
    {
        if let Some(region) = self.regions.get_mut(region)
        {
            region.end_line = number;
        }
    }
}

/// The directive name and its argument, when `line` is a directive: `#` after optional blanks,
/// then optional blanks, then the name.
fn Directive_Of(line: &str) -> Option<(String, &str)>
{
    let after_hash = line.trim_start().strip_prefix('#')?.trim_start();
    let name: String = after_hash.chars().take_while(|character| return character.is_alphabetic()).collect();
    let argument = after_hash.get(name.len()..).unwrap_or("").trim();

    return Some((name, argument));
}

/// The condition as the payload carries it: without a trailing comment, whitespace collapsed.
fn Condition_Text(argument: &str) -> String
{
    let expression = argument.split_once("//").map_or(argument, |(expression, _)| return expression);
    return expression.split_whitespace().collect::<Vec<&str>>().join(" ");
}

/// The symbol a `#define` or `#undef` names: its one identifier, before any comment.
fn Symbol_Of(argument: &str) -> Option<String>
{
    let expression = argument.split_once("//").map_or(argument, |(expression, _)| return expression);
    let words: Vec<&str> = expression.split_whitespace().collect();
    let [symbol] = words.as_slice()
    else
    {
        return None;
    };
    let is_identifier = symbol.chars().next().is_some_and(|first| return first.is_alphabetic() || first == '_')
        && symbol.chars().all(|character| return character.is_alphanumeric() || character == '_')
        && *symbol != "true"
        && *symbol != "false";

    return is_identifier.then(|| return (*symbol).to_owned());
}

fn Failure(line: usize, message: &str) -> ParseFailure
{
    return ParseFailure { line, message: message.to_owned() };
}

#[cfg(test)]
mod tests;
