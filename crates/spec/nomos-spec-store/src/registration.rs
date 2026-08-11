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

use std::collections::BTreeMap;
use std::path::Path;

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

/// Why a registration directory was refused.
///
/// Every condition here is a refusal and not a skip. A skipped registration is a governing
/// record that leaves the store without anybody being told, and an absent record is exactly
/// what the guard downstream of this reader exists to catch — so absence must never become
/// success on the way in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RegistrationError
{
    /// The directory, or a file in it, did not read.
    Unreadable
    {
        at: String,
        cause: String,
    },
    /// The directory holds no registration at all.
    Empty
    {
        at: String,
    },
    /// A file in the directory is not named `<ID>.record`.
    NotARegistration
    {
        at: String,
    },
    /// A registration's stem is not an identifier this store could use.
    NotAnIdentifier
    {
        at: String,
        stem: String,
    },
    /// A registration names no record.
    NoPath
    {
        at: String,
    },
    /// A registration names two records.
    RepeatedKey
    {
        at: String,
        key: String,
    },
    /// A registration carries a line this format does not define.
    UnknownKey
    {
        at: String,
        key: String,
    },
    /// A registration names something that is not a record file under `docs/records`.
    Outside
    {
        at: String,
        named: String,
    },
    /// A registration names a record file that is not on disk.
    Absent
    {
        at: String,
        named: String,
    },
    /// Two registrations name one record file.
    Shared
    {
        named: String,
        first: String,
        second: String,
    },
}

impl RegistrationError
{
    /// What went wrong, and what the reader would have had to guess to continue.
    pub(crate) fn Describe(&self) -> String
    {
        return match self
        {
            Self::Unreadable { at, cause } => format!(
                "{at} did not read: {cause}. A registration directory that half-opens is a \
                 governing list that is quietly short."
            ),
            Self::Empty { at } => format!(
                "{at} holds no *.{REGISTRATION_EXTENSION} file. An empty governing table is \
                 the vacuous outcome this arrangement exists to prevent, so it is refused \
                 rather than produced."
            ),
            Self::NotARegistration { at } => format!(
                "{at} is in the registration directory and is not a \
                 *.{REGISTRATION_EXTENSION} file. A typo'd extension would be a record \
                 silently dropped, so nothing in this directory is ignored."
            ),
            Self::NotAnIdentifier { at, stem } => format!(
                "{at} has the stem `{stem}`, which is not a record identifier. The stem is \
                 the identity; `od-foo-001` is not an identifier this store uses."
            ),
            Self::NoPath { at } => format!(
                "{at} carries no `{PATH_KEY}:` line, so it names no record. A registration \
                 that names nothing is a phantom governing record."
            ),
            Self::RepeatedKey { at, key } => format!(
                "{at} carries `{key}:` more than once. Resolving that by taking the first is \
                 the defect, not the fix."
            ),
            Self::UnknownKey { at, key } => format!(
                "{at} carries `{key}`, which this format does not define. The only key is \
                 `{PATH_KEY}:`; comments start with `#`."
            ),
            Self::Outside { at, named } => format!(
                "{at} names `{named}`, which is not a markdown file under \
                 `{RECORD_DIRECTORY}/`. A registration may only name a record."
            ),
            Self::Absent { at, named } => format!(
                "{at} names `{named}`, which is not on disk. Left to `include_str!`, the \
                 error would name a generated file instead of the registration that is wrong."
            ),
            Self::Shared { named, first, second } => format!(
                "`{first}` and `{second}` both name `{named}`. Two identities over one \
                 document would make the two generated tables disagree in length."
            ),
        };
    }
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

        let file = entry.path();
        let at = file.display().to_string();

        if file
            .extension()
            .is_none_or(|extension| return extension != REGISTRATION_EXTENSION)
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

        let text = std::fs::read_to_string(&file).map_err(|error| {
            return RegistrationError::Unreadable {
                at: at.clone(),
                cause: error.to_string(),
            };
        })?;

        let named = Named_Record(&text, &at)?;
        Check_Is_A_Record(&named, &at, root)?;

        found.push(Registration {
            id: stem.to_owned(),
            path: named,
        });
    }

    if found.is_empty()
    {
        return Err(RegistrationError::Empty {
            at: directory.display().to_string(),
        });
    }

    let mut by_record: BTreeMap<String, String> = BTreeMap::new();
    for registration in &found
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

    // Byte order on the identifier. `read_dir` order is not deterministic across platforms
    // and must not reach a generated table, or two machines build two different binaries
    // from one commit.
    found.sort_by(|left, right| return left.id.cmp(&right.id));

    return Ok(found);
}

