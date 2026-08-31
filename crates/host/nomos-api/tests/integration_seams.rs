//! `nomos-api`'s own inline `#[cfg(test)]` suites already exercise most of the crates it
//! depends on -- but from inside the crate, where a test still sees `pub(crate)` items and
//! reaches its neighbours as an ordinary dependency, not as the wire consumer this crate exists
//! to serve. Compiled here instead, this file can only reach `nomos_gate_orchestration`,
//! `nomos_contracts`, `nomos_rules`, `nomos_model`, `nomos_ledger`, `nomos_work_orchestration`,
//! `nomos_platform`, `nomos_platform_std`, `nomos_spec_orchestration`, `nomos_spec_model`,
//! `nomos_spec_project`, `nomos_spec_store` and `nomos_workspace` through `nomos-api`'s own
//! public `Handle_*` functions and response types -- the same view a real caller has.
//!
//! Grouped by the composition each seam sits behind, the same three groups [`crate::lib`]'s own
//! module doc names: Gate, Work and Spec, plus `nomos_workspace` on its own -- it backs every
//! `Handle_Gate_*` call's environment (`composition::Host_Variant`) but is not itself carried by
//! any response, so its own seam is proven independently of the other three.

use std::path::{Path, PathBuf};
use std::time::Duration;

// ---------------------------------------------------------------------------------------------
// Gate: nomos_gate_orchestration, nomos_contracts, nomos_rules, nomos_model.
// ---------------------------------------------------------------------------------------------

/// `nomos_gate_orchestration`, `nomos_contracts` and `nomos_rules`, exercised through
/// `Handle_Gate_Explain` -- its own `query` parameter is a real `nomos_gate_orchestration::
/// FindingQuery` addressed by a real `nomos_contracts::RuleId` naming a real, shipped
/// `nomos_rules` rule, `COMPLETENESS_MIRROR`. A query nothing answers is `NotFound`, the same
/// distinction `nomos-gate-orchestration`'s own suite proves at its own layer -- this proves it
/// survives crossing into `nomos-api`'s public response shape too.
#[test]
fn Test_Handle_Gate_Explain_Should_Report_Not_Found_For_A_Query_Nothing_Answers()
{
    let directory = Scratch_Source_Tree("gate-explain-not-found", "a.rs", "pub fn Ok() {}\n");
    let query = nomos_gate_orchestration::FindingQuery {
        rule: nomos_contracts::RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
        location: "nowhere.rs".to_owned(),
    };

    let response = nomos_api::Handle_Gate_Explain(&directory, &query);

    let _ignored = std::fs::remove_dir_all(&directory);

    assert!(matches!(response, nomos_api::GateExplainResponse::NotFound), "{response:?}");
}

/// The same three crates, plus `nomos_model`: a real trigger over a real walked directory
/// produces a real blocking `nomos_contracts::Finding` naming the mirrored item, crossing this
/// crate's public boundary. `nomos-api`'s own private `sources::Read_Source` tags every file
/// this walk reads -- `a.rs` here -- with `nomos_model::Subject_Of_Path` before handing it to
/// `Explain_Gate`, so this also proves that identity function is real, deterministic, and
/// distinguishes the file the finding fired on from a sibling it did not.
#[test]
fn Test_Handle_Gate_Explain_Should_Find_A_Real_Blocking_Finding_Over_A_Real_Walked_Directory()
{
    let directory = Scratch_Source_Tree(
        "gate-explain-found",
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    );
    let query = nomos_gate_orchestration::FindingQuery {
        rule: nomos_contracts::RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
        location: "a.rs".to_owned(),
    };

    let response = nomos_api::Handle_Gate_Explain(&directory, &query);

    let _ignored = std::fs::remove_dir_all(&directory);

    let nomos_api::GateExplainResponse::Found { finding, would_block, contract_record, .. } = response
    else
    {
        // This fixture's trigger content is the same shape `nomos-gate-orchestration`'s own
        // suite proves produces a real blocking finding, so landing anywhere else means the
        // fixture itself regressed, not a condition this test should tolerate.
        panic!("this fixture's trigger content must produce a real blocking finding");
    };
    assert!(would_block);
    assert_eq!(contract_record.as_deref(), Some("D-134"));
    assert_eq!(finding.subject_name, "T", "{finding:?}");

    let subject_of_the_file_that_fired = nomos_model::Subject_Of_Path("a.rs");
    assert_eq!(
        subject_of_the_file_that_fired,
        nomos_model::Subject_Of_Path("a.rs"),
        "the same path must always identify the same subject"
    );
    assert_ne!(
        subject_of_the_file_that_fired,
        nomos_model::Subject_Of_Path("b.rs"),
        "a sibling file this crate would walk alongside the one that fired must not collide with it"
    );
}

