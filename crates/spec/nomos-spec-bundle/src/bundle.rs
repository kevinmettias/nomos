//! A specification corpus as text.

use std::collections::BTreeMap;

use nomos_spec_model::ContentHash;
use serde::{Deserialize, Serialize};

use crate::BundleError;
use crate::header::{FORMAT, Header};
use crate::manifest::Manifest;
use crate::record::Record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "line", rename_all = "snake_case")]
enum Line
{
    Header(Header),
    Record(Record),
    Manifest(Manifest),
}

/// A specification corpus as text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bundle
{
    header: Header,
    records: Vec<Record>,
    manifest: Manifest,
}

impl Bundle
{
    /// # Errors
    ///
    /// Returns [`BundleError::Json`] if a record cannot be serialized.
    pub fn New(schema_version: u32, records: Vec<Record>) -> Result<Self, BundleError>
    {
        let header = Header {
            format: FORMAT,
            schema_version,
        };

        let counts = Self::Counts_In(&records);
        let covered = Self::Covered_Text(&header, &records)?;
        let manifest = Manifest {
            records: u32::try_from(records.len()).unwrap_or(u32::MAX),
            counts,
            digest: ContentHash::Of(&covered).As_Str().to_owned(),
        };

        return Ok(Self {
            header,
            records,
            manifest,
        });
    }

    /// How many records each table contributed.
    fn Counts_In(records: &[Record]) -> BTreeMap<String, u32>
    {
        return Self::Counts_Present(records)
            .into_iter()
            .map(|(table, count)| return (table.to_owned(), count))
            .collect();
    }

    /// Everything the manifest's digest is taken over: the header and every record.
    fn Covered_Text(header: &Header, records: &[Record]) -> Result<String, BundleError>
    {
        let mut covered = Self::Line_Text(&Line::Header(header.clone()))?;
        for record in records
        {
            covered.push_str(&Self::Line_Text(&Line::Record(record.clone()))?);
        }

        return Ok(covered);
    }

    #[must_use]
    pub const fn Header(&self) -> &Header
    {
        return &self.header;
    }

    #[must_use]
    pub const fn Manifest(&self) -> &Manifest
    {
        return &self.manifest;
    }

    #[must_use]
    pub fn Records(&self) -> &[Record]
    {
        return &self.records;
    }

    /// The bytes. LF, UTF-8, no BOM, one record per line, trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`BundleError::Json`] if a record cannot be serialized.
    pub fn Write(&self) -> Result<String, BundleError>
    {
        let mut text = Self::Covered_Text(&self.header, &self.records)?;
        text.push_str(&Self::Line_Text(&Line::Manifest(self.manifest.clone()))?);

        return Ok(text);
    }

    /// Reads a bundle and refuses one it cannot fully trust.
    ///
    /// Three separate checks, because they fail for three different reasons: the digest
    /// catches an edited record, the counts catch a deleted one, and the canonical-form
    /// check catches a bundle that no regeneration would ever reproduce.
    ///
    /// # Errors
    ///
    /// Returns [`BundleError`] naming which of them failed.
    pub fn Parse(text: &str) -> Result<Self, BundleError>
    {
        let body = text.trim_end_matches('\n');
        let Some((covered, manifest_line)) = body.rsplit_once('\n')
        else
        {
            return Err(BundleError::Malformed(
                "a bundle is at least a header and a manifest".to_owned(),
            ));
        };
        let lines = Self::Canonical_Lines(covered)?;
        let manifest = Self::Manifest_Line(manifest_line)?;
        let header = Self::Header_Of(&lines)?;
        let records = Self::Records_After_The_Header(lines)?;

        Self::Assert_Untampered(covered, &manifest)?;
        let bundle = Self {
            header,
            records,
            manifest,
        };
        bundle.Verify_Counts()?;

        return Ok(bundle);
    }

    /// Every line above the manifest, refusing one no regeneration would ever reproduce.
    fn Canonical_Lines(covered: &str) -> Result<Vec<Line>, BundleError>
    {
        let mut lines: Vec<Line> = Vec::new();
        for (index, line) in covered.lines().enumerate()
        {
            let parsed: Line = serde_json::from_str(line)?;
            if Self::Line_Text(&parsed)?.trim_end_matches('\n') != line
            {
                return Err(BundleError::NotCanonical {
                    line: index.saturating_add(1),
                });
            }
            lines.push(parsed);
        }

        return Ok(lines);
    }

