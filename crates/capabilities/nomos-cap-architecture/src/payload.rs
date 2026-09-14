//! The wire shape of a `nomos.architecture.declaration.v1` payload, its canonical encoding,
//! and the generic relation queries every party asks of it.
//!
//! # What this crate owns and what it does not
//!
//! It owns the *shape* of an architecture declaration: that there are named components, that
//! packages belong to them, that components admit dependencies on other components, that pairs
//! of packages may be excepted from their component's own peer rule, and that a package may be
//! declared a write authority with named doors. It owns the queries over that shape.
//!
//! It owns none of the content. There is no component named here, no package named here, and
//! no default. A repository dividing itself into `Domain`, `Infrastructure` and `Api` is
//! expressed by this schema exactly as well as one dividing itself into twelve zones, which is
//! the property `OD-RULES-029` says a declaration format must have before it has externalized
//! anything: "a zone lattice with reasons behind each cell ... is this workspace's
//! architecture. It is not every repository's, and a declaration format that cannot express a
//! different one has not externalized the declaration, only moved it."
//!
//! # Why the queries live here rather than in the rule
//!
//! They are relation lookups over a declaration, not judgments. A rule that owned them would
//! own the schema, which is the half `OD-RULES-029`'s triple is about; and the two parties
//! outside a check run -- `nomos gate admits` and the editor's architectural component --
//! would then have to reach a rules crate to ask a question that is not a rule's.

pub(crate) mod authority;
pub(crate) mod exception;
pub(crate) mod membership;
pub(crate) mod permission;
pub(crate) mod refusal;

use authority::Authority;
use exception::Exception;
use membership::Membership;
use permission::Permission;
use refusal::Refusal;

/// A `member` line's own fields, after its tag.
const MEMBER_FIELDS: usize = 2;
/// A `permits` line's own fields, after its tag.
const PERMITS_FIELDS: usize = 2;
/// An `exception` line's own fields, after its tag.
const EXCEPTION_FIELDS: usize = 2;
/// A `door` line's own fields, after its tag.
const DOOR_FIELDS: usize = 2;

/// A repository's whole declared architecture — empty when it declares none.
///
/// An empty payload is a real answer and not an absence: it means this repository has not
/// declared an architecture, which `OD-RULES-003` decided is `Applicability::NotApplicable`
/// for the rules that read it rather than a gap in every one of its members. Telling that
/// case apart from "declared, and this member was left out" is the whole reason the
/// declaration had to leave the rule's own crate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArchitecturePayload
{
    /// Every component this repository divides itself into, in declared order.
    pub components: Vec<String>,
    /// Which component each member belongs to.
    pub membership: Vec<Membership>,
    /// Which component may depend on which.
    pub permissions: Vec<Permission>,
    /// The package pairs excepted from their shared component's own peer rule.
    pub exceptions: Vec<Exception>,
    /// The packages declared sole write authorities, and the doors into each.
    pub authorities: Vec<Authority>,
}

impl ArchitecturePayload
{
    /// Whether this repository declared an architecture at all.
    ///
    /// A declaration with components but no members is still a declaration: it says what the
    /// repository divides itself into and has not yet placed anything, which is a different
    /// claim from declaring nothing.
    #[must_use]
    pub fn Declares_An_Architecture(&self) -> bool
    {
        return !self.components.is_empty();
    }

    /// `package`'s own declared component, or `None` when this declaration does not place it.
    ///
    /// `None` is the answer for a package this declaration never names, and there is
    /// deliberately no fallback to anything else. A lookup that still answered from some other
    /// source when the declaration is silent would leave the declaration decorative.
    #[must_use]
    pub fn Component_Of(&self, package: &str) -> Option<&str>
    {
        return self
            .membership
            .iter()
            .find(|placed| return placed.package == package)
            .map(|placed| return placed.component.as_str());
    }

    /// Whether a package in `from` may depend on a package in `to`, by component alone.
    ///
    /// Two packages sharing a component are [`Self::Excepts`]'s question, not this one's, and
    /// this answers only what the declaration states: a repository that declares no `from -> from`
    /// permission gets `false` for one, which is how this workspace's own declaration keeps
    /// peers from naming each other without that rule being written here.
    #[must_use]
    pub fn Permits(&self, from: &str, to: &str) -> bool
    {
        return self
            .permissions
            .iter()
            .any(|permission| return permission.from == from && permission.to == to);
    }

    /// Whether `from` naming `to` is one of the package pairs this declaration excepts.
    ///
    /// Directed, because an exception is a statement about one real dependency and not a
    /// blanket exemption for a pair.
    #[must_use]
    pub fn Excepts(&self, from: &str, to: &str) -> bool
    {
        return self
            .exceptions
            .iter()
            .any(|exception| return exception.from == from && exception.to == to);
    }

    /// The doors into `package`, or `None` when this declaration does not call it an authority.
    #[must_use]
    pub fn Doors_Into(&self, package: &str) -> Option<&[String]>
    {
        return self
            .authorities
            .iter()
            .find(|authority| return authority.package == package)
            .map(|authority| return authority.doors.as_slice());
    }
}

