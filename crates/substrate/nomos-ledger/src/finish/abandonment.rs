//! A claim given up, and what was said about it.

use serde::Deserialize;
use serde::Serialize;
use nomos_platform::Timestamp;
/// A claim somebody gave up on purpose, and why.
///
/// Kept on the item rather than in the transition that produced it, which is the whole
/// point. [`ItemState::Declined`] carries its reason and survives for exactly one reason:
/// it is part of a *state*, and states are what get written down. An abandonment is a
/// *transition*, and a transition leaves nothing behind unless something on the item is
/// given the job of holding it. This is that job.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Abandonment
{
    /// Who gave it up.
    pub holder: String,
    /// Why, in the words they gave.
    pub reason: String,
    /// When they gave it up.
    #[serde(serialize_with = "nomos_platform::timestamp_serde::Write_Unix_Seconds")]
    #[serde(deserialize_with = "nomos_platform::timestamp_serde::Read_Unix_Seconds")]
    pub abandoned_at: Timestamp,
}
