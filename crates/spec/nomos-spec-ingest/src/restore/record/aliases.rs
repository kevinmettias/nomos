//! Naming a restored node, and leaving a name alone when it is not this restoration's to give.
//!
//! An alias is the corpus's own word for a node, so these decide whether the name the
//! documents actually use resolves. They came out of `record.rs` when it crossed the
//! file-size review trigger, and their tests came with them -- a Rust unit test's companion
//! is the file the test is written in, so a function and its test have to move together.

use super::super::{BTreeMap, IngestError, Member, RestorationReport, SpecificationStore};

/// Points an alias at the node, unless the name is contested.
///
/// A name more than one restored member claims is left unclaimed, and one another authority
/// already owns is reported and left where it is — a restoration may not repoint a name it
/// did not mint.
pub(super) fn Claim_Alias(
    store: &mut SpecificationStore,
    member: &Member,
    node_uid: i64,
    report: &mut RestorationReport,
) -> Result<(), IngestError>
{
    let Some(alias) = &member.alias
    else
    {
        return Ok(());
    };
    if report.ambiguous_names.contains(alias)
    {
        return Ok(());
    }

    if !Alias_Resolves_To(store, alias, node_uid)?
    {
        report.contested_aliases.push(alias.clone());
    }

    return Ok(());
}

/// Names claimed by more than one restored member.
///
/// Reported and withheld rather than resolved by a rule such as "the domain model wins".
/// Two nodes really do carry the name; picking one is an answer the corpus does not give.
pub(super) fn Ambiguous_Names(members: &[Member]) -> Vec<String>
{
    let mut claims: BTreeMap<&str, u32> = BTreeMap::new();
    for alias in members.iter().filter_map(|member| return member.alias.as_deref())
    {
        let counter = claims.entry(alias).or_insert(0);
        *counter = counter.saturating_add(1);
    }

    return claims
        .into_iter()
        .filter(|(_, claimed)| return *claimed > 1)
        .map(|(alias, _)| return alias.to_owned())
        .collect();
}

/// Whether the alias now resolves to this node.
///
/// `false` where something else already owns it. Reported rather than ignored: an alias
/// silently pointing at another node makes "resolve by name" answer confidently and
/// wrongly, which is worse than not resolving at all.
pub(super) fn Alias_Resolves_To(store: &SpecificationStore, alias: &str, node_uid: i64) -> Result<bool, IngestError>
{
    use super::Sql_Result;

    let inserted = store.Connection().execute(
        "INSERT OR IGNORE INTO node_aliases (alias, node_uid) VALUES (?1, ?2)",
        rusqlite::params![alias, node_uid],
    );
    Sql_Result(inserted)?;

    let selected_owner = store.Connection().query_row(
        "SELECT node_uid FROM node_aliases WHERE alias = ?1",
        rusqlite::params![alias],
        |row| row.get(0),
    );
    let owner: i64 = Sql_Result(selected_owner)?;

    return Ok(owner == node_uid);
}

#[cfg(test)]
mod tests
{
    use super::super::{A_Located_Member_With_Its_Node, LocatedMemberWithNode, NodeRow, SpecificationStore};
    use super::*;
    use crate::Origin;

    #[test]
    fn Test_Claim_Alias_Should_Point_The_Alias_At_The_Node_Unless_It_Is_Ambiguous()
    {
        let LocatedMemberWithNode { mut store, member, node_uid, .. } = A_Located_Member_With_Its_Node();
        let alias = member.alias.clone().expect("the domain model row carries an alias");

        let mut ambiguous_report = RestorationReport {
            ambiguous_names: vec![alias.clone()],
            ..RestorationReport::default()
        };
        Claim_Alias(&mut store, &member, node_uid, &mut ambiguous_report)
            .expect("the domain model's name is unclaimed, so the alias lands on its node");
        assert!(ambiguous_report.contested_aliases.is_empty());
        let claimed = Claimed_Alias_Count(&store, &alias);
        assert_eq!(claimed, 0, "an ambiguous alias must not be claimed");

        let mut report = RestorationReport::default();
        Claim_Alias(&mut store, &member, node_uid, &mut report)
            .expect("the domain model's name is unclaimed, so the alias lands on its node");
        assert!(report.contested_aliases.is_empty());
        let owner = Alias_Owner(&store, &alias);
        assert_eq!(owner, node_uid);
    }

    fn Claimed_Alias_Count(store: &SpecificationStore, alias: &str) -> i64
    {
        let claimed: i64 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM node_aliases WHERE alias = ?1",
                rusqlite::params![&alias],
                |row| row.get(0),
            )
            .expect("the count runs over the store's own alias table, which the claim just wrote");

        return claimed;
    }

    fn Alias_Owner(store: &SpecificationStore, alias: &str) -> i64
    {
        let owner: i64 = store
            .Connection()
            .query_row(
                "SELECT node_uid FROM node_aliases WHERE alias = ?1",
                rusqlite::params![&alias],
                |row| row.get(0),
            )
            .expect("reads the alias");

        return owner;
    }

    #[test]
    fn Test_Ambiguous_Names_Should_Count_Aliases_Claimed_By_More_Than_One_Member()
    {
        let shared = Shared_Alias_Member();
        let other = Member {
            id: "GLS-SHARED".to_owned(),
            alias: Some("Shared".to_owned()),
            ..shared.clone()
        };
        let unique = Member {
            id: "CDM-Y".to_owned(),
            name: "Y".to_owned(),
            alias: Some("Unique".to_owned()),
            ..shared.clone()
        };

        let ambiguous = Ambiguous_Names(&[shared, other, unique]);

        assert_eq!(ambiguous, vec!["Shared".to_owned()]);
    }

    fn Shared_Alias_Member() -> Member
    {
        use crate::Restored;

        let shared = Member {
            id: "CDM-X".to_owned(),
            family: Restored::CanonicalDomainModel,
            name: "X".to_owned(),
            document: "02-core.md".to_owned(),
            origin: Origin::Row {
                block_ordinal: 1,
                row_ordinal: 1,
            },
            alias: Some("Shared".to_owned()),
        };

        return shared;
    }

    #[test]
    fn Test_Alias_Resolves_To_Should_Report_False_When_Another_Node_Already_Owns_It()
    {
        let mut store =
            SpecificationStore::In_Memory().expect("an in-memory store opens over no file, so this has no failure path");
        let first = store
            .Upsert_Node(NodeRow {
                node_id: "CDM-A",
                kind: "concept",
                authority: "canonical",
                representation: "record",
                title: "A",
            })
            .expect("CDM-A is an identifier this test has not minted, so the upsert writes a row");
        let second = store
            .Upsert_Node(NodeRow {
                node_id: "CDM-B",
                kind: "concept",
                authority: "canonical",
                representation: "record",
                title: "B",
            })
            .expect("CDM-B is an identifier this test has not minted, so the upsert writes a row");

        assert!(Alias_Resolves_To(&store, "Shared", first).expect("the alias table is empty, so the claim wins"));
        assert!(
            !Alias_Resolves_To(&store, "Shared", second).expect("the alias table answers the owner it holds"),
            "a second node was allowed to take a claimed alias"
        );
    }
}
