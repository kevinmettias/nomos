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
        return format!("{} {}", self.Fault(), self.Because());
    }

    /// The fault itself, naming the file and what it said.
    fn Fault(&self) -> String
    {
        return match self
        {
            Self::Unreadable { at, cause } => format!("{at} did not read: {cause}."),
            Self::Empty { at } => format!("{at} holds no *.{REGISTRATION_EXTENSION} file."),
            Self::NotARegistration { at } => format!(
                "{at} is in the registration directory and is not a \
                 *.{REGISTRATION_EXTENSION} file."
            ),
            Self::NotAnIdentifier { at, stem } =>
            {
                format!("{at} has the stem `{stem}`, which is not a record identifier.")
            }
            Self::NoPath { at } =>
            {
                format!("{at} carries no `{PATH_KEY}:` line, so it names no record.")
            }
            Self::RepeatedKey { at, key } => format!("{at} carries `{key}:` more than once."),
            Self::UnknownKey { at, key } => format!(
                "{at} carries `{key}`, which this format does not define; the only key is \
                 `{PATH_KEY}:`."
            ),
            Self::Outside { at, named } => format!(
                "{at} names `{named}`, which is not a markdown file under \
                 `{RECORD_DIRECTORY}/`."
            ),
            Self::Absent { at, named } => format!("{at} names `{named}`, which is not on disk."),
            Self::Shared { named, first, second } =>
            {
                format!("`{first}` and `{second}` both name `{named}`.")
            }
        };
    }

    /// Why that is refused rather than passed over.
    ///
    /// Held apart from the fault because it is the invariant half: the fault names a file
    /// that differs every time, and this is the sentence that does not.
    const fn Because(&self) -> &'static str
    {
        return match self
        {
            Self::Unreadable { .. } =>
            {
                "A registration directory that half-opens is a governing list that is quietly \
                 short."
            }
            Self::Empty { .. } =>
            {
                "An empty governing table is the vacuous outcome this arrangement exists to \
                 prevent, so it is refused rather than produced."
            }
            Self::NotARegistration { .. } =>
            {
                "A typo'd extension would be a record silently dropped, so nothing in this \
                 directory is ignored."
            }
            Self::NotAnIdentifier { .. } =>
            {
                "The stem is the identity; `od-foo-001` is not an identifier this store uses."
            }
            Self::NoPath { .. } => "A registration that names nothing is a phantom governing record.",
            Self::RepeatedKey { .. } => "Resolving that by taking the first is the defect, not the fix.",
            Self::UnknownKey { .. } => "Comments start with `#`.",
            Self::Outside { .. } => "A registration may only name a record.",
            Self::Absent { .. } =>
            {
                "Left to `include_str!`, the error would name a generated file instead of the \
                 registration that is wrong."
            }
            Self::Shared { .. } =>
            {
                "Two identities over one document would make the two generated tables \
                 disagree in length."
            }
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
    let named = Named_Record(&text, &at)?;
    Check_Is_A_Record(&named, &at, root)?;

    return Ok(Registration {
        id: stem.to_owned(),
        path: named,
    });
}

/// Two registrations naming one record would make the two generated tables disagree in
/// length, so the second one is refused rather than taken.
fn Assert_One_Identity_Per_Record(found: &[Registration]) -> Result<(), RegistrationError>
{
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

/// The record a registration body names.
///
/// Blank lines and `#` comments are ignored, and that is the only tolerance. Anything else
/// is refused rather than passed over.
fn Named_Record(text: &str, at: &str) -> Result<String, RegistrationError>
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
        return RegistrationError::NoPath { at: at.to_owned() };
    });
}

/// The path one line names, or `None` for a blank line or a comment.
///
/// `held` is what an earlier line already named, because a second `path:` is refused rather
/// than resolved by taking one of them.
fn Path_Line(
    line: &str,
    at: &str,
    held: Option<&str>,
) -> Result<Option<String>, RegistrationError>
{
    if line.is_empty() || line.starts_with('#')
    {
        return Ok(None);
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
    if held.is_some()
    {
        return Err(RegistrationError::RepeatedKey {
            at: at.to_owned(),
            key: PATH_KEY.to_owned(),
        });
    }

    return Ok(Some(value.trim().to_owned()));
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
mod tests;
