//! The wire shape of a `nomos.controlflow.reachability.v1` payload, and its canonical
//! encoding.

pub(crate) mod payload_refusal;
pub(crate) mod reachability_payload;
pub(crate) mod reachability_site;

use payload_refusal::PayloadRefusal;
use reachability_payload::ReachabilityPayload;
use reachability_site::ReachabilitySite;

/// How one flagged arm's body was shaped, restricted to the four `OD-RULES-008` names as
/// syntactically obvious without name or type resolution.
#[doc = include_str!("../docs/api/arm_shape.md")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmShape
{
    /// `Err(binding) => {}` — an arm whose body is a block with no statements at all.
    Empty,
    /// The arm's body is a bare `continue`, with no other effect.
    BareContinue,
    /// The arm's body is a bare `return`, with no value — the path forward from here
    /// carries nothing a `Finding` could have come from.
    BareReturn,
    /// The arm's body's tail expression is a call to `Ok(...)` — treating a fact-read
    /// failure as though it had succeeded.
    TailOk,
}

impl ArmShape
{
    /// This shape's canonical string label, the spelling `Encode_Payload` writes and
    /// [`From_Label`] parses back.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Empty => "empty",
            Self::BareContinue => "bare-continue",
            Self::BareReturn => "bare-return",
            Self::TailOk => "tail-ok",
        };
    }

    /// Parses a shape back from the label [`Label`] produces, returning `None` for any
    /// other string.
    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            "empty" => Some(Self::Empty),
            "bare-continue" => Some(Self::BareContinue),
            "bare-return" => Some(Self::BareReturn),
            "tail-ok" => Some(Self::TailOk),
            _ => None,
        };
    }
}

