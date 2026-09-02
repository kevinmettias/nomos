//! Reading `standards.json`'s `goals`, `max_subsystems_per_goal` and `subsystems` blocks
//! into a goal policy.
//!
//! The three keys are exactly the ones code-standards' own `kernel/config/limits` declares:
//! `goals` and `max_subsystems_per_goal` at the top level, and `subsystems` as an array
//! whose name field is spelled `subsystem`, not `name`. That last spelling is easy to get
//! wrong from the prose and was read off the struct tag rather than guessed.
//!
//! Verified directly against this repository's own `standards.json` before this reader was
//! written: it declares none of the three. That is the opt-out the rule is built around
//! rather than a gap to fill here — a goal cannot be inferred from code, so a provider that
//! invented one for this repository would be deciding something only the repository can
//! decide.

use nomos_cap_goals_policy::{GoalsPolicyPayload, SubsystemDeclaration};
use nomos_platform::{FileSystem, FileSystemError};
use std::path::Path;

const STANDARDS_JSON: &str = "standards.json";

const GOALS_KEY: &str = "goals";
const CEILING_KEY: &str = "max_subsystems_per_goal";
const SUBSYSTEMS_KEY: &str = "subsystems";
const SUBSYSTEM_NAME_KEY: &str = "subsystem";

/// `standards.json` could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalsPolicyError
{
    pub reason: String,
}

impl core::fmt::Display for GoalsPolicyError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// The goal policy `root`'s own `standards.json` declares — declaring nothing when the file
/// is absent or names none of the three keys.
///
/// Declaration order is preserved for both goals and subsystems rather than sorted, and
/// that is the deterministic choice here rather than the sloppy one: code-standards' own
/// audit reports orphaned goals in declared order, so re-ordering them would change the
/// order of a consumer's findings for no reason. A JSON array's order is the file's own.
///
/// # Errors
///
/// [`GoalsPolicyError`] if `standards.json` exists but could not be read for a reason other
/// than absence, is not valid JSON, declares any of the three keys with the wrong JSON type,
/// declares a non-string goal, or declares one subsystem name twice. A subsystem entry with
/// no name is skipped rather than refused, matching code-standards' own `Goal_Subsystems`,
/// which drops it: an unnamed part is one this audit has nothing to say about, not a
/// malformed file.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<GoalsPolicyPayload, GoalsPolicyError>
{
    let path = root.join(STANDARDS_JSON);
    let text = match filesystem.Read_To_String(&path)
    {
        Ok(text) => text,
        Err(FileSystemError::NotFound { .. }) => return Ok(GoalsPolicyPayload::default()),
        Err(error) =>
        {
            return Err(GoalsPolicyError {
                reason: format!("{STANDARDS_JSON} could not be read: {error}"),
            });
        }
    };

    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| GoalsPolicyError {
        reason: format!("{STANDARDS_JSON} is not valid JSON: {error}"),
    })?;

    return Ok(GoalsPolicyPayload {
        goals: Goals_In(&value)?,
        max_subsystems_per_goal: Ceiling_In(&value)?,
        subsystems: Subsystems_In(&value)?,
    });
}

fn Goals_In(value: &serde_json::Value) -> Result<Vec<String>, GoalsPolicyError>
{
    return String_List_At(value, GOALS_KEY, GOALS_KEY);
}

/// The declared ceiling, with anything at or below zero read as unbounded — code-standards'
/// own option text says "zero or below means unbounded", and its audit guards on
/// `maxPerGoal > 0`, so a negative is a way of saying "not this bound" rather than an error.
fn Ceiling_In(value: &serde_json::Value) -> Result<u32, GoalsPolicyError>
{
    let Some(declared) = value.get(CEILING_KEY)
    else
    {
        return Ok(0);
    };

    let Some(ceiling) = declared.as_i64()
    else
    {
        return Err(GoalsPolicyError {
            reason: format!("{STANDARDS_JSON}'s {CEILING_KEY} is not an integer"),
        });
    };

    return Ok(u32::try_from(ceiling).unwrap_or(0));
}

