//! The wire shape of a `nomos.goals.policy.v1` payload, and its canonical encoding.

mod refusal;
mod subsystem_declaration;

pub use refusal::Refusal;
pub use subsystem_declaration::SubsystemDeclaration;

/// A repository's whole declared goal policy.
///
/// All three fields travel together, and that is a departure from code-standards worth
/// stating. There, the goal set and the ceiling are the check's own options while the
/// subsystem-to-goal mapping is a column of a separate architecture declaration four checks
/// share — `check-goal-traceability`'s `spec.go` says so and explains that splitting the
/// column out would let a subsystem's name and the goals it serves drift apart. This
/// workspace has no separate architecture-declaration capability to put that column in, and
/// the judgment is unanswerable without all three, so one capability carries all three and
/// the column stays beside the names it is about — the same property by a different route.
///
/// An empty `goals` means the repository has not opted in, and every rule reading this
/// reports nothing at all: `check-goal-traceability`'s own `AuditTraceability` returns
/// early on it, because a goal cannot be inferred from code and inventing one would impose
/// one repository's plan on every other. That is scripting's "unconfigured means silence"
/// shape rather than naming's and limits' "unconfigured means the prior default".
///
/// `max_subsystems_per_goal` of zero means unbounded, matching the Go implementation's own
/// `maxPerGoal > 0` guard: leaving it unset keeps the two-way traceability audit and drops
/// only the spread bound.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GoalsPolicyPayload
{
    pub goals: Vec<String>,
    pub max_subsystems_per_goal: u32,
    pub subsystems: Vec<SubsystemDeclaration>,
}

const GOAL_TAG: &str = "goal";
const CEILING_TAG: &str = "ceiling";
const SUBSYSTEM_TAG: &str = "subsystem";
const SERVES_TAG: &str = "serves";

/// Encodes a payload as tab-separated lines, the same shape every other capability payload
/// in this workspace uses: diffable by a person, written in one place with no derive between
/// the data and the bytes. No header line — this payload answers for the workspace as a
/// whole, not for one member.
///
/// Goals first, then the ceiling, then each subsystem immediately followed by what it
/// serves, so a reader sees a part and its purposes together rather than having to join two
/// lists by eye.
#[must_use]
pub fn Encode_Payload(payload: &GoalsPolicyPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for goal in &payload.goals
    {
        Push_Row(&mut encoded, &[GOAL_TAG, goal]);
    }

    if payload.max_subsystems_per_goal > 0
    {
        Push_Row(&mut encoded, &[CEILING_TAG, &payload.max_subsystems_per_goal.to_string()]);
    }

    for subsystem in &payload.subsystems
    {
        Push_Row(&mut encoded, &[SUBSYSTEM_TAG, &subsystem.name]);
        for goal in &subsystem.goals
        {
            Push_Row(&mut encoded, &[SERVES_TAG, &subsystem.name, goal]);
        }
    }

    return encoded.into_bytes();
}

fn Push_Row(encoded: &mut String, fields: &[&str])
{
    encoded.push_str(&fields.join("\t"));
    encoded.push('\n');
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, a line carries an unrecognized tag or the
/// wrong number of fields for the tag it carries, any field is empty, a second `ceiling`
/// line is present, a `ceiling` value is not a number, a `subsystem` is declared twice, or a
/// `serves` line names a subsystem no `subsystem` line declared. That last one is the
/// refusal that matters: a mapping whose left-hand side is ungrounded would let a rule
/// report a part the repository never declared.
pub fn Parse_Payload(bytes: &[u8]) -> Result<GoalsPolicyPayload, Refusal>
{
    let text = Decode_Utf8(bytes)?;

    let mut payload = GoalsPolicyPayload::default();
    let mut ceiling_seen = false;

    for line in text.lines()
    {
        let fields = Split_Fields(line)?;
        Apply_Line(&mut payload, &fields, line, &mut ceiling_seen)?;
    }

    return Ok(payload);
}

/// Decodes `bytes` as UTF-8, or refuses.
fn Decode_Utf8(bytes: &[u8]) -> Result<&str, Refusal>
{
    return core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    });
}

