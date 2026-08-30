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

impl<'a, Text> From<&'a Text> for WorkflowText<'a>
where
    Text: AsRef<str> + ?Sized,
{
    fn from(value: &'a Text) -> Self
    {
        return WorkflowText(value.as_ref());
    }
}

impl<'a> WorkflowText<'a>
{
    /// The workflow's text as a plain string.
    #[must_use]
    pub fn As_Text(&self) -> &'a str
    {
        return self.0;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_As_Text_Should_Return_The_Wrapped_String_Unchanged()
    {
        let workflow = "name: gate\n".to_owned();
        let text: WorkflowText<'_> = WorkflowText::from(&workflow);

        assert_eq!(text.As_Text(), "name: gate\n");
    }
}
