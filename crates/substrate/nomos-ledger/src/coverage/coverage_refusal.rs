//! Why an item was refused under a repository's predicate-coverage rules, in one vocabulary
//! both verbs that judge it share.

/// Why an item's predicate does not satisfy what its repository declares about coverage.
///
/// One type carried by two refusals, [`crate::AddRefusal::Coverage`] and
/// [`crate::ClaimRefusal::Coverage`], because `OD-GATE-036` says `work widen` refuses "the same
/// way" `work add` does. Two copies of these arms, one per verb, would be two renderings of one
/// rule, free to drift into saying different things about the same predicate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoverageRefusal
{
    /// The territory reaches a path a rule declares, and the predicate carries neither every
    /// argument that rule requires nor any one argument that satisfies it on its own.
    ///
    /// Names what is missing rather than only that something is, because the remedy is to add
    /// exactly those arguments and an author told only "uncovered" has to go and read the
    /// declaration to find out which.
    Uncovered
    {
        /// The record that decided the rule, as the declaration names it.
        record: String,
        /// The item's own paths that reach a path the rule declares, as authored.
        reaching: Vec<String>,
        /// The required arguments the predicate does not carry, in the order the rule lists
        /// them.
        missing: Vec<String>,
        /// The arguments any one of which would have satisfied the rule alone. Empty when the
        /// rule declares none.
        sufficient: Vec<String>,
    },
    /// The repository has a declaration and it could not be read or parsed.
    ///
    /// Refuses every item rather than none. Which paths an unreadable declaration names is
    /// exactly what nobody knows, and treating it as declaring nothing would let one misspelled
    /// key switch every rule off with nothing said — the declaration's own worst case reading
    /// as a clean pass.
    Unreadable
    {
        /// What went wrong, naming the file.
        cause: String,
    },
}

impl CoverageRefusal
{
    /// A sentence an author can act on, naming the missing arguments and the record, or the
    /// file that could not be read.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Uncovered { record, reaching, missing, sufficient } => format!(
                "{} reaches a path {record} declares, and the predicate does not carry {}. That rule \
                 requires every argument it names{}",
                reaching.join(", "),
                missing.join(", "),
                Alternative(sufficient),
            ),
            Self::Unreadable { cause } => format!(
                "the predicate-coverage declaration could not be read, so no territory can be judged \
                 against it: {cause}"
            ),
        };
    }
}

/// The clause naming what satisfies a rule on its own, or nothing when the rule names none.
fn Alternative(sufficient: &[String]) -> String
{
    if sufficient.is_empty()
    {
        return String::new();
    }

    return format!(", or any one of {} alone", sufficient.join(", "));
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Describe_Should_Name_The_Missing_Arguments_And_The_Record()
    {
        let refusal = CoverageRefusal::Uncovered {
            record: "OD-EXAMPLE-001".to_owned(),
            reaching: vec!["crates/rules/a.rs".to_owned()],
            missing: vec!["nomos-cli".to_owned(), "nomos-integration-tests".to_owned()],
            sufficient: vec!["--workspace".to_owned()],
        };

        let said = refusal.Describe();

        assert!(said.contains("OD-EXAMPLE-001"), "{said}");
        assert!(said.contains("crates/rules/a.rs"), "{said}");
        assert!(said.contains("nomos-cli, nomos-integration-tests"), "{said}");
        assert!(said.contains("--workspace alone"), "{said}");
    }

    #[test]
    fn Test_Describe_Should_Carry_The_Cause_Of_An_Unreadable_Declaration()
    {
        let refusal = CoverageRefusal::Unreadable { cause: "nomos-predicate-coverage.json: expected `,`".to_owned() };

        assert!(refusal.Describe().contains("expected `,`"), "{}", refusal.Describe());
    }

    #[test]
    fn Test_Alternative_Should_Say_Nothing_When_No_Argument_Suffices_Alone()
    {
        assert_eq!(Alternative(&[]), "");
        assert_eq!(Alternative(&["--workspace".to_owned()]), ", or any one of --workspace alone");
    }
}
