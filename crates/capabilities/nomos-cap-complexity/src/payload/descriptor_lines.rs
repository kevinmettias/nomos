//! A [`MetricDescriptor`] as the ten `descriptor` lines that carry it, both ways.

use super::aggregation::Aggregation;
use super::directionality::Directionality;
use super::metric_descriptor::MetricDescriptor;
use super::payload_refusal::PayloadRefusal;

/// Every property of `descriptor` under the name its line carries, in MET-006's own order.
pub(super) fn Fields_Of(descriptor: &MetricDescriptor) -> [(&'static str, String); 10]
{
    return [
        ("unit", descriptor.unit.clone()),
        ("subject-kinds", descriptor.subject_kinds.clone()),
        ("aggregation", descriptor.aggregation.Label().to_owned()),
        ("weighting", descriptor.weighting.clone()),
        ("normalization", descriptor.normalization.clone()),
        ("missing-data", descriptor.missing_data.clone()),
        ("directionality", descriptor.directionality.Label().to_owned()),
        ("baseline", descriptor.baseline.clone()),
        ("uncertainty", descriptor.uncertainty.clone()),
        ("snapshot-comparability", descriptor.snapshot_comparability.clone()),
    ];
}

/// The descriptor lines read so far, each property at most once.
///
/// A payload missing any one of the ten is refused rather than completed with a guess: a
/// descriptor with a property left out is the reading MET-007 forbids a client to supply for
/// itself.
#[derive(Default)]
pub(super) struct DescriptorLines
{
    unit: Option<String>,
    subject_kinds: Option<String>,
    aggregation: Option<Aggregation>,
    weighting: Option<String>,
    normalization: Option<String>,
    missing_data: Option<String>,
    directionality: Option<Directionality>,
    baseline: Option<String>,
    uncertainty: Option<String>,
    snapshot_comparability: Option<String>,
}

impl DescriptorLines
{
    /// Reads one `descriptor` line's field and value, `rest` being what follows its tag.
    pub(super) fn Read_Line(&mut self, rest: &str, line: &str) -> Result<(), PayloadRefusal>
    {
        let Some((field, value)) = rest.split_once('\t')
        else
        {
            return Err(Refusal(&format!("descriptor line does not have a field and a value: {line:?}")));
        };

        return match field
        {
            "unit" => Set_Once(&mut self.unit, value.to_owned(), line),
            "subject-kinds" => Set_Once(&mut self.subject_kinds, value.to_owned(), line),
            "aggregation" => Set_Once(&mut self.aggregation, Label(Aggregation::From_Label(value), line)?, line),
            "weighting" => Set_Once(&mut self.weighting, value.to_owned(), line),
            "normalization" => Set_Once(&mut self.normalization, value.to_owned(), line),
            "missing-data" => Set_Once(&mut self.missing_data, value.to_owned(), line),
            "directionality" => Set_Once(&mut self.directionality, Label(Directionality::From_Label(value), line)?, line),
            "baseline" => Set_Once(&mut self.baseline, value.to_owned(), line),
            "uncertainty" => Set_Once(&mut self.uncertainty, value.to_owned(), line),
            "snapshot-comparability" => Set_Once(&mut self.snapshot_comparability, value.to_owned(), line),
            _ => Err(Refusal(&format!("descriptor line names no MET-006 property: {line:?}"))),
        };
    }

    /// The descriptor the lines read describe, or a refusal naming the first property no line
    /// declared.
    pub(super) fn Into_Descriptor(self) -> Result<MetricDescriptor, PayloadRefusal>
    {
        return Ok(MetricDescriptor {
            unit: Declared(self.unit, "unit")?,
            subject_kinds: Declared(self.subject_kinds, "subject-kinds")?,
            aggregation: Declared(self.aggregation, "aggregation")?,
            weighting: Declared(self.weighting, "weighting")?,
            normalization: Declared(self.normalization, "normalization")?,
            missing_data: Declared(self.missing_data, "missing-data")?,
            directionality: Declared(self.directionality, "directionality")?,
            baseline: Declared(self.baseline, "baseline")?,
            uncertainty: Declared(self.uncertainty, "uncertainty")?,
            snapshot_comparability: Declared(self.snapshot_comparability, "snapshot-comparability")?,
        });
    }
}

/// Stores `value` in `slot` unless an earlier line already did: two lines for one property are
/// two answers to one question, and neither is chosen.
fn Set_Once<Value>(slot: &mut Option<Value>, value: Value, line: &str) -> Result<(), PayloadRefusal>
{
    if slot.is_some()
    {
        return Err(Refusal(&format!("descriptor property declared twice: {line:?}")));
    }

    *slot = Some(value);
    return Ok(());
}

/// A label's parsed value, or a refusal naming the line whose label was not one.
fn Label<Value>(parsed: Option<Value>, line: &str) -> Result<Value, PayloadRefusal>
{
    return parsed.ok_or_else(|| return Refusal(&format!("unrecognized descriptor label: {line:?}")));
}

fn Declared<Value>(slot: Option<Value>, field: &str) -> Result<Value, PayloadRefusal>
{
    return slot.ok_or_else(|| return Refusal(&format!("no descriptor line declares {field}")));
}

fn Refusal(reason: &str) -> PayloadRefusal
{
    return PayloadRefusal { reason: reason.to_owned() };
}
