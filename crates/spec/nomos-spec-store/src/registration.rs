//! Which records govern this build, read from one file per record.
//!
//! A record becomes governing by having a registration under
//! `crates/spec/nomos-spec-store/records/`, named for its identifier. Writing the record
//! file does not make it governing; the registration does. That is two independent acts by
//! a human, which is exactly what
//! `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing` compares — and it is why the
//! input to this reader is the registration directory and never `docs/records`. Deriving
//! the governing list from `docs/records` would make that guard compare a directory against
//! itself and pass having checked nothing, which is the remedy `OD-LEDGER-007` refused and
//! `OD-SPEC-007` answers.
//!
//! # Why this is a module and not twenty lines inside `build.rs`
//!
//! `build.rs` reaches it with `#[path = "src/registration.rs"] mod registration;`, because
//! a build script cannot depend on its own crate. Copying the parse into `build.rs` is how
//! two readers of one format come to disagree, which this workspace has already written
//! down twice — `tests/contract/src/universes.rs` and `OD-COMPLETENESS-001`.
//!
//! `lib.rs` declares it `#[cfg(test)] mod registration;`. The library itself never calls
//! this reader — it consumes the tables `build.rs` generated — so an unconditional `mod`
//! would make every item here dead code under `-D warnings`. Under `cfg(test)` the module
//! compiles into the lib test binary and its refusals run. A parser whose refusals nobody
//! has ever executed is a parser whose refusals are a claim, and the refusals are the whole
//! of the value here: a malformed registration that is *skipped* rather than *refused* is a
//! governing record that quietly stops governing, which is `OD-SPEC-005`'s defect wearing a
//! build script's clothes.

use std::path::Path;

// Explicit `#[path]` because this file is itself loaded two ways — `lib.rs` says
// `#[cfg(test)] mod registration;` (ordinary resolution: children under `registration/`) but
// `build.rs` says `#[path = "src/registration.rs"] mod registration;`, and once a module is
// reached through an explicit `#[path]`, rustc stops inferring a same-named subdirectory for
// ITS children and looks beside the path's own directory instead — an implicit `mod error;`
// here would resolve to `src/error.rs` under the build-script compilation and fail to find
// it. Spelling the path keeps both compilations pointed at the same file.
#[path = "registration/error.rs"]
mod error;

pub(crate) use error::RegistrationError;

/// The only directory, relative to the repository root, a registration may name.
const RECORD_DIRECTORY: &str = "docs/records";

/// The extension that makes a file in the registration directory a registration.
const REGISTRATION_EXTENSION: &str = "record";

/// The one key a registration body may carry.
const PATH_KEY: &str = "path";

/// One record's registration.
///
/// Two fields and no more. The identifier is the file stem and appears nowhere inside the
/// file, because an `id:` key would be a second place for the identity to be wrong. The
/// body names the path, because a slug is not derivable from an identifier and renaming a
/// record's slug should touch that record's own registration and nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Registration
{
    /// The record identifier, which is the file stem.
    pub(crate) id: String,
    /// The record file this registration names: repository-relative, forward slashes.
    pub(crate) path: String,
}

/// Every registration under `directory`, sorted by identifier.
///
/// `directory` is a parameter and not a constant on purpose, and it is the structural half
/// of the guard against this reader ever being pointed at `docs/records`: a unit test hands
/// it a synthetic directory holding one registration while the repository holds dozens, and
/// a reader that consulted anything but its argument fails it.
///
/// `root` is the repository root, and is used only to check that a named record is really
/// there. Nothing under it is enumerated.
///
/// # Errors
///
/// Returns [`RegistrationError`] on every malformed input. Nothing is skipped; see the
/// enum's own documentation for why absence must not become success.
pub(crate) fn Registrations_In(
    directory: &Path,
    root: &Path,
) -> Result<Vec<Registration>, RegistrationError>
{
    let entries = std::fs::read_dir(directory).map_err(|error| {
        return RegistrationError::Unreadable {
            at: directory.display().to_string(),
            cause: error.to_string(),
        };
    })?;
    let mut found: Vec<Registration> = Vec::new();
    for entry in entries
    {
        let entry = entry.map_err(|error| {
            return RegistrationError::Unreadable {
                at: directory.display().to_string(),
                cause: error.to_string(),
            };
        })?;
        let registration = Read_Registration(&entry.path(), root)?;

        found.push(registration);
    }
    if found.is_empty()
    {
        return Err(RegistrationError::Empty {
            at: directory.display().to_string(),
        });
    }
    Assert_One_Identity_Per_Record(&found)?;
    // Byte order on the identifier. `read_dir` order is not deterministic across platforms
    // and must not reach a generated table, or two machines build two different binaries
    // from one commit.
    found.sort_by(|left, right| return left.id.cmp(&right.id));

    return Ok(found);
}

