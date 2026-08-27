//! The name of one step within a workflow, kept apart from the workflow's text purely by
//! type.

/// The name of one step within a workflow, distinguished from [`crate::WorkflowText`] for the
/// reason given on that type.
#[derive(Clone, Copy, Debug)]
pub struct StepName<'a>(&'a str);

impl<'a> From<&'a str> for StepName<'a>
{
    fn from(value: &'a str) -> Self
    {
        return StepName(value);
    }
}

impl<'a> From<&'a String> for StepName<'a>
{
    fn from(value: &'a String) -> Self
    {
        return StepName(value.as_str());
    }
}

impl<'a> StepName<'a>
{
    /// The step name as a plain string.
    #[must_use]
    pub fn As_Str(&self) -> &'a str
    {
        return self.0;
    }
}
