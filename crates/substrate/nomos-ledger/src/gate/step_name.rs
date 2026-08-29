//! The name of one step within a workflow, kept apart from the workflow's text purely by
//! type.

/// The name of one step within a workflow, distinguished from [`crate::WorkflowText`] for the
/// reason given on that type.
#[derive(Clone, Copy, Debug)]
pub struct StepName<'a>(&'a str);

impl<'a, Text> From<&'a Text> for StepName<'a>
where
    Text: AsRef<str> + ?Sized,
{
    fn from(value: &'a Text) -> Self
    {
        return StepName(value.as_ref());
    }
}

impl<'a> StepName<'a>
{
    /// The step name as a plain string.
    #[must_use]
    pub fn As_Text(&self) -> &'a str
    {
        return self.0;
    }
}