/// One registration file: its identity from the stem, and the record it names.
fn Read_Registration(file: &Path, root: &Path) -> Result<Registration, RegistrationError>
{
    let at = file.display().to_string();
    let is_registration = file
        .extension()
        .is_some_and(|extension| return extension == REGISTRATION_EXTENSION);
    if !is_registration
    {
        return Err(RegistrationError::NotARegistration { at });
    }
    let Some(stem) = file.file_stem().and_then(|stem| return stem.to_str())
    else
    {
        return Err(RegistrationError::NotARegistration { at });
    };
    if !Is_Identifier(stem)
    {
        return Err(RegistrationError::NotAnIdentifier {
            at,
            stem: stem.to_owned(),
        });
    }
    let text = std::fs::read_to_string(file).map_err(|error| {
        return RegistrationError::Unreadable {
            at: at.clone(),
            cause: error.to_string(),
        };
    })?;
    let named = Named_Record(&text, At(&at))?;
    Check_Is_A_Record(&named, At(&at), root)?;

    return Ok(Registration {
        id: stem.to_owned(),
        path: named,
    });
}

/// Whether a stem is a record identifier: `ARC-SPECDB-001`, `D-129`, `OD-LEDGER-007`.
///
/// Hyphen-separated segments of upper-case ASCII letters and digits, the first of which
/// begins with a letter. Written out rather than matched against a pattern because this
/// reader carries no dependency, and a build script that pulled in a regular expression
/// engine would move the crate's band and the contracts allowlist with it.
fn Is_Identifier(stem: &str) -> bool
{
    if !stem.starts_with(|character: char| return character.is_ascii_uppercase())
    {
        return false;
    }

    return stem.split('-').all(|segment| {
        return !segment.is_empty()
            && segment.chars().all(|character| {
                return character.is_ascii_uppercase() || character.is_ascii_digit();
            });
    });
}

/// The record a registration body names.
///
/// Blank lines and `#` comments are ignored, and that is the only tolerance. Anything else
/// is refused rather than passed over.
fn Named_Record(text: &str, at: At<'_>) -> Result<String, RegistrationError>
{
    let mut named: Option<String> = None;

    for line in text.lines()
    {
        // Trimming also removes a carriage return a CRLF checkout would leave behind, so the
        // path a registration names does not depend on how the tree was cloned.
        let found = Path_Line(line.trim(), at, named.as_deref())?;

        named = found.or(named);
    }

    return named.ok_or_else(|| {
        return RegistrationError::NoPath { at: at.0.to_owned() };
    });
}

/// Where a registration lives, for a refusal to name.
///
/// Distinct from the plain `&str` positions it sits beside — `text`, `line`, `named` — so a
/// caller cannot pass the registration's own location where its content was meant, or the
/// reverse.
#[derive(Clone, Copy)]
struct At<'a>(&'a str);

/// The path one line names, or `None` for a blank line or a comment.
///
/// `held` is what an earlier line already named, because a second `path:` is refused rather
/// than resolved by taking one of them.
fn Path_Line(
    line: &str,
    at: At<'_>,
    held: Option<&str>,
) -> Result<Option<String>, RegistrationError>
{
    if Is_Blank_Or_Comment(line)
    {
        return Ok(None);
    }

    let value = Path_Key_Value(line, at)?;

    Check_Not_Repeated(held, at)?;

    return Ok(Some(value.trim().to_owned()));
}

/// Whether a line carries nothing worth parsing: blank, or a comment.
fn Is_Blank_Or_Comment(line: &str) -> bool
{
    return line.is_empty() || line.starts_with('#');
}

/// The line's value, once its key is confirmed to be `PATH_KEY` — refused if the line does
/// not split into `key: value` at all, or if the key is anything else.
fn Path_Key_Value<'a>(line: &'a str, at: At<'_>) -> Result<&'a str, RegistrationError>
{
    let at = at.0;

    let Some((key, value)) = line.split_once(':')
    else
    {
        return Err(RegistrationError::UnknownKey {
            at: at.to_owned(),
            key: line.to_owned(),
        });
    };
    if key.trim() != PATH_KEY
    {
        return Err(RegistrationError::UnknownKey {
            at: at.to_owned(),
            key: key.trim().to_owned(),
        });
    }

    return Ok(value);
}