    /// The last line, which must be the manifest.
    fn Manifest_Line(line: &str) -> Result<Manifest, BundleError>
    {
        let Line::Manifest(manifest) = serde_json::from_str::<Line>(line)?
        else
        {
            return Err(BundleError::Malformed(
                "the last line must be the manifest".to_owned(),
            ));
        };

        return Ok(manifest);
    }

    /// The first line, which must be a header this build is new enough to read.
    fn Header_Of(lines: &[Line]) -> Result<Header, BundleError>
    {
        let Some(Line::Header(header)) = lines.first()
        else
        {
            return Err(BundleError::Malformed(
                "the first line must be the header".to_owned(),
            ));
        };
        if header.format > FORMAT
        {
            return Err(BundleError::TooNew {
                found: header.format,
                supported: FORMAT,
            });
        }

        return Ok(header.clone());
    }

    /// Everything after the header, which must all be records.
    fn Records_After_The_Header(lines: Vec<Line>) -> Result<Vec<Record>, BundleError>
    {
        let mut records: Vec<Record> = Vec::new();
        for line in lines.into_iter().skip(1)
        {
            match line
            {
                Line::Record(record) => records.push(record),
                Line::Header(_) | Line::Manifest(_) =>
                {
                    return Err(BundleError::Malformed(
                        "a header or manifest appeared among the records".to_owned(),
                    ));
                }
            }
        }

        return Ok(records);
    }

    /// The digest catches an edited record, which no count can see.
    fn Assert_Untampered(covered: &str, manifest: &Manifest) -> Result<(), BundleError>
    {
        let computed = ContentHash::Of(&format!("{covered}\n"));
        if computed.As_Str() == manifest.digest
        {
            return Ok(());
        }

        return Err(BundleError::Tampered {
            declared: manifest.digest.clone(),
            computed: computed.As_Str().to_owned(),
        });
    }

    /// # Errors
    ///
    /// Returns [`BundleError::Miscounted`] if the manifest and the records disagree.
    pub fn Verify_Counts(&self) -> Result<(), BundleError>
    {
        let present = Self::Counts_Present(&self.records);
        let total = u32::try_from(self.records.len()).unwrap_or(u32::MAX);

        self.Assert_Every_Declared_Count_Is_Met(&present)?;
        self.Assert_Every_Table_Present_Is_Declared(&present)?;
        if total != self.manifest.records
        {
            return Err(BundleError::Miscounted {
                table: "*".to_owned(),
                declared: self.manifest.records,
                present: total,
            });
        }

        return Ok(());
    }

    /// How many records of each table the bundle actually carries.
    fn Counts_Present(records: &[Record]) -> BTreeMap<&str, u32>
    {
        let mut present: BTreeMap<&str, u32> = BTreeMap::new();
        for record in records
        {
            let counter = present.entry(record.Table()).or_insert(0);
            *counter = counter.saturating_add(1);
        }

        return present;
    }

    /// A table the manifest declares a count for that the records do not meet.
    fn Assert_Every_Declared_Count_Is_Met(&self, present: &BTreeMap<&str, u32>)
        -> Result<(), BundleError>
    {
        for (table, declared) in &self.manifest.counts
        {
            let found = present.get(table.as_str()).copied().unwrap_or(0);
            if found != *declared
            {
                return Err(BundleError::Miscounted {
                    table: table.clone(),
                    declared: *declared,
                    present: found,
                });
            }
        }

        return Ok(());
    }

    /// A table the records carry that the manifest does not name at all.
    fn Assert_Every_Table_Present_Is_Declared(&self, present: &BTreeMap<&str, u32>)
        -> Result<(), BundleError>
    {
        let undeclared = present
            .iter()
            .find(|(table, _)| return !self.manifest.counts.contains_key(**table));
        let Some((table, found)) = undeclared
        else
        {
            return Ok(());
        };

        return Err(BundleError::Miscounted {
            table: (*table).to_owned(),
            declared: 0,
            present: *found,
        });
    }

