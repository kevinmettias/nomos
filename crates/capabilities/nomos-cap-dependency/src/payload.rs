//! The wire shape of a `nomos.dependency.edges.v1` payload, and its canonical encoding.

/// Which of the manifest's three dependency tables an edge was declared in.
///
/// Not a `bool`: a dev-dependency and a build-dependency are both distinct from a normal
/// one, and a rule that treats them alike is a rule choosing to, not a fact that could not
/// tell them apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyKind
{
    Normal,
    Dev,
    Build,
}

impl DependencyKind
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Normal => "normal",
            Self::Dev => "dev",
            Self::Build => "build",
        };
    }

    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            "normal" => Some(Self::Normal),
            "dev" => Some(Self::Dev),
            "build" => Some(Self::Build),
            _ => None,
        };
    }
}

/// One edge: this package names `target` as a dependency of kind `kind`, optionally.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyEdge
{
    /// The dependency's own package name, as Cargo resolved it — not the manifest's
    /// `package = "..."` rename, if one was given, because a rule comparing this against
    /// `bands.rs`'s table must compare against the same name the table is written in.
    pub target: String,
    pub kind: DependencyKind,
    pub optional: bool,
}

/// One package's full set of first-party dependency edges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyPayload
{
    pub package: String,
    pub edges: Vec<DependencyEdge>,
}

/// Encodes a payload as tab-separated lines, the same shape `nomos-cap-syntax` uses and
/// for the same two reasons: diffable by a person, and written in one place with no
/// derive between the data and the bytes.
#[must_use]
pub fn Encode_Payload(payload: &DependencyPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    encoded.push_str("package\t");
    encoded.push_str(&payload.package);
    encoded.push('\n');

    for edge in &payload.edges
    {
        encoded.push_str("edge\t");
        encoded.push_str(&edge.target);
        encoded.push('\t');
        encoded.push_str(edge.kind.Label());
        encoded.push('\t');
        encoded.push_str(if edge.optional { "optional" } else { "required" });
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
/// # Errors
///
/// [`PayloadRefusal`] if the bytes are not valid UTF-8, the first line does not name a
/// package, or an edge line does not have exactly the four fields this schema declares.
pub fn Parse_Payload(bytes: &[u8]) -> Result<DependencyPayload, PayloadRefusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| PayloadRefusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut lines = text.lines();
    let package = Package_Line(lines.next())?;
    let mut edges = Vec::new();

    for line in lines
    {
        edges.push(Edge_Line(line)?);
    }

    return Ok(DependencyPayload { package, edges });
}

fn Package_Line(line: Option<&str>) -> Result<String, PayloadRefusal>
{
    let Some(line) = line
    else
    {
        return Err(PayloadRefusal {
            reason: "empty payload; expected a package line".to_owned(),
        });
    };

    let Some(name) = line.strip_prefix("package\t")
    else
    {
        return Err(PayloadRefusal {
            reason: format!("first line is not a package declaration: {line:?}"),
        });
    };

    return Ok(name.to_owned());
}

fn Edge_Line(line: &str) -> Result<DependencyEdge, PayloadRefusal>
{
    let Some(rest) = line.strip_prefix("edge\t")
    else
    {
        return Err(PayloadRefusal {
            reason: format!("line is not an edge: {line:?}"),
        });
    };

    let fields: Vec<&str> = rest.split('\t').collect();
    let [target, kind, optionality] = fields.as_slice()
    else
    {
        return Err(PayloadRefusal {
            reason: format!("edge line does not have exactly three fields: {line:?}"),
        });
    };

    let Some(kind) = DependencyKind::From_Label(kind)
    else
    {
        return Err(PayloadRefusal {
            reason: format!("unrecognized dependency kind: {kind:?}"),
        });
    };

    let optional = match *optionality
    {
        "optional" => true,
        "required" => false,
        other =>
        {
            return Err(PayloadRefusal {
                reason: format!("unrecognized optionality: {other:?}"),
            });
        }
    };

    return Ok(DependencyEdge {
        target: (*target).to_owned(),
        kind,
        optional,
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Sample() -> DependencyPayload
    {
        return DependencyPayload {
            package: "nomos-rules".to_owned(),
            edges: vec![
                DependencyEdge {
                    target: "nomos-capability".to_owned(),
                    kind: DependencyKind::Normal,
                    optional: false,
                },
                DependencyEdge {
                    target: "nomos-analysis".to_owned(),
                    kind: DependencyKind::Dev,
                    optional: true,
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
            "package\tnomos-rules\n\
             edge\tnomos-capability\tnormal\trequired\n\
             edge\tnomos-analysis\tdev\toptional\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Be_Refused()
    {
        assert!(Parse_Payload(&[]).is_err());
    }

    #[test]
    fn Test_A_Malformed_Edge_Line_Should_Be_Refused()
    {
        let bytes = b"package\tsomething\nedge\tonly-one-field\n";
        assert!(Parse_Payload(bytes).is_err());
    }

    #[test]
    fn Test_A_Package_With_No_Edges_Should_Round_Trip()
    {
        let payload = DependencyPayload {
            package: "nomos-contracts".to_owned(),
            edges: Vec::new(),
        };
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("a package with no dependencies is valid");

        assert_eq!(decoded, payload);
    }
}
