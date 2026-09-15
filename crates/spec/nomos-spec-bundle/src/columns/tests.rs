//! What the column declaration promises, exercised.

use super::*;

fn Schema(columns: &[&str]) -> Vec<String>
{
    return columns.iter().map(|column| return (*column).to_owned()).collect();
}

#[test]
fn Test_A_Fully_Declared_Table_Should_Pass()
{
    let declared = &[("uid", Carried::Surrogate), ("path", Carried::Field("path"))];

    assert!(Compare_Schema_To_Declaration("t", &Schema(&["uid", "path"]), declared).is_ok());
}

/// The failure the guard exists for: a column joins the schema and nothing carries it.
#[test]
fn Test_An_Undeclared_Column_Should_Be_Refused()
{
    let declared = &[("uid", Carried::Surrogate)];

    let refusal = Compare_Schema_To_Declaration("t", &Schema(&["uid", "note"]), declared)
        .expect_err("an undeclared column must be refused");

    assert!(
        matches!(refusal, BundleError::UncoveredColumn { ref column, .. } if column == "note"),
        "{refusal}"
    );
}

/// A declaration that stopped describing anything would otherwise go on passing.
#[test]
fn Test_A_Declaration_The_Schema_Dropped_Should_Be_Refused()
{
    let declared = &[("uid", Carried::Surrogate), ("gone", Carried::Field("gone"))];

    let refusal = Compare_Schema_To_Declaration("t", &Schema(&["uid"]), declared)
        .expect_err("a stale declaration must be refused");

    assert!(
        matches!(refusal, BundleError::PhantomColumn { ref column, .. } if column == "gone"),
        "{refusal}"
    );
}

/// The length of the two-byte record this fixture declares, which no column in the schema
/// holds — the point of the case is the field's NAME, not its value.
const FIXTURE_BYTE_LENGTH: i64 = 2;

#[test]
fn Test_A_Declared_Field_The_Record_Lacks_Should_Be_Refused()
{
    let record = Record::Blob(crate::row::blob::Blob {
        sha256: "sha256:aa".to_owned(),
        byte_length: FIXTURE_BYTE_LENGTH,
        encoding: crate::row::blob::encoding::Encoding::Utf8,
        content: "hi".to_owned(),
    });
    let declared = &[("sha256", Carried::Field("digest"))];

    let refusal = Assert_Fields_Exist("blobs", declared, std::slice::from_ref(&record))
        .expect_err("a field the record does not have must be refused");

    assert!(
        matches!(refusal, BundleError::Uncarried { ref field, .. } if field == "digest"),
        "{refusal}"
    );
    assert!(
        Assert_Fields_Exist("blobs", &[("sha256", Carried::Field("sha256"))], &[record])
            .is_ok(),
        "the real field must satisfy it, or the check proves nothing"
    );
}

/// Every table the store knows about declares its columns somewhere.
#[test]
fn Test_Every_Table_Should_Have_A_Coverage_Entry()
{
    for table in Table::All()
    {
        assert!(
            COVERAGE.iter().any(|coverage| coverage.table == table.Name()),
            "{} declares no columns, so every one of them is uncovered",
            table.Name()
        );
    }
}
