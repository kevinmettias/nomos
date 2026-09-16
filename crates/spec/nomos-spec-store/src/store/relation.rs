//! What a relation type constrains, and the checks its registration must pass.

use crate::StoreError;

mod from_node_id;
mod inverse_relation_type;
mod constraint;
mod tier;
mod type_name;
mod to_node_id;

pub use from_node_id::FromNodeId;
pub use inverse_relation_type::InverseRelationType;
pub use constraint::Constraint;
pub use tier::Tier;
pub use type_name::TypeName;
pub use to_node_id::ToNodeId;

/// The constraint declares something at every end, or the refusal names the type that
/// declared nothing.
pub(super) fn Assert_Constraint_Is_Declared(name: &str, constraint: &Constraint<'_>) -> Result<(), StoreError>
{
    let is_unconstrained = constraint.domain.is_empty() || constraint.range.is_empty() || constraint.max_per_node == 0;
    if is_unconstrained
    {
        return Err(StoreError::UnconstrainedRelationType { name: name.to_owned() });
    }

    return Ok(());
}

/// A set of node kinds, sorted and serialized.
///
/// Sorted before serializing so the same set of kinds always writes the same bytes,
/// regardless of the order a caller happened to list them in — the bundle's byte-identical
/// round trip depends on it exactly the way it depends on every other ordering in this
/// store being by natural key rather than by insertion order.
pub(super) fn Sorted_Kinds_Json(kinds: &[&str]) -> Result<String, StoreError>
{
    let mut sorted: Vec<&str> = kinds.to_vec();
    sorted.sort_unstable();

    return serde_json::to_string(&sorted).map_err(|error| return StoreError::Sql(error.to_string()));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::NodeRow;
    use crate::SpecificationStore;
    use super::super::AUTHORED;

    struct TestNodeId<'a>(&'a str);
    struct TestNodeKind<'a>(&'a str);

    /// The cardinality these tests declare for the `joins` relation type.
    const JOINS_MAX_PER_NODE: u32 = 4;

    #[test]
    fn Test_Assert_Constraint_Is_Declared_Should_Refuse_An_Empty_Domain_Range_Or_Cardinality()
    {
        assert!(
            Assert_Constraint_Is_Declared(
                "t",
                &Constraint { domain: &[], range: &["w"], max_per_node: 1 }
            )
            .is_err()
        );
        assert!(
            Assert_Constraint_Is_Declared(
                "t",
                &Constraint { domain: &["w"], range: &[], max_per_node: 1 }
            )
            .is_err()
        );
        assert!(
            Assert_Constraint_Is_Declared(
                "t",
                &Constraint { domain: &["w"], range: &["w"], max_per_node: 0 }
            )
            .is_err()
        );
        assert!(
            Assert_Constraint_Is_Declared(
                "t",
                &Constraint { domain: &["w"], range: &["w"], max_per_node: 1 }
            )
            .is_ok()
        );
    }

    #[test]
    fn Test_Sorted_Kinds_Json_Should_Serialize_Kinds_In_Sorted_Order()
    {
        assert_eq!(
            Sorted_Kinds_Json(&["widget", "gadget"]).expect("serializes"),
            "[\"gadget\",\"widget\"]"
        );
        assert_eq!(
            Sorted_Kinds_Json(&["gadget", "widget"]).expect("serializes"),
            "[\"gadget\",\"widget\"]"
        );
    }

    /// `OD-SPEC-012`: a relation type declaring nothing is refused where it is registered,
    /// not left to write an edge that later discovers there was nothing to check.
    #[test]
    fn Test_Registering_A_Relation_Type_With_No_Domain_Should_Be_Refused()
    {
        let mut store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");

        let refusal = store.Put_Relation_Type(
            "nothing",
            "seed",
            &Constraint { domain: &[], range: &["widget"], max_per_node: 1 },
        );
        let message = format!("{refusal:?}");

        assert!(
            matches!(refusal, Err(StoreError::UnconstrainedRelationType { ref name }) if name == "nothing"),
            "an empty domain must be refused at registration: {message}"
        );
    }

    #[test]
    fn Test_Registering_A_Relation_Type_With_Zero_Cardinality_Should_Be_Refused()
    {
        let mut store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");

        let refusal = store.Put_Relation_Type(
            "nothing",
            "seed",
            &Constraint { domain: &["widget"], range: &["widget"], max_per_node: 0 },
        );

        assert!(
            matches!(refusal, Err(StoreError::UnconstrainedRelationType { .. })),
            "a zero cardinality must be refused at registration: {refusal:?}"
        );
    }

    /// The refusal names the type, the endpoint that failed, its kind, and the role it
    /// failed — everything `done_when` asks a domain/range refusal to say.
    #[test]
    fn Test_An_Edge_Whose_Endpoint_Kind_Is_Not_Admitted_Should_Be_Refused_By_Name()
    {
        let mut store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");
        Insert_Node(&mut store, TestNodeId("A"), TestNodeKind("widget"));
        Insert_Node(&mut store, TestNodeId("B"), TestNodeKind("gadget"));
        Put_Joins_Relation_Type(&mut store, JOINS_MAX_PER_NODE);

        let error = store.Put_Relation("A", "joins", "B").expect_err("B is a gadget, not a widget");

        let StoreError::RelationEndpoint { relation_type, role, node_id, kind, admits } = error
        else
        {
            // Test-only: the refusal must be this exact variant for the assertions below to
            // check anything, so a different variant is a defect in the fixture, not a case
            // the test needs to keep running for.
            panic!("wrong variant: {error:?}");
        };
        assert_eq!(relation_type, "joins", "which type");
        assert_eq!(role, "range", "which end");
        assert_eq!(node_id, "B", "which endpoint");
        assert_eq!(kind, "gadget", "what it is");
        assert_eq!(admits, vec!["widget".to_owned()], "what would satisfy it");
    }

    /// The refusal names the type, the node that is full, and the cap it declared.
    #[test]
    fn Test_An_Edge_Past_The_Declared_Cardinality_Should_Be_Refused_By_Name()
    {
        let mut store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");
        Insert_Node(&mut store, TestNodeId("A"), TestNodeKind("widget"));
        Insert_Node(&mut store, TestNodeId("B"), TestNodeKind("widget"));
        Insert_Node(&mut store, TestNodeId("C"), TestNodeKind("widget"));
        Put_Joins_Relation_Type(&mut store, 1);
        store.Put_Relation("A", "joins", "B").expect("the first edge fits the cap of 1");

        let error =
            store.Put_Relation("A", "joins", "C").expect_err("a second edge exceeds the cap of 1");

        let StoreError::RelationCardinality { relation_type, node_id, max_per_node } = error
        else
        {
            // Test-only: the refusal must be this exact variant for the assertions below to
            // check anything, so a different variant is a defect in the fixture, not a case
            // the test needs to keep running for.
            panic!("wrong variant: {error:?}");
        };
        assert_eq!(relation_type, "joins", "which type");
        assert_eq!(node_id, "A", "which node is full");
        assert_eq!(max_per_node, 1, "what cap it declared");
    }

    /// Re-ingesting the same edge is idempotent, the same property `INSERT OR IGNORE`
    /// already gives every other edge — it must not count a second time against the cap.
    #[test]
    fn Test_Re_Writing_The_Same_Edge_Should_Not_Count_Against_Cardinality()
    {
        let mut store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");
        Insert_Node(&mut store, TestNodeId("A"), TestNodeKind("widget"));
        Insert_Node(&mut store, TestNodeId("B"), TestNodeKind("widget"));
        Put_Joins_Relation_Type(&mut store, 1);
        store.Put_Relation("A", "joins", "B").expect("joins admits widget at both ends, so this first edge fits");

        store.Put_Relation("A", "joins", "B").expect("an idempotent re-write must not refuse");
    }

    /// A placeholder minted by [`SpecificationStore::Reference_Node`] carries no real kind
    /// yet, so the domain/range check defers rather than judging it against the sentinel.
    #[test]
    fn Test_A_Placeholder_Endpoint_Should_Be_Exempt_From_The_Kind_Check()
    {
        let mut store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");
        Insert_Node(&mut store, TestNodeId("A"), TestNodeKind("widget"));
        store.Reference_Node("B").expect("mints a placeholder");
        Put_Joins_Relation_Type(&mut store, JOINS_MAX_PER_NODE);

        store
            .Put_Relation("A", "joins", "B")
            .expect("a placeholder endpoint is exempt from the range check");
    }

    fn Insert_Node(store: &mut SpecificationStore, node_id: TestNodeId<'_>, kind: TestNodeKind<'_>)
    {
        store
            .Upsert_Node(NodeRow {
                node_id: node_id.0,
                kind: kind.0,
                authority: AUTHORED,
                representation: "record",
                title: node_id.0,
            })
            .expect("mints a node");
    }

    /// The `joins` relation type, admitting only `widget` at either end, at the cardinality
    /// the caller asks for.
    fn Put_Joins_Relation_Type(store: &mut SpecificationStore, max_per_node: u32)
    {
        store
            .Put_Relation_Type(
                "joins",
                "seed",
                &Constraint { domain: &["widget"], range: &["widget"], max_per_node },
            )
            .expect("the joins type declares a domain, a range and a nonzero cap");
    }
}
