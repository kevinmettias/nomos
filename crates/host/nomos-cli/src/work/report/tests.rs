//! What [`super`]'s report functions say, exercised.
//!
//! Split from `report.rs` itself once that file passed the ~500-line review trigger --
//! `report.rs` is the rendering logic, this is its own coverage, the same split this
//! workspace already keeps between `spec.rs` and `spec/tests.rs`.

use super::*;

/// The instant the fixture reports at: one second after the ending it describes, so the
/// report runs strictly later than the event it is about.
const REPORTED_AT_UNIX_SECONDS: i64 = 2;

/// When the fixture's verification ran, later still, so the record reads as a predicate
/// that had already finished by the time the ending is reported.
const VERIFIED_AT_UNIX_SECONDS: i64 = 5;

/// When the held claims in the fixtures expire. Nothing is compared against it -- the
/// assertions are about which word a refusal maps to -- so it only has to be far enough
/// out that the claim reads as live.
const HELD_UNTIL_UNIX_SECONDS: i64 = 10;

/// The expiry the successful claim's fixture is given, and therefore the one its line has
/// to print back; named so the two cannot drift apart.
const CLAIM_EXPIRES_AT_UNIX_SECONDS: i64 = 100;

/// `work audit` answers for items somebody could act on, and nobody can act on a declined
/// one.
///
/// `P10-AUDIT-STATE` settled that once: an audit that reported blockers for finished work
/// made forty-four lines nobody could do anything about. A newly reachable terminal state
/// is the obvious way to reopen it.
#[test]
fn Test_Blocking_Refusal_Should_Answer_Nothing_For_A_Declined_Item()
{
    let mut item = Item("T-1");
    item.Decline("superseded by T-2", "agent-a", Timestamp::From_Unix_Seconds(1));
    let document = LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items: vec![item],
    };
    let found = document.items.first().expect("the fixture has an item");

    assert!(
        Blocking_Refusal(&document, found, Timestamp::From_Unix_Seconds(REPORTED_AT_UNIX_SECONDS))
            .is_none(),
        "audit answered for an item nobody can act on"
    );
}

#[test]
fn Test_Amendment_Note_Should_Be_Empty_When_Nothing_Is_Amended()
{
    assert_eq!(Amendment_Note(&Territory::Of_Files(Vec::<String>::new())), "");
}

#[test]
fn Test_Amendment_Note_Should_Name_Every_Amended_Path()
{
    let amending = Territory::Of_Files(vec![
        "docs/records/A.md".to_owned(),
        "docs/records/B.md".to_owned(),
    ]);

    assert_eq!(
        Amendment_Note(&amending),
        ", amending docs/records/A.md, docs/records/B.md"
    );
}

#[test]
fn Test_Code_For_Refusal_Should_Map_Each_Refusal_To_Its_Own_Exit_Code()
{
    assert_eq!(
        Code_For_Refusal(&AddRefusal::AlreadyPresent { item: ItemId::New("T-1") }),
        ExitCode::Conflict,
        "a taken identifier is a conflict, not a retryable one"
    );
    assert_eq!(
        Code_For_Refusal(&AddRefusal::RecordPublished {
            identifier: "OD-X-001".to_owned(),
            file: "docs/records/OD-X-001-a.md".to_owned(),
        }),
        ExitCode::Conflict,
        "a record already spent is a conflict too"
    );
    assert_eq!(
        Code_For_Refusal(&AddRefusal::WouldBeInvalid {
            violations: vec!["reserves nothing".to_owned()],
        }),
        ExitCode::ValidationError,
        "an item's own declaration is the caller's to correct"
    );
    assert_eq!(
        Code_For_Refusal(&AddRefusal::LedgerUnusable { cause: "disk full".to_owned() }),
        ExitCode::StoreError,
        "a ledger that cannot be used at all stops an agent rather than sending it to retry"
    );
}

/// An ending with no board behind it, for the cases that assert the line the verb
/// always prints rather than the fanout that depends on one.
fn Ended_Without_A_Board(item: &ItemId) -> Ended<'_>
{
    return Ended { item, board: None, now: Timestamp::From_Unix_Seconds(0) };
}

