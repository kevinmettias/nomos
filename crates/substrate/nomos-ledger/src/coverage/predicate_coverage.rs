//! What a repository declares an item's predicate must carry, and the one comparison that judges
//! a predicate against it.

use std::path::Path;

use nomos_model::Intersection;
use nomos_platform::{FileSystem, FileSystemError};
use serde::Deserialize;

use crate::CoverageRefusal;
use crate::Territory;
use crate::VerificationPredicate;

/// The file, at a repository's root, that declares its predicate-coverage rules.
///
/// A repository without one declares no rule, and every item satisfies it. That is what keeps
/// a ledger serving some other repository untouched by a rule this one decided: the rule is the
/// repository's to write, never the ledger's to assume.
pub const PREDICATE_COVERAGE: &str = "nomos-predicate-coverage.json";

/// What a repository declares an item's predicate must carry when the item's territory reaches
/// certain paths. `OD-GATE-036`.
///
/// The ledger compares argument tokens and nothing else. It does not know that `-p` names a
/// cargo package or that `--workspace` means every member; a rule says which tokens it wants,
/// and a predicate carries a token when one of its arguments is exactly that string. So the
/// same comparison serves a repository that verifies with some other tool, and a rule that
/// wanted interpretation would have to be written as the tokens that interpretation produces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PredicateCoverage
{
    /// The rules the repository declares. Empty for a repository with no declaration.
    Declared(Vec<CoverageRule>),
    /// A declaration exists and could not be read or parsed.
    ///
    /// Kept as its own state rather than collapsed into "declares nothing", because that
    /// collapse is how one misspelled key would switch every rule off with nothing said. Judged,
    /// it refuses whatever it is asked about: see [`CoverageRefusal::Unreadable`].
    Unreadable
    {
        /// What went wrong, naming the file.
        cause: String,
    },
}

/// One rule: the paths that trigger it, the arguments a predicate must then carry, and any
/// argument that satisfies it on its own.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRule
{
    /// The record that decided the rule, which a refusal names so an author can read why.
    pub record: String,
    /// Territory paths that trigger the rule. An item reaches one when a path it reserves is
    /// that path, lies beneath it, or contains it — the containment [`Territory::Intersect`]
    /// already decides, and not a second one.
    pub paths: Vec<String>,
    /// Arguments a predicate must carry, every one of them, each compared as one whole token.
    pub requires: Vec<String>,
    /// Arguments any one of which satisfies the rule on its own.
    #[serde(default)]
    pub satisfied_by: Vec<String>,
}

/// The declaration file's shape: the rules under one key, refusing any key nothing reads.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration
{
    rules: Vec<CoverageRule>,
}

impl PredicateCoverage
{
    /// A repository that declares no rule.
    ///
    /// What a caller with nothing to declare passes. It is honest in the way an empty published
    /// territory is: it says no rule is known, and every item satisfies no rule.
    #[must_use]
    pub const fn Undeclared() -> Self
    {
        return Self::Declared(Vec::new());
    }

    /// The rules `text` declares.
    ///
    /// # Errors
    ///
    /// Text that is not the declaration's shape, including a key nothing reads — refused rather
    /// than ignored, because a misspelled `requires` would otherwise parse as a rule requiring
    /// nothing — and a rule that names no record, no path or no required argument, or names an
    /// empty one. Such a rule would refuse nothing while reading as a rule.
    pub fn From_Json(text: &str) -> Result<Self, String>
    {
        let declaration: Declaration = serde_json::from_str(text).map_err(|error| return error.to_string())?;

        if let Some(problem) = declaration.rules.iter().find_map(Problem_With)
        {
            return Err(problem);
        }

        return Ok(Self::Declared(declaration.rules));
    }

    /// What the declaration at `root` says, read through `filesystem`.
    ///
    /// Absent is [`Self::Undeclared`]. Present and unreadable, or present and malformed, is
    /// [`Self::Unreadable`], never undeclared — the one state a missing file and a broken one
    /// must not share.
    #[must_use]
    pub fn In_Repository(root: &Path, filesystem: &impl FileSystem) -> Self
    {
        let text = match filesystem.Read_To_String(&root.join(PREDICATE_COVERAGE))
        {
            Ok(text) => text,
            Err(FileSystemError::NotFound { .. }) => return Self::Undeclared(),
            Err(error) => return Self::Unreadable { cause: format!("{PREDICATE_COVERAGE}: {error}") },
        };

        return Self::From_Json(&text)
            .unwrap_or_else(|cause| return Self::Unreadable { cause: format!("{PREDICATE_COVERAGE}: {cause}") });
    }

