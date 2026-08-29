//! Content from one of `UntrustedPromptOrigin`'s five sources. A marker wrapper,
//! deliberately without any method that turns its text into an authorization -- the
//! type carries no way to satisfy `AGT-EXEC-003`'s negative constraint, which is the
//! constraint enforced.

use serde::{Deserialize, Serialize};

use super::UntrustedPromptOrigin;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UntrustedPromptContent
{
    pub origin: UntrustedPromptOrigin,
    pub text: String,
}
