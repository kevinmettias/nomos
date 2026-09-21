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
/// Re-addressed twice, deliberately. First by `P10-SYNTAX-V2`: `nomos.syntax.items.v2`
/// carries two more fields per item, so every fact keyed under the v1 bytes has a new
/// payload digest — `OD-SYNTAX-002` is the sentence that had to be read before that diff
/// merged. Second by `OD-CAPABILITY-010`: a named-field struct's `shape` now carries its
/// own field list instead of always `.`, and the shared fixture this golden is captured
/// over declares one, so every fact for a struct with named fields has a new payload
/// digest too.
pub(crate) const PARSED_GOLDEN: &str = "f5e3f679152a9eca336ae37d0315fadb";

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
/// `nomos-lang-go`'s golden, over [`crate::productions::GO_FIXTURE`]. Captured on this
/// platform when `nomos-lang-go` first declared `SyntaxFactProduction`, the same way every
/// other constant in this file was captured on the platform that first declared its domain.
///
/// Re-addressed by `OD-CAPABILITY-010`: `GO_FIXTURE`'s `Held` struct has a named field, so
/// its `shape` now carries real data instead of always `.`, the same re-addressing
/// [`PARSED_GOLDEN`] took for the identical reason on the Rust side.
pub(crate) const GO_GOLDEN: &str = "42a188d75caad7d13be97610fc619bbe";
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
///
/// Moved a third time by `P103-REPEATED-TEXT` (`6803c249`), and this one moved for the
/// simplest reason a bundle's bytes can move: a bundle declares the schema version it was
/// exported from. [`nomos_spec_bundle::Header`] carries `schema_version`, that migration
/// added migration 8 (`repeated-text-declarations`) to a list that held seven, and the header
/// is the first covered line of every bundle. So the digest had to move, and a digest that
/// had *not* moved would have been the defect -- it would mean a bundle exported under a new
/// schema was indistinguishable from one exported under the old.
///
/// Attributed rather than assumed, because a golden re-pinned to whatever the code now emits
/// is a golden that has stopped pinning anything. `cargo test -p nomos-integration-tests
/// --test determinism Bundle` passes at `9a7f3a35`, that commit's parent, producing exactly
/// the value this constant held before; it fails at `6803c249` producing exactly the value it
/// holds now. One commit, one cause, and the cause is the intended consequence of adding a
/// declared table rather than a corruption of what was already there.
///
/// Nothing else in the fixture changed: `P103-REPEATED-TEXT` touched no file under
/// `tests/integration`, and the new table is empty in this fixture, so not one record line
/// differs. The whole move is the header's version field.
pub(crate) const BUNDLE_GOLDEN: &str = "21b4117f8b5ec2bb8c54a2036dc5fff4";
pub(crate) const PROJECTION_GOLDEN: &str = "9cecf39961bbd638111f82382eafd643";