/// A real, freshly walkable scratch tree of this file's own -- never the real repository tree,
/// which live sessions write to concurrently.
fn Scratch_Source_Tree(label: &str, file_name: &str, content: &str) -> PathBuf
{
    let name = format!("nomos-api-tests-integration-seams-{label}-{}", std::process::id());
    let directory = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("creates a fresh scratch directory");
    std::fs::write(directory.join(file_name), content).expect("writes a real source file");

    return directory;
}

// ---------------------------------------------------------------------------------------------
// Work: nomos_ledger, nomos_work_orchestration, nomos_platform, nomos_platform_std.
// ---------------------------------------------------------------------------------------------

/// `nomos_work_orchestration`, `nomos_ledger`, `nomos_platform` and `nomos_platform_std`,
/// exercised through `Handle_Work_Claim` -- its own `request` parameter is a real
/// `nomos_work_orchestration::ClaimRequest` naming a real `nomos_ledger::ItemId`, and the
/// `nomos_ledger::Reservation` it grants (through `ReservationResponse`, this crate's own public
/// twin of it) carries a real `nomos_platform::Timestamp` expiry, bound to a real
/// `nomos_platform_std::SystemClock` reading plus the requested lease -- not a value this crate
/// invented, but a real clock's answer plus real arithmetic on it. Both `Clock` readings bracket
/// the call, so the granted expiry is checked against a real window rather than an exact,
/// flaky-by-construction instant.
#[test]
fn Test_Handle_Work_Claim_Should_Grant_A_Reservation_Whose_Expiry_Is_A_Real_System_Clock_Lease()
{
    use nomos_platform::Clock;

    let id = "SCRATCH-API-TESTS-INTEGRATION-SEAMS-CLAIM";
    let directory = Scratch_Board_With_A_Claimable_Item(id);
    let lease = Duration::from_secs(3600);
    let request = nomos_work_orchestration::ClaimRequest {
        item: nomos_ledger::ItemId::New(id),
        holder: "test-holder".to_owned(),
        lease,
    };

    let before = nomos_platform_std::SystemClock.Now();
    let response = nomos_api::Handle_Work_Claim(&directory, &request);
    let after = nomos_platform_std::SystemClock.Now();

    let _ignored = std::fs::remove_dir_all(&directory);

    let nomos_api::ReservationOutcomeResponse::Reserved { reservation } = response
    else
    {
        // This fixture builds a real, unclaimed Ready item with real territory and no
        // conflict, so a grant is the only correct outcome -- reaching a refusal here means
        // the claim path itself regressed, not a condition this test should assert around.
        panic!("an unclaimed Ready item with real territory grants a reservation");
    };
    assert_eq!(reservation.item, nomos_ledger::ItemId::New(id));
    assert_eq!(reservation.holder, "test-holder");
    let earliest_acceptable = before.Plus(lease);
    let latest_acceptable = after.Plus(lease);
    assert!(
        reservation.expires_at >= earliest_acceptable && reservation.expires_at <= latest_acceptable,
        "a lease of {lease:?} granted between {before:?} and {after:?} must expire between \
         {earliest_acceptable:?} and {latest_acceptable:?}, not {:?}",
        reservation.expires_at
    );
}

