//! A block repeated across enough sections to be boilerplate.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template
{
    pub text: String,
    pub sections: u32,
    pub documents: Vec<String>,
    pub declared: Option<&'static str>,
}
