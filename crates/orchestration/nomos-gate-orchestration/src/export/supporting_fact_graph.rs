//! [`SupportingFactGraph`], what each rule's judgment depended on, as `GraphML`.

use super::run_status::RunStatus;
use crate::GateRunResult;
use nomos_analysis::ReadOutcome;
use nomos_check_orchestration::{CheckOutcome, FactRead, SupportingFacts};
use nomos_contracts::Guarantee;
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// The `GraphML` document's opening, through every attribute key it declares.
///
/// Every key a node, an edge or the graph itself carries is declared here with its type, which
/// is the property of `GraphML` that decided the format: a reader receives `reads` as a number
/// and an absent guarantee as an absent attribute, not as a string it must reinterpret.
const PROLOGUE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">
  <key id="executionSuccessful" for="graph" attr.name="executionSuccessful" attr.type="boolean"/>
  <key id="reasons" for="graph" attr.name="reasons" attr.type="string"/>
  <key id="kind" for="node" attr.name="kind" attr.type="string"/>
  <key id="support" for="node" attr.name="support" attr.type="string"/>
  <key id="capability" for="node" attr.name="capability" attr.type="string"/>
  <key id="provider" for="node" attr.name="provider" attr.type="string"/>
  <key id="providerVersion" for="node" attr.name="providerVersion" attr.type="string"/>
  <key id="outcome" for="edge" attr.name="outcome" attr.type="string"/>
  <key id="degradedTo" for="edge" attr.name="degradedTo" attr.type="string"/>
  <key id="reads" for="edge" attr.name="reads" attr.type="long"/>
  <key id="variant" for="edge" attr.name="variant" attr.type="string"/>
  <key id="soundness" for="edge" attr.name="soundness" attr.type="string"/>
  <key id="completeness" for="edge" attr.name="completeness" attr.type="string"/>
  <key id="incremental" for="edge" attr.name="incremental" attr.type="string"/>
  <key id="evidence" for="edge" attr.name="evidence" attr.type="string"/>
  <graph id="supporting-facts" edgedefault="directed">
"#;

/// The document's close.
const EPILOGUE: &str = "  </graph>\n</graphml>\n";

/// The dependency graph a run already holds, as `GraphML`: every rule, the capability families
/// each rule's judgment read and from which provider, and how each read came back.
///
/// This is the check's supporting-fact trail (`OD-HOST-016`), which is the only graph a
/// [`GateRunResult`] carries; the crate dependency graph a run materializes stays in a fact store
/// the run does not return. A rule node's `support` is that trail's own four-way answer --
/// `read`, `not-fact-backed`, `raised-by-materialization` or `unrecorded` -- so a rule that asked
/// for nothing is never drawn as one whose reads came back empty. Edges run from a rule to a
/// capability node, which is one capability family as one provider version answers it.
///
/// The graph's own `executionSuccessful` and `reasons` carry the run's status, so a check that
/// judged nothing is a graph that says so, not an empty graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportingFactGraph
{
    /// The run's own status.
    status: RunStatus,
    /// Every rule the rule table holds, by id, with its trail's answer.
    rules: Vec<RuleNode>,
    /// Every capability-and-provider pair some rule read.
    capabilities: BTreeSet<CapabilityNode>,
    /// Every read, sorted.
    edges: Vec<ReadEdge>,
}

/// One rule and the trail's answer for it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RuleNode
{
    /// The rule's id.
    rule: String,
    /// The trail's answer, as [`Support_Word`] spells it.
    support: &'static str,
}

/// One capability family, as one provider version answers it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CapabilityNode
{
    /// The capability family.
    capability: String,
    /// The provider the read was addressed to.
    provider: String,
    /// The contract version that provider's offer names.
    provider_version: String,
}

/// One rule's reduced read of one capability node.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReadEdge
{
    /// The rule that read.
    rule: String,
    /// What it read.
    target: CapabilityNode,
    /// How the read came back, as [`Outcome_Word`] spells it.
    outcome: &'static str,
    /// The applicability a degraded read came back at, for a degraded read only.
    degraded_to: Option<&'static str>,
    /// How many reads stood behind this tuple.
    reads: usize,
    /// The declared completeness of the fact that answered, for a read that was answered.
    guarantee: Option<Guarantee>,
    /// How the fact that answered was come by, for a read that was answered.
    evidence: Option<&'static str>,
}

