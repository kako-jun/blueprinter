use std::fs;
use std::path::Path;

#[test]
fn mermaid_poc_assets_and_docs_stay_in_sync() {
    let fixture_paths = [
        "tests/fixtures/mermaid/flowchart.mmd",
        "tests/fixtures/mermaid/sequence.mmd",
        "tests/fixtures/mermaid/er-diagram.mmd",
    ];
    for path in fixture_paths {
        assert!(Path::new(path).exists(), "missing fixture: {path}");
    }

    assert!(
        Path::new("scripts/mermaid-poc.sh").exists(),
        "missing PoC script"
    );

    let readme = fs::read_to_string("README.md").expect("read README.md");
    assert!(readme.contains("scripts/mermaid-poc.sh"));

    let poc_doc = fs::read_to_string("docs/poc.md").expect("read docs/poc.md");
    assert!(poc_doc.contains("tests/fixtures/mermaid/flowchart.mmd"));
    assert!(poc_doc.contains("tests/fixtures/mermaid/sequence.mmd"));
    assert!(poc_doc.contains("tests/fixtures/mermaid/er-diagram.mmd"));
    assert!(poc_doc.contains("scripts/mermaid-poc.sh"));

    let roadmap = fs::read_to_string("docs/roadmap.md").expect("read docs/roadmap.md");
    assert!(roadmap.contains("Phase 2.5: Mermaid見た目PoC（#20）"));
}