    /// The refusal `paths` earn under this declaration with `predicate` as their item's
    /// predicate, or `None` when every rule they reach is satisfied.
    ///
    /// `paths` are what is being judged: an item's whole territory when it is added, and only
    /// what a widening adds when it is widened, since the rest was judged when it was reserved.
    /// No paths reach nothing, so they are refused by nothing, an unreadable declaration
    /// included. The first unsatisfied rule is the one reported, in the order the declaration
    /// lists them.
    ///
    /// The program itself is not one of the predicate's arguments; only what follows it is
    /// compared. An item with no predicate carries no argument.
    #[must_use]
    pub fn Shortfall(&self, paths: &[String], predicate: Option<&VerificationPredicate>) -> Option<CoverageRefusal>
    {
        if paths.is_empty()
        {
            return None;
        }

        let rules = match self
        {
            Self::Declared(rules) => rules,
            Self::Unreadable { cause } => return Some(CoverageRefusal::Unreadable { cause: cause.clone() }),
        };
        let carried = Carried_Arguments(predicate);

        return rules.iter().find_map(|rule| return rule.Shortfall(paths, &carried));
    }
}

impl CoverageRule
{
    /// The refusal `paths` earn under this one rule, given the arguments their predicate
    /// carries.
    ///
    /// A satisfying argument is looked for before the required ones, because one is enough
    /// and reporting the required ones as missing beside it would send an author to add
    /// arguments the predicate does not need.
    fn Shortfall(&self, paths: &[String], carried: &[&str]) -> Option<CoverageRefusal>
    {
        let reaching = self.Reaching(paths);
        let satisfied_alone = self.satisfied_by.iter().any(|argument| return carried.contains(&argument.as_str()));

        if reaching.is_empty() || satisfied_alone
        {
            return None;
        }

        let missing: Vec<String> = self
            .requires
            .iter()
            .filter(|argument| return !carried.contains(&argument.as_str()))
            .cloned()
            .collect();

        if missing.is_empty()
        {
            return None;
        }

        return Some(CoverageRefusal::Uncovered {
            record: self.record.clone(),
            reaching,
            missing,
            sufficient: self.satisfied_by.clone(),
        });
    }

    /// Those of `paths` that reach a path this rule declares, in the order given.
    ///
    /// One path at a time, so the refusal can name which. [`Territory::Intersect`] decides
    /// each, which is what makes a directory reach the files beneath it in both directions.
    fn Reaching(&self, paths: &[String]) -> Vec<String>
    {
        let declared = Territory::Of_Files(self.paths.iter().cloned());

        return paths
            .iter()
            .filter(|path| {
                let mine = Territory::Of_Files([(*path).clone()]);
                return matches!(mine.Intersect(&declared), Intersection::Overlaps(_));
            })
            .cloned()
            .collect();
    }
}

/// The arguments `predicate` carries, without its program, as tokens to compare.
fn Carried_Arguments(predicate: Option<&VerificationPredicate>) -> Vec<&str>
{
    return predicate.map_or_else(Vec::new, |predicate| {
        return predicate.argv.iter().skip(1).map(String::as_str).collect();
    });
}

