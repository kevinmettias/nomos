//! [`ToolDriver`], which tool produced the run and which rules it could have fired.

use super::reporting_descriptor::ReportingDescriptor;
use super::sarif_result::SarifResult;
use crate::{GateCommand, GateOutcome};
use nomos_rules::RuleOffer;
use serde::Serialize;
use std::collections::BTreeMap;

/// The product, not this crate: a consumer reads `tool.driver.name` as the analyzer it is
/// looking at, and `nomos-mcp` already identifies this product's tool server under the same
/// word.
const TOOL_NAME: &str = "nomos";

/// This crate's own version, the same `CARGO_PKG_VERSION` `nomos-mcp`'s `ServerIdentity`
/// reports for the product over a wire. `env!` resolves against the crate that calls it, so
/// this is the version of the projection and of the judgment it projects.
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// SARIF 2.1.0 §3.19's `toolComponent` object, in its `driver` role.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct ToolDriver
{
    /// [`TOOL_NAME`].
    pub(crate) name: &'static str,
    /// [`TOOL_VERSION`].
    pub(crate) version: &'static str,
    /// Every rule that could have fired, sorted by id.
    pub(crate) rules: Vec<ReportingDescriptor>,
}

impl ToolDriver
{
    /// The driver for a run whose results are `results`.
    ///
    /// "Every rule that could have fired" is this build's rule registry -- the same registry
    /// [`crate::Run`] plans and a bare check runs in full -- with any rule a result cites that
    /// the registry does not hold added by id. Neither judgment carries a record of the `rules`
    /// narrowing a `GateCommand` may have applied, so the registry is the widest honest answer
    /// and the only one the judgment alone can reconstruct. A `BTreeMap` keyed by id, so the listing is sorted and duplicate-free
    /// without depending on the order either source arrives in.
    pub(crate) fn Of(results: &[SarifResult]) -> Self
    {
        let mut rules: BTreeMap<String, ReportingDescriptor> = BTreeMap::new();

        for offer in Registered_Offers()
        {
            rules.insert(offer.rule.As_Str().to_owned(), ReportingDescriptor::Registered(&offer));
        }

        for result in results
        {
            rules.entry(result.Rule_Id().to_owned()).or_insert_with(|| return ReportingDescriptor::Bare(result.Rule_Id()));
        }

        return Self { name: TOOL_NAME, version: TOOL_VERSION, rules: rules.into_values().collect() };
    }
}

/// What this build's registry holds, or nothing when its own composition is contradictory --
/// a state [`crate::Registered`]'s doc says is not reachable today, and one
/// this projection would still rather list findings' own rules under than refuse a log over.
fn Registered_Offers() -> Vec<RuleOffer>
{
    return match crate::Run(&GateCommand::default())
    {
        GateOutcome::Planned(plan) => plan.rules,
        GateOutcome::Contradictory(_) => Vec::new(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::Finding_At;
    use std::path::Path;

    #[test]
    fn Test_Of_Should_Name_The_Product_And_A_Non_Empty_Version()
    {
        let driver = ToolDriver::Of(&[]);

        assert_eq!(driver.name, "nomos");
        assert!(!driver.version.is_empty());
    }

    /// The listing is the registry a real plan reports, not an empty one and not only the
    /// rules that happened to fire.
    #[test]
    fn Test_Of_Should_List_Every_Rule_The_Registry_Holds_Sorted_By_Id()
    {
        let offered = Planned_Rules();

        let driver = ToolDriver::Of(&[]);

        let listed: Vec<&str> = driver.rules.iter().map(|rule| return rule.id.as_str()).collect();
        let mut expected: Vec<&str> = offered.iter().map(|offer| return offer.rule.As_Str()).collect();
        expected.sort_unstable();
        assert_eq!(listed, expected);
        assert!(driver.rules.iter().all(|rule| return rule.properties.is_some()), "every registered rule carries its citation");
    }

    /// The falsifier for the merge: a result citing a rule no registry holds still resolves to
    /// a descriptor, and a result citing a registered rule does not duplicate it.
    #[test]
    fn Test_Of_Should_Add_A_Results_Unregistered_Rule_By_Id_And_Not_Duplicate_A_Registered_One()
    {
        let registered_id = Planned_First_Rule_Id();
        let unregistered = SarifResult::Of(Path::new("."), &Finding_At("a-rule-no-registry-holds", "a.rs:1"), None);
        let registered = SarifResult::Of(Path::new("."), &Finding_At(&registered_id, "a.rs:1"), None);

        let driver = ToolDriver::Of(&[unregistered, registered]);

        let bare = driver.rules.iter().find(|rule| return rule.id == "a-rule-no-registry-holds").expect("an unregistered rule a result cites is listed");
        assert!(bare.properties.is_none(), "{bare:?}");
        assert_eq!(driver.rules.iter().filter(|rule| return rule.id == registered_id).count(), 1);
        assert!(driver.rules.windows(2).all(|pair| return pair.first().map(|left| return &left.id) < pair.get(1).map(|right| return &right.id)), "sorted and distinct");
    }

    fn Planned_First_Rule_Id() -> String
    {
        return Planned_Rules().first().map(|offer| return offer.rule.As_Str().to_owned()).expect("this workspace ships real rules");
    }

    /// Every rule this build's registry offers, as [`crate::Run`] plans it.
    fn Planned_Rules() -> Vec<RuleOffer>
    {
        let GateOutcome::Planned(plan) = crate::Run(&GateCommand::default())
        else
        {
            panic!("this crate's own rule registry composes cleanly");
        };

        return plan.rules;
    }
}
