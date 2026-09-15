//! [`Demanded_Families`]' own contract, exercised: a selection demands exactly what its
//! rules declare, and nothing else.

use super::Demanded_Families;
use nomos_contracts::RuleId;
use nomos_rules::{RequiredFact, DESCRIPTORS};

/// A rule's declared family is demanded when that rule is selected.
///
/// `NESTING_DEPTH` is the case, not an example. Its descriptor declares
/// `RequiredFact::LimitsPolicy` and the hand-written guard this derivation replaced named
/// five rules that did not include it, so selecting it alone materialized no limits-policy
/// fact and `Check_Nesting_Depth` fell back to `MAX_NESTING_DEPTH`'s built-in default
/// instead of the limit the repository configured -- silently, with no finding and no
/// `MissingCapability`. A full run hid it, because its five siblings were selected too.
#[test]
fn Test_A_Rule_Selected_Alone_Should_Demand_The_Family_It_Declares()
{
    let demanded = Demanded_Families(&[RuleId::New(nomos_rules::NESTING_DEPTH)]);

    assert!(
        demanded.contains(&RequiredFact::LimitsPolicy),
        "NESTING_DEPTH declares RequiredFact::LimitsPolicy and selecting it alone demanded \
         {demanded:?}. This is the drift the derivation exists to close: a rule that runs \
         without the fact it declared judges against a built-in default and says nothing."
    );
}

/// A family no selected rule declares is not demanded.
///
/// The converse control. Without it the assertion above is satisfied by a derivation that
/// demands everything always, which would be correct and useless -- every run would pay
/// for every provider, and the selection `RuleSelector` exists to express would buy
/// nothing.
#[test]
fn Test_A_Family_No_Selected_Rule_Declares_Should_Not_Be_Demanded()
{
    let demanded = Demanded_Families(&[RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE)]);

    assert!(
        demanded.is_empty(),
        "NO_TRAILING_WHITESPACE declares no required fact -- its descriptor's requires is \
         empty -- and selecting it alone demanded {demanded:?}. A derivation that demands \
         a family nobody asked for makes every narrowed run pay for every provider."
    );
}

/// The demand is exactly the union over the selected rules' declarations.
///
/// Checked against `DESCRIPTORS` itself rather than against a list written here, so a
/// rule whose declaration changes moves this with it and no second statement of the
/// relation can appear for the first one to drift against.
#[test]
fn Test_The_Demand_Should_Be_The_Union_Of_What_The_Selected_Rules_Declare()
{
    for descriptor in DESCRIPTORS
    {
        let demanded = Demanded_Families(&[RuleId::New(descriptor.id)]);

        for family in descriptor.requires
        {
            assert!(
                demanded.contains(family),
                "{} declares {family:?} and selecting it demanded {demanded:?}",
                descriptor.id
            );
        }

        assert!(
            demanded.len() == descriptor.requires.len(),
            "{} declares {:?} and selecting it alone demanded {demanded:?} -- a family \
             nothing selected asked for",
            descriptor.id,
            descriptor.requires
        );
    }
}

/// Selecting everything demands every family any rule declares, once each.
#[test]
fn Test_Selecting_Everything_Should_Demand_Each_Family_Once()
{
    let demanded = Demanded_Families(&[]);

    for descriptor in DESCRIPTORS
    {
        for family in descriptor.requires
        {
            assert!(
                demanded.contains(family),
                "{} declares {family:?} and a run selecting everything demanded {demanded:?}",
                descriptor.id
            );
        }
    }

    Assert_Each_Family_Demanded_Once(&demanded);
}

/// The second half of the claim above: no family appears twice, since one run considers each
/// family's section exactly once. Extracted rather than left inline because it is the whole
/// of what that test asserts after its first loop, and the loop it replaces there was longer
/// than the assertion it carried.
fn Assert_Each_Family_Demanded_Once(demanded: &[RequiredFact])
{
    let mut seen = Vec::new();
    for family in demanded
    {
        assert!(
            !seen.contains(&family),
            "{family:?} appears twice in {demanded:?}, so its section would be considered \
             more than once for one run"
        );
        seen.push(family);
    }
}
