//! What `nomos-mcp` may depend on, asserted from the real graph rather than left to good
//! behavior.
//!
//! `nomos-mcp`'s own module doc draws a stricter boundary than its sibling
//! `nomos-api-transport` needs: rather than calling `nomos_api::Handle_*` directly and
//! having to be policed by name the way `transport_registry.rs` polices that crate, it
//! depends on `nomos-api-transport` alone and reaches every Gate verb through that crate's
//! own `NomosApiService`, under the name the client asked for. (Until `OD-HOST-013` it did
//! that by re-serializing each `tools/call` into a synthetic JSON-RPC line and handing it
//! to that crate's own parser -- a round trip through a wire format neither side was
//! reading off a wire, which existed only because the two had no shared contract to meet
//! at. The boundary this module asserts is the same either way: it is the dependency edge,
//! not the calling convention.) A
//! dependency on `nomos-api` or on any orchestration crate would be the only way that
//! boundary could quietly widen -- there would be no call to grep for the way
//! `transport_registry.rs` greps for one, because the crate would not need a call to reach a
//! handler it already depended on directly. Read from `cargo metadata` rather than from
//! `Cargo.toml`'s own text, the same reason [`Workspace`] itself exists: a resolved graph
//! cannot be fooled by a dependency named under a feature or a target this platform does not
//! build.

use nomos_contract_tests::Workspace;

/// The crate whose registry this module bounds.
const CRATE: &str = "nomos-mcp";

/// The one internal crate `CRATE` may depend on.
const TRANSPORT: &str = "nomos-api-transport";

/// `nomos-mcp` depends on `nomos-api-transport` and no other workspace member.
#[test]
fn Test_The_Mcp_Crate_Should_Depend_On_Nothing_But_The_Transport_It_Projects()
{
    let workspace = Workspace::Load();
    let package = workspace.Get(CRATE).unwrap_or_else(|| panic!("{CRATE} is not a workspace member"));

    let internal: Vec<&String> = package.direct_dependencies.iter().filter(|name| return name.starts_with("nomos-")).collect();

    assert_eq!(
        internal,
        vec![TRANSPORT],
        "{CRATE} depends on {internal:?} inside this workspace; it must depend on {TRANSPORT} \
         alone. A tools/call reaches a Gate handler only through {TRANSPORT}'s own served \
         surface, under a name that crate's own registry admits -- a second internal \
         dependency here is the only way that could quietly widen, since there \
         would be no call to police the way nomos-api-transport's own registry \
         already is."
    );
}

/// The assertion above has a real subject: `nomos-mcp` is a real workspace member with at
/// least one real dependency, not an empty package that would satisfy the exact-match above
/// by depending on nothing at all.
#[test]
fn Test_The_Assertion_Should_Have_A_Real_Subject()
{
    let workspace = Workspace::Load();
    let package = workspace.Get(CRATE).unwrap_or_else(|| panic!("{CRATE} is not a workspace member"));

    assert!(
        package.direct_dependencies.contains(TRANSPORT),
        "{CRATE} does not depend on {TRANSPORT} at all, so the exact-match assertion above \
         would pass over a crate that projects nothing"
    );
}
