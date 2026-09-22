//! Every `Check_*` function `nomos-rules` exports is either composed into a real run, or
//! carries a stated reason it is not — never an unexamined gap nobody looked at since the
//! last time somebody happened to diff the two lists by hand.
//!
//! Building a rule first and composing it in a later item is a deliberate, sensible
//! pattern here: composing an unmeasured rule can redden the gate by hundreds of findings,
//! and this workspace has done exactly that on purpose before (`OD-RULES-012-WIRE-CLEAN-
//! TEXT-RULES`, `P27-RULES-TWO-MORE-UNCOMPOSED`). The defect this test exists to close is
//! not the pattern — it is that the gap was found by hand, each time, by whoever happened
//! to diff the two lists. `P46-UNCOMPOSED-RULES-ARE-COUNTED` measured it once, on
//! 2026-09-05: seventy-five declared, fifty-six composed, nineteen unaccounted for. This
//! test is the guard that makes that measurement stay current on its own.
//!
//! # Why this reads two files' own text rather than calling a shared function
//!
//! `nomos_rules::DESCRIPTORS` already answers a narrower, different question —
//! `tests/contract/tests/rule_descriptors.rs` already checks it against
//! `nomos_check_orchestration::Composed_Rules()` — but `DESCRIPTORS` is populated only for
//! rules already composed; a rule this crate exports and nobody has composed yet has no
//! descriptor and is invisible to that comparison by construction. There is no reflection
//! in Rust and no third list declaring "every `Check_*` function this crate has," so this
//! test reads the two texts that already are that list, honestly, the same way
//! `tests/contract/tests/boundaries/readme.rs` reads `README.md`'s own text rather than a
//! restated copy of it:
//!
//! - `tests/contract/surface/nomos-rules.txt`, the committed, blessed public-surface
//!   snapshot, for every `Check_*` function this crate exports today.
//! - `crates/rules/nomos-rules/src/rule_descriptor.rs`'s own source, for every `Check_*`
//!   function the `DESCRIPTORS` table actually names.
//!
//! That second file used to be `nomos-check-orchestration`'s `run_context.rs`, anchored on
//! `fn With_Composed_Rules`, because that is where the seventy composed closures were
//! written. `OD-RULES-027` moved each rule's judgment into its own descriptor and the
//! composition root now derives its run from the table, so the function this reader
//! anchored on no longer exists and the composed set is declared one crate lower. The
//! reading is otherwise unchanged, and deliberately still textual: a descriptor carries its
//! check as a `fn` pointer, which has no name at run time, so `DESCRIPTORS` itself cannot
//! answer *which* function is composed -- only its own source text can.
//!
//! `OD-RULES-034` added a second way for a row to name no function at all: a rule of the
//! declared archetype carries a `DeclaredTextRule` literal instead of a judgment pointer, so
//! its rule is composed while the function it was written as reads here as uncomposed. That
//! is not a false reading and the fix is not to teach this reader about the form -- the
//! function really is out of the run now, and it stays exported because it is the baseline
//! that record's falsifier compares the declaration against. Each one says so below.
//!
//! A function added to the surface and never composed falls into the difference; a
//! function in the difference with no entry in [`ACCOUNTED_FOR`] fails this test.

use std::collections::BTreeSet;
use std::path::Path;

