//! The registry MCP's `tools/list` reports, and the one `tools/call` dispatches by name.

use nomos_api_transport::ServedMethod;
use serde_json::{json, Value};
use xvpe_remote_call::ToolDescriptor;

/// One operation this server exposes as an MCP tool.
///
/// A thin second name for [`ServedMethod`]'s own variants -- not a wider registry and not a
/// narrower one. `OD-HOST-007`'s Gate verbs, plus `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2`'s
/// Correction verb and `OD-HOST-014`'s Check verb, are the whole of what an MCP client sees
/// here too, the same boundary `nomos-api-transport` already draws and proves
/// against `nomos-api`'s own blessed surface: this crate never calls a `nomos_api::Handle_*`
/// function or depends on `nomos-api` at all, so widening what a `tools/call` can reach
/// would first have to widen [`ServedMethod`] itself, in that crate, where the exclusion is
/// already policed. This type adds only what `tools/list` needs beside a name -- a human
/// description and a JSON Schema for its arguments -- neither of which that crate has a
/// reason to carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServedTool(ServedMethod);

impl ServedTool
{
    /// Every tool this server lists, in the order `tools/list` reports them -- the same order
    /// [`ServedMethod::REGISTRY`] declares, since this projects that registry rather than
    /// keeping a second one beside it.
    ///
    /// Mirrored by `Test_The_Tool_Registry_Should_Name_The_Same_Operations_As_The_Served_Method_Registry`.
    /// A further `ServedMethod` variant added to that registry without a matching entry here
    /// would otherwise vanish silently: nothing else compares this array's length or order
    /// against the registry it claims to project. `OD-HOST-014`'s decision 5 names this array
    /// as one of the three places an admitting increment must edit.
    pub const REGISTRY: [Self; 6] = [
        Self(ServedMethod::GatePlan),
        Self(ServedMethod::GateRun),
        Self(ServedMethod::GateExplain),
        Self(ServedMethod::GateCompare),
        Self(ServedMethod::Correction),
        Self(ServedMethod::Check),
    ];

    /// This tool's canonical name -- `nomos_contracts::OperationName`'s own identity,
    /// projected without renaming. That type's own doc states the rule directly: "an MCP tool
    /// name ... may [not] disagree with it or with each other," so a `nomos.gate.run` MCP tool
    /// is the same string `nomos-api-transport`'s wire method already is, not a
    /// `snake_case`-flavored second spelling.
    #[must_use]
    pub fn Name(self) -> &'static str
    {
        return self.0.Name();
    }

    /// The tool `name` spells, or `None` for a name outside the registry -- the identical
    /// "absence is not an error here" contract [`ServedMethod::Named`] states for the wire
    /// method this projects.
    #[must_use]
    pub fn Named(name: &str) -> Option<Self>
    {
        return ServedMethod::Named(name).map(Self);
    }

    /// This tool, as the engine's own described entry.
    ///
    /// The schema is rendered here rather than authored as text: written as a
    /// literal it would be one long unreadable line per tool, and a typo in it
    /// would be a schema nothing rejects. Built as a document and rendered at
    /// this one boundary, it is readable where it is written and text where it
    /// crosses.
    #[must_use]
    pub fn Descriptor(self) -> ToolDescriptor
    {
        return ToolDescriptor::Of(self.Name(), self.Description(), self.Input_Schema().to_string());
    }

    /// What a client reads before deciding whether to call this tool.
    fn Description(self) -> &'static str
    {
        return match self.0
        {
            ServedMethod::GatePlan => "Reports what this workspace's rule registry holds, without walking or judging any tree.",
            ServedMethod::GateRun => "Walks and judges a tree, and reports the findings and disposition of a real gate run over it.",
            ServedMethod::GateExplain => "Explains one named finding from a prior run: what it means and whether it would keep a run from passing.",
            ServedMethod::GateCompare => {
                "Judges two trees, or one tree under two policies, and reports what changed: findings that are new, findings that are gone, and findings that moved between blocking and suppressed, baselined or calibrated."
            }
            ServedMethod::Correction => {
                "Walks a tree, stages a fix for the first real blocking correction claim it finds (a stale doc mirror or trailing whitespace), and, only if asked, commits it."
            }
            ServedMethod::Check => {
                "Walks a tree and runs every registered rule over it, reporting the findings, the files and facts examined, and whether every rule could look. Unlike nomos.gate.run it applies no suppression, baseline or coverage policy and reaches no disposition."
            }
        };
    }

    /// This tool's arguments, as the JSON Schema the parameter type `nomos-api-transport`
    /// itself deserializes already accepts -- hand-written rather than derived, the same "no
    /// library bought a shape this crate can spell itself" choice that crate's own module doc
    /// makes about JSON-RPC framing. `nomos_api_transport::GateParameters`'s,
    /// `FindingParameters`'s and `CheckParameters`' own doc comments are this schema's
    /// authority; a field added or renamed there without a matching edit here is a schema
    /// silently describing arguments
    /// the operation no longer takes, which `dispatch::tests` exercises by calling every
    /// listed tool with the arguments its own schema shows a client.
    fn Input_Schema(self) -> Value
    {
        return match self.0
        {
            ServedMethod::GatePlan => Plan_Schema(),
            ServedMethod::GateRun => Run_Schema(),
            ServedMethod::GateCompare => Compare_Schema(),
            ServedMethod::GateExplain => Explain_Schema(),
            ServedMethod::Correction => Correction_Schema(),
            ServedMethod::Check => Check_Schema(),
        };
    }
}