#[test]
fn Test_Report_Finish_Should_Print_The_Verified_Command_On_Success()
{
    let mut output = Vec::new();

    let code = Report_Finish(
        &Ended_Without_A_Board(&ItemId::New("T-1")),
        Ok(nomos_ledger::VerificationRecord {
            argv: vec!["cargo".to_owned(), "test".to_owned()],
            exit_code: 0,
            output_tail: String::new(),
            verified_at: Timestamp::From_Unix_Seconds(VERIFIED_AT_UNIX_SECONDS),
            gate: None,
            revision: None,
        }),
        &mut output,
    );

    assert_eq!(code, ExitCode::Ok);
    assert!(
        String::from_utf8(output).unwrap().contains("verified by `cargo test`"),
        "success must name what was run"
    );
}

#[test]
fn Test_Report_Finish_Should_Report_A_Validation_Error_When_The_Predicate_Judged_The_Work()
{
    let mut output = Vec::new();

    let code = Report_Finish(
        &Ended_Without_A_Board(&ItemId::New("T-1")),
        Err(FinishRefusal::PredicateFailed {
            item: ItemId::New("T-1"),
            exit_code: 1,
            output_tail: "assertion failed".to_owned(),
        }),
        &mut output,
    );

    assert_eq!(
        code,
        ExitCode::ValidationError,
        "a predicate that ran and said no judged the work, not the tooling"
    );
    assert!(String::from_utf8(output).unwrap().contains("not finished"));
}

#[test]
fn Test_Refusal_Label_Should_Name_Each_Refusal_By_Its_Own_Word()
{
    assert_eq!(
        Refusal_Label(&ClaimRefusal::DependencyUnmet {
            item: ItemId::New("T-1"),
            dependency: ItemId::New("T-0"),
            state: "Ready".to_owned(),
        }),
        "waiting"
    );
    assert_eq!(
        Refusal_Label(&ClaimRefusal::DependencyDeclined {
            item: ItemId::New("T-1"),
            dependency: ItemId::New("T-0"),
            state: "Declined".to_owned(),
        }),
        "stranded"
    );
    assert_eq!(
        Refusal_Label(&ClaimRefusal::HeldBy {
            holder: "agent-a".to_owned(),
            until: Timestamp::From_Unix_Seconds(HELD_UNTIL_UNIX_SECONDS),
            item: ItemId::New("T-2"),
        }),
        "held"
    );
    assert_eq!(
        Refusal_Label(&ClaimRefusal::Lapsed {
            item: ItemId::New("T-1"),
            holder: "agent-a".to_owned(),
            since: Timestamp::From_Unix_Seconds(HELD_UNTIL_UNIX_SECONDS),
        }),
        "lapsed"
    );
    assert_eq!(
        Refusal_Label(&ClaimRefusal::NoSuchItem { item: ItemId::New("T-9") }),
        "snagged",
        "everything the four named words do not cover falls to the catch-all"
    );
}

#[test]
fn Test_Report_Claim_Should_Print_Who_Holds_It_Until_When_On_Success()
{
    let mut output = Vec::new();

    let code = Report_Claim(
        Ok(nomos_ledger::Reservation {
            item: ItemId::New("T-1"),
            holder: "agent-a".to_owned(),
            expires_at: Timestamp::From_Unix_Seconds(CLAIM_EXPIRES_AT_UNIX_SECONDS),
        }),
        &mut output,
    );

    assert_eq!(code, ExitCode::Ok);
    assert_eq!(
        String::from_utf8(output).unwrap(),
        format!("T-1 held by agent-a until unix {CLAIM_EXPIRES_AT_UNIX_SECONDS}\n")
    );
}

#[test]
fn Test_Report_Claim_Should_Report_A_Retryable_Refusal_As_Claim_Unavailable()
{
    let mut output = Vec::new();

    let code = Report_Claim(
        Err(ClaimRefusal::HeldBy {
            holder: "agent-a".to_owned(),
            until: Timestamp::From_Unix_Seconds(HELD_UNTIL_UNIX_SECONDS),
            item: ItemId::New("T-1"),
        }),
        &mut output,
    );

    assert_eq!(code, ExitCode::ClaimUnavailable);
    assert!(String::from_utf8(output).unwrap().starts_with("refused:"));
}