/// Every currently-uncomposed `Check_*` function, and the real, checked reason it stays
/// that way. A function removed from here without also being composed makes this test fail
/// for the honest reason `done_when` names: a rule exported tomorrow without either falls
/// out of the test.
///
/// Reasons are grouped by kind, not alphabetically, so the shape of the population reads
/// rather than just its membership.
const ACCOUNTED_FOR: &[(&str, &str)] = &[
    // Measured directly against this workspace's own real tree (a temporary local
    // composition, never committed) before this table was written. Each would add real,
    // non-trivial blocking-finding volume the same way OD-RULES-012's own three exclusions
    // already do -- composing any of these four is real, separate work that has to look at
    // every finding it would add, not a table-row edit.
    (
        "Check_Constants_Are_The_Exception_To_Function_Scope_Use",
        "65 blocking findings measured against this workspace's own tree; each would need review before composing.",
    ),
    (
        "Check_Naming_Clarity",
        "63 blocking findings measured against this workspace's own tree; each would need review before composing.",
    ),
    (
        "Check_One_Public_Type_Per_File",
        "67 blocking findings measured against this workspace's own tree; each would need review before composing.",
    ),
    (
        "Check_File_Size_Review_Trigger",
        "42 blocking findings measured against this workspace's own tree; each would need review before composing.",
    ),
    // run_context.rs's own module doc, above `Rule_Findings`, already names these three as
    // deliberately excluded for the identical reason -- reproduced here as a citation, not
    // re-measured.
    (
        "Check_Boolean_Predicates",
        "run_context.rs's own doc: composing would add 45 findings against this workspace's own tree, named there as a later, separate change.",
    ),
    (
        "Check_No_Decorative_Section_Dividers",
        "run_context.rs's own doc: composing would add 71 findings against this workspace's own tree, named there as a later, separate change.",
    ),
    (
        "Check_Test_Names_Describe_Behavior",
        "run_context.rs's own doc: composing would add 217 findings against this workspace's own tree, named there as a later, separate change.",
    ),
    (
        "Check_Panics_Are_Justified_Documented_And_Validated",
        "run_context.rs's own doc names wiring a rule this tree fails as a different, larger change than composing one it already satisfies.",
    ),
    (
        "Check_Unwrap_Expect_Discipline",
        "run_context.rs's own doc names wiring a rule this tree fails as a different, larger change than composing one it already satisfies.",
    ),
    // The three facade rules: composing surfaced 59 "unexplained" aliases that are this
    // workspace's own settled `pub use id::Id as EntityId;` idiom -- a real disagreement
    // between two standards, not a defect in either -- so facade.rs's own module doc
    // leaves composition to a later increment that can weigh it.
    (
        "Check_A_Consumer_Imports_Through_The_Facade",
        "facade.rs's own doc: composing surfaced 59 real, settled re-export aliases this rule cannot yet tell apart from an unexplained one.",
    ),
    (
        "Check_A_Facade_Publishes_A_Child_One_Way",
        "facade.rs's own doc leaves composition of all three facade rules to a later increment that can weigh the alias disagreement together.",
    ),
    (
        "Check_A_Renamed_Facade_Re_Export_Names_The_Contract",
        "facade.rs's own doc leaves composition of all three facade rules to a later increment that can weigh the alias disagreement together.",
    ),
    // A named, already-decided architectural reason each, not a volume problem.
    (
        "Check_Declared_Role_Matches_Surface",
        "Additive and unwired by design: its own subject is Agent-required and it names no runtable case Run's own composition covers yet (OD-RULES-009, OD-GATE-017, OD-CORRECTIONS-001, OD-EXECUTOR-003).",
    ),
    (
        "Check_Domain_Values_Are_Distinct_Types",
        "OD-CAPABILITY-011 just decided the syntax payload's stringly-typed shape field changes; composing a rule that judges that exact field ahead of the change it names would judge a shape about to move.",
    ),
    (
        "Check_Function_Arity_Policy",
        "Not a standalone rule: the shared, configurable engine PARAMETER_COUNT and GO_HELPERS_PACKAGE_FIVE_INPUTS already call with their own policy. It has no rule id of its own; composing it directly would compose an already-composed check a third time.",
    ),
    (
        "Check_Project_Owned_Function_Names_Use_Upper_Snake_Case",
        "The identical judgment as the already-composed naming-convention, relabeled under the code-standards rule id for numbering compatibility. Composing both would double-report every naming violation under two ids at once.",
    ),
    // The three rules OD-RULES-034's declared form replaced. Each rule is composed -- its
    // DESCRIPTORS row is a declaration now rather than a function pointer, which is why this
    // reader, which reads the table's own source text for `Check_*` names, no longer sees
    // one. The function stays exported because it is the baseline the record's own falsifier
    // compares against, byte for byte, and a baseline that had been deleted would leave the
    // comparison with nothing on the other side.
    (
        "Check_Every_Allow_Carries_A_Justification",
        "every-allow-carries-a-justification is composed, as a declaration: OD-RULES-034's form carries the row and checks::rust_text::declarations' own acceptance test compares this function's findings against the declaration's, byte for byte, over this tree.",
    ),
    (
        "Check_Inline_Always_Justification",
        "inline-always-requires-justification is composed, as a declaration, for the identical reason: this function is the byte-identical baseline that comparison reads.",
    ),
    (
        "Check_A_Disabled_Test_States_Why",
        "a-disabled-test-states-why is composed, as a declaration, for the identical reason: this function is the byte-identical baseline that comparison reads.",
    ),
];