/// Refuses a second `path:` line rather than resolving it by taking one of the two.
fn Check_Not_Repeated(held: Option<&str>, at: At<'_>) -> Result<(), RegistrationError>
{
    if held.is_some()
    {
        return Err(RegistrationError::RepeatedKey {
            at: at.0.to_owned(),
            key: PATH_KEY.to_owned(),
        });
    }

    return Ok(());
}

/// The two halves [`Check_Is_A_Record`] computes about a named path, bundled rather than
/// passed as two adjacent bools — a position is not a name, and `Is_A_Record_Path(named,
/// true, false)` said nothing at the call site about which half was which.
struct PathShape
{
    inside: bool,
    markdown: bool,
}

/// That what a registration names is a record file, and that it is there.
fn Check_Is_A_Record(named: &str, at: At<'_>, root: &Path) -> Result<(), RegistrationError>
{
    let at = at.0;
    let inside = named
        .strip_prefix(RECORD_DIRECTORY)
        .and_then(|rest| return rest.strip_prefix('/'))
        .is_some_and(|rest| return !rest.is_empty() && !rest.contains('/'));
    let markdown = Path::new(named)
        .extension()
        .is_some_and(|extension| return extension == "md");
    if !Is_A_Record_Path(named, PathShape { inside, markdown })
    {
        return Err(RegistrationError::Outside {
            at: at.to_owned(),
            named: named.to_owned(),
        });
    }
    if !root.join(named).is_file()
    {
        return Err(RegistrationError::Absent {
            at: at.to_owned(),
            named: named.to_owned(),
        });
    }

    return Ok(());
}

/// Whether a registered path is one this store will read a record from.
///
/// `shape.inside` and `shape.markdown` are the two halves already computed above. The other
/// two clauses are traversal: a `..` or a backslash would leave the record directory while
/// still spelling a name that looks as though it sits inside it.
fn Is_A_Record_Path(named: &str, shape: PathShape) -> bool
{
    return shape.inside && shape.markdown && !named.contains("..") && !named.contains('\\');
}

/// Two registrations naming one record would make the two generated tables disagree in
/// length, so the second one is refused rather than taken.
fn Assert_One_Identity_Per_Record(found: &[Registration]) -> Result<(), RegistrationError>
{
    use std::collections::BTreeMap;

    let mut by_record: BTreeMap<String, String> = BTreeMap::new();

    for registration in found
    {
        if let Some(first) = by_record.insert(registration.path.clone(), registration.id.clone())
        {
            return Err(RegistrationError::Shared {
                named: registration.path.clone(),
                first,
                second: registration.id.clone(),
            });
        }
    }

    return Ok(());
}

/// Coverage for the two items declared in this file. `registration/tests.rs` is a separate
/// `.rs` file and so cannot address a function declared in this one, no matter how
/// thoroughly its refusals exercise `Registrations_In` and `Describe` underneath.
#[cfg(test)]
mod inline_coverage
{
    use super::*;
    use std::path::PathBuf;

    /// A record that is really on disk, so a fixture can be well-formed.
    const A_REAL_RECORD: &str = "docs/records/OD-GATE-001-a-skipped-test-reports-ok.md";

    /// How many directories up the repository root sits from the crate's own directory.
    const ROOT_ANCESTORS: usize = 3;

    #[test]
    fn Test_Registrations_In_Should_Read_A_Well_Formed_Directory()
    {
        let directory = Synthetic_Directory("well-formed");
        std::fs::write(
            directory.join("OD-GATE-001.record"),
            format!("path: {A_REAL_RECORD}\n"),
        )
        .expect("writes a fixture");

        let found = Registrations_In(&directory, &Root())
            .expect("the synthetic directory holds one well-formed registration");

        assert_eq!(
            found,
            vec![Registration {
                id: "OD-GATE-001".to_owned(),
                path: A_REAL_RECORD.to_owned(),
            }]
        );
        std::fs::remove_dir_all(&directory)
            .expect("the synthetic directory this test wrote into can be removed");
    }

    fn Synthetic_Directory(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-registration-inline-{name}-{}", std::process::id()));
        if path.exists()
        {
            std::fs::remove_dir_all(&path)
                .expect("the previous run's synthetic directory is removable");
        }
        std::fs::create_dir_all(&path).expect("a test needs a temporary directory");
        return path;
    }

    fn Root() -> PathBuf
    {
        return Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(ROOT_ANCESTORS)
            .expect("the crate sits three directories below the repository root")
            .to_path_buf();
    }

    #[test]
    fn Test_Describe_Should_Combine_What_Went_Wrong_With_Why_It_Is_Refused()
    {
        let refusal = RegistrationError::Empty { at: "docs/records".to_owned() };

        let description = refusal.Describe();

        assert!(description.contains("docs/records"), "{description}");
        assert!(
            description.contains("vacuous outcome"),
            "the fault and the invariant both belong in one sentence: {description}"
        );
    }
}

#[cfg(test)]
mod tests;
