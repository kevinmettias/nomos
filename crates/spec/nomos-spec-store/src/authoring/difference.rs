//! Working out what an edit changed, before anything is written.

use nomos_spec_model::{
    ContentHash, Normalize_Whitespace, Record, RecordFrontMatter, RecordRelation,
    Render_Record, Segment, SourceBlock,
};

use crate::BlockChange;
use crate::IdentityChange;
use crate::NormativeOutcome;
use crate::Kind_Label;

use super::RelationChanges;

/// What a staged block turned out to be against the blocks that were already there.
enum Match
{
    /// One of them is this block, and this is what became of it — nothing worth reporting,
    /// where it neither moved nor was reflowed.
    Held(Option<BlockChange>),
    /// Nothing before this edit carries this wording.
    New,
}

/// What became of each block, before and after.
pub(crate) fn Block_Changes(before: &[SourceBlock], after: &[SourceBlock]) -> Vec<BlockChange>
{
    let mut taken = vec![false; before.len()];
    let mut changes = Vec::new();
    let mut added: Vec<&SourceBlock> = Vec::new();

    for block in after
    {
        match Claimed_Block(before, &mut taken, block)
        {
            Match::Held(change) => changes.extend(change),
            Match::New => added.push(block),
        }
    }

    let removed = Untaken_Blocks(before, &taken);
    let paired = Paired_Changes(&added, &removed);

    changes.extend(paired);
    changes.sort_by_key(Position_Of);

    return changes;
}

/// Claims the block this staged one is — verbatim first, then under the normalizer — so that
/// no earlier block is matched twice, and says what became of it.
fn Claimed_Block(before: &[SourceBlock], taken: &mut [bool], block: &SourceBlock) -> Match
{
    if let Some(index) = Unmatched_Index(before, taken, |candidate| return candidate.text == block.text)
    {
        Mark_Taken(taken, index);
        let found = before.get(index);
        let moved = Displaced_Block(found, block.ordinal);

        return Match::Held(moved);
    }
    if let Some(index) = Unmatched_Index(before, taken, |candidate| {
        return Normalize_Whitespace(&candidate.text) == Normalize_Whitespace(&block.text);
    })
    {
        Mark_Taken(taken, index);

        return Match::Held(Some(BlockChange::Reflowed {
            ordinal: block.ordinal,
        }));
    }

    return Match::New;
}

fn Unmatched_Index(
    before: &[SourceBlock],
    taken: &[bool],
    predicate: impl Fn(&SourceBlock) -> bool,
) -> Option<usize>
{
    for (index, block) in before.iter().enumerate()
    {
        if Is_Free(taken, index) && predicate(block)
        {
            return Some(index);
        }
    }

    return None;
}

fn Mark_Taken(taken: &mut [bool], index: usize)
{
    if let Some(slot) = taken.get_mut(index)
    {
        *slot = true;
    }
}

/// The same wording at a different ordinal moved. At the same ordinal, nothing happened to
/// it, and reporting that would be noise in a preview an author has to read every time.
fn Displaced_Block(found: Option<&SourceBlock>, to: u32) -> Option<BlockChange>
{
    return found
        .filter(|block| return block.ordinal != to)
        .map(|block| {
            return BlockChange::Moved {
                from: block.ordinal,
                to,
            };
        });
}

/// The blocks no staged block claimed, which are the ones the edit removed.
fn Untaken_Blocks<'a>(before: &'a [SourceBlock], taken: &[bool]) -> Vec<&'a SourceBlock>
{
    return before
        .iter()
        .enumerate()
        .filter(|(index, _)| return Is_Free(taken, *index))
        .map(|(_, block)| return block)
        .collect();
}

/// An addition and a removal at one ordinal are a rewording, and saying so is more use to a
/// reader than two lines that do not mention each other.
fn Paired_Changes(added: &[&SourceBlock], removed: &[&SourceBlock]) -> Vec<BlockChange>
{
    let mut changes = Vec::new();

    for block in added
    {
        let change = Added_Or_Reworded(block, removed);
        changes.push(change);
    }
    for gone in removed
    {
        if !added.iter().any(|block| return block.ordinal == gone.ordinal)
        {
            changes.push(BlockChange::Removed {
                ordinal: gone.ordinal,
                kind: Kind_Label(gone.kind).to_owned(),
            });
        }
    }

    return changes;
}

