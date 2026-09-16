//! [`Handle_Spec_Profiles`] and its own [`ProfilesResponse`].

use nomos_spec_project::Profile;
use serde::Serialize;

/// Lists every shipped projection profile, exactly as `nomos spec profiles` would, and
/// hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Spec_Profiles() -> ProfilesResponse
{
    return ProfilesResponse::From(nomos_spec_orchestration::Profiles());
}

/// What a real `nomos spec profiles` produced, in a shape `serde_json` can hand across a
/// wire.
///
/// A tagged enum, the same shape `ListResponse` already uses: `nomos_spec_project::
/// ProjectError` does not derive `Serialize` -- nothing needed a wire format for it before
/// this crate existed -- so the refusal case is named rather than collapsed into an empty
/// success. `Profiles`' own doc calls that refusal "a defect in this build, not in
/// anything the caller did": real, but not expected to fire in practice, the same
/// never-actually-taken arm `nomos-cli`'s own `gate/tests.rs` already exhausts other exit
/// codes for.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ProfilesResponse
{
    /// The embedded profile catalogue parsed.
    Listed
    {
        /// Every shipped projection profile.
        profiles: Vec<Profile>,
    },
    /// The embedded profile catalogue itself failed to parse.
    Unreadable
    {
        /// What went wrong, as `ProjectError`'s own `Display` renders it.
        cause: String,
    },
}

impl ProfilesResponse
{
    pub(crate) fn From(profiles: Result<Vec<Profile>, nomos_spec_project::ProjectError>) -> Self
    {
        return match profiles
        {
            Ok(profiles) => Self::Listed { profiles },
            Err(error) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::Assert_Round_Trips_As_Json;

    /// The catalogue is embedded in the binary, so this is a real end-to-end exercise of
    /// this workspace's own shipped profiles -- no fixture, no scratch directory, the same
    /// zero-setup shape `Profiles()` itself has.
    #[test]
    fn Test_Handle_Spec_Profiles_Should_List_This_Workspaces_Own_Shipped_Profiles()
    {
        let response = Handle_Spec_Profiles();

        let ProfilesResponse::Listed { profiles } = response
        else
        {
            // The profile catalogue this test reads is embedded in the binary this test
            // itself was compiled into, so a parse failure here means the workspace's own
            // shipped catalogue is broken, not a runtime condition this test should recover
            // from.
            panic!("this workspace's own embedded catalogue parses");
        };
        assert!(!profiles.is_empty());
    }

    #[test]
    fn Test_From_Should_Round_Trip_As_Json()
    {
        let response = Handle_Spec_Profiles();

        Assert_Round_Trips_As_Json(&response, "listed")
            .expect("a listed response serializes and parses back as a tagged object");
    }
}