/// `line` split on tabs, refused if any field is empty.
fn Split_Fields(line: &str) -> Result<Vec<&str>, Refusal>
{
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.iter().any(|field| return field.is_empty())
    {
        return Err(Refusal {
            reason: format!("line {line:?} has an empty field"),
        });
    }

    return Ok(fields);
}

/// Applies one line's already-split `fields` to `payload`, tracking whether a `ceiling`
/// line has already been seen so a second one can be refused.
fn Apply_Line(payload: &mut GoalsPolicyPayload, fields: &[&str], line: &str, ceiling_seen: &mut bool) -> Result<(), Refusal>
{
    match fields
    {
        [GOAL_TAG, goal] => payload.goals.push((*goal).to_owned()),
        [CEILING_TAG, ceiling] => Apply_Ceiling_Line(payload, *ceiling, SourceLine(line), ceiling_seen)?,
        [SUBSYSTEM_TAG, name] => Apply_Subsystem_Line(payload, *name)?,
        [SERVES_TAG, name, goal] => Apply_Serves_Line(payload, Name(name), Goal(goal), SourceLine(line))?,
        _ =>
        {
            return Err(Refusal {
                reason: format!("line has an unrecognized tag or field count: {line:?}"),
            });
        }
    }

    return Ok(());
}

/// The whole row line a tagged value was parsed from, carried only for its own error
/// message.
struct SourceLine<'a>(&'a str);

/// A `ceiling` line's own handling: refuse a second one, otherwise record it.
fn Apply_Ceiling_Line(payload: &mut GoalsPolicyPayload, ceiling: &str, line: SourceLine<'_>, ceiling_seen: &mut bool) -> Result<(), Refusal>
{
    if *ceiling_seen
    {
        return Err(Refusal {
            reason: format!("a second `ceiling` line is not allowed: {:?}", line.0),
        });
    }
    payload.max_subsystems_per_goal = ceiling.parse().map_err(|_error| Refusal {
        reason: format!("`ceiling` value is not a number: {:?}", line.0),
    })?;
    *ceiling_seen = true;

    return Ok(());
}

/// A `subsystem` line's own handling: refuse a name declared twice, otherwise declare it.
fn Apply_Subsystem_Line(payload: &mut GoalsPolicyPayload, name: &str) -> Result<(), Refusal>
{
    if Position_Of(&payload.subsystems, name).is_some()
    {
        return Err(Refusal {
            reason: format!("subsystem {name:?} is declared twice"),
        });
    }
    payload.subsystems.push(SubsystemDeclaration {
        name: name.to_owned(),
        goals: Vec::new(),
    });

    return Ok(());
}

/// A subsystem's own declared name, distinguished from the adjacent goal name and source
/// line it travels beside so a caller cannot transpose them.
struct Name<'a>(&'a str);

/// A goal a subsystem serves, distinguished from the adjacent subsystem name it travels
/// beside so a caller cannot transpose them.
struct Goal<'a>(&'a str);