impl SupportingFactGraph
{
    /// The graph of what a gate run's check depended on.
    #[must_use]
    pub fn Of_Gate_Run(result: &GateRunResult) -> Self
    {
        return Self::Of(RunStatus::Of_Gate_Run(result), &result.check_outcome);
    }

    /// The graph of what a check run depended on.
    #[must_use]
    pub fn Of_Check_Run(outcome: &CheckOutcome) -> Self
    {
        return Self::Of(RunStatus::Of_Check_Run(outcome), outcome);
    }

    /// `outcome`'s trail beside `status`; a check that judged nothing has no trail and no nodes.
    fn Of(status: RunStatus, outcome: &CheckOutcome) -> Self
    {
        let mut graph = Self { status, rules: Vec::new(), capabilities: BTreeSet::new(), edges: Vec::new() };
        let CheckOutcome::Judged { supporting_facts, .. } = outcome
        else
        {
            return graph;
        };

        let mut rules: Vec<&str> = nomos_rules::DESCRIPTORS.iter().map(|descriptor| return descriptor.id).collect();
        rules.sort_unstable();
        for rule in rules
        {
            let facts = supporting_facts.Facts_For(rule);
            if let SupportingFacts::Read(reads) = &facts
            {
                for read in reads
                {
                    let edge = ReadEdge::Of(rule, read);
                    graph.capabilities.insert(edge.target.clone());
                    graph.edges.push(edge);
                }
            }
            graph.rules.push(RuleNode { rule: rule.to_owned(), support: Support_Word(&facts) });
        }

        return graph;
    }

    /// The graph as a `GraphML` document.
    #[must_use]
    pub fn Rendered(&self) -> String
    {
        let mut text = PROLOGUE.to_owned();
        Data(&mut text, "    ", "executionSuccessful", &self.status.is_execution_successful.to_string());
        if !self.status.reasons.is_empty()
        {
            Data(&mut text, "    ", "reasons", &self.status.reasons.join("\n"));
        }

        for rule in &self.rules
        {
            let _ = writeln!(text, "    <node id=\"{}\">", Escaped(&Rule_Id(&rule.rule)));
            Data(&mut text, "      ", "kind", "rule");
            Data(&mut text, "      ", "support", rule.support);
            text.push_str("    </node>\n");
        }
        for capability in &self.capabilities
        {
            let _ = writeln!(text, "    <node id=\"{}\">", Escaped(&Capability_Id(capability)));
            Data(&mut text, "      ", "kind", "capability");
            Data(&mut text, "      ", "capability", &capability.capability);
            Data(&mut text, "      ", "provider", &capability.provider);
            Data(&mut text, "      ", "providerVersion", &capability.provider_version);
            text.push_str("    </node>\n");
        }
        for (ordinal, edge) in self.edges.iter().enumerate()
        {
            edge.Write(&mut text, ordinal);
        }

        text.push_str(EPILOGUE);
        return text;
    }
}

impl ReadEdge
{
    /// `rule`'s reduced read `read`.
    fn Of(rule: &str, read: &FactRead) -> Self
    {
        let (outcome, degraded_to) = Outcome_Word(read.outcome);

        return Self {
            rule: rule.to_owned(),
            target: CapabilityNode {
                capability: read.capability.As_Str().to_owned(),
                provider: read.provider.As_Str().to_owned(),
                provider_version: read.provider_version.to_string(),
            },
            outcome,
            degraded_to,
            reads: read.reads,
            guarantee: read.guarantee,
            evidence: read.evidence.map(nomos_contracts::EvidenceClass::Label),
        };
    }

    /// This edge as a `GraphML` `edge` element, numbered `ordinal`.
    fn Write(&self, text: &mut String, ordinal: usize)
    {
        let _ = writeln!(
            text,
            "    <edge id=\"e{ordinal}\" source=\"{}\" target=\"{}\">",
            Escaped(&Rule_Id(&self.rule)),
            Escaped(&Capability_Id(&self.target))
        );
        Data(text, "      ", "outcome", self.outcome);
        if let Some(applicability) = self.degraded_to
        {
            Data(text, "      ", "degradedTo", applicability);
        }
        Data(text, "      ", "reads", &self.reads.to_string());
        if let Some(guarantee) = self.guarantee
        {
            Data(text, "      ", "variant", &guarantee.variant.to_string());
            Data(text, "      ", "soundness", &guarantee.soundness.to_string());
            Data(text, "      ", "completeness", &guarantee.completeness.to_string());
            Data(text, "      ", "incremental", &guarantee.incremental.to_string());
        }
        if let Some(evidence) = self.evidence
        {
            Data(text, "      ", "evidence", evidence);
        }
        text.push_str("    </edge>\n");
    }
}

