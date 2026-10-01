//! The wire shape of a `nomos.standards.corpus.v1` payload, and its canonical encoding.

mod declaration_issue;
mod declaration_kind;
mod declared_rule;
mod document_declaration;
mod refusal;
mod rule_severity;

pub use declaration_issue::DeclarationIssue;
pub use declaration_kind::DeclarationKind;
pub use declared_rule::DeclaredRule;
pub use document_declaration::DocumentDeclaration;
pub use refusal::Refusal;
pub use rule_severity::RuleSeverity;

use nomos_contracts::GateCategory;

/// Every document a repository's declared standards corpus holds, and what each one says it is.
///
/// # The two populations, and why both travel
///
/// `documents` is the population on disk: one row per markdown file the reader reached, in
/// whichever corpus root the repository declared. `rules` is the subset that declares
/// `kind: rule`, each carrying the severity, gate and owners it declared.
///
/// Both are carried because the claim this capability is measured against is an *equality*
/// between them and reality — "every document on disk is reported" — and a reader that
/// reported only the rules could satisfy a rule count while dropping a document it could not
/// read and inventing one it could. A count over a set of paths cannot be satisfied that way.
///
/// # Unconfigured is not empty
///
/// A repository that declares no corpus produces a payload with no `roots`, and that is a
/// different thing from a corpus that holds no documents: the first is a repository this
/// capability has nothing to say about, and the second is a declared corpus that is empty.
/// Every rule reading this capability reports nothing at all in the first case, so an
/// unconfigured repository judges exactly as it did before this capability existed.
///
/// # An issue is not an omission
///
/// A document whose declaration will not parse appears in `documents` *and* in `issues` —
/// never in `rules`, and never nowhere. Dropping it from the population would report a smaller
/// corpus that looks clean. [`Parse_Payload`] enforces the other half of that: a document
/// declaring `kind: rule` must arrive with either a `rule` row or an `issue` row, so a payload
/// cannot record a rule document and say nothing further about it.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct StandardsCorpusPayload
{
    /// The corpus roots the repository declared, repository-relative and sorted.
    pub roots: Vec<String>,
    /// Every markdown document the reader reached, sorted by path.
    pub documents: Vec<DocumentDeclaration>,
    /// Every document that declared itself a rule and declared it completely, sorted by path.
    pub rules: Vec<DeclaredRule>,
    /// Every document whose declaration could not be read, sorted by path.
    pub issues: Vec<DeclarationIssue>,
}

const CORPUS_TAG: &str = "corpus";
const DOCUMENT_TAG: &str = "document";
const RULE_TAG: &str = "rule";
const ISSUE_TAG: &str = "issue";

/// Encodes a payload as tab-separated lines, the same shape every other capability payload in
/// this workspace uses: diffable by a person, written in one place with no derive between the
/// data and the bytes. No header line — this payload answers for the workspace as a whole, not
/// for one member.
///
/// Every list is sorted here rather than trusted to arrive sorted, so the encoded bytes are a
/// function of the payload alone: two readers that reached the same documents in different
/// walk orders produce identical bytes.
///
/// `enforced_by` is joined with commas and is one field. The corpus's own schema requires at
/// least one entry, so a rule with no declared owner is an [`DeclarationIssue`] rather than a
/// row — [`Parse_Payload`] refuses the empty field an empty list would write.
#[must_use]
pub fn Encode_Payload(payload: &StandardsCorpusPayload) -> Vec<u8>
{
    let mut encoded = String::new();

    let mut roots: Vec<&String> = payload.roots.iter().collect();
    roots.sort();
    for root in roots
    {
        Push_Row(&mut encoded, &[CORPUS_TAG, root]);
    }

    let mut documents: Vec<&DocumentDeclaration> = payload.documents.iter().collect();
    documents.sort();
    for document in documents
    {
        Push_Row(&mut encoded, &[DOCUMENT_TAG, &document.path, document.kind.Label()]);
    }

    let mut rules: Vec<&DeclaredRule> = payload.rules.iter().collect();
    rules.sort_by(|left, right| return left.path.cmp(&right.path));
    for rule in rules
    {
        Push_Row(&mut encoded, &[RULE_TAG, &rule.path, &rule.id, &rule.title, rule.severity.Label(), Gate_Declaration(rule.gate), &rule.enforced_by.join(",")]);
    }

    let mut issues: Vec<&DeclarationIssue> = payload.issues.iter().collect();
    issues.sort();
    for issue in issues
    {
        Push_Row(&mut encoded, &[ISSUE_TAG, &issue.path, &issue.reason]);
    }

    return encoded.into_bytes();
}