#[test]
fn Test_Report_Decline_Should_Name_The_Item_On_Success()
{
    let mut output = Vec::new();

    let code = Report_Decline(&Ended_Without_A_Board(&ItemId::New("T-1")), Ok(()), &mut output);

    assert_eq!(code, ExitCode::Ok);
    assert_eq!(String::from_utf8(output).unwrap(), "T-1 declined\n");
}

#[test]
fn Test_Report_Decline_Should_Report_A_Non_Retryable_Refusal_As_A_Conflict()
{
    let mut output = Vec::new();

    let code = Report_Decline(
        &Ended_Without_A_Board(&ItemId::New("T-1")),
        Err(ClaimRefusal::NotClaimable {
            item: ItemId::New("T-1"),
            state: "Done".to_owned(),
        }),
        &mut output,
    );

    assert_eq!(code, ExitCode::Conflict);
    assert!(String::from_utf8(output).unwrap().starts_with("refused:"));
}

#[test]
fn Test_Report_Release_Should_Say_Released_On_Success()
{
    let mut output = Vec::new();

    let code = Report_Release(Ok(()), &mut output);

    assert_eq!(code, ExitCode::Ok);
    assert_eq!(String::from_utf8(output).unwrap(), "released\n");
}

#[test]
fn Test_Report_Validation_Should_Print_Both_Schema_Versions_On_Success()
{
    let mut output = Vec::new();
    let document = LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items: Vec::new(),
    };

    let code = Report_Validation(Ok(document), &mut output);

    assert_eq!(code, ExitCode::Ok);
    assert!(String::from_utf8(output).unwrap().contains("ledger is valid"));
}

#[test]
fn Test_Report_Validation_Should_Report_A_Malformed_Ledger_As_A_Store_Error()
{
    let mut output = Vec::new();

    let code = Report_Validation(
        Err(LedgerError::Malformed { cause: "not json".to_owned() }),
        &mut output,
    );

    assert_eq!(code, ExitCode::StoreError);
}

#[test]
fn Test_Print_Blocked_Should_Print_The_Items_Identifier_Label_And_The_Refusals_Description()
{
    let item = Item("T-1");
    let refusal = ClaimRefusal::HeldBy {
        holder: "agent-a".to_owned(),
        until: Timestamp::From_Unix_Seconds(HELD_UNTIL_UNIX_SECONDS),
        item: ItemId::New("T-2"),
    };
    let mut output = Vec::new();

    Print_Blocked(&item, &refusal, &mut output);

    let printed = String::from_utf8(output)
        .expect("the reporter writes only bytes of Rust strings into this buffer, so it holds UTF-8");
    assert!(printed.contains("T-1"), "{printed}");
    assert!(printed.contains("held"), "{printed}");
    assert!(printed.contains(&refusal.Describe()), "{printed}");
}

#[test]
fn Test_Report_Error_Should_Map_An_Invalid_Ledger_To_A_Validation_Error()
{
    let mut output = Vec::new();

    let code = Report_Error(
        &LedgerError::Invalid { violations: vec!["dup".to_owned()] },
        &mut output,
    );

    assert_eq!(code, ExitCode::ValidationError);
    assert!(String::from_utf8(output).unwrap().contains("ledger is invalid"));
}

#[test]
fn Test_Report_Error_Should_Map_An_Unreadable_Ledger_To_A_Store_Error()
{
    let mut output = Vec::new();

    let code = Report_Error(
        &LedgerError::Unreadable { cause: "permission denied".to_owned() },
        &mut output,
    );

    assert_eq!(code, ExitCode::StoreError);
}

/// A minimal, ready item: enough to give [`Print_Blocked`] something to print.
/// A board where `T-2` depends on `T-1`, with `T-1` already ended the given way.
///
/// Both halves matter: a dependent that names the ended item is what the fanout is for,
/// and whether the ending freed or stranded it is exactly what the label has to say.
fn A_Board_Where_T2_Depends_On_T1(end: impl FnOnce(&mut LedgerItem)) -> LedgerDocument
{
    let mut ended = Item("T-1");
    end(&mut ended);
    let mut dependent = Item("T-2");
    dependent.depends_on = vec![ItemId::New("T-1")];

    return LedgerDocument { schema_version: SCHEMA_VERSION, items: vec![ended, dependent] };
}

