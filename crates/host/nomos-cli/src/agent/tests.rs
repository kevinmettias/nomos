//! What `nomos agent` promises, exercised.
//!
//! Split by the group each test drives -- `execute`'s flags, `judge-role`'s, and the two
//! vocabularies the printed usage text has to agree with -- because what a reader comes here
//! to find is the group rather than the assertion. The one translation every module shares,
//! a command line into arguments, lives here beside the `mod` list.

mod effort_usage;
mod execute_parsing;
mod exit_codes;
mod judge_role;

/// Splits a command line into the arguments a caller would hand the parser -- the one
/// translation every test below needs, so each case states its own command line as text.
fn Arguments(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}