/// The trail's answer for one rule, in the word a node carries.
const fn Support_Word(facts: &SupportingFacts) -> &'static str
{
    return match facts
    {
        SupportingFacts::Read(_) => "read",
        SupportingFacts::NotFactBacked => "not-fact-backed",
        SupportingFacts::RaisedByMaterialization => "raised-by-materialization",
        SupportingFacts::Unrecorded => "unrecorded",
    };
}

/// How a read came back, and the applicability a degraded one came back at.
const fn Outcome_Word(outcome: ReadOutcome) -> (&'static str, Option<&'static str>)
{
    return match outcome
    {
        ReadOutcome::Materialized => ("materialized", None),
        ReadOutcome::Absent => ("absent", None),
        ReadOutcome::Superseded => ("superseded", None),
        ReadOutcome::Degraded(applicability) => ("degraded", Some(applicability.Label())),
    };
}

/// A rule node's id.
fn Rule_Id(rule: &str) -> String
{
    return format!("rule:{rule}");
}

/// A capability node's id: the family, the provider and its version, which together are what
/// makes one node distinct from another.
fn Capability_Id(capability: &CapabilityNode) -> String
{
    return format!("capability:{}@{}@{}", capability.capability, capability.provider, capability.provider_version);
}

/// Appends one `data` element for `key`, indented by `indent`, its value escaped.
fn Data(text: &mut String, indent: &str, key: &str, value: &str)
{
    let _ = writeln!(text, "{indent}<data key=\"{key}\">{}</data>", Escaped(value));
}