/// A decline strands every dependent permanently, and the whole point of
/// `OD-LEDGER-038` is that the person who typed it finds out then rather than later.
#[test]
fn Test_Report_Decline_Should_Name_The_Dependent_It_Just_Stranded()
{
    let document = A_Board_Where_T2_Depends_On_T1(|item| {
        item.Decline("not work after all", "agent-a", Timestamp::From_Unix_Seconds(1));
    });
    let item = ItemId::New("T-1");
    let ended =
        Ended { item: &item, board: Some(&document), now: Timestamp::From_Unix_Seconds(REPORTED_AT_UNIX_SECONDS) };
    let mut output = Vec::new();

    let code = Report_Decline(&ended, Ok(()), &mut output);

    let printed = String::from_utf8(output).expect("the reporter writes only bytes of Rust strings into this buffer, so it holds UTF-8");
    assert_eq!(code, ExitCode::Ok);
    assert!(printed.contains("depending on T-1:"), "{printed}");
    // `stranded`, the word `list` already uses for a dependency that will never finish,
    // rather than a second vocabulary invented at the point of ending.
    assert!(printed.contains("stranded T-2"), "{printed}");
}

/// The same line for the ending that frees rather than strands: a dependent that was
/// waiting is claimable now, and saying so is what tells a reader what to pick up next.
#[test]
fn Test_Report_Finish_Should_Name_The_Dependent_It_Just_Freed()
{
    let document = A_Board_Where_T2_Depends_On_T1(|item| {
        item.state = ItemState::Done;
    });
    let item = ItemId::New("T-1");
    let ended =
        Ended { item: &item, board: Some(&document), now: Timestamp::From_Unix_Seconds(REPORTED_AT_UNIX_SECONDS) };
    let mut output = Vec::new();

    let code = Report_Finish(&ended, Ok(A_Verification_Record()), &mut output);

    let printed = String::from_utf8(output).expect("the reporter writes only bytes of Rust strings into this buffer, so it holds UTF-8");
    assert_eq!(code, ExitCode::Ok);
    assert!(printed.contains("depending on T-1:"), "{printed}");
    assert!(printed.contains("ready T-2"), "{printed}");
}

/// Most endings free nobody. A header over an empty list every time would teach a
/// reader to skip the line, which is the one place it has to be read.
#[test]
fn Test_An_Ending_Nothing_Depends_On_Should_Print_No_Fanout_At_All()
{
    let document = LedgerDocument { schema_version: SCHEMA_VERSION, items: vec![Item("T-1")] };
    let item = ItemId::New("T-1");
    let ended =
        Ended { item: &item, board: Some(&document), now: Timestamp::From_Unix_Seconds(REPORTED_AT_UNIX_SECONDS) };
    let mut output = Vec::new();

    let code = Report_Decline(&ended, Ok(()), &mut output);

    assert_eq!(code, ExitCode::Ok);
    assert_eq!(String::from_utf8(output).expect("the reporter writes only bytes of Rust strings into this buffer, so it holds UTF-8"), "T-1 declined
");
}

/// A verification record, for the cases that care about what was printed after it.
fn A_Verification_Record() -> nomos_ledger::VerificationRecord
{
    return nomos_ledger::VerificationRecord {
        argv: vec!["cargo".to_owned(), "test".to_owned()],
        exit_code: 0,
        output_tail: String::new(),
        verified_at: Timestamp::From_Unix_Seconds(VERIFIED_AT_UNIX_SECONDS),
        gate: None,
        revision: None,
    };
}

fn Item(id: &str) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: format!("item {id}"),
        why: "because".to_owned(),
        done_when: "it prints".to_owned(),
        kind: nomos_ledger::ItemKind::Correction,
        origin: nomos_ledger::ItemOrigin::Proposed,
        territory: Territory::Of_Files(vec!["src/a.rs".to_owned()]),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
        declined: None,
    };
}