/// An addition at an ordinal something was removed from is that block, reworded.
fn Added_Or_Reworded(block: &SourceBlock, removed: &[&SourceBlock]) -> BlockChange
{
    let Some(gone) = removed.iter().find(|gone| return gone.ordinal == block.ordinal)
    else
    {
        return BlockChange::Added {
            ordinal: block.ordinal,
            kind: Kind_Label(block.kind).to_owned(),
        };
    };

    return BlockChange::Reworded {
        ordinal: block.ordinal,
        before: ContentHash::Of(&gone.text).As_String_Slice().to_owned(),
        after: ContentHash::Of(&block.text).As_String_Slice().to_owned(),
    };
}

const fn Position_Of(change: &BlockChange) -> u32
{
    return match change
    {
        BlockChange::Added { ordinal, .. }
        | BlockChange::Removed { ordinal, .. }
        | BlockChange::Reworded { ordinal, .. }
        | BlockChange::Reflowed { ordinal } => *ordinal,
        BlockChange::Moved { to, .. } => *to,
    };
}

/// Which front matter fields the edit changes.
pub(crate) fn Identity_Changes(before: &RecordFrontMatter, after: &RecordFrontMatter) -> Vec<IdentityChange>
{
    let mut changes = Vec::new();

    for (field, was, now) in [
        ("type", before.kind.clone(), after.kind.clone()),
        ("title", before.title.clone(), after.title.clone()),
        ("status", before.status.clone(), after.status.clone()),
        ("authority", before.authority.clone(), after.authority.clone()),
        ("version", before.version.to_string(), after.version.to_string()),
        ("tags", before.tags.join(", "), after.tags.join(", ")),
    ]
    {
        if was != now
        {
            changes.push(IdentityChange {
                field: field.to_owned(),
                before: was,
                after: now,
            });
        }
    }

    return changes;
}

pub(crate) fn Relation_Changes(before: &[RecordRelation], after: &[RecordRelation]) -> RelationChanges
{
    let added = after
        .iter()
        .filter(|relation| return !before.contains(relation))
        .cloned()
        .collect();
    let removed = before
        .iter()
        .filter(|relation| return !after.contains(relation))
        .cloned()
        .collect();

    return RelationChanges { added, removed };
}

/// Where a statement's canonical text sits before and after the edit.
///
/// Compared under the normalizer, because that is what decides whether two spellings are one
/// statement everywhere else in this system. A statement whose block was merely reflowed is
/// held, not moved.
pub(crate) fn Located_Statement(canonical_text: &str, before: &[SourceBlock], after: &[SourceBlock]) -> NormativeOutcome
{
    let needle = Normalize_Whitespace(canonical_text);
    let carrying = |blocks: &[SourceBlock]| {
        return blocks
            .iter()
            .find(|block| return Normalize_Whitespace(&block.text).contains(&needle))
            .map(|block| return block.ordinal);
    };

    return match (carrying(before), carrying(after))
    {
        (None, _) => NormativeOutcome::Unlocatable,
        (Some(from), None) => NormativeOutcome::Gone { from },
        (Some(from), Some(to)) if from == to => NormativeOutcome::Held { block: from },
        (Some(from), Some(to)) => NormativeOutcome::Moved { from, to },
    };
}

/// Why the staged text is not what this surface would have written.
pub(crate) fn Why_Not_Canonical(markdown: &str, record: &Record) -> String
{
    return match Render_Record(&record.front_matter, &Segment(&record.body))
    {
        Err(error) => error.to_string(),
        Ok(rendered) => First_Difference(Expected(&rendered), Found(markdown)),
    };
}

/// What this surface would have written.
struct Expected<'a>(&'a str);
/// What the staged text actually holds.
struct Found<'a>(&'a str);

fn First_Difference(expected: Expected<'_>, found: Found<'_>) -> String
{
    let expected = expected.0;
    let found = found.0;

    for (index, (left, right)) in expected.lines().zip(found.lines()).enumerate()
    {
        if left != right
        {
            return format!(
                "line {}: this surface writes {left:?} and the staged text has {right:?}",
                index.saturating_add(1)
            );
        }
    }

    let (written, staged) = (expected.lines().count(), found.lines().count());
    if written != staged
    {
        return format!("this surface writes {written} line(s) and the staged text has {staged}");
    }

    return "every line matches, so the difference is in the leading or trailing whitespace"
        .to_owned();
}

/// Whether the block at this index is still unclaimed.
///
/// An index past the end reads as taken, so a walk that runs off the slice matches nothing
/// rather than pairing a change with a block that is not there.
fn Is_Free(taken: &[bool], index: usize) -> bool
{
    return !taken.get(index).copied().unwrap_or(true);
}