/// The record a registration body names.
///
/// Blank lines and `#` comments are ignored, and that is the only tolerance. Anything else
/// is refused rather than passed over.
fn Named_Record(text: &str, at: &str) -> Result<String, RegistrationError>
{
    let mut named: Option<String> = None;

    for line in text.lines()
    {
        // Trimming also removes a carriage return a CRLF checkout would leave behind, so
        // the path a registration names does not depend on how the tree was cloned.
        let line = line.trim();

        if line.is_empty() || line.starts_with('#')
        {
            continue;
        }

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

        if named.is_some()
        {
            return Err(RegistrationError::RepeatedKey {
                at: at.to_owned(),
                key: PATH_KEY.to_owned(),
            });
        }

        named = Some(value.trim().to_owned());
    }

    return named.ok_or_else(|| {
        return RegistrationError::NoPath { at: at.to_owned() };
    });
}

/// Whether a registered path is one this store will read a record from.
///
/// `inside` and `markdown` are the two halves already computed above. The other two clauses
/// are traversal: a `..` or a backslash would leave the record directory while still
/// spelling a name that looks as though it sits inside it.
fn Is_A_Record_Path(named: &str, inside: bool, markdown: bool) -> bool
{
    return inside && markdown && !named.contains("..") && !named.contains('\\');
}