fn Subsystems_In(value: &serde_json::Value) -> Result<Vec<SubsystemDeclaration>, GoalsPolicyError>
{
    let Some(declared) = value.get(SUBSYSTEMS_KEY)
    else
    {
        return Ok(Vec::new());
    };

    let Some(entries) = declared.as_array()
    else
    {
        return Err(GoalsPolicyError {
            reason: format!("{STANDARDS_JSON}'s {SUBSYSTEMS_KEY} is not an array"),
        });
    };

    let mut subsystems: Vec<SubsystemDeclaration> = Vec::new();
    for entry in entries
    {
        let Some(name) = entry.get(SUBSYSTEM_NAME_KEY).and_then(serde_json::Value::as_str)
        else
        {
            continue;
        };
        if name.is_empty()
        {
            continue;
        }

        if subsystems.iter().any(|declared_already| return declared_already.name == name)
        {
            return Err(GoalsPolicyError {
                reason: format!("{STANDARDS_JSON} declares subsystem {name:?} twice"),
            });
        }

        subsystems.push(SubsystemDeclaration {
            name: name.to_owned(),
            goals: String_List_At(entry, GOALS_KEY, &format!("{SUBSYSTEMS_KEY}[{name}].{GOALS_KEY}"))?,
        });
    }

    return Ok(subsystems);
}