/// `nomos.gate.plan`'s arguments: none, and a schema saying so rather than leaving a client to
/// guess whether an empty object is an omission or a mistake.
fn Plan_Schema() -> Value
{
    return json!({
        "type": "object",
        "properties": {},
        "additionalProperties": false,
    });
}

/// `nomos.gate.run`'s arguments -- the four fields every judging verb takes.
fn Run_Schema() -> Value
{
    return json!({
        "type": "object",
        "properties": {
            "root": {
                "type": "string",
                "description": "The tree to judge. Absent, this server's own working directory.",
            },
            "include": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Which files under root a run judges. Empty selects everything.",
            },
            "exclude": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Which files under root a run does not judge, applied after include.",
            },
            "rules": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Which rules' findings count toward the disposition. Empty selects every registered rule.",
            },
        },
        "additionalProperties": false,
    });
}

/// `nomos.gate.compare`'s arguments: the same side twice, named `baseline` and `candidate`.
fn Compare_Schema() -> Value
{
    return json!({
        "type": "object",
        "properties": {
            "baseline": Side_Schema(),
            "candidate": Side_Schema(),
        },
        "additionalProperties": false,
    });
}

/// One side of `nomos.gate.compare` -- the whole of `Run_Schema`'s arguments, since both sides
/// are judged by the same operation.
fn Side_Schema() -> Value
{
    return json!({
        "type": "object",
        "properties": {
            "root": {
                "type": "string",
                "description": "The tree this side judges. Absent, this server's own working directory.",
            },
            "include": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Which files under root this side judges. Empty selects everything.",
            },
            "exclude": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Which files under root this side does not judge, applied after include.",
            },
            "rules": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Which rules' findings count toward this side's disposition. Empty selects every registered rule.",
            },
        },
        "additionalProperties": false,
    });
}

/// `nomos.gate.explain`'s arguments: the two fields a finding is named by, both required.
fn Explain_Schema() -> Value
{
    return json!({
        "type": "object",
        "properties": {
            "root": {
                "type": "string",
                "description": "The tree to judge before answering. Absent, this server's own working directory.",
            },
            "rule": {
                "type": "string",
                "description": "The rule the finding to explain was produced by.",
            },
            "location": {
                "type": "string",
                "description": "One location that finding names, as a reader of a run's own output already sees it.",
            },
        },
        "required": ["rule", "location"],
        "additionalProperties": false,
    });
}

/// `nomos.gate.correction`'s arguments: which tree, and whether the fix is only staged.
fn Correction_Schema() -> Value
{
    return json!({
        "type": "object",
        "properties": {
            "root": {
                "type": "string",
                "description": "The tree to correct. Absent, this server's own working directory.",
            },
            "commit": {
                "type": "boolean",
                "description": "Whether to actually write the corrected file, or only stage and validate it. Absent, false.",
            },
        },
        "additionalProperties": false,
    });
}

/// `nomos.check.run`'s arguments: which tree, and nothing else.
///
/// One property rather than the four `Run_Schema` shows, because `nomos check` itself takes
/// one -- `nomos_api_transport::CheckParameters`' own doc is this schema's authority, the
/// same way [`Run_Schema`]'s is `GateParameters`'.
fn Check_Schema() -> Value
{
    return json!({
        "type": "object",
        "properties": {
            "root": {
                "type": "string",
                "description": "The tree to judge. Absent, this server's own working directory.",
            },
        },
        "additionalProperties": false,
    });
}

#[cfg(test)]
mod tests
{
    use super::ServedTool;
    use crate::test_support::Field_At;

    /// Every registered tool round-trips through its own name -- so a name added to
    /// [`ServedTool::Name`] without a matching [`ServedTool::REGISTRY`] entry is unreachable
    /// and this fails rather than silently listing nothing for it.
    #[test]
    fn Test_Every_Registered_Tool_Should_Be_Found_By_Its_Own_Name()
    {
        for tool in ServedTool::REGISTRY
        {
            assert_eq!(ServedTool::Named(tool.Name()), Some(tool), "{tool:?}");
        }
    }

    /// A name outside the registry is refused rather than guessed at.
    #[test]
    fn Test_A_Name_Outside_The_Registry_Should_Not_Resolve()
    {
        assert_eq!(ServedTool::Named("nomos.work.list"), None);
        assert_eq!(ServedTool::Named("gate_run"), None);
    }

    /// Every listing carries a name, a non-empty description and an input schema -- the
    /// three fields a client needs to decide whether and how to call the tool.
    #[test]
    fn Test_Every_Listing_Should_Carry_A_Name_A_Description_And_An_Input_Schema()
    {
        for tool in ServedTool::REGISTRY
        {
            let descriptor = tool.Descriptor();
            let schema: serde_json::Value =
                serde_json::from_str(&descriptor.input_schema).expect("a schema is a document");
            assert_eq!(descriptor.name, tool.Name(), "{descriptor:?}");
            assert!(!descriptor.description.is_empty(), "{descriptor:?}");
            assert_eq!(Field_At(&schema, "/type"), "object", "{schema}");
        }
    }

    /// [`ServedTool::REGISTRY`]'s own claimed mirror: a further `ServedMethod` variant would
    /// not silently vanish from this projection, because this compares the two registries by
    /// name and order rather than trusting them to stay in step.
    #[test]
    fn Test_The_Tool_Registry_Should_Name_The_Same_Operations_As_The_Served_Method_Registry()
    {
        let tools: Vec<&str> = ServedTool::REGISTRY.iter().map(|tool| return tool.Name()).collect();
        let methods: Vec<&str> = nomos_api_transport::ServedMethod::REGISTRY.iter().map(|method| return method.Name()).collect();

        assert_eq!(tools, methods);
    }
}