/// That what a registration names is a record file, and that it is there.
fn Check_Is_A_Record(named: &str, at: &str, root: &Path) -> Result<(), RegistrationError>
{
    let inside = named
        .strip_prefix(RECORD_DIRECTORY)
        .and_then(|rest| return rest.strip_prefix('/'))
        .is_some_and(|rest| return !rest.is_empty() && !rest.contains('/'));

    let markdown = Path::new(named)
        .extension()
        .is_some_and(|extension| return extension == "md");

    if !Is_A_Record_Path(named, inside, markdown)
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

#[cfg(test)]
mod tests
{
    use super::{Registration, RegistrationError, Registrations_In};
    use std::path::{Path, PathBuf};

    /// A record that is really on disk, so a fixture can be well-formed.
    const A_REAL_RECORD: &str = "docs/records/OD-GATE-001-a-skipped-test-reports-ok.md";

    /// A second one, for the ordering and duplication cases.
    const ANOTHER_REAL_RECORD: &str = "docs/records/D-129-the-store-is-the-identity-substrate.md";

    fn Root() -> PathBuf
    {
        return Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("the crate sits three directories below the repository root")
            .to_path_buf();
    }

    /// A directory of this test's own, emptied first so a previous run cannot supply a file.
    fn Synthetic(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-registration-{name}-{}", std::process::id()));
        Cleared(&path);
        std::fs::create_dir_all(&path).expect("a test needs a temporary directory");
        return path;
    }

    /// Removes a fixture directory, tolerating the one failure that is not one.
    ///
    /// A directory that is already absent is the state this asks for, so `NotFound` is
    /// success. Anything else is said out loud rather than discarded, in both directions it
    /// is called from: a setup that quietly cannot delete hands the test a file a previous
    /// run wrote — which is exactly what `Synthetic` promises it cannot — and a teardown
    /// that quietly cannot delete leaves one directory per run with nothing reporting it.
    fn Cleared(path: &Path)
    {
        if let Err(cause) = std::fs::remove_dir_all(path)
            && cause.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("{} could not be cleared: {cause}", path.display());
        }
    }

    fn Write(directory: &Path, name: &str, body: &str)
    {
        std::fs::write(directory.join(name), body).expect("a test needs to write its fixture");
    }

    fn Read(directory: &Path) -> Result<Vec<Registration>, RegistrationError>
    {
        return Registrations_In(directory, &Root());
    }

    // -----------------------------------------------------------------------
    // The refusals. Each asserts `Err`, not a short `Ok`.
    // -----------------------------------------------------------------------

    #[test]
    fn Test_An_Empty_Directory_Should_Refuse_Rather_Than_Produce_An_Empty_Table()
    {
        let directory = Synthetic("empty");

        let refusal = Read(&directory).expect_err("an empty governing table must be refused");

        assert!(
            matches!(refusal, RegistrationError::Empty { .. }),
            "{}",
            refusal.Describe()
        );
        Cleared(&directory);
    }

    #[test]
    fn Test_A_Missing_Directory_Should_Refuse()
    {
        let directory = Synthetic("missing");
        std::fs::remove_dir_all(&directory).expect("the test removes what it made");

        let refusal = Read(&directory).expect_err("a directory that is not there is not `no records`");

        assert!(
            matches!(refusal, RegistrationError::Unreadable { .. }),
            "{}",
            refusal.Describe()
        );
    }

    #[test]
    fn Test_A_File_That_Is_Not_A_Registration_Should_Refuse()
    {
        for intruder in ["README.md", "OD-FOO-001.record.bak", "notes.txt"]
        {
            let directory = Synthetic("intruder");
            Write(&directory, "OD-GATE-001.record", &format!("path: {A_REAL_RECORD}\n"));
            Write(&directory, intruder, "anything\n");

            let refusal = Read(&directory)
                .expect_err("a file in the registration directory is never ignored");

            assert!(
                matches!(
                    refusal,
                    RegistrationError::NotARegistration { .. }
                        | RegistrationError::NotAnIdentifier { .. }
                ),
                "{intruder}: {}",
                refusal.Describe()
            );
            Cleared(&directory);
        }
    }

    #[test]
    fn Test_A_Registration_With_No_Path_Line_Should_Refuse()
    {
        let directory = Synthetic("no-path");
        Write(&directory, "OD-GATE-001.record", "# only a comment\n\n");

        let refusal = Read(&directory).expect_err("a registration naming nothing is a phantom");

        assert!(
            matches!(refusal, RegistrationError::NoPath { .. }),
            "{}",
            refusal.Describe()
        );
        Cleared(&directory);
    }

    #[test]
    fn Test_A_Registration_With_Two_Path_Lines_Should_Refuse()
    {
        let directory = Synthetic("two-paths");
        Write(
            &directory,
            "OD-GATE-001.record",
            &format!("path: {A_REAL_RECORD}\npath: {ANOTHER_REAL_RECORD}\n"),
        );

        let refusal = Read(&directory).expect_err("ambiguity must not be resolved by guessing");

        assert!(
            matches!(refusal, RegistrationError::RepeatedKey { .. }),
            "{}",
            refusal.Describe()
        );
        Cleared(&directory);
    }

    #[test]
    fn Test_An_Unknown_Key_Should_Refuse()
    {
        let directory = Synthetic("unknown-key");
        Write(
            &directory,
            "OD-GATE-001.record",
            &format!("id: OD-GATE-001\npath: {A_REAL_RECORD}\n"),
        );

        let refusal = Read(&directory).expect_err("a key this format does not define is refused");

        assert!(
            matches!(refusal, RegistrationError::UnknownKey { .. }),
            "{}",
            refusal.Describe()
        );
        Cleared(&directory);
    }

    #[test]
    fn Test_A_Registration_Naming_A_File_That_Is_Not_There_Should_Refuse()
    {
        let directory = Synthetic("phantom");
        Write(
            &directory,
            "OD-FOO-001.record",
            "path: docs/records/OD-FOO-001-nothing-wrote-this.md\n",
        );

        let refusal = Read(&directory).expect_err("a registration naming nothing is refused here");

        assert!(
            matches!(refusal, RegistrationError::Absent { .. }),
            "{}",
            refusal.Describe()
        );
        Cleared(&directory);
    }

    #[test]
    fn Test_A_Registration_Naming_A_Path_Outside_The_Record_Directory_Should_Refuse()
    {
        for named in [
            "../../secrets.md",
            "docs/records/../../Cargo.toml",
            "docs/notes/OD-GATE-001.md",
            "docs/records/OD-GATE-001-a-skipped-test-reports-ok.txt",
        ]
        {
            let directory = Synthetic("outside");
            Write(&directory, "OD-GATE-001.record", &format!("path: {named}\n"));

            let refusal =
                Read(&directory).expect_err("a registration may only name a record file");

            assert!(
                matches!(refusal, RegistrationError::Outside { .. }),
                "{named}: {}",
                refusal.Describe()
            );
            Cleared(&directory);
        }
    }

    #[test]
    fn Test_Two_Registrations_Naming_One_Record_Should_Refuse()
    {
        let directory = Synthetic("shared");
        Write(&directory, "OD-GATE-001.record", &format!("path: {A_REAL_RECORD}\n"));
        Write(&directory, "OD-GATE-002.record", &format!("path: {A_REAL_RECORD}\n"));

        let refusal = Read(&directory).expect_err("two identities over one document is refused");

        assert!(
            matches!(refusal, RegistrationError::Shared { .. }),
            "{}",
            refusal.Describe()
        );
        Cleared(&directory);
    }

    #[test]
    fn Test_A_Stem_That_Is_Not_An_Identifier_Should_Refuse()
    {
        for stem in ["od-foo-001", "OD FOO", "OD--FOO-001", "1D-FOO"]
        {
            let directory = Synthetic("stem");
            Write(
                &directory,
                &format!("{stem}.record"),
                &format!("path: {A_REAL_RECORD}\n"),
            );

            let refusal = Read(&directory).expect_err("the stem is the identity");

            assert!(
                matches!(refusal, RegistrationError::NotAnIdentifier { .. }),
                "{stem}: {}",
                refusal.Describe()
            );
            Cleared(&directory);
        }
    }

    // -----------------------------------------------------------------------
    // The positive controls. Without them a reader that refused everything
    // would satisfy every test above.
    // -----------------------------------------------------------------------

    #[test]
    fn Test_A_Well_Formed_Directory_Should_Be_Read()
    {
        let directory = Synthetic("well-formed");
        Write(&directory, "OD-GATE-001.record", &format!("path: {A_REAL_RECORD}\n"));
        Write(&directory, "D-129.record", &format!("path: {ANOTHER_REAL_RECORD}\n"));

        let read = Read(&directory).expect("two well-formed registrations must read");

        assert_eq!(
            read,
            vec![
                Registration {
                    id: "D-129".to_owned(),
                    path: ANOTHER_REAL_RECORD.to_owned(),
                },
                Registration {
                    id: "OD-GATE-001".to_owned(),
                    path: A_REAL_RECORD.to_owned(),
                },
            ]
        );
        Cleared(&directory);
    }

    #[test]
    fn Test_The_Result_Should_Be_Sorted_By_Identifier()
    {
        let directory = Synthetic("sorted");
        Write(&directory, "OD-GATE-001.record", &format!("path: {A_REAL_RECORD}\n"));
        Write(&directory, "D-129.record", &format!("path: {ANOTHER_REAL_RECORD}\n"));

        let identifiers: Vec<String> = Read(&directory)
            .expect("reads")
            .into_iter()
            .map(|registration| return registration.id)
            .collect();

        let mut sorted = identifiers.clone();
        sorted.sort();
        assert_eq!(
            identifiers, sorted,
            "`read_dir` order reached the table, so two machines build two binaries from \
             one commit"
        );
        Cleared(&directory);
    }

    #[test]
    fn Test_Comments_And_Blank_Lines_Should_Be_Ignored()
    {
        let directory = Synthetic("comments");
        Write(
            &directory,
            "OD-GATE-001.record",
            &format!("# why this file exists\n\n   \n#: not a key\npath:   {A_REAL_RECORD}   \n\n"),
        );

        let read = Read(&directory).expect("the only tolerance is comments and blank lines");

        assert_eq!(read.first().map(|found| return found.path.as_str()), Some(A_REAL_RECORD));
        Cleared(&directory);
    }

    #[test]
    fn Test_A_Carriage_Return_Should_Not_Reach_The_Path()
    {
        let directory = Synthetic("crlf");
        Write(
            &directory,
            "OD-GATE-001.record",
            &format!("# a CRLF checkout\r\npath: {A_REAL_RECORD}\r\n"),
        );

        let read = Read(&directory).expect("a CRLF registration is still a registration");

        assert_eq!(read.first().map(|found| return found.path.as_str()), Some(A_REAL_RECORD));
        Cleared(&directory);
    }

    /// The reader consults the directory it is given and nothing else.
    ///
    /// The structural half of the guard against this ever being pointed at `docs/records`.
    /// A synthetic directory holds one registration while the repository holds dozens, so a
    /// reader that consulted anything but its argument returns the wrong number here. The
    /// other half is textual and lives beside the guard it protects; a textual check is a
    /// textual check and is labelled as one there.
    #[test]
    fn Test_The_Reader_Should_Enumerate_The_Directory_It_Is_Given()
    {
        let directory = Synthetic("given");
        Write(&directory, "OD-GATE-001.record", &format!("path: {A_REAL_RECORD}\n"));

        let read = Read(&directory).expect("one registration is a valid directory");

        assert_eq!(
            read.len(),
            1,
            "the reader returned more than the directory it was handed holds: {read:?}"
        );
        Cleared(&directory);
    }

    /// And the real directory reads, so the synthetic cases are not the only ones exercised.
    #[test]
    fn Test_The_Real_Directory_Should_Read()
    {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("records");

        let read = Registrations_In(&directory, &Root()).expect("the repository's own registrations must read");

        let files = std::fs::read_dir(&directory)
            .expect("the registration directory is in the repository")
            .flatten()
            .filter(|entry| {
                return entry
                    .path()
                    .extension()
                    .is_some_and(|extension| return extension == "record");
            })
            .count();

        assert_eq!(read.len(), files);
        assert!(!read.is_empty(), "no registration was read, so this checked nothing");
    }
}