/// The list of strings at `key`, or empty when the key is absent. `where_named` is how the
/// location is spelled in a refusal, since the same key appears at the top level and inside
/// each subsystem and a reader of the error needs to know which one it was.
fn String_List_At(value: &serde_json::Value, key: &str, where_named: &str) -> Result<Vec<String>, GoalsPolicyError>
{
    let Some(declared) = value.get(key)
    else
    {
        return Ok(Vec::new());
    };

    let Some(entries) = declared.as_array()
    else
    {
        return Err(GoalsPolicyError {
            reason: format!("{STANDARDS_JSON}'s {where_named} is not an array"),
        });
    };

    let mut listed = Vec::new();
    for entry in entries
    {
        let Some(text) = entry.as_str()
        else
        {
            return Err(GoalsPolicyError {
                reason: format!("{STANDARDS_JSON}'s {where_named} has a non-string entry"),
            });
        };
        listed.push(text.to_owned());
    }

    return Ok(listed);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    #[test]
    fn Test_Discover_Workspace_Should_Read_This_Repository_As_Having_Declared_No_Goals()
    {
        let payload = Discover_Workspace(&Repository_Root(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        assert_eq!(
            payload,
            GoalsPolicyPayload::default(),
            "this workspace has not taken goal traceability on, and the reader must say so rather than invent it"
        );
    }

    #[test]
    fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_Missing_File()
    {
        let payload = Discover_Workspace(&PathBuf::from("this/path/does/not/exist"), &StdFileSystem)
            .expect("a missing standards.json declares nothing rather than failing");

        assert_eq!(payload, GoalsPolicyPayload::default());
    }

    /// A [`FileSystem`] that hands back fixed text instead of reading a real path, the
    /// boundary this crate's own module doc names as the one place a caller substitutes a
    /// real filesystem.
    struct FakeFileSystem
    {
        text: String,
    }

    impl FileSystem for FakeFileSystem
    {
        fn Read_To_String(&self, _path: &Path) -> Result<String, FileSystemError>
        {
            return Ok(self.text.clone());
        }

        fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
        {
            unimplemented!("this reader never writes")
        }

        fn Exists(&self, _path: &Path) -> bool
        {
            true
        }
    }

    fn Declaring(value: &serde_json::Value) -> FakeFileSystem
    {
        return FakeFileSystem { text: value.to_string() };
    }

    #[test]
    fn Test_Discover_Workspace_Should_Read_Goals_A_Ceiling_And_The_Subsystem_Mapping()
    {
        let filesystem = Declaring(&serde_json::json!({
            "goals": ["render", "simulate"],
            "max_subsystems_per_goal": 2,
            "subsystems": [
                { "subsystem": "graphics", "paths": ["src/graphics"], "goals": ["render"] },
                { "subsystem": "utils", "paths": ["src/utils"] }
            ]
        }));

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(payload.goals, vec!["render".to_owned(), "simulate".to_owned()]);
        assert_eq!(payload.max_subsystems_per_goal, 2);
        assert_eq!(payload.subsystems.len(), 2, "{payload:?}");
        let graphics = payload.subsystems.first().expect("asserted len 2 above");
        assert_eq!(graphics.name, "graphics");
        assert_eq!(graphics.goals, vec!["render".to_owned()]);
        let utils = payload.subsystems.get(1).expect("asserted len 2 above");
        assert!(utils.goals.is_empty(), "a part declaring no goals is the purposeless case, kept: {utils:?}");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Read_A_Ceiling_At_Or_Below_Zero_As_Unbounded()
    {
        let filesystem = Declaring(&serde_json::json!({ "max_subsystems_per_goal": -1 }));

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(payload.max_subsystems_per_goal, 0, "zero or below is a way of saying `not this bound`");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Skip_A_Subsystem_With_No_Name()
    {
        let filesystem = Declaring(&serde_json::json!({
            "subsystems": [{ "paths": ["src/orphan"] }, { "subsystem": "storage" }]
        }));

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(payload.subsystems.len(), 1, "an unnamed part is dropped, not refused: {payload:?}");
        assert_eq!(payload.subsystems.first().expect("asserted len 1 above").name, "storage");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Preserve_Declaration_Order()
    {
        let filesystem = Declaring(&serde_json::json!({ "goals": ["zeta", "alpha"] }));

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(payload.goals, vec!["zeta".to_owned(), "alpha".to_owned()], "declared order, not sorted order");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_Invalid_Json()
    {
        let filesystem = FakeFileSystem { text: "not json at all".to_owned() };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

        assert!(error.reason.contains("not valid JSON"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_Array_Goals_Declaration()
    {
        let filesystem = Declaring(&serde_json::json!({ "goals": "render" }));

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("goals is a list");

        assert!(error.reason.contains("goals is not an array"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_String_Goal()
    {
        let filesystem = Declaring(&serde_json::json!({ "goals": [5] }));

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a goal is a name");

        assert!(error.reason.contains("non-string entry"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Name_Which_Goals_List_Was_Malformed()
    {
        let filesystem = Declaring(&serde_json::json!({
            "subsystems": [{ "subsystem": "graphics", "goals": [5] }]
        }));

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a goal is a name here too");

        assert!(
            error.reason.contains("subsystems[graphics].goals"),
            "the same key appears twice, so a refusal must say which: {}",
            error.reason
        );
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_Integer_Ceiling()
    {
        let filesystem = Declaring(&serde_json::json!({ "max_subsystems_per_goal": "many" }));

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a ceiling is a count");

        assert!(error.reason.contains("is not an integer"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_Array_Subsystems_Declaration()
    {
        let filesystem = Declaring(&serde_json::json!({ "subsystems": {} }));

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("subsystems is a list");

        assert!(error.reason.contains("subsystems is not an array"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_One_Subsystem_Declared_Twice()
    {
        let filesystem = Declaring(&serde_json::json!({
            "subsystems": [{ "subsystem": "graphics" }, { "subsystem": "graphics" }]
        }));

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("one part, one declaration");

        assert!(error.reason.contains("twice"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_File_Naming_None_Of_The_Three_Keys()
    {
        let filesystem = Declaring(&serde_json::json!({ "suppression": {} }));

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON with no goal block");

        assert_eq!(payload, GoalsPolicyPayload::default());
    }

    fn Repository_Root() -> PathBuf
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .map(PathBuf::from)
            .expect("this crate sits three levels below the workspace root");
    }
}
