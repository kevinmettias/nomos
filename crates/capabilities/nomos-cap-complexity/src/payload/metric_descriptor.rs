//! [`MetricDescriptor`], what a number means and how it may be read.

use super::aggregation::Aggregation;
use super::directionality::Directionality;

/// Every property MET-006 requires a metric descriptor to define, one field each: "Every metric
/// descriptor shall define unit, subject kinds, aggregation operator, weighting, normalization,
/// missing-data behavior, directionality, baseline requirements, uncertainty/statistical
/// treatment, and snapshot comparability."
///
/// Carried in every payload rather than implied by the capability's name, so a reader holding a
/// fact holds the terms it may read the number under, and MET-007's "A client may not aggregate
/// or blend a metric beyond the descriptor’s declared semantics" has something to be checked
/// against. The two properties a reader acts on -- how values combine and which way is worse --
/// are enums a reader can match on; the rest are stated as text because nothing yet reads them
/// mechanically, and a type invented for them now would be guessed rather than needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricDescriptor
{
    /// What one unit of the value counts.
    pub unit: String,
    /// What kind of thing the value is measured of.
    pub subject_kinds: String,
    /// Whether and how values may be combined across subjects.
    pub aggregation: Aggregation,
    /// How subjects are weighted when values are combined.
    pub weighting: String,
    /// Whether the value is normalized, and against what.
    pub normalization: String,
    /// What a subject with no value means.
    pub missing_data: String,
    /// Which way the value gets worse.
    pub directionality: Directionality,
    /// What, if anything, a value must be compared against to mean something.
    pub baseline: String,
    /// How exact the value is, and why it may be off.
    pub uncertainty: String,
    /// When two values of this metric, from two snapshots, may be compared.
    pub snapshot_comparability: String,
}
