//! The ledger finds a refused gate step's Blocking findings by a label it cannot name.
//!
//! A finish refused by the gate's `Rules` step leads its refusal with every line of the report
//! labelled Blocking, and [`nomos_ledger::Is_Labelled_Blocking`] is what decides which lines
//! those are. It compares against a copy of the label, because `nomos-ledger` does not depend
//! on `nomos-contracts`, which renders it: `GateCategory::Label` spells it, `Finding::Describe`
//! puts it in brackets at the head of a finding's line, and `nomos gate run` prints that line
//! as it is. Were either side's spelling to move, no line would be lifted, the refusal would
//! fall back to its old tail, and the ledger's own tests, whose fixture is spelled the way the
//! label is today, would stay green.
//!
//! This crate can name both, so the two are held together here. The label is not typed in this
//! file, which would be a third copy: a real finding is rendered and the ledger is asked about
//! the line it renders.

use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// Every gate category but Blocking. No finding in one of these refused a run, so none of
/// their lines is lifted.
const CATEGORIES_NOT_LIFTED: [GateCategory; 3] = [GateCategory::Review, GateCategory::Unreachable, GateCategory::Advisory];

/// The byte the fixture's subject digest is filled with. Any one byte would do; the subject
/// plays no part in the line the label heads.
const SUBJECT_DIGEST_BYTE: u8 = 7;

/// The line `nomos gate run`'s report holds for a finding in `gate`, rendered by the function
/// the report renders it with.
fn Report_Line(gate: GateCategory) -> String
{
    let finding = Finding {
        rule: RuleId::New("a-fixture-rule"),
        subject: SubjectId::From_Digest(Digest128::From_Bytes([SUBJECT_DIGEST_BYTE; Digest128::BYTE_LENGTH])),
        subject_name: "src/lib.rs:12".to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate,
        summary: "a fixture finding's summary".to_owned(),
        locations: vec!["src/lib.rs".to_owned()],
        address: None,
    };

    return finding.Describe();
}

/// A Blocking finding's line, as `nomos-contracts` renders it, is one the ledger lifts. A
/// change to the label's spelling, to the brackets around it, or to the ledger's copy of
/// either fails this.
#[test]
fn Test_The_Ledger_Should_Lift_The_Line_A_Blocking_Finding_Renders()
{
    let line = Report_Line(GateCategory::Blocking);

    assert!(
        nomos_ledger::Is_Labelled_Blocking(&line),
        "nomos-ledger does not take the line nomos-contracts renders for a Blocking finding to be \
         labelled Blocking, so a finish refused by such a finding would lift nothing: {line}"
    );
}

/// The ledger's copy is the Blocking label and not merely something every finding line
/// begins with: no other category's line is lifted.
#[test]
fn Test_The_Ledger_Should_Lift_No_Line_Another_Gate_Category_Renders()
{
    for category in CATEGORIES_NOT_LIFTED
    {
        let line = Report_Line(category);

        assert!(
            !nomos_ledger::Is_Labelled_Blocking(&line),
            "nomos-ledger takes the line rendered for a {category} finding to be labelled Blocking: {line}"
        );
    }
}
