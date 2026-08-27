//! The text of a whole workflow file, kept apart from a step's name purely by type.

/// The text of a whole GitHub Actions workflow file, distinguished from [`crate::StepName`]
/// purely by type.
///
/// [`crate::Derive_Step`] takes one of each as adjacent parameters, and two parameters that
/// both read as `&str` there let a caller swap the workflow for the step name and have the
/// compiler accept it. `From<&str>` and `From<&String>` both convert into this, so no
/// existing call site needs to change shape to adopt it — every one already passes a
/// borrowed string.
#[derive(Clone, Copy, Debug)]
pub struct WorkflowText<'a>(&'a str);

impl<'a> From<&'a str> for WorkflowText<'a>
{
    fn from(value: &'a str) -> Self
    {
        return WorkflowText(value);
    }
}

impl<'a> From<&'a String> for WorkflowText<'a>
{
    fn from(value: &'a String) -> Self
    {
        return WorkflowText(value.as_str());
    }
}

impl<'a> WorkflowText<'a>
{
    /// The workflow's text as a plain string.
    #[must_use]
    pub fn As_Str(&self) -> &'a str
    {
        return self.0;
    }
}