/// Encodes a payload as tab-separated lines, the same shape `nomos-cap-dependency` and
/// `nomos-cap-syntax` both use and for the same two reasons: diffable by a person, and
/// written in one place with no derive between the data and the bytes.
#[must_use]
pub fn Encode_Payload(payload: &ReachabilityPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for site in &payload.sites
    {
        encoded.push_str("site\t");
        encoded.push_str(&site.function);
        encoded.push('\t');
        encoded.push_str(&site.binding);
        encoded.push('\t');
        encoded.push_str(site.shape.Label());
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// Reads a payload back out of its canonical encoding.
#[doc = include_str!("../docs/api/arm_shape.md")]
pub fn Parse_Payload(bytes: &[u8]) -> Result<ReachabilityPayload, PayloadRefusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| PayloadRefusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut sites = Vec::new();
    for line in text.lines()
    {
        sites.push(Site_Line(line)?);
    }

    return Ok(ReachabilityPayload { sites });
}

fn Site_Line(line: &str) -> Result<ReachabilitySite, PayloadRefusal>
{
    let rest = Strip_Site_Prefix(line)?;
    let (function, binding, shape) = Split_Site_Fields(rest, line)?;
    let shape = Parse_Arm_Shape(shape)?;

    return Ok(ReachabilitySite {
        function: function.to_owned(),
        binding: binding.to_owned(),
        shape,
    });
}

/// Strips the `site\t` tag every site line must open with.
fn Strip_Site_Prefix(line: &str) -> Result<&str, PayloadRefusal>
{
    let Some(rest) = line.strip_prefix("site\t")
    else
    {
        return Err(PayloadRefusal {
            reason: format!("line is not a site: {line:?}"),
        });
    };

    return Ok(rest);
}

/// Splits a site line's tagged remainder into its three tab-separated fields.
fn Split_Site_Fields<'a>(rest: &'a str, line: &str) -> Result<(&'a str, &'a str, &'a str), PayloadRefusal>
{
    let fields: Vec<&str> = rest.split('\t').collect();
    let [function, binding, shape] = fields.as_slice()
    else
    {
        return Err(PayloadRefusal {
            reason: format!("site line does not have exactly three fields: {line:?}"),
        });
    };

    return Ok((*function, *binding, *shape));
}

/// Resolves a site line's shape field to the [`ArmShape`] it names.
fn Parse_Arm_Shape(shape: &str) -> Result<ArmShape, PayloadRefusal>
{
    let Some(shape) = ArmShape::From_Label(shape)
    else
    {
        return Err(PayloadRefusal {
            reason: format!("unrecognized arm shape: {shape:?}"),
        });
    };

    return Ok(shape);
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
            "site\tPayload_Of\tapplicability\tempty\n\
             site\tWalk_Sources\tapplicability\ttail-ok\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    /// [`ArmShape::Label`] has no test of its own: the round-trip test above drives it only as
    /// a side effect of encoding [`Sample`], which never exercises `BareContinue` or
    /// `BareReturn`. This is the direct address, over every variant.
    #[test]
    fn Test_Label_Should_Produce_A_Distinct_String_Per_Arm_Shape()
    {
        assert_eq!(ArmShape::Empty.Label(), "empty");
        assert_eq!(ArmShape::BareContinue.Label(), "bare-continue");
        assert_eq!(ArmShape::BareReturn.Label(), "bare-return");
        assert_eq!(ArmShape::TailOk.Label(), "tail-ok");
    }

    /// [`ArmShape::From_Label`] is exercised indirectly wherever [`Parse_Payload`] reads a
    /// shape field, but no test is named for it directly. This drives every label it must
    /// resolve, plus the one it must refuse.
    #[test]
    fn Test_From_Label_Should_Resolve_Every_Known_Label_Back_To_Its_Shape()
    {
        assert_eq!(ArmShape::From_Label("empty"), Some(ArmShape::Empty));
        assert_eq!(ArmShape::From_Label("bare-continue"), Some(ArmShape::BareContinue));
        assert_eq!(ArmShape::From_Label("bare-return"), Some(ArmShape::BareReturn));
        assert_eq!(ArmShape::From_Label("tail-ok"), Some(ArmShape::TailOk));
        assert_eq!(ArmShape::From_Label("bogus"), None);
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Round_Trip_To_No_Sites()
    {
        let decoded = Parse_Payload(&[]).expect("a file with no flagged sites is valid");
        assert_eq!(decoded, ReachabilityPayload { sites: Vec::new() });
    }

    #[test]
    fn Test_A_Malformed_Site_Line_Should_Be_Refused()
    {
        for bytes in Malformed_Site_Field_Counts()
        {
            let error = Parse_Payload(bytes).expect_err("a site line without three fields must be refused");
            assert!(
                error.reason.contains("does not have exactly three fields"),
                "expected a field-count refusal for {bytes:?}, got: {}",
                error.reason
            );
        }
    }

    /// Every case here undershoots the three tab-separated fields a site line must have —
    /// one field, or two — so each must be refused for that reason specifically, not merely
    /// refused for some reason or other.
    fn Malformed_Site_Field_Counts() -> Vec<&'static [u8]>
    {
        return vec![b"site\tonly-one-field\n", b"site\ttwo\tfields\n", b"site\t\n"];
    }

    #[test]
    fn Test_An_Unrecognized_Shape_Should_Be_Refused()
    {
        for bytes in Unrecognized_Arm_Shape_Labels()
        {
            let error = Parse_Payload(bytes).expect_err("an unrecognized shape label must be refused");
            assert!(
                error.reason.contains("unrecognized arm shape"),
                "expected an unrecognized-shape refusal for {bytes:?}, got: {}",
                error.reason
            );
        }
    }

    /// Every case here has exactly three fields, so it reaches shape resolution and is
    /// refused there specifically — not for a field count or a missing prefix.
    fn Unrecognized_Arm_Shape_Labels() -> Vec<&'static [u8]>
    {
        return vec![
            b"site\tf\tapplicability\tsomething-else\n",
            b"site\tf\tapplicability\tEMPTY\n",
            b"site\tf\tapplicability\t\n",
        ];
    }

    fn Sample() -> ReachabilityPayload
    {
        return ReachabilityPayload {
            sites: vec![
                ReachabilitySite {
                    function: "Payload_Of".to_owned(),
                    binding: "applicability".to_owned(),
                    shape: ArmShape::Empty,
                },
                ReachabilitySite {
                    function: "Walk_Sources".to_owned(),
                    binding: "applicability".to_owned(),
                    shape: ArmShape::TailOk,
                },
            ],
        };
    }
}
