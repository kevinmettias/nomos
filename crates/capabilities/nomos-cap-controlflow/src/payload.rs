//! The wire shape of a `nomos.controlflow.reachability.v1` payload, and its canonical
//! encoding.

/// How one flagged arm's body was shaped, restricted to the four `OD-RULES-008` names as
/// syntactically obvious without name or type resolution.
///
/// Deliberately closed: a shape this enum cannot name is a shape a `Syntactic`-level
/// provider does not flag, by construction — extending recognition to a new shape is a new
/// variant here, not a default arm quietly widening what "obvious" means.
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

/// One `Err(binding) => <body>` arm this file's provider found and could classify as one
/// of [`ArmShape`]'s four obvious-defect shapes.
///
/// Absence from a payload is not a claim of cleanliness — an arm whose body is anything
/// else is simply not recorded, the same "no finding is not confirmed clean" convention
/// every rule in this workspace already keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachabilitySite
{
    /// The nearest enclosing function's own name, unqualified. Tier-1's canonical subject
    /// is a control-flow edge inside one function body in one file; a fully qualified path
    /// through enclosing `impl`/`mod` blocks is real information a sound provider should
    /// carry, and is not attempted here.
    pub function: String,
    /// The identifier the `Err(...)` pattern binds. Not asserted to be `applicability` by
    /// this type — the provider that writes it is the one narrowing to that name.
    pub binding: String,
    pub shape: ArmShape,
}

/// One file's full set of flagged reachability sites.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachabilityPayload
{
    pub sites: Vec<ReachabilitySite>,
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

/// A payload could not be decoded under this schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadRefusal
{
    pub reason: String,
}

impl core::fmt::Display for PayloadRefusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Reads a payload back out of its canonical encoding.
///
/// An empty byte string is a valid payload — a file with no flagged sites at all — unlike
/// [`nomos_cap_dependency::Parse_Payload`], which always expects a leading `package` line.
/// This schema has no such header line to require.
///
/// # Errors
///
/// [`PayloadRefusal`] if the bytes are not valid UTF-8, or a line does not have exactly the
/// four tab-separated fields this schema declares.
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
    let Some(rest) = line.strip_prefix("site\t")
    else
    {
        return Err(PayloadRefusal {
            reason: format!("line is not a site: {line:?}"),
        });
    };

    let fields: Vec<&str> = rest.split('\t').collect();
    let [function, binding, shape] = fields.as_slice()
    else
    {
        return Err(PayloadRefusal {
            reason: format!("site line does not have exactly three fields: {line:?}"),
        });
    };

    let Some(shape) = ArmShape::From_Label(shape)
    else
    {
        return Err(PayloadRefusal {
            reason: format!("unrecognized arm shape: {shape:?}"),
        });
    };

    return Ok(ReachabilitySite {
        function: (*function).to_owned(),
        binding: (*binding).to_owned(),
        shape,
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

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

    #[test]
    fn Test_A_Payload_Should_Round_Trip_Through_Its_Own_Encoding()
    {
        let payload = Sample();
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_The_Encoding_Should_Be_Stable_And_Diffable()
    {
        let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("ASCII and tabs");

        assert_eq!(
            rendered,
            "site\tPayload_Of\tapplicability\tempty\n\
             site\tWalk_Sources\tapplicability\ttail-ok\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
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
        let bytes = b"site\tonly-one-field\n";
        assert!(Parse_Payload(bytes).is_err());
    }

    #[test]
    fn Test_An_Unrecognized_Shape_Should_Be_Refused()
    {
        let bytes = b"site\tf\tapplicability\tsomething-else\n";
        assert!(Parse_Payload(bytes).is_err());
    }
}
