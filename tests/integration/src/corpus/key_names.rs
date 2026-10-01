//! Naming fact keys against the corpus they are about.
//!
//! For diagnostics, and it reads no slice. An invalidation assertion that names the subjects
//! it reached is the difference between "two facts recomputed" — which is satisfied by
//! recomputing the wrong two — and a claim worth making.

use crate::Corpus;
use nomos_analysis::FactKey;

/// The subjects named by a set of invalidated keys, resolved back to corpus paths.
///
/// Keys carry digests, and an assertion that compares digests is an assertion nobody
/// can read when it fails. This maps them back through the corpus so a failure says
/// `alpha/one.rs` rather than thirty-two hex characters.
#[must_use]
pub fn Name_Keys(corpus: &Corpus, keys: &[FactKey]) -> Vec<String>
{
    let mut named: Vec<String> = keys
        .iter()
        .map(|key| {
            let subject = corpus
                .files
                .iter()
                .find(|file| return file.subject == key.subject)
                .map(|file| return file.path.clone())
                .or_else(|| {
                    return corpus
                        .files
                        .iter()
                        .find(|file| return file.group_subject == key.subject)
                        .map(|file| return file.group.clone());
                })
                .unwrap_or_else(|| return format!("<unknown subject {}>", key.subject));

            return format!("{} of {subject}", key.contract);
        })
        .collect();
    named.sort();

    return named;
}
