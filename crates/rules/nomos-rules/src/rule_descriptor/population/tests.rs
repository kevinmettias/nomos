//! Which sources each kind of population holds, and the name a report gives it.

use super::*;
use nomos_model::Subject_Of_Path;

fn Source(path: &str, text: &str) -> SourceFile
{
    let mut source = SourceFile::New(path, Subject_Of_Path(path), text);
    source.language = crate::Recognized_Language_In_Tests(path);
    return source;
}

fn Starts_With_Bang(source: &SourceFile) -> bool
{
    return source.text.starts_with("#!");
}

#[test]
fn Test_Each_Population_Should_Hold_Exactly_Its_Own_Sources()
{
    let rust = Source("src/lib.rs", "pub fn One() {}\n");
    let go = Source("main.go", "package main\n");
    let script = Source("tools/run", "#!/bin/sh\n");

    let languages: &[&str] = &[crate::RUST_LANGUAGE, crate::GO_LANGUAGE];
    let kind = Population::Kind { name: "scripts", holds: Starts_With_Bang };

    assert!([&rust, &go, &script].iter().all(|source| return Population::Every.Holds(source)));
    assert_eq!([&rust, &go, &script].map(|source| return Population::Language(crate::GO_LANGUAGE).Holds(source)), [false, true, false]);
    assert_eq!([&rust, &go, &script].map(|source| return Population::Languages(languages).Holds(source)), [true, true, false]);
    assert_eq!([&rust, &go, &script].map(|source| return kind.Holds(source)), [false, false, true]);
}

#[test]
fn Test_A_Population_Should_Be_Named_For_What_It_Holds()
{
    let languages: &[&str] = &[crate::RUST_LANGUAGE, crate::GO_LANGUAGE];

    assert_eq!(Population::Every.Name(), "every source");
    assert_eq!(Population::Language(crate::GO_LANGUAGE).Name(), "go sources");
    assert_eq!(Population::Languages(languages).Name(), "rust or go sources");
    assert_eq!(Population::Kind { name: "scripts", holds: Starts_With_Bang }.Name(), "scripts");
}
