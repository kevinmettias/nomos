//! The two text transforms every markup writer here needs: one escapes, one slugs.
//!
//! Neither is a format's own. A slug is what an identity becomes when it has to name an anchor
//! or a graph node, and an escape is what authored text becomes when it has to survive a
//! markup grammar -- both are asked by more than one writer, so both live here rather than in
//! whichever writer happened to need one first.

pub(super) fn Escape_Html(value: &str) -> String
{
    return value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
}

pub(super) fn Slug_Of_Text(value: &str) -> String
{
    let mut slug = String::new();
    let mut pending = false;

    for character in value.chars()
    {
        if character.is_ascii_alphanumeric()
        {
            if pending && !slug.is_empty()
            {
                slug.push('-');
            }
            pending = false;
            slug.extend(character.to_lowercase());
        }
        else
        {
            pending = true;
        }
    }

    return slug;
}
