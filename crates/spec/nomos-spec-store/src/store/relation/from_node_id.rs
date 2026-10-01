/// The identifier of a relation edge's source node.
///
/// Distinct from `ToNodeId` even though both carry a node identifier, because the two
/// sit next to each other at every edge-writing call site — `Write_Relation(from, type, to)`
/// — and a position is not a name. Naming the role rather than leaving both `&str` is what
/// keeps a transposed pair a compile error instead of a silently reversed edge.
#[derive(Clone, Copy, Debug)]
pub struct FromNodeId<'a>(pub &'a str);

impl<'a> From<&'a str> for FromNodeId<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for FromNodeId<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
