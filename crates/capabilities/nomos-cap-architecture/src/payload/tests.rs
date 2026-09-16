//! The encoding round trip, the refusals, and the relation queries.
//!
//! Every fixture below declares an architecture this workspace does not have. That is
//! deliberate and it is the property under test: if these read naturally with components
//! called `Domain`, `Infrastructure` and `Api`, then nothing in this crate knows what a zone
//! is, which is what `OD-RULES-029` requires of a declaration format before it has
//! externalized anything.

use super::*;

#[test]
fn Test_Parse_Payload_Should_Round_Trip_A_Declaration_Through_Its_Own_Encoding()
{
    let payload = Sample();
    let encoded = Encode_Payload(&payload);
    let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

    assert_eq!(decoded, payload);
}

#[test]
fn Test_Encode_Payload_Should_Produce_Stable_Diffable_Bytes()
{
    let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("ASCII and tabs");

    assert_eq!(
        rendered,
        "component\tDomain\n\
         component\tInfrastructure\n\
         component\tApi\n\
         member\tbilling\tDomain\n\
         member\tpostgres\tInfrastructure\n\
         member\thttp\tApi\n\
         permits\tApi\tDomain\n\
         permits\tInfrastructure\tDomain\n\
         exception\tbilling\tbilling-core\n\
         authority\tledger-store\n\
         door\tledger-store\tbilling\n"
    );
    assert!(!rendered.contains('\r'), "line endings must not be local");
}

#[test]
fn Test_An_Empty_Byte_String_Should_Decode_To_A_Repository_That_Declares_Nothing()
{
    let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");

    assert!(!decoded.Declares_An_Architecture());
    assert_eq!(decoded.Component_Of("billing"), None);
}

/// Components without members is still a declaration: the repository said what it divides
/// itself into and has not placed anything yet, which is not the same claim as declaring
/// nothing.
#[test]
fn Test_Components_With_No_Members_Should_Still_Declare_An_Architecture()
{
    let decoded = Parse_Payload(b"component\tDomain\n").expect("one component line");

    assert!(decoded.Declares_An_Architecture());
}

#[test]
fn Test_A_Line_With_An_Unknown_Tag_Should_Be_Refused()
{
    let error = Parse_Payload(b"zone\tDomain\n").expect_err("a tag this schema does not name must be refused");

    assert!(error.reason.contains("no tag this schema names"), "{}", error.reason);
}

#[test]
fn Test_A_Two_Field_Line_Missing_A_Field_Should_Be_Refused()
{
    for bytes in Short_Lines()
    {
        let error = Parse_Payload(bytes).expect_err("a line short of its tag's fields must be refused");
        assert!(
            error.reason.contains("does not have exactly 2 fields"),
            "expected a field-count refusal for {bytes:?}, got: {}",
            error.reason
        );
    }
}

fn Short_Lines() -> Vec<&'static [u8]>
{
    return vec![b"component\tDomain\nmember\tbilling\n", b"component\tDomain\npermits\tDomain\n", b"exception\tbilling\n"];
}

/// The refusal `OD-RULES-003` asks for by name. A statement about a component the declaration
/// never declared is incomplete, and reporting it is a different thing from dropping it.
#[test]
fn Test_A_Member_In_An_Undeclared_Component_Should_Be_Refused()
{
    let error = Parse_Payload(b"component\tDomain\nmember\tbilling\tInfrastructure\n")
        .expect_err("a member of a component no component line declares must be refused");

    assert!(error.reason.contains("which no component line declares"), "{}", error.reason);
    assert!(error.reason.contains("Infrastructure"), "the refusal names the component: {}", error.reason);
}

#[test]
fn Test_A_Permission_Naming_An_Undeclared_Component_Should_Be_Refused()
{
    let error = Parse_Payload(b"component\tDomain\npermits\tDomain\tApi\n")
        .expect_err("a permission naming a component no component line declares must be refused");

    assert!(error.reason.contains("which no component line declares"), "{}", error.reason);
}