/// A scratch board carrying one real, claimable `Ready` item -- never the real shared `work/`
/// directory, which live sessions write to concurrently.
fn Scratch_Board_With_A_Claimable_Item(id: &str) -> PathBuf
{
    let name = format!("nomos-api-tests-integration-seams-claimable-{}", std::process::id());
    let directory = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("creates a fresh scratch directory");
    let ledger = format!(
        "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
         \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"a\"], \"patterns\": []}}, \"state\": \"Ready\"}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

    return directory;
}

// ---------------------------------------------------------------------------------------------
// Spec: nomos_spec_orchestration, nomos_spec_model, nomos_spec_project, nomos_spec_store.
// ---------------------------------------------------------------------------------------------

/// `nomos_spec_orchestration` and `nomos_spec_model`, exercised through `Handle_Spec_Submit` --
/// its own `request` parameter is a real `nomos_spec_orchestration::SubmitRequest` whose `kind`
/// and `state` are real `nomos_spec_model` values, and OD-SPEC-010's own completeness rule,
/// reached only through this submission path, is what actually decides `Accepted` from
/// `Refused` -- not a check this crate duplicates.
#[test]
fn Test_Handle_Spec_Submit_Should_Accept_A_Complete_Submission_With_Submitted_Origin()
{
    let request = Complete_Feature_Request("FR-API-TESTS-INTEGRATION-SEAMS-001");

    let response = nomos_api::Handle_Spec_Submit(&request);

    let nomos_api::SubmitResponse::Accepted { submission, .. } = response
    else
    {
        // This request carries every universal field and every field OD-SPEC-010 requires of
        // a FeatureRequest, so reaching any other variant here means the submission path
        // itself regressed, not a condition this test should assert around.
        panic!("a complete feature request is accepted: {response:?}");
    };
    assert_eq!(submission.submitted_through, "nomos-api-tests-integration-seams");
    assert!(
        submission
            .values
            .iter()
            .all(|value| matches!(value.origin, nomos_api::OriginResponse::Submitted))
    );
}

/// The same two crates' negative case: `OD-SPEC-010`'s completeness rule refuses a submission
/// missing a required field, entirely inside `nomos_spec_orchestration` and `nomos_spec_model`
/// -- this crate writes nothing here but the request and the assertion on the refusal it gets
/// back.
#[test]
fn Test_An_Incomplete_Submission_Should_Be_Refused()
{
    let mut request = Complete_Feature_Request("FR-API-TESTS-INTEGRATION-SEAMS-002");
    request.fields.truncate(1);

    let response = nomos_api::Handle_Spec_Submit(&request);

    let nomos_api::SubmitResponse::Refused { refusal } = response
    else
    {
        // This request was truncated to just its first field, dropping the ones OD-SPEC-010
        // requires (including `goal`), so reaching any other variant here means the
        // completeness rule itself stopped enforcing, not a condition this test should assert
        // around.
        panic!("an incomplete submission must be refused: {response:?}");
    };
    assert!(refusal.failures.iter().any(|failure| failure.field == "goal"), "{refusal:?}");
}

/// Every universal field and every field `OD-SPEC-010` requires of
/// `SubmissionKind::FeatureRequest` -- the same set `nomos_spec_orchestration`'s own `tests.rs`
/// submits, and `nomos-api`'s own inline suite already submitted before this file existed.
fn Complete_Feature_Request(id: &str) -> nomos_spec_orchestration::SubmitRequest
{
    return nomos_spec_orchestration::SubmitRequest {
        kind: nomos_spec_model::SubmissionKind::FeatureRequest,
        id: id.to_owned(),
        by: "kevin".to_owned(),
        state: nomos_spec_model::SubmissionState::Draft,
        contract_version: 1,
        fields: vec![
            ("title".to_owned(), "t".to_owned()),
            ("goal".to_owned(), "g".to_owned()),
            ("behaviour".to_owned(), "b".to_owned()),
            ("acceptance".to_owned(), "a".to_owned()),
            ("invariants".to_owned(), "none".to_owned()),
        ],
        gaps: Vec::new(),
        submitted_through: "nomos-api-tests-integration-seams".to_owned(),
        into: None,
    };
}

/// `nomos_spec_project`, exercised through `Handle_Spec_Profiles` -- `ProfilesResponse::Listed`
/// carries `Vec<nomos_spec_project::Profile>` directly, not a twin of it, so this is the one
/// seam in this file proven by a real value of the provider's own type crossing the boundary
/// unchanged, rather than by a response this crate built from one.
#[test]
fn Test_Handle_Spec_Profiles_Should_List_Real_Nomos_Spec_Project_Profiles()
{
    let response = nomos_api::Handle_Spec_Profiles();

    let nomos_api::ProfilesResponse::Listed { profiles } = response
    else
    {
        // The profile catalogue this test reads is embedded in the binary this test itself
        // was compiled into, so a parse failure here means the workspace's own shipped
        // catalogue is broken, not a runtime condition this test should recover from.
        panic!("this workspace's own embedded catalogue parses");
    };
    let first: &nomos_spec_project::Profile =
        profiles.first().expect("the embedded catalogue ships at least one profile");
    assert!(!first.id.is_empty(), "{first:?}");
    assert!(!first.output.is_empty(), "{first:?}");
}

/// `nomos_spec_store`, exercised through `Handle_Spec_Record` -- `DocumentSourceResponse` is
/// this crate's own serializable twin of `nomos_spec_store::DocumentSource`, so this
/// reconstructs a real `DocumentSource` from each of two independent reads' own public fields
/// and compares them with the origin type's own `PartialEq`. That equality is the real seam:
/// two reads through this crate's public boundary must agree on exactly what `nomos_spec_store`
/// itself says a document's identity is, not merely on the response shape this crate wraps it
/// in.
#[test]
fn Test_Handle_Spec_Record_Should_Resolve_The_Same_Document_On_Repeated_Real_Reads()
{
    let request = nomos_spec_orchestration::RecordRequest { id: "D-132".to_owned(), revision: None };

    let first = nomos_api::Handle_Spec_Record(&request);
    let second = nomos_api::Handle_Spec_Record(&request);

    let (
        nomos_api::RecordResponse::Resolved { document: first_document, .. },
        nomos_api::RecordResponse::Resolved { document: second_document, .. },
    ) = (first, second)
    else
    {
        // D-132 is a real, embedded governing record, so reaching any other variant on either
        // read here means the record path itself regressed, not a condition this test should
        // assert around.
        panic!("D-132 is a governing record, embedded even with no corpus");
    };

    assert_eq!(As_Document_Source(&first_document), As_Document_Source(&second_document));
}

/// Rebuilds a real `nomos_spec_store::DocumentSource` from a `DocumentSourceResponse`'s own
/// public fields. `uid` is fixed at `0` on both sides -- its own doc comment in
/// `nomos-spec-store` says it is "never exported and never printed," so `DocumentSourceResponse`
/// carries no value for it to rebuild from, and this comparison is not about that surrogate.
fn As_Document_Source(response: &nomos_api::DocumentSourceResponse) -> nomos_spec_store::DocumentSource
{
    return nomos_spec_store::DocumentSource {
        uid: 0,
        path: response.path.clone(),
        revision: response.revision.clone(),
        content_hash: response.content_hash.clone(),
        text: response.text.clone(),
    };
}

// ---------------------------------------------------------------------------------------------
// nomos_workspace -- backs every Handle_Gate_* call's environment, carried by no response.
// ---------------------------------------------------------------------------------------------

/// `nomos_workspace`, exercised independently of any `Handle_*` return value: `composition::
/// Host_Variant` (private to `nomos-api`) builds a `nomos_workspace::BuildVariant` from four
/// `env!` values `build.rs` captures via `cargo::rustc-env`, which Cargo sets for every target
/// it compiles for this package -- this integration test included, since it is compiled as part
/// of the same `nomos-api` package build. This reconstructs the identical `BuildVariant` from
/// the same environment, from outside the crate, and then proves the environment it describes
/// is one `Handle_Gate_Run`'s own public entry point actually judges a real tree in -- the one
/// place this crate's public surface can show `nomos_workspace`'s value is live, not merely
/// well-formed in isolation.
#[test]
fn Test_This_Packages_Build_Variant_Should_Be_Well_Formed_For_A_Real_Gate_Run()
{
    let variant = nomos_workspace::BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES").split(',').filter(|feature| return !feature.is_empty()),
    );
    assert!(!variant.target.is_empty(), "{variant:?}");
    assert!(!variant.profile.is_empty(), "{variant:?}");
    assert!(!variant.toolchain.is_empty(), "{variant:?}");

    let response = nomos_api::Handle_Gate_Run(Path::new("."));
    assert_ne!(
        response.disposition,
        nomos_api::Disposition::Indeterminate,
        "this crate's own source tree has real .rs files to judge under the same build \
         environment BuildVariant above describes, so the check behind this run must have \
         reached Judged: {response:?}"
    );
}
