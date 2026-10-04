//! `nomos work show --item <text>` naming an item by the start of its id, and `claim` refusing to.

use super::super::{
    ClaimRequest, ExitCode, ItemId, LedgerDocument, Render_Show, Run, ShowView, Territory, WorkCommand,
};
use super::listing::{LISTING_NOW, Listing_Item};
use nomos_platform::Timestamp;

/// What `show --item <text>` prints on a board of the items `ids` name, and its exit code.
fn Shown_On(ids: &[&str], text: &str) -> (String, ExitCode)
{
    let view = ShowView {
        document: LedgerDocument {
            schema_version: nomos_ledger::SCHEMA_VERSION,
            items: ids.iter().map(|id| return Listing_Item(id)).collect(),
        },
        now: Timestamp::From_Unix_Seconds(LISTING_NOW),
        current_revision: None,
    };
    let mut output = Vec::new();

    let code = Render_Show(&ItemId::New(text), Ok(view), &mut output);

    return (String::from_utf8(output).expect("Render_Show writes only str into the buffer"), code);
}

/// The id `show` printed on its first line, which is the item it resolved the text to.
fn First_Id(shown: &str) -> &str
{
    return shown.split_whitespace().next().unwrap_or_default();
}

/// The hyphen is the boundary: `P19` names `P19-…` and never `P190-…`, however few items begin
/// with it.
#[test]
fn Test_Show_Should_Never_Resolve_A_Number_To_A_Longer_Number()
{
    let (beside, code) = Shown_On(&["P190-LONGER", "P19-NAMED"], "P19");
    assert_eq!(code, ExitCode::Ok, "{beside}");
    assert_eq!(First_Id(&beside), "P19-NAMED", "{beside}");

    let (alone, code) = Shown_On(&["P190-LONGER"], "P19");
    assert_eq!(code, ExitCode::Conflict, "P19 begins no id up to a hyphen:\n{alone}");
    assert!(alone.contains("no item named P19"), "{alone}");
}

/// The start of exactly one id, up to a hyphen, shows that item.
#[test]
fn Test_Show_Should_Resolve_The_Start_Of_Exactly_One_Id()
{
    let (shown, code) = Shown_On(&["P7-A-LONG-TITLE", "P8-ANOTHER"], "P7");

    assert_eq!(code, ExitCode::Ok, "{shown}");
    assert_eq!(First_Id(&shown), "P7-A-LONG-TITLE", "{shown}");
}

/// An exact id wins outright, even when it also begins a longer id.
#[test]
fn Test_Show_Should_Show_An_Exact_Id_That_Also_Begins_A_Longer_One()
{
    let (shown, code) = Shown_On(&["T-1-LONGER", "T-1"], "T-1");

    assert_eq!(code, ExitCode::Ok, "{shown}");
    assert_eq!(First_Id(&shown), "T-1", "{shown}");
}

/// Several matches are all named and none is chosen.
#[test]
fn Test_Show_Should_Name_Every_Id_A_Text_Begins_And_Show_None()
{
    let (shown, code) = Shown_On(&["P1-MODEL", "P1-GATE", "P2-OTHER"], "P1");

    assert_eq!(code, ExitCode::Usage, "{shown}");
    assert!(shown.contains("  P1-MODEL\n") && shown.contains("  P1-GATE\n"), "{shown}");
    assert!(!shown.contains("P2-OTHER"), "{shown}");
    assert!(!shown.contains("state:"), "no item was shown:\n{shown}");
}

/// `claim` takes the whole id. Through the real composition root and a real ledger, the start of
/// an id that `show` would resolve claims nothing and leaves the file as it was; the whole id is
/// the control that does claim.
#[test]
fn Test_Claim_Should_Refuse_The_Start_Of_An_Id_That_Show_Resolves()
{
    let directory = std::env::temp_dir().join(format!("nomos-work-claim-prefix-{}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    std::fs::write(
        directory.join("ledger.json"),
        format!("{{\"schema_version\":{},\"items\":[]}}", nomos_ledger::SCHEMA_VERSION),
    )
    .expect("a scratch ledger");
    let mut item = Listing_Item("T-1-LONGER");
    item.territory = Territory::Of_Files(["src/claimed.rs"]);
    let added = Run(&WorkCommand::Add { item: Box::new(item), amending: Territory::Empty() }, &directory, &mut Vec::new());
    let before = std::fs::read(directory.join("ledger.json")).expect("the ledger was written");

    let by_prefix = Run(&WorkCommand::Claim(Claim_Of("T-1")), &directory, &mut Vec::new());
    let after_refusal = std::fs::read(directory.join("ledger.json")).expect("the ledger is readable");
    let by_id = Run(&WorkCommand::Claim(Claim_Of("T-1-LONGER")), &directory, &mut Vec::new());

    let _ignored = std::fs::remove_dir_all(&directory);
    assert_eq!(added, ExitCode::Ok);
    assert_ne!(by_prefix, ExitCode::Ok, "claim must not resolve the start of an id");
    assert_eq!(before, after_refusal, "a refused claim leaves the ledger as it was");
    assert_eq!(by_id, ExitCode::Ok, "the whole id is the control that does claim");
}

fn Claim_Of(text: &str) -> ClaimRequest
{
    return ClaimRequest {
        item: ItemId::New(text),
        holder: "agent-a".to_owned(),
        lease: std::time::Duration::from_secs(60),
    };
}