/// A `serves` line's own handling: refuse a subsystem no `subsystem` line declared,
/// otherwise record the goal it serves.
fn Apply_Serves_Line(payload: &mut GoalsPolicyPayload, name: Name<'_>, goal: Goal<'_>, line: SourceLine<'_>) -> Result<(), Refusal>
{
    let Some(position) = Position_Of(&payload.subsystems, name.0)
    else
    {
        return Err(Refusal {
            reason: format!("`serves` names a subsystem that was never declared: {:?}", line.0),
        });
    };
    let Some(subsystem) = payload.subsystems.get_mut(position)
    else
    {
        return Err(Refusal {
            reason: format!("`serves` names a subsystem that was never declared: {:?}", line.0),
        });
    };
    subsystem.goals.push(goal.0.to_owned());

    return Ok(());
}

fn Position_Of(subsystems: &[SubsystemDeclaration], name: &str) -> Option<usize>
{
    return subsystems.iter().position(|subsystem| return subsystem.name == name);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Parse_Payload_Should_Round_Trip_A_Payload_Through_Its_Own_Encoding()
    {
        let payload = Sample();
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_Encode_Payload_Should_Produce_Stable_Diffable_Bytes()
    {
        let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("ASCII and tabs");

        assert_eq!(
            rendered,
            "goal\trender\ngoal\tsimulate\nceiling\t1\nsubsystem\tgraphics\nserves\tgraphics\trender\nsubsystem\tutils\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Policy_That_Declares_Nothing()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");

        assert_eq!(decoded, GoalsPolicyPayload::default());
    }

    #[test]
    fn Test_A_Subsystem_Serving_Nothing_Should_Survive_The_Round_Trip()
    {
        let decoded = Parse_Payload(b"goal\trender\nsubsystem\tutils\n")
            .expect("the literal carries a `goal` row and a `subsystem` row, each with the one field its tag takes");

        assert_eq!(decoded.subsystems.len(), 1, "{decoded:?}");
        let declared = decoded.subsystems.first().expect("asserted len 1 above");
        assert_eq!(declared.name, "utils");
        assert!(declared.goals.is_empty(), "a purposeless part is declared and serves nothing: {declared:?}");
    }

    #[test]
    fn Test_An_Unset_Ceiling_Should_Decode_As_Unbounded()
    {
        let decoded = Parse_Payload(b"goal\trender\n")
            .expect("the literal carries one `goal` row, the tag `Apply_Line` takes with a single field");

        assert_eq!(decoded.max_subsystems_per_goal, 0, "zero is this payload's spelling of unbounded");
    }

    #[test]
    fn Test_A_Line_With_An_Empty_Field_Should_Be_Refused()
    {
        let error = Parse_Payload(b"goal\t\n").expect_err("an empty value must be refused");

        assert!(error.reason.contains("empty field"), "{}", error.reason);
    }

    #[test]
    fn Test_An_Unrecognized_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"purpose\trender\n").expect_err("an unrecognized tag must be refused");

        assert!(error.reason.contains("unrecognized tag"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Serves_Line_With_Two_Fields_Should_Be_Refused()
    {
        let error = Parse_Payload(b"serves\tgraphics\n").expect_err("serves names both sides or neither");

        assert!(error.reason.contains("unrecognized tag or field count"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Second_Ceiling_Line_Should_Be_Refused()
    {
        let error = Parse_Payload(b"ceiling\t1\nceiling\t2\n").expect_err("a second ceiling must be refused");

        assert!(error.reason.contains("second `ceiling` line"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Ceiling_That_Is_Not_A_Number_Should_Be_Refused()
    {
        let error = Parse_Payload(b"ceiling\tmany\n").expect_err("a ceiling is a count");

        assert!(error.reason.contains("not a number"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Subsystem_Declared_Twice_Should_Be_Refused()
    {
        let error = Parse_Payload(b"subsystem\tutils\nsubsystem\tutils\n").expect_err("one part, one declaration");

        assert!(error.reason.contains("declared twice"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Serves_Line_For_An_Undeclared_Subsystem_Should_Be_Refused()
    {
        let error = Parse_Payload(b"serves\tghost\trender\n").expect_err("the mapping must be grounded");

        assert!(error.reason.contains("never declared"), "{}", error.reason);
    }

    fn Sample() -> GoalsPolicyPayload
    {
        return GoalsPolicyPayload {
            goals: vec!["render".to_owned(), "simulate".to_owned()],
            max_subsystems_per_goal: 1,
            subsystems: vec![
                SubsystemDeclaration {
                    name: "graphics".to_owned(),
                    goals: vec!["render".to_owned()],
                },
                SubsystemDeclaration {
                    name: "utils".to_owned(),
                    goals: Vec::new(),
                },
            ],
        };
    }
}