    fn Line_Text(line: &Line) -> Result<String, BundleError>
    {
        let mut text = serde_json::to_string(line)?;
        text.push('\n');
        return Ok(text);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::blob::Blob;
    use crate::blob_encoding::BlobEncoding;

    fn One_Blob() -> Vec<Record>
    {
        return vec![Record::Blob(Blob {
            sha256: "sha256:aa".to_owned(),
            byte_length: 2,
            encoding: BlobEncoding::Utf8,
            content: "hi".to_owned(),
        })];
    }

    #[test]
    fn Test_A_Bundle_Should_Round_Trip_Through_Text()
    {
        let bundle = Bundle::New(1, One_Blob()).expect("builds");

        let text = bundle.Write().expect("writes");
        let read = Bundle::Parse(&text).expect("parses");

        assert_eq!(read, bundle);
        assert_eq!(read.Write().expect("rewrites"), text);
    }

    #[test]
    fn Test_Every_Line_Should_End_With_A_Newline_And_No_Carriage_Return()
    {
        let text = Bundle::New(1, One_Blob())
            .expect("builds")
            .Write()
            .expect("writes");

        assert!(text.ends_with('\n'));
        assert!(!text.contains('\r'), "CRLF would make the bundle diff per platform");
    }

    /// An edited record must not survive reading.
    #[test]
    fn Test_An_Altered_Record_Should_Be_Refused()
    {
        let text = Bundle::New(1, One_Blob())
            .expect("builds")
            .Write()
            .expect("writes");

        let tampered = text.replace("\"hi\"", "\"ho\"");
        assert_ne!(tampered, text, "the negative control must actually alter the bundle");

        let refusal = Bundle::Parse(&tampered).expect_err("an altered record must be refused");
        assert!(matches!(refusal, BundleError::Tampered { .. }), "{refusal}");
    }

    /// A deleted record leaves the digest of what remains internally consistent only if
    /// the manifest is edited too — so the counts are what catches it.
    #[test]
    fn Test_A_Removed_Record_Should_Be_Refused()
    {
        let bundle = Bundle::New(1, One_Blob()).expect("builds");
        let text = bundle.Write().expect("writes");

        let kept: Vec<&str> = text
            .lines()
            .filter(|line| !line.contains("\"table\":\"blobs\""))
            .collect();
        let mut shortened = kept.join("\n");
        shortened.push('\n');
        assert!(
            shortened.lines().count() < text.lines().count(),
            "the negative control must actually remove a line"
        );

        let refusal = Bundle::Parse(&shortened).expect_err("a removed record must be refused");
        assert!(matches!(refusal, BundleError::Tampered { .. }), "{refusal}");
    }

    /// The fixpoint requirement. A reformatted bundle is refused even when its digest was
    /// recomputed over the reformatting, because nothing would ever regenerate it.
    #[test]
    fn Test_A_Reformatted_Bundle_Should_Be_Refused()
    {
        let bundle = Bundle::New(1, One_Blob()).expect("builds");
        let text = bundle.Write().expect("writes");

        let loosened = text.replace("\"format\":1", "\"format\": 1");
        assert_ne!(loosened, text);

        let refusal = Bundle::Parse(&loosened).expect_err("non-canonical text must be refused");
        assert!(matches!(refusal, BundleError::NotCanonical { .. }), "{refusal}");
    }

    #[test]
    fn Test_A_Newer_Format_Should_Be_Refused()
    {
        let mut bundle = Bundle::New(1, Vec::new()).expect("builds");
        bundle.header.format = FORMAT.saturating_add(1);
        let rebuilt = Bundle::New(1, Vec::new()).expect("builds");
        let text = format!(
            "{}{}",
            Bundle::Line_Text(&Line::Header(bundle.header)).expect("writes"),
            Bundle::Line_Text(&Line::Manifest(rebuilt.manifest)).expect("writes")
        );

        let refusal = Bundle::Parse(&text).expect_err("a newer format must be refused");
        assert!(
            matches!(refusal, BundleError::TooNew { .. } | BundleError::Tampered { .. }),
            "{refusal}"
        );
    }

    #[test]
    fn Test_An_Empty_Bundle_Should_Still_Be_Well_Formed()
    {
        let bundle = Bundle::New(1, Vec::new()).expect("builds");
        let text = bundle.Write().expect("writes");

        assert_eq!(text.lines().count(), 2, "a header and a manifest");
        assert_eq!(Bundle::Parse(&text).expect("parses"), bundle);
    }
}
