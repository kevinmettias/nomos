//! Digests captured on one platform, for the scopes whose claim spans platforms.
//!
//! A `CrossPlatform` claim is verified by capturing reference output on one platform and
//! comparing from the others. In a repository the other platforms are not present to ask,
//! so the capture is committed and every platform's CI compares against it — which is the
//! same instrument, with the reference stored rather than fetched.
//!
//! A legitimate change to an encoding changes these, and that is the point: the diff then
//! says "every fact ever keyed under the old bytes has been re-addressed", which is a
//! sentence somebody should have to read before merging.

/// The parser's golden.
///
/// Re-addressed once, deliberately, by `P10-SYNTAX-V2`: `nomos.syntax.items.v2` carries two
/// more fields per item, so every fact keyed under the v1 bytes has a new payload digest.
/// `OD-SYNTAX-002` is the sentence that had to be read before this diff merged.
pub(crate) const PARSED_GOLDEN: &str = "6ec4ad9fe9c81ab2358e3c3756a3f1f5";

/// The rollup's golden.
///
/// It pins more than a payload encoding. The bytes carry the derived fact's key, the index,
/// and every dependency edge with its outcome — so this constant moves if the member
/// ordering changes, if the edge order changes, or if what counts as a read changes. Each of
/// those is a change to what a later invalidation will reach, which is worth a sentence in a
/// diff.
pub(crate) const ROLLED_GOLDEN: &str = "a9dc834595e753e498f3a981020b2214";

/// The reachability offer's golden, over the fixture that finds no site to flag.
pub(crate) const REACHABILITY_GOLDEN: &str = "45ad5b30ad5a0a0299749b362f65fe83";
pub(crate) const SCANNED_GOLDEN: &str = "e692ad97796279579ca5cd77764e08e5";
pub(crate) const SNAPSHOT_GOLDEN: &str = "1fb5fb67d666b0bb983f3b71e7e09f93";

/// The bundle's golden, and the one whose scope claim reaches furthest.
///
/// `CrossBinary` is verified by capturing a reference from one build and comparing from
/// others. The reference is this constant, and what it is worth is bounded by that: it is
/// compared by every later build of this workspace, which is a real comparison across
/// recompilation and is not yet a comparison across compiler versions.
/// `docs/records/OD-DETERMINISM-002` says so rather than implying more.
///
/// It also pins the store's schema version, which travels in the bundle header. A
/// migration therefore moves this constant, and that is right rather than unfortunate:
/// a bundle written under one schema and read under another is exactly the interchange
/// case the `CrossBinary` claim is about, and the diff is where somebody says so.
/// Moved by `P10-SUBMISSION-LAYOUT` (`86971ce`), which is the case the paragraph above
/// describes rather than an exception to it.
///
/// That commit added the `submissions`, `submission_values` and `submission_gaps` tables as
/// store schema migration 6 and extended the exporter to cover them, so both halves of what
/// this pins moved: the schema version travelling in the bundle header, and the column
/// coverage the export asserts. The first failure was not a digest mismatch at all — it was
/// `UncoveredColumn { table: "submissions", column: "uid" }`, the exporter refusing to write
/// a bundle it could not fully describe, which is that check working.
///
/// Recaptured here rather than by that commit's author because they could not: the item's
/// territory is the four `crates/spec` crates and its predicate does not reach
/// `nomos-integration-tests`, so nothing they ran could see this constant and nothing they
/// were entitled to write could change it. `OD-LEDGER-003` decided that a per-item predicate
/// stays narrower than the gate and that the gate at push time is what closes the gap, so
/// this is the designed outcome rather than a defect somebody let through — and this comment
/// is the sentence that decision expects somebody to read.
///
/// Moved a second time by `P13-SPEC-PRODUCTIONS-FIXTURE-2`. `OD-SPEC-012`/`P10-EDGE-CONSTRAINTS-2`
/// added `relation_types.domain_kinds_json`, `range_kinds_json` and `max_per_node`
/// (`NOT NULL`, no default); `tests/integration/tests/determinism/spec_productions.rs`'s
/// `SPEC_RELATIONS` fixture had never learned the two JSON columns, so its raw
/// `INSERT INTO relation_types` failed its `NOT NULL` constraint before `Export` ever ran —
/// this golden was never actually being compared against that fixture's real bytes, only
/// pinned at whatever `Export` had last produced before those columns existed. Supplying the
/// three columns (matching `Put_Relation_Type`'s own shape, the same pattern
/// `P13-GRAPH-EDGES-FIXTURE` already used for `nomos-spec-project`'s sibling fixture) let the
/// bundle build for the first time under the current schema, and its bytes now include the
/// domain/range/cardinality `SpecificationBundle`'s `RelationType` carries per
/// `OD-SPEC-012` decision 6 — a real encoding change this constant had not yet seen.
pub(crate) const BUNDLE_GOLDEN: &str = "6d4589efb6920ec2294231ef235b63cf";
pub(crate) const PROJECTION_GOLDEN: &str = "9cecf39961bbd638111f82382eafd643";