fn Push_Row(encoded: &mut String, fields: &[&str])
{
    encoded.push_str(&fields.join("\t"));
    encoded.push('\n');
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, a line carries no tag or an unrecognized one,
/// a line carries the wrong number of fields for its tag, any field is empty, a severity, gate
/// or kind is not in the vocabulary it is written in, a document or rule path is declared
/// twice, a `rule` line names a document no `document` line declared as a rule, or a document
/// declaring `kind: rule` arrives with neither a `rule` line nor an `issue` line.
///
/// The last two are the refusals that matter: a rule whose document is ungrounded would let a
/// rule report over a corpus the repository never declared, and a rule document nobody said
/// anything further about is exactly the silent omission this capability exists to prevent.
pub fn Parse_Payload(bytes: &[u8]) -> Result<StandardsCorpusPayload, Refusal>
{
    let text = Decode_Utf8(bytes)?;

    let mut payload = StandardsCorpusPayload::default();
    for line in text.lines()
    {
        Apply_Line(&mut payload, line)?;
    }

    Check_Grounded(&payload)?;

    return Ok(payload);
}

/// Decodes `bytes` as UTF-8, or refuses.
fn Decode_Utf8(bytes: &[u8]) -> Result<&str, Refusal>
{
    return core::str::from_utf8(bytes).map_err(|error| Refusal {
        reason: format!("not UTF-8: {error}"),
    });
}

/// Applies one line to `payload`, dispatching on its tag.
///
/// The tag is split off first rather than the whole line being split on tabs, because an
/// `issue` line's reason is free text that may itself contain a tab.
fn Apply_Line(payload: &mut StandardsCorpusPayload, line: &str) -> Result<(), Refusal>
{
    let Some((tag, rest)) = line.split_once('\t')
    else
    {
        return Err(Refusal {
            reason: format!("line has no tag: {line:?}"),
        });
    };

    return match tag
    {
        CORPUS_TAG => Apply_Corpus_Line(payload, rest, line),
        DOCUMENT_TAG => Apply_Document_Line(payload, rest, line),
        RULE_TAG => Apply_Rule_Line(payload, rest, line),
        ISSUE_TAG => Apply_Issue_Line(payload, rest),
        _ =>
        {
            Err(Refusal {
                reason: format!("line has an unrecognized tag: {line:?}"),
            })
        }
    };
}

/// A `corpus` line: the root the repository declared.
fn Apply_Corpus_Line(payload: &mut StandardsCorpusPayload, rest: &str, line: &str) -> Result<(), Refusal>
{
    let [root] = Split_Exactly::<1>(rest, line)?;
    payload.roots.push(root.to_owned());

    return Ok(());
}

/// A `document` line: one document on disk and the kind it declares.
fn Apply_Document_Line(payload: &mut StandardsCorpusPayload, rest: &str, line: &str) -> Result<(), Refusal>
{
    let [path, kind] = Split_Exactly::<2>(rest, line)?;
    let Some(kind) = DeclarationKind::From_Label(kind)
    else
    {
        return Err(Refusal {
            reason: format!("kind is not in the corpus's vocabulary: {line:?}"),
        });
    };

    payload.documents.push(DocumentDeclaration {
        path: path.to_owned(),
        kind,
    });

    return Ok(());
}

/// A `rule` line: one document that declared itself a rule, with the whole of its declaration.
fn Apply_Rule_Line(payload: &mut StandardsCorpusPayload, rest: &str, line: &str) -> Result<(), Refusal>
{
    let [path, id, title, severity, gate, owners] = Split_Exactly::<6>(rest, line)?;
    let Some(severity) = RuleSeverity::Declared_By(severity)
    else
    {
        return Err(Refusal {
            reason: format!("severity is not in the corpus's vocabulary: {line:?}"),
        });
    };
    let Some(gate) = Declared_Gate(gate)
    else
    {
        return Err(Refusal {
            reason: format!("gate is not in the corpus's vocabulary: {line:?}"),
        });
    };

    payload.rules.push(DeclaredRule {
        path: path.to_owned(),
        id: id.to_owned(),
        title: title.to_owned(),
        severity,
        gate,
        enforced_by: owners.split(',').map(str::to_owned).collect(),
    });

    return Ok(());
}

const BLOCKING_DECLARATION: &str = "blocking";
const ADVISORY_DECLARATION: &str = "advisory";
const UNREACHABLE_DECLARATION: &str = "unreachable";
const REVIEW_DECLARATION: &str = "review";

/// The gate category a document declared by writing `declared`, or `None` where the corpus's
/// vocabulary has no such category.
///
/// Public because the front-matter reader in `nomos-repo-policy` parses a document's `gate:`
/// line with it. A reader with its own copy of these four spellings would be the second place
/// the corpus's vocabulary is written down, and the drop this crate's own `Encode_Payload` had
/// — a report spelling written where a corpus spelling is read — is exactly the defect two
/// copies produce.
#[must_use]
pub fn Declared_Gate(declared: &str) -> Option<GateCategory>
{
    return match declared.trim()
    {
        BLOCKING_DECLARATION => Some(GateCategory::Blocking),
        ADVISORY_DECLARATION => Some(GateCategory::Advisory),
        UNREACHABLE_DECLARATION => Some(GateCategory::Unreachable),
        REVIEW_DECLARATION => Some(GateCategory::Review),
        _ => None,
    };
}

/// `gate` as the corpus's own vocabulary spells it — the inverse of [`Declared_Gate`].
///
/// Deliberately not [`GateCategory::Label`], which is `PascalCase`: that is the spelling this
/// workspace writes a gate in its own reports, while a corpus document writes the lowercase
/// enum its schema declares. The two name the identical four states, and the whole reason
/// [`super::DeclaredRule::gate`] is a `GateCategory` rather than a type of this crate's own is
/// that there is no translation between them — but "no translation between the states" is not
/// "one spelling for both readers", and a payload written in the report's spelling would not
/// decode through the document's. Writing the document's spelling here keeps this encoding's
/// vocabulary identical to the one the front-matter reader parses, so the bytes on the wire and
/// the bytes on disk cannot drift.
fn Gate_Declaration(gate: GateCategory) -> &'static str
{
    return match gate
    {
        GateCategory::Blocking => BLOCKING_DECLARATION,
        GateCategory::Advisory => ADVISORY_DECLARATION,
        GateCategory::Unreachable => UNREACHABLE_DECLARATION,
        GateCategory::Review => REVIEW_DECLARATION,
    };
}

/// An `issue` line: a document whose declaration could not be read.
fn Apply_Issue_Line(payload: &mut StandardsCorpusPayload, rest: &str) -> Result<(), Refusal>
{
    let Some((path, reason)) = rest.split_once('\t')
    else
    {
        return Err(Refusal {
            reason: format!("issue line names no reason: {rest:?}"),
        });
    };
    Refuse_If_Empty(path, rest)?;

    payload.issues.push(DeclarationIssue {
        path: path.to_owned(),
        reason: reason.to_owned(),
    });

    return Ok(());
}

/// `rest` split on tabs into exactly the `N` non-empty fields a line of this tag carries.
///
/// A fixed-size array rather than a slice, so that each caller destructures its own fields by
/// name and no position is ever reached that the check above did not establish. The workspace
/// denies `clippy::indexing_slicing` outright, which is what makes a `fields[3]` written after
/// a length check unusable here even though the check would have made it safe.
fn Split_Exactly<'a, const N: usize>(rest: &'a str, line: &str) -> Result<[&'a str; N], Refusal>
{
    let carried = rest.split('\t').count();
    if carried != N
    {
        return Err(Refusal {
            reason: format!("line expects {N} fields but carries {carried}: {line:?}"),
        });
    }

    let mut fields = [""; N];
    for (field, declared) in fields.iter_mut().zip(rest.split('\t'))
    {
        if declared.is_empty()
        {
            return Err(Refusal {
                reason: format!("line has an empty field: {line:?}"),
            });
        }
        *field = declared;
    }

    return Ok(fields);
}

/// Refuses an empty field, naming `line`.
fn Refuse_If_Empty(field: &str, line: &str) -> Result<(), Refusal>
{
    if field.is_empty()
    {
        return Err(Refusal {
            reason: format!("line has an empty field: {line:?}"),
        });
    }

    return Ok(());
}

/// Refuses the payload shapes that would let a document slip out of the population.
///
/// Run after every line rather than during, so a payload is judged the same way whatever order
/// its rows arrive in.
fn Check_Grounded(payload: &StandardsCorpusPayload) -> Result<(), Refusal>
{
    Refuse_Duplicates(&payload.documents.iter().map(|document| return document.path.as_str()).collect::<Vec<_>>())?;
    Refuse_Duplicates(&payload.rules.iter().map(|rule| return rule.path.as_str()).collect::<Vec<_>>())?;

    for rule in &payload.rules
    {
        Refuse_Unless_Declared_A_Rule(payload, &rule.path)?;
    }
    for document in &payload.documents
    {
        if document.kind.Is_Rule()
        {
            Refuse_Unless_Accounted_For(payload, &document.path)?;
        }
    }

    return Ok(());
}

/// Refuses unless `path` is a document the payload declared as a rule.
fn Refuse_Unless_Declared_A_Rule(payload: &StandardsCorpusPayload, path: &str) -> Result<(), Refusal>
{
    let declared = payload.documents.iter().any(|document| return document.path == path && document.kind.Is_Rule());
    if !declared
    {
        return Err(Refusal {
            reason: format!("rule {path:?} names a document never declared as a rule"),
        });
    }

    return Ok(());
}

/// Refuses a document declaring `kind: rule` that the payload says nothing further about.
fn Refuse_Unless_Accounted_For(payload: &StandardsCorpusPayload, path: &str) -> Result<(), Refusal>
{
    let accounted = payload.rules.iter().any(|rule| return rule.path == path)
        || payload.issues.iter().any(|issue| return issue.path == path);
    if !accounted
    {
        return Err(Refusal {
            reason: format!("document {path:?} declares kind: rule but carries neither a rule nor an issue"),
        });
    }

    return Ok(());
}

/// Refuses a path that appears twice in `paths`.
fn Refuse_Duplicates(paths: &[&str]) -> Result<(), Refusal>
{
    let mut seen: Vec<&str> = Vec::new();
    for path in paths
    {
        if seen.contains(path)
        {
            return Err(Refusal {
                reason: format!("{path:?} is declared twice"),
            });
        }
        seen.push(path);
    }

    return Ok(());
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Parse_Payload_Should_Round_Trip_A_Payload_Through_Its_Own_Encoding()
    {
        let payload = Sample();
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

        assert_eq!(decoded, payload);
    }

    /// Canonicalization is a property of the encoding, not of whatever order the reader
    /// walked the corpus in: the same payload encoded twice is the same bytes.
    #[test]
    fn Test_Encode_Payload_Should_Be_A_Function_Of_The_Payload_Alone()
    {
        let mut shuffled = Sample();
        shuffled.roots.reverse();
        shuffled.documents.reverse();
        shuffled.rules.reverse();

        assert_eq!(Encode_Payload(&shuffled), Encode_Payload(&Sample()));
    }

    #[test]
    fn Test_Encode_Payload_Should_Produce_Stable_Diffable_Bytes()
    {
        let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("ASCII and tabs");

        assert_eq!(
            rendered,
            "corpus\tdocs/standards\n\
             document\tdocs/standards/broken.md\trule\n\
             document\tdocs/standards/index.md\tindex\n\
             document\tdocs/standards/naming.md\trule\n\
             document\tdocs/standards/style.md\tundeclared\n\
             rule\tdocs/standards/naming.md\tnaming\tNaming\tMUST NOT\tblocking\treview\n\
             issue\tdocs/standards/broken.md\tthe fence never closes\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Payload_That_Declares_Nothing()
    {
        let decoded = Parse_Payload(&[]).expect("no header line makes an empty payload unambiguous");

        assert_eq!(decoded, StandardsCorpusPayload::default());
        assert!(decoded.roots.is_empty(), "a payload that declares nothing is not a declared empty corpus");
    }

    #[test]
    fn Test_A_Severity_Outside_The_Vocabulary_Should_Be_Refused()
    {
        let error = Parse_Payload(b"document\tcss/a.md\trule\nrule\tcss/a.md\ta\tA\tSHALL\treview\treview\n")
            .expect_err("the corpus's severity vocabulary is closed");

        assert!(error.reason.contains("severity"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Gate_Outside_The_Vocabulary_Should_Be_Refused()
    {
        let error = Parse_Payload(b"document\tcss/a.md\trule\nrule\tcss/a.md\ta\tA\tMUST\tsometimes\treview\n")
            .expect_err("the corpus's gate vocabulary is closed");

        assert!(error.reason.contains("gate"), "{}", error.reason);
    }

    /// The encoding's gate spelling and the corpus's are one vocabulary, not two that happen to
    /// agree on the sample. This is the test the `PascalCase` spelling failed: `Label` produces a
    /// report's spelling, which `Declared_Gate` correctly refuses.
    #[test]
    fn Test_Every_Gate_Category_Should_Round_Trip_Through_The_Corpus_Spelling()
    {
        for gate in [GateCategory::Review, GateCategory::Unreachable, GateCategory::Advisory, GateCategory::Blocking]
        {
            assert_eq!(Declared_Gate(Gate_Declaration(gate)), Some(gate), "{gate:?}");
        }
    }

    /// And the report's own spelling is not the corpus's, so a payload written in it is refused
    /// rather than silently read as something a document never declared.
    #[test]
    fn Test_The_Report_Spelling_Should_Not_Decode_As_A_Declared_Gate()
    {
        assert_eq!(Declared_Gate(GateCategory::Blocking.Label()), None, "PascalCase is this workspace's word");
    }

    #[test]
    fn Test_A_Rule_With_No_Declared_Owner_Should_Not_Survive_Its_Own_Encoding()
    {
        let mut payload = Sample();
        let rule = payload.rules.pop().expect("Sample declares one rule");
        payload.rules.push(DeclaredRule {
            enforced_by: Vec::new(),
            ..rule
        });

        let error = Parse_Payload(&Encode_Payload(&payload)).expect_err("an ownerless rule is an issue, not a row");

        assert!(error.reason.contains("empty field"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Rule_Naming_An_Undeclared_Document_Should_Be_Refused()
    {
        let error = Parse_Payload(b"rule\tcss/ghost.md\tghost\tGhost\tMUST\treview\treview\n")
            .expect_err("a rule's document must be declared");

        assert!(error.reason.contains("never declared as a rule"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Rule_Naming_A_Document_That_Is_Not_A_Rule_Should_Be_Refused()
    {
        let error = Parse_Payload(b"document\tcss/a.md\tindex\nrule\tcss/a.md\ta\tA\tMUST\treview\treview\n")
            .expect_err("a document is a rule because it declares kind: rule");

        assert!(error.reason.contains("never declared as a rule"), "{}", error.reason);
    }

    /// The silent omission this capability exists to prevent: a document says it is a rule,
    /// and the payload records neither what it declares nor why it could not be read.
    #[test]
    fn Test_A_Rule_Document_With_Neither_A_Rule_Nor_An_Issue_Should_Be_Refused()
    {
        let error = Parse_Payload(b"document\tcss/a.md\trule\n").expect_err("a rule document must be accounted for");

        assert!(error.reason.contains("neither a rule nor an issue"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Document_Declared_Twice_Should_Be_Refused()
    {
        let error = Parse_Payload(b"document\tcss/a.md\tindex\ndocument\tcss/a.md\tindex\n")
            .expect_err("one document, one row");

        assert!(error.reason.contains("declared twice"), "{}", error.reason);
    }

    #[test]
    fn Test_An_Unrecognized_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"standard\tcss/a.md\n").expect_err("an unrecognized tag must be refused");

        assert!(error.reason.contains("unrecognized tag"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Line_With_Too_Few_Fields_Should_Be_Refused()
    {
        let error = Parse_Payload(b"rule\tcss/a.md\n").expect_err("a rule row carries six fields");

        assert!(error.reason.contains("expects 6 fields"), "{}", error.reason);
    }

    /// An issue's reason is free text, so it is the one field that may itself carry a tab.
    #[test]
    fn Test_An_Issue_Reason_Carrying_A_Tab_Should_Survive_The_Round_Trip()
    {
        let payload = Parse_Payload(b"issue\tcss/a.md\tline 3:\tseverity is not in the vocabulary\n")
            .expect("the reason is split off once, so a tab inside it is content");

        assert_eq!(payload.issues.len(), 1, "{payload:?}");
        assert_eq!(
            payload.issues.first().expect("asserted len 1 above").reason,
            "line 3:\tseverity is not in the vocabulary"
        );
    }

    /// The sample is written in canonical order, which is what makes the byte test above a
    /// statement about the encoding rather than about this literal's own order.
    ///
    /// `broken.md` is the shape worth reading twice: it declares `kind: rule`, so it is a rule
    /// document and not a rule, and its issue carries the reason. That is the one document the
    /// population must not lose.
    fn Sample() -> StandardsCorpusPayload
    {
        return StandardsCorpusPayload {
            roots: vec!["docs/standards".to_owned()],
            documents: vec![
                DocumentDeclaration {
                    path: "docs/standards/broken.md".to_owned(),
                    kind: DeclarationKind::Rule,
                },
                DocumentDeclaration {
                    path: "docs/standards/index.md".to_owned(),
                    kind: DeclarationKind::Index,
                },
                DocumentDeclaration {
                    path: "docs/standards/naming.md".to_owned(),
                    kind: DeclarationKind::Rule,
                },
                DocumentDeclaration {
                    path: "docs/standards/style.md".to_owned(),
                    kind: DeclarationKind::Undeclared,
                },
            ],
            rules: vec![DeclaredRule {
                path: "docs/standards/naming.md".to_owned(),
                id: "naming".to_owned(),
                title: "Naming".to_owned(),
                severity: RuleSeverity::MustNot,
                gate: GateCategory::Blocking,
                enforced_by: vec!["review".to_owned()],
            }],
            issues: vec![DeclarationIssue {
                path: "docs/standards/broken.md".to_owned(),
                reason: "the fence never closes".to_owned(),
            }],
        };
    }
}