#[test]
fn Test_A_Door_Without_Its_Authority_Should_Be_Refused()
{
    let error = Parse_Payload(b"door\tledger-store\tbilling\n").expect_err("a door into an undeclared authority must be refused");

    assert!(error.reason.contains("which no authority line declares"), "{}", error.reason);
}

#[test]
fn Test_Component_Of_Should_Answer_From_The_Declaration_Alone()
{
    let payload = Sample();

    assert_eq!(payload.Component_Of("billing"), Some("Domain"));
    assert_eq!(payload.Component_Of("http"), Some("Api"));
    assert_eq!(payload.Component_Of("never-declared"), None);
}

#[test]
fn Test_Permits_Should_Answer_Exactly_What_The_Declaration_States()
{
    let payload = Sample();

    assert!(payload.Permits(Depending("Api"), Depended("Domain")));
    assert!(payload.Permits(Depending("Infrastructure"), Depended("Domain")));
    assert!(!payload.Permits(Depending("Domain"), Depended("Api")), "the reverse is not declared");
    assert!(!payload.Permits(Depending("Api"), Depended("Infrastructure")), "an undeclared pair is not permitted");
}

/// A declaration that states no self-permission gets `false` for one, which is how a
/// repository keeps two members of one component from naming each other. It is a property of
/// the declaration and not a rule written into this crate: a repository that does declare
/// `permits X X` gets `true`, and this crate has no opinion about that.
#[test]
fn Test_Permits_Should_Follow_The_Declaration_For_A_Component_Against_Itself()
{
    let payload = Sample();
    assert!(!payload.Permits(Depending("Domain"), Depended("Domain")), "the sample declares no self-permission");

    let permissive = Parse_Payload(b"component\tDomain\npermits\tDomain\tDomain\n").expect("a declaration may permit a component against itself");
    assert!(permissive.Permits(Depending("Domain"), Depended("Domain")), "a declared self-permission is honoured");
}

#[test]
fn Test_Excepts_Should_Be_Directed()
{
    let payload = Sample();

    assert!(payload.Excepts(Depending("billing"), Depended("billing-core")));
    assert!(
        !payload.Excepts(Depending("billing-core"), Depended("billing")),
        "an exception is one real dependency, not a pair exemption"
    );
}

#[test]
fn Test_Doors_Into_Should_Answer_None_For_A_Package_That_Is_Not_An_Authority()
{
    let payload = Sample();

    assert_eq!(payload.Doors_Into("ledger-store"), Some(["billing".to_owned()].as_slice()));
    assert_eq!(payload.Doors_Into("billing"), None);
}

/// An authority nothing may reach is representable, and means what it says.
#[test]
fn Test_An_Authority_With_No_Doors_Should_Decode_As_One()
{
    let decoded = Parse_Payload(b"authority\tsealed\n").expect("an authority line alone");

    assert_eq!(decoded.Doors_Into("sealed"), Some([].as_slice()));
}

/// A repository whose architecture is nothing like this workspace's, expressed without a
/// single word this crate had to know in advance.
fn Sample() -> ArchitecturePayload
{
    return ArchitecturePayload {
        components: vec!["Domain".to_owned(), "Infrastructure".to_owned(), "Api".to_owned()],
        membership: vec![
            Membership { package: "billing".to_owned(), component: "Domain".to_owned() },
            Membership { package: "postgres".to_owned(), component: "Infrastructure".to_owned() },
            Membership { package: "http".to_owned(), component: "Api".to_owned() },
        ],
        permissions: vec![
            Permission { from: "Api".to_owned(), to: "Domain".to_owned() },
            Permission { from: "Infrastructure".to_owned(), to: "Domain".to_owned() },
        ],
        exceptions: vec![Exception { from: "billing".to_owned(), to: "billing-core".to_owned() }],
        authorities: vec![Authority { package: "ledger-store".to_owned(), doors: vec!["billing".to_owned()] }],
    };
}