/// `value` with the five characters XML reserves replaced by their entities, which is enough
/// for both attribute values and element text.
fn Escaped(value: &str) -> String
{
    return value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;");
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::export::xml_reader::{Element, Parsed};
    use crate::sarif::fixtures::{Complete_Gate_Result, Empty_Gate_Findings};
    use crate::NoVerdict;

    /// The `data` values `element` carries, by key.
    fn Data_Of(element: &Element) -> std::collections::BTreeMap<String, String>
    {
        return element
            .children
            .iter()
            .filter(|child| return child.name == "data")
            .map(|child| return (child.attributes.get("key").cloned().unwrap_or_default(), child.text.clone()))
            .collect();
    }

    /// The document's one `graph` element.
    fn Graph_Of(root: &Element) -> &Element
    {
        assert_eq!(root.name, "graphml");
        return root.children.iter().find(|child| return child.name == "graph").expect("a GraphML document holds a graph");
    }

    /// Every `data` key the document uses is declared, for the element kind that uses it, and
    /// every edge's two ends are nodes the document holds -- the two things a `GraphML` reader
    /// refuses a document over.
    fn Assert_Well_Formed_Graph_Document(root: &Element)
    {
        let declared: BTreeSet<(String, String)> = root
            .children
            .iter()
            .filter(|child| return child.name == "key")
            .map(|key| return (key.attributes.get("id").cloned().unwrap_or_default(), key.attributes.get("for").cloned().unwrap_or_default()))
            .collect();
        let graph = Graph_Of(root);
        let nodes: BTreeSet<String> = graph.children.iter().filter(|child| return child.name == "node").filter_map(|node| return node.attributes.get("id").cloned()).collect();

        for key in Data_Of(graph).keys()
        {
            assert!(declared.contains(&(key.clone(), "graph".to_owned())), "graph key {key} is declared");
        }
        for element in &graph.children
        {
            for key in Data_Of(element).keys()
            {
                assert!(declared.contains(&(key.clone(), element.name.clone())), "{} key {key} is declared", element.name);
            }
            if element.name == "edge"
            {
                for end in ["source", "target"]
                {
                    let named = element.attributes.get(end).cloned().unwrap_or_default();
                    assert!(nodes.contains(&named), "edge {end} {named} is a node");
                }
            }
        }
    }

    /// Over a real gate run, the completeness-mirror rule's node says it read, and an edge runs
    /// from it to the syntax capability it read, carrying a read count and the answering fact's
    /// variant. A real run rather than a constructed trail, because the trail's recording is the
    /// check service's and a fixture could only restate what it was built to contain.
    #[test]
    fn Test_A_Real_Run_Should_Draw_Each_Rule_And_What_Its_Judgment_Read()
    {
        let root = std::env::temp_dir().join(format!("nomos-gate-support-graph-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("the temp directory is writable");
        let text = "/// A list.\n/// Mirrored by `Test_Graph_Ghost`.\npub const TABLES: &[&str] = &[];\n";
        std::fs::write(root.join("a.rs"), text).expect("writes one real source");

        let command = crate::GateCommand {
            root: root.clone(),
            rules: crate::RuleSelector { include: vec![nomos_contracts::RuleId::New(nomos_rules::COMPLETENESS_MIRROR)] },
            ..crate::GateCommand::default()
        };
        let source = nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), text.to_owned());
        let providers = nomos_composer_providers::Standard_Providers();
        let environment = crate::GateEnvironment {
            variant: nomos_workspace::BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>()),
            launcher: &nomos_platform_std::StdProgramLauncher,
            filesystem: &nomos_platform_std::StdFileSystem,
            environment: &nomos_platform_std::StdEnvironment,
            now: nomos_platform::Timestamp::From_Unix_Seconds(0),
            providers: &providers,
        };
        let result = crate::Run_Gate(Some(vec![source]), environment, &command, crate::Fresh_Run_Id(nomos_platform::Timestamp::From_Unix_Seconds(0)));
        let _ignored = std::fs::remove_dir_all(&root);

        let root_element = Parsed(&SupportingFactGraph::Of_Gate_Run(&result).Rendered());
        Assert_Well_Formed_Graph_Document(&root_element);
        let graph = Graph_Of(&root_element);
        let rule_id = format!("rule:{}", nomos_rules::COMPLETENESS_MIRROR);
        let rule = graph.children.iter().find(|child| return child.attributes.get("id") == Some(&rule_id)).expect("the rule that ran has a node");
        assert_eq!(Data_Of(rule).get("support").map(String::as_str), Some("read"), "{rule:?}");
        let edge = graph
            .children
            .iter()
            .find(|child| {
                return child.name == "edge"
                    && child.attributes.get("source") == Some(&rule_id)
                    && child.attributes.get("target").is_some_and(|target| return target.starts_with("capability:nomos.cap.syntax"));
            })
            .expect("the rule's read of the syntax capability is an edge");
        let data = Data_Of(edge);
        assert!(data.get("reads").and_then(|reads| return reads.parse::<usize>().ok()).is_some_and(|reads| return reads > 0), "{data:?}");
        assert!(data.contains_key("variant"), "{data:?}");
    }

    /// A run that reached no verdict is a graph that says so and why.
    #[test]
    fn Test_A_Run_That_Reached_No_Verdict_Should_Say_So_On_The_Graph()
    {
        let mut result = Complete_Gate_Result(Empty_Gate_Findings());
        result.no_verdict = Some(NoVerdict::IncompleteCoverage);

        let root_element = Parsed(&SupportingFactGraph::Of_Gate_Run(&result).Rendered());

        Assert_Well_Formed_Graph_Document(&root_element);
        let data = Data_Of(Graph_Of(&root_element));
        assert_eq!(data.get("executionSuccessful").map(String::as_str), Some("false"), "{data:?}");
        assert!(data.get("reasons").is_some_and(|reasons| return reasons.contains("require-completeness")), "{data:?}");
    }

    /// Text XML reserves survives the round trip through an attribute and a data element.
    #[test]
    fn Test_Escaped_Should_Round_Trip_Every_Reserved_Character()
    {
        let awkward = "a & b < c > d \" e ' f";
        let root_element = Parsed(&format!("<graphml><graph a=\"{0}\"><data key=\"k\">{0}</data></graph></graphml>", Escaped(awkward)));

        let graph = Graph_Of(&root_element);
        assert_eq!(graph.attributes.get("a").map(String::as_str), Some(awkward));
        assert_eq!(Data_Of(graph).get("k").map(String::as_str), Some(awkward));
    }
}