#[test]
fn Test_Every_Exported_Rule_Is_Composed_Or_Accounted_For()
{
    let root = nomos_contract_tests::Workspace::Workspace_Root();
    let exported = Exported_Check_Functions(&root);
    let composed = Composed_Check_Functions(&root);

    assert!(!exported.is_empty(), "no Check_ function was read from the surface snapshot; the reader found nothing to compare");
    assert!(!composed.is_empty(), "no Check_ function was read from run_context.rs; the reader found nothing to compare");

    let accounted: BTreeSet<&str> = ACCOUNTED_FOR.iter().map(|(name, _)| return *name).collect();
    assert_eq!(
        accounted.len(),
        ACCOUNTED_FOR.len(),
        "ACCOUNTED_FOR names one function twice; a duplicated reason hides whether the real function is covered"
    );

    let uncomposed: BTreeSet<&str> = exported.difference(&composed).map(String::as_str).collect();

    let unaccounted: Vec<&str> = uncomposed.difference(&accounted).copied().collect();
    assert!(
        unaccounted.is_empty(),
        "these exported rules are composed into no real run and carry no stated reason in ACCOUNTED_FOR: {unaccounted:#?}.\n\
         Either compose the rule (checked first against this workspace's own real tree, per this crate's own established discipline), \
         or add a row here naming a real, checked reason."
    );

    let stale: Vec<&str> = accounted.difference(&uncomposed).copied().collect();
    assert!(
        stale.is_empty(),
        "ACCOUNTED_FOR names a reason for a rule that is no longer in the gap -- either it is composed now (drop the row) or it is no \
         longer exported (drop the row): {stale:#?}"
    );
}

/// Every `Check_*` function named in the committed, blessed public-surface snapshot.
fn Exported_Check_Functions(root: &Path) -> BTreeSet<String>
{
    let text = std::fs::read_to_string(root.join("tests/contract/surface/nomos-rules.txt"))
        .expect("the committed nomos-rules surface snapshot must read");

    return text
        .lines()
        .filter_map(|line| return line.strip_prefix("pub fn nomos_rules::"))
        .filter_map(Check_Function_Name)
        .collect();
}

/// Every `Check_*` function the `DESCRIPTORS` table names, read from that table's own
/// source text rather than from a second, hand-maintained list.
fn Composed_Check_Functions(root: &Path) -> BTreeSet<String>
{
    let text = std::fs::read_to_string(root.join("crates/rules/nomos-rules/src/rule_descriptor.rs"))
        .expect("rule_descriptor.rs must read");

    let Some(start) = text.find("pub const DESCRIPTORS")
    else
    {
        panic!("rule_descriptor.rs no longer declares DESCRIPTORS; this reader's own anchor moved");
    };
    let body = text.get(start..).unwrap_or_default();
    let Some(end) = body.find("\n];\n")
    else
    {
        panic!("the DESCRIPTORS table's own closing bracket was not found the way this reader expects");
    };
    let body = body.get(..end).unwrap_or_default();

    let mut found = BTreeSet::new();
    let mut searched_from = 0usize;
    while let Some(offset) = body.get(searched_from..).and_then(|rest| return rest.find("Check_"))
    {
        let start = searched_from.saturating_add(offset);
        let rest = body.get(start..).unwrap_or_default();
        let name_length = rest
            .find(|character: char| return !(character.is_ascii_alphanumeric() || character == '_'))
            .unwrap_or(rest.len());
        if let Some(name) = rest.get(..name_length)
        {
            found.insert(name.to_owned());
        }
        searched_from = start.saturating_add(name_length.max(1));
    }

    return found;
}

/// Whether `path`, the part of a surface line after `pub fn nomos_rules::`, names a bare
/// (non-method) `Check_*` function -- `Check_X(...)`, not `SomeType::Check_X(...)`.
fn Check_Function_Name(path: &str) -> Option<String>
{
    let name = path.split('(').next().unwrap_or(path);
    if !name.starts_with("Check_") || name.contains("::")
    {
        return None;
    }

    return Some(name.to_owned());
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Check_Function_Name_Should_Accept_A_Bare_Function_And_Reject_A_Method()
    {
        assert_eq!(Check_Function_Name("Check_Something(sources: &[SourceFile]) -> Vec<Finding>"), Some("Check_Something".to_owned()));
        assert_eq!(Check_Function_Name("SomeType::Check_Something(&self) -> bool"), None);
        assert_eq!(Check_Function_Name("Something_Else(sources: &[SourceFile]) -> Vec<Finding>"), None);
    }
}
