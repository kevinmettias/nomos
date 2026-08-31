//! Why a registration was refused.

use super::{PATH_KEY, RECORD_DIRECTORY, REGISTRATION_EXTENSION};

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Describe_Should_Join_The_Fault_And_The_Reason()
    {
        let error = RegistrationError::NoPath { at: "OD-EXAMPLE-001.record".to_owned() };

        let described = error.Describe();

        assert_eq!(
            described,
            "OD-EXAMPLE-001.record carries no `path:` line, so it names no record. A registration \
             that names nothing is a phantom governing record."
        );
    }
}