/// What is wrong with `rule` as a declaration, or `None` when nothing is.
fn Problem_With(rule: &CoverageRule) -> Option<String>
{
    if rule.record.trim().is_empty()
    {
        return Some("a rule names no record, so a refusal under it could not say why".to_owned());
    }

    let lists = [("paths", &rule.paths), ("requires", &rule.requires), ("satisfied_by", &rule.satisfied_by)];
    for (key, entries) in lists
    {
        if entries.iter().any(|entry| return entry.trim().is_empty())
        {
            return Some(format!("the rule {} names an empty entry in `{key}`", rule.record));
        }
    }

    if rule.paths.is_empty() || rule.requires.is_empty()
    {
        return Some(format!(
            "the rule {} names no path or no required argument, so it would refuse nothing",
            rule.record
        ));
    }

    return None;
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A declaration of one rule, shaped like the one this repository commits.
    const ONE_RULE: &str = r#"{
        "rules": [
            {
                "record": "OD-EXAMPLE-001",
                "paths": ["crates/rules", "crates/orchestration/nomos-check-orchestration"],
                "requires": ["nomos-cli", "nomos-integration-tests"],
                "satisfied_by": ["--workspace"]
            }
        ]
    }"#;

    fn Declared() -> PredicateCoverage
    {
        return PredicateCoverage::From_Json(ONE_RULE).expect("the fixture is the declaration's own shape");
    }

    fn Predicate(arguments: &str) -> VerificationPredicate
    {
        return VerificationPredicate::From_String_Arguments(arguments.split_whitespace().map(str::to_owned).collect());
    }

    fn Paths(paths: &[&str]) -> Vec<String>
    {
        return paths.iter().map(|path| return (*path).to_owned()).collect();
    }

    #[test]
    fn Test_Undeclared_Should_Refuse_Nothing()
    {
        let refusal = PredicateCoverage::Undeclared().Shortfall(&Paths(&["crates/rules/a.rs"]), None);

        assert_eq!(refusal, None);
    }

    #[test]
    fn Test_From_Json_Should_Read_Every_Field_Of_A_Rule()
    {
        let PredicateCoverage::Declared(rules) = Declared()
        else
        {
            panic!("a well-formed declaration reads as declared rules");
        };

        assert_eq!(
            rules,
            vec![CoverageRule {
                record: "OD-EXAMPLE-001".to_owned(),
                paths: Paths(&["crates/rules", "crates/orchestration/nomos-check-orchestration"]),
                requires: Paths(&["nomos-cli", "nomos-integration-tests"]),
                satisfied_by: Paths(&["--workspace"]),
            }]
        );
    }

    #[test]
    fn Test_From_Json_Should_Refuse_A_Key_Nothing_Reads()
    {
        let misspelled = ONE_RULE.replace("\"requires\"", "\"required\"");

        let error = PredicateCoverage::From_Json(&misspelled).expect_err("a misspelled key is refused, not ignored");

        assert!(error.contains("required"), "{error}");
    }

    #[test]
    fn Test_Problem_With_Should_Refuse_A_Rule_That_Would_Refuse_Nothing()
    {
        let requiring_nothing = ONE_RULE.replace(r#"["nomos-cli", "nomos-integration-tests"]"#, "[]");
        let blank_path = ONE_RULE.replace(r#""crates/rules""#, r#""  ""#);

        let nothing = PredicateCoverage::From_Json(&requiring_nothing).expect_err("a rule requiring nothing is refused");
        let blank = PredicateCoverage::From_Json(&blank_path).expect_err("an empty path is refused");

        assert!(nothing.contains("would refuse nothing"), "{nothing}");
        assert!(blank.contains("`paths`"), "{blank}");
    }

    #[test]
    fn Test_Shortfall_Should_Name_Every_Required_Argument_A_Reaching_Predicate_Lacks()
    {
        let predicate = Predicate("cargo test -p nomos-rules -p nomos-cli");

        let refusal = Declared().Shortfall(&Paths(&["crates/rules/nomos-rules/src/a.rs", "README.md"]), Some(&predicate));

        assert_eq!(
            refusal,
            Some(CoverageRefusal::Uncovered {
                record: "OD-EXAMPLE-001".to_owned(),
                reaching: Paths(&["crates/rules/nomos-rules/src/a.rs"]),
                missing: Paths(&["nomos-integration-tests"]),
                sufficient: Paths(&["--workspace"]),
            })
        );
    }

    #[test]
    fn Test_Shortfall_Should_Pass_Every_Required_Argument_Or_One_That_Suffices_Alone()
    {
        let both = Predicate("cargo test -p nomos-cli -p nomos-integration-tests");
        let workspace = Predicate("cargo test --workspace");
        let reaching = Paths(&["crates/rules/nomos-rules"]);

        assert_eq!(Declared().Shortfall(&reaching, Some(&both)), None);
        assert_eq!(Declared().Shortfall(&reaching, Some(&workspace)), None);
    }

    #[test]
    fn Test_Shortfall_Should_Pass_A_Territory_That_Reaches_No_Declared_Path()
    {
        let predicate = Predicate("cargo test -p nomos-ledger");

        let refusal = Declared().Shortfall(&Paths(&["crates/substrate/nomos-ledger", "crates/rulesets"]), Some(&predicate));

        assert_eq!(refusal, None, "a sibling whose name begins the same is not beneath the path");
    }

    #[test]
    fn Test_Shortfall_Should_Count_A_Directory_Containing_A_Declared_Path_As_Reaching_It()
    {
        let refusal = Declared().Shortfall(&Paths(&["crates"]), Some(&Predicate("cargo test -p nomos-ledger")));

        assert!(matches!(refusal, Some(CoverageRefusal::Uncovered { .. })), "{refusal:?}");
    }

    #[test]
    fn Test_Shortfall_Should_Refuse_An_Item_With_No_Predicate_And_Never_Compare_The_Program()
    {
        let program_only = Predicate("nomos-cli");

        let without = Declared().Shortfall(&Paths(&["crates/rules"]), None);
        let named_as_program = Declared().Shortfall(&Paths(&["crates/rules"]), Some(&program_only));

        assert!(matches!(&without, Some(CoverageRefusal::Uncovered { missing, .. }) if missing.len() == 2), "{without:?}");
        assert!(matches!(&named_as_program, Some(CoverageRefusal::Uncovered { missing, .. }) if missing.len() == 2), "{named_as_program:?}");
    }

    #[test]
    fn Test_Shortfall_Should_Refuse_Anything_Under_An_Unreadable_Declaration_And_Nothing_When_No_Path_Is_Judged()
    {
        let unreadable = PredicateCoverage::Unreadable { cause: "expected `,`".to_owned() };

        let judged = unreadable.Shortfall(&Paths(&["docs/a.md"]), None);
        let nothing_judged = unreadable.Shortfall(&[], None);

        assert_eq!(judged, Some(CoverageRefusal::Unreadable { cause: "expected `,`".to_owned() }));
        assert_eq!(nothing_judged, None);
    }

    #[test]
    fn Test_Carried_Arguments_Should_Leave_Out_The_Program()
    {
        let predicate = Predicate("cargo test --workspace");

        assert_eq!(Carried_Arguments(Some(&predicate)), vec!["test", "--workspace"]);
        assert!(Carried_Arguments(None).is_empty());
    }

    #[test]
    fn Test_Reaching_Should_Keep_The_Order_The_Paths_Were_Given()
    {
        let PredicateCoverage::Declared(rules) = Declared()
        else
        {
            panic!("a well-formed declaration reads as declared rules");
        };
        let rule = rules.first().expect("the fixture declares one rule");

        let reaching = rule.Reaching(&Paths(&["crates/rules/b", "docs", "crates/rules/a"]));

        assert_eq!(reaching, Paths(&["crates/rules/b", "crates/rules/a"]));
    }

    #[test]
    fn Test_In_Repository_Should_Tell_An_Absent_Declaration_From_A_Broken_One()
    {
        use nomos_platform_std::StdFileSystem;

        let root = std::env::temp_dir().join(format!("nomos-ledger-predicate-coverage-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory under the temporary directory is creatable");

        let absent = PredicateCoverage::In_Repository(&root, &StdFileSystem);
        std::fs::write(root.join(PREDICATE_COVERAGE), "{ \"rules\": [").expect("the scratch directory is writable");
        let broken = PredicateCoverage::In_Repository(&root, &StdFileSystem);
        std::fs::write(root.join(PREDICATE_COVERAGE), ONE_RULE).expect("the scratch directory is writable");
        let declared = PredicateCoverage::In_Repository(&root, &StdFileSystem);
        let _ignored = std::fs::remove_dir_all(&root);

        assert_eq!(absent, PredicateCoverage::Undeclared());
        assert!(matches!(&broken, PredicateCoverage::Unreadable { cause } if cause.starts_with(PREDICATE_COVERAGE)), "{broken:?}");
        assert_eq!(declared, Declared());
    }
}
