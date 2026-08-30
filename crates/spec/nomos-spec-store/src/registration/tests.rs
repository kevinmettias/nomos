//! The registration reader's refusals, executed rather than asserted.

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

/// File names that sit in the registration directory without being one.
fn Intruders() -> [&'static str; 3]
{
    return ["README.md", "OD-FOO-001.record.bak", "notes.txt"];
}

#[test]
fn Test_A_File_That_Is_Not_A_Registration_Should_Refuse()
{
    for intruder in Intruders()
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

/// Paths a registration might name that are not a record file under the record directory.
fn Paths_Outside_The_Record_Directory() -> [&'static str; 4]
{
    return [
        "../../secrets.md",
        "docs/records/../../Cargo.toml",
        "docs/notes/OD-GATE-001.md",
        "docs/records/OD-GATE-001-a-skipped-test-reports-ok.txt",
    ];
}

#[test]
fn Test_A_Registration_Naming_A_Path_Outside_The_Record_Directory_Should_Refuse()
{
    for named in Paths_Outside_The_Record_Directory()
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

/// Stems that are not identifiers this store could use.
fn Stems_That_Are_Not_Identifiers() -> [&'static str; 4]
{
    return ["od-foo-001", "OD FOO", "OD--FOO-001", "1D-FOO"];
}

#[test]
fn Test_A_Stem_That_Is_Not_An_Identifier_Should_Refuse()
{
    for stem in Stems_That_Are_Not_Identifiers()
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

/// Bodies that carry incidental noise around the one `path:` line that matters — comments,
/// blank lines, or the carriage return a CRLF checkout leaves behind — paired with the
/// reason each is tolerated. All of them must still yield the same one path.
fn Bodies_With_Incidental_Noise() -> [(&'static str, String); 2]
{
    return [
        (
            "the only tolerance is comments and blank lines",
            format!("# why this file exists\n\n   \n#: not a key\npath:   {A_REAL_RECORD}   \n\n"),
        ),
        (
            "a CRLF registration is still a registration",
            format!("# a CRLF checkout\r\npath: {A_REAL_RECORD}\r\n"),
        ),
    ];
}

#[test]
fn Test_Incidental_Noise_Around_The_Path_Line_Should_Not_Reach_The_Path()
{
    for (why, body) in Bodies_With_Incidental_Noise()
    {
        let directory = Synthetic("noise");
        Write(&directory, "OD-GATE-001.record", &body);

        let read = Read(&directory).expect(why);

        assert_eq!(
            read.first().map(|found| return found.path.as_str()),
            Some(A_REAL_RECORD),
            "{why}"
        );
        Cleared(&directory);
    }
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
