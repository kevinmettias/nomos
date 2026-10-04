//! The sites family's productions, one per provider that offers it, each over a fixture of its
//! own because the fixture every other production shares holds no labeled jump.

use crate::productions::{Fact_Context, Go_Context, Go_Subject_Of, Subject_Of};

/// The source the sites offer is measured over: its own, because the shared fixture holds no
/// labeled jump and a golden over a family that recorded nothing would pin the stances and not
/// one record. Each file carries what could make a walk's order unstable -- jumps at three loop
/// depths, two labels sharing a name in two functions, a labeled block and a closure the outer
/// label does not cross.
const SITES_FIXTURE: [(&str, &str); 2] = [
    (
        "src/search.rs",
        "pub fn search(grid: &[Vec<i32>], needle: i32) -> bool {\n\
         \x20   'rows: for row in grid {\n\
         \x20       'cells: for cell in row {\n\
         \x20           if *cell < 0 { continue 'rows; }\n\
         \x20           if *cell == needle { break 'rows; }\n\
         \x20           if *cell == 0 { continue 'cells; }\n\
         \x20       }\n\
         \x20   }\n\
         \x20   false\n\
         }\n",
    ),
    (
        "src/drain.rs",
        "pub fn first(kinds: &[i32]) {\n\
         \x20   'l: for kind in kinds {\n\
         \x20       match kind { 0 => break 'l, _ => continue }\n\
         \x20   }\n\
         }\n\
         pub fn second(kinds: &[i32]) -> i32 {\n\
         \x20   'l: loop {\n\
         \x20       let skip = || loop { break 'l; };\n\
         \x20       'blk: { if kinds.is_empty() { break 'blk; } }\n\
         \x20       break 'l;\n\
         \x20   }\n\
         \x20   0\n\
         }\n",
    ),
];

/// The sites offer's facts over [`SITES_FIXTURE`], rendered the way [`Reachability_Production`](crate::productions::Reachability_Production)
/// renders its own: the path, the key, the payload.
pub(crate) fn Sites_Production() -> Vec<u8>
{
    let context = Fact_Context();
    let mut rendered = Vec::new();

    for (path, source) in SITES_FIXTURE
    {
        let fact = match nomos_lang_rust::sites::Materialize_Sites_Fact(Subject_Of(path), source, context)
        {
            nomos_lang_rust::Materialization::Materialized(fact) => fact,
            nomos_lang_rust::Materialization::Unparseable(failure) =>
            {
                panic!("the fixture must parse; {path} did not: {failure}");
            }
        };

        rendered.extend_from_slice(format!("file\t{path}\n").as_bytes());
        rendered.extend_from_slice(format!("key\t{}\n", fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&fact.payload.bytes);
    }

    return rendered;
}

/// The Go source the Go sites offer is measured over: jumps where Go's bare `break` is caught by a
/// `switch` and where it is not, a `continue` from inside a `select`, a func literal in a package
/// `var`, and a closure the outer label does not cross.
const GO_SITES_FIXTURE: [(&str, &str); 2] = [
    (
        "drain.go",
        "package p\n\nfunc drain(kinds []int) {\nLoop:\n    for _, kind := range kinds {\n        if kind < 0 {\n            break Loop\n        }\n        switch kind {\n        case 0:\n            break Loop\n        default:\n            continue Loop\n        }\n    }\n}\n",
    ),
    (
        "run.go",
        "package p\n\nvar run = func(grid [][]int, done chan int) {\nOuter:\n    for range grid {\n        select {\n        case <-done:\n            continue Outer\n        }\n        go func() {\n        Inner:\n            for range grid {\n                break Inner\n            }\n        }()\n    }\n}\n",
    ),
];

/// The Go sites offer's facts over [`GO_SITES_FIXTURE`], rendered the way [`Sites_Production`] renders
/// the Rust offer's.
pub(crate) fn Go_Sites_Production() -> Vec<u8>
{
    let context = Go_Context();
    let mut rendered = Vec::new();

    for (path, source) in GO_SITES_FIXTURE
    {
        let fact = match nomos_lang_go::sites::Materialize_Sites_Fact(Go_Subject_Of(path), source, context)
        {
            nomos_lang_go::Materialization::Materialized(fact) => fact,
            nomos_lang_go::Materialization::Unparseable(failure) =>
            {
                panic!("the fixture must parse; {path} did not: {failure}");
            }
        };

        rendered.extend_from_slice(format!("file\t{path}\n").as_bytes());
        rendered.extend_from_slice(format!("key\t{}\n", fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&fact.payload.bytes);
    }

    return rendered;
}

/// The C# source the C# sites offer is measured over: a loop, a `break`, and the `goto` label C# does
/// have, none of which moves the decline.
const CSHARP_SITES_FIXTURE: &str = "class C\n{\n    void F(int[] xs)\n    {\n        foreach (var x in xs) { if (x == 0) { break; } }\n        retry: goto retry;\n    }\n}\n";

/// The C# sites offer's fact over [`CSHARP_SITES_FIXTURE`], rendered the same way: one file, its
/// key and a payload carrying the decline and no record.
pub(crate) fn Csharp_Sites_Production() -> Vec<u8>
{
    let shared = Fact_Context();
    let context = nomos_lang_csharp::FactContext { snapshot: shared.snapshot, variant: shared.variant, configuration: shared.configuration, generation: shared.generation };
    let path = "C.cs";
    let fact = match nomos_lang_csharp::sites::Materialize_Sites_Fact(Subject_Of(path), CSHARP_SITES_FIXTURE, context)
    {
        nomos_lang_csharp::Materialization::Materialized(fact) => fact,
        nomos_lang_csharp::Materialization::Unparseable(failure) => panic!("a decline does not wait on a parse: {failure}"),
    };
    let mut rendered = Vec::new();

    rendered.extend_from_slice(format!("file\t{path}\n").as_bytes());
    rendered.extend_from_slice(format!("key\t{}\n", fact.Key().Digest()).as_bytes());
    rendered.extend_from_slice(&fact.payload.bytes);

    return rendered;
}
