//! [`ChoiceRecord`], the fields [`super::CorrectionChoice::New`] takes beyond `selected`.

use crate::{CorrectionId, RankingCriterion};

/// `COR-012`'s own five recorded fields, apart from `selected` itself -- grouped into one
/// value so [`super::CorrectionChoice::New`] stays within this crate's own parameter-count
/// limit.
pub struct ChoiceRecord
{
    pub objective_weights: Vec<(RankingCriterion, u32)>,
    pub rejected_alternatives: Vec<CorrectionId>,
    pub predicted_side_effects: Vec<String>,
    pub unresolved_tradeoffs: Vec<String>,
    pub verification_obligations: Vec<String>,
}