/// Encodes a declaration as tab-separated lines, the same shape every sibling capability in
/// this workspace uses: diffable by a person, written in one place with no derive between the
/// data and the bytes.
///
/// No header line, the same reason `nomos_cap_limits_policy::Encode_Payload` has none: this
/// payload answers for the workspace as a whole, so there is no identifying value to write
/// ahead of the rows.
#[must_use]
pub fn Encode_Payload(payload: &ArchitecturePayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for component in &payload.components
    {
        encoded.push_str("component\t");
        encoded.push_str(component);
        encoded.push('\n');
    }
    for placed in &payload.membership
    {
        Push_Pair(&mut encoded, "member", &placed.package, &placed.component);
    }
    for permission in &payload.permissions
    {
        Push_Pair(&mut encoded, "permits", &permission.from, &permission.to);
    }
    for exception in &payload.exceptions
    {
        Push_Pair(&mut encoded, "exception", &exception.from, &exception.to);
    }
    for authority in &payload.authorities
    {
        encoded.push_str("authority\t");
        encoded.push_str(&authority.package);
        encoded.push('\n');

        for door in &authority.doors
        {
            Push_Pair(&mut encoded, "door", &authority.package, door);
        }
    }

    return encoded.into_bytes();
}

/// One `tag\tleft\tright` line.
fn Push_Pair(encoded: &mut String, tag: &str, left: &str, right: &str)
{
    encoded.push_str(tag);
    encoded.push('\t');
    encoded.push_str(left);
    encoded.push('\t');
    encoded.push_str(right);
    encoded.push('\n');
}

/// Reads a declaration back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, a line carries a tag this schema does not
/// name, a line does not have exactly the fields its tag declares, or a `member` or `permits`
/// line names a component the declaration never declared. That last one is the case
/// `OD-RULES-003` asks a rule to report as a refusal rather than silently drop: a declaration
/// that is incomplete in that way is not the same as a repository that declared nothing.
pub fn Parse_Payload(bytes: &[u8]) -> Result<ArchitecturePayload, Refusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    })?;

    let mut payload = ArchitecturePayload::default();
    for line in text.lines()
    {
        Read_Line(line, &mut payload)?;
    }

    Assert_Components_Declared(&payload)?;

    return Ok(payload);
}

/// One line of the encoding, into `payload`.
fn Read_Line(line: &str, payload: &mut ArchitecturePayload) -> Result<(), Refusal>
{
    if let Some(name) = line.strip_prefix("component\t")
    {
        payload.components.push(name.to_owned());

        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("member\t")
    {
        let [package, component] = Pair(line, rest, MEMBER_FIELDS)?;
        payload.membership.push(Membership { package: package.to_owned(), component: component.to_owned() });

        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("permits\t")
    {
        let [from, to] = Pair(line, rest, PERMITS_FIELDS)?;
        payload.permissions.push(Permission { from: from.to_owned(), to: to.to_owned() });

        return Ok(());
    }

    return Read_Exception_Or_Authority(line, payload);
}

/// The three tags [`Read_Line`] did not take, split out so neither function exceeds this
/// workspace's own nesting limit.
fn Read_Exception_Or_Authority(line: &str, payload: &mut ArchitecturePayload) -> Result<(), Refusal>
{
    if let Some(rest) = line.strip_prefix("exception\t")
    {
        let [from, to] = Pair(line, rest, EXCEPTION_FIELDS)?;
        payload.exceptions.push(Exception { from: from.to_owned(), to: to.to_owned() });

        return Ok(());
    }
    if let Some(package) = line.strip_prefix("authority\t")
    {
        payload.authorities.push(Authority { package: package.to_owned(), doors: Vec::new() });

        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("door\t")
    {
        let [package, door] = Pair(line, rest, DOOR_FIELDS)?;

        return Add_Door(payload, package, door);
    }

    return Err(Refusal {
        reason: format!("line carries no tag this schema names: {line:?}"),
    });
}

/// `door` onto the authority it names, refused when no `authority` line declared one.
fn Add_Door(payload: &mut ArchitecturePayload, package: &str, door: &str) -> Result<(), Refusal>
{
    let Some(authority) = payload.authorities.iter_mut().find(|authority| return authority.package == package)
    else
    {
        return Err(Refusal {
            reason: format!("door line names {package:?}, which no authority line declares"),
        });
    };

    authority.doors.push(door.to_owned());

    return Ok(());
}

/// A two-field line's own fields, or a refusal naming the line that did not have two.
fn Pair<'a>(line: &str, rest: &'a str, fields: usize) -> Result<[&'a str; 2], Refusal>
{
    let split: Vec<&str> = rest.splitn(fields, '\t').collect();
    let [left, right] = split.as_slice()
    else
    {
        return Err(Refusal {
            reason: format!("line does not have exactly {fields} fields after its tag: {line:?}"),
        });
    };

    return Ok([*left, *right]);
}

/// Every component a `member` or `permits` line names is one the declaration declared.
///
/// The check `OD-RULES-003` asks for by name: an incomplete declaration is refused rather than
/// silently dropping the statements that reference what is missing.
fn Assert_Components_Declared(payload: &ArchitecturePayload) -> Result<(), Refusal>
{
    for placed in &payload.membership
    {
        Assert_Declared(payload, &placed.component, "member")?;
    }
    for permission in &payload.permissions
    {
        Assert_Declared(payload, &permission.from, "permits")?;
        Assert_Declared(payload, &permission.to, "permits")?;
    }

    return Ok(());
}

/// One component name, checked against the declared set.
fn Assert_Declared(payload: &ArchitecturePayload, component: &str, tag: &str) -> Result<(), Refusal>
{
    if payload.components.iter().any(|declared| return declared == component)
    {
        return Ok(());
    }

    return Err(Refusal {
        reason: format!("a {tag} line names component {component:?}, which no component line declares"),
    });
}

#[cfg(test)]
mod tests;
