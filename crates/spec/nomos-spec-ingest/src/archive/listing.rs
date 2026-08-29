//! What an archive holds, as against how its bytes are fetched.

/// The files an archive holds, sorted.
///
/// Separate from [`Archive`](crate::Archive) because answering "what is in here" and
/// answering "give me that one" need different state and are asked at different times: a
/// walk over twenty revisions reads every listing and opens almost none of them. Sorted at
/// construction, so no caller's iteration order depends on how the archive was written.
pub struct Listing
{
    paths: Vec<String>,
}

impl Listing
{
    /// The listing of an already-open archive.
    #[must_use]
    pub(crate) fn Of(paths: Vec<String>) -> Self
    {
        return Self { paths };
    }

    /// Every file in the archive, sorted. Never empty: `Archive::Open` refuses that.
    #[must_use]
    pub fn Paths(&self) -> &[String]
    {
        return &self.paths;
    }

    /// Whether the archive holds a file at exactly this path.
    #[must_use]
    pub fn Has_Path(&self, entry: &str) -> bool
    {
        return self.paths.iter().any(|name| name == entry);
    }

    /// Entries whose path ends with `suffix`, sorted.
    #[must_use]
    pub fn Ending_With(&self, suffix: &str) -> Vec<String>
    {
        return self
            .paths
            .iter()
            .filter(|name| name.ends_with(suffix))
            .cloned()
            .collect();
    }
}
