//! The wire shape of a `nomos.goals.policy.v1` payload, and its canonical encoding.

/// One declared part of a system and the purposes it claims to serve.
///
/// A subsystem with an empty `goals` is the interesting case rather than a degenerate one:
/// it is exactly what `check-goal-traceability` calls a purposeless part, so the encoding
/// has to be able to say "this part is declared and serves nothing" distinctly from "this
/// part was never declared". That is why a subsystem gets a line of its own rather than
/// being implied by the goals it serves.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SubsystemDeclaration
{
    pub name: String,
    pub goals: Vec<String>,
}

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

/// A payload's bytes did not decode: not UTF-8, a line with the wrong shape, a line with an
/// empty field, a second `ceiling` line, a `ceiling` that is not a number, or a `serves`
/// line naming a subsystem no `subsystem` line declared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
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
    let text = core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut payload = GoalsPolicyPayload::default();
    let mut ceiling_seen = false;

    for line in text.lines()
    {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.iter().any(|field| return field.is_empty())
        {
            return Err(Refusal {
                reason: format!("line {line:?} has an empty field"),
            });
        }

        match fields.as_slice()
        {
            [GOAL_TAG, goal] => payload.goals.push((*goal).to_owned()),
            [CEILING_TAG, ceiling] =>
            {
                if ceiling_seen
                {
                    return Err(Refusal {
                        reason: format!("a second `ceiling` line is not allowed: {line:?}"),
                    });
                }
                payload.max_subsystems_per_goal = ceiling.parse().map_err(|_error| Refusal {
                    reason: format!("`ceiling` value is not a number: {line:?}"),
                })?;
                ceiling_seen = true;
            }
            [SUBSYSTEM_TAG, name] =>
            {
                if Position_Of(&payload.subsystems, name).is_some()
                {
                    return Err(Refusal {
                        reason: format!("subsystem {name:?} is declared twice"),
                    });
                }
                payload.subsystems.push(SubsystemDeclaration {
                    name: (*name).to_owned(),
                    goals: Vec::new(),
                });
            }
            [SERVES_TAG, name, goal] =>
            {
                let Some(position) = Position_Of(&payload.subsystems, name)
                else
                {
                    return Err(Refusal {
                        reason: format!("`serves` names a subsystem that was never declared: {line:?}"),
                    });
                };
                let Some(subsystem) = payload.subsystems.get_mut(position)
                else
                {
                    return Err(Refusal {
                        reason: format!("`serves` names a subsystem that was never declared: {line:?}"),
                    });
                };
                subsystem.goals.push((*goal).to_owned());
            }
            _ =>
            {
                return Err(Refusal {
                    reason: format!("line has an unrecognized tag or field count: {line:?}"),
                });
            }
        }
    }

    return Ok(payload);
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
        let decoded = Parse_Payload(b"goal\trender\nsubsystem\tutils\n").expect("well-formed");

        assert_eq!(decoded.subsystems.len(), 1, "{decoded:?}");
        let declared = decoded.subsystems.first().expect("asserted len 1 above");
        assert_eq!(declared.name, "utils");
        assert!(declared.goals.is_empty(), "a purposeless part is declared and serves nothing: {declared:?}");
    }

    #[test]
    fn Test_An_Unset_Ceiling_Should_Decode_As_Unbounded()
    {
        let decoded = Parse_Payload(b"goal\trender\n").expect("well-formed");

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
