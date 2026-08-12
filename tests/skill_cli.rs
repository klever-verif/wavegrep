use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

mod common;
use common::wavepeek_cmd;

fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("skills")
        .join("wavepeek")
}

fn extract(destination: &Path) {
    wavepeek_cmd()
        .arg("skill")
        .arg(destination)
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("Extracted wavepeek skill to"));
}

fn files_below(root: &Path) -> Vec<PathBuf> {
    fn visit(root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).expect("directory should be readable") {
            let path = entry.expect("entry should be readable").path();
            if path.is_dir() {
                visit(root, &path, paths);
            } else {
                paths.push(
                    path.strip_prefix(root)
                        .expect("path should be below root")
                        .into(),
                );
            }
        }
    }

    let mut paths = Vec::new();
    visit(root, root, &mut paths);
    paths.sort();
    paths
}

#[test]
fn skill_extracts_complete_bundle_into_missing_directory() {
    let temp = tempdir().expect("temporary directory should be created");
    let destination = temp.path().join("wavepeek");

    extract(&destination);

    let mut expected = files_below(&source_root());
    expected.retain(|path| path != Path::new("examples/.gitkeep"));
    expected.push(PathBuf::from("manifest.json"));
    expected.sort();
    assert_eq!(files_below(&destination), expected);

    for relative in expected
        .iter()
        .filter(|path| path.as_os_str() != "manifest.json")
    {
        assert_eq!(
            fs::read(destination.join(relative)).expect("extracted file should be readable"),
            fs::read(source_root().join(relative)).expect("source file should be readable"),
            "content mismatch for {}",
            relative.display()
        );
    }
    assert!(destination.join("examples").is_dir());
    assert!(
        fs::read_dir(destination.join("examples"))
            .expect("examples should be readable")
            .next()
            .is_none()
    );

    let manifest: Value = serde_json::from_slice(
        &fs::read(destination.join("manifest.json")).expect("manifest should be readable"),
    )
    .expect("manifest should be valid JSON");
    assert_eq!(manifest["wavepeek_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(manifest["bundle_format_version"], 1);
    assert_eq!(
        manifest
            .as_object()
            .expect("manifest should be object")
            .len(),
        2
    );
}

#[test]
fn skill_accepts_existing_empty_directory() {
    let temp = tempdir().expect("temporary directory should be created");
    let destination = temp.path().join("wavepeek");
    fs::create_dir(&destination).expect("empty destination should be created");

    extract(&destination);

    assert!(destination.join("SKILL.md").is_file());
}

#[test]
fn skill_rejects_nonempty_directory_without_modifying_it() {
    let temp = tempdir().expect("temporary directory should be created");
    let destination = temp.path().join("wavepeek");
    fs::create_dir(&destination).expect("destination should be created");
    fs::write(destination.join(".existing"), "keep me").expect("sentinel should be written");

    wavepeek_cmd()
        .arg("skill")
        .arg(&destination)
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("fatal: file:"))
        .stderr(predicate::str::contains("is not empty"));

    assert_eq!(
        fs::read_to_string(destination.join(".existing")).expect("sentinel should remain"),
        "keep me"
    );
    assert_eq!(files_below(&destination), [PathBuf::from(".existing")]);
}

#[test]
fn skill_rejects_file_destination() {
    let temp = tempdir().expect("temporary directory should be created");
    let destination = temp.path().join("wavepeek");
    fs::write(&destination, "keep me").expect("destination file should be written");

    wavepeek_cmd()
        .arg("skill")
        .arg(&destination)
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("is not a directory"));

    assert_eq!(fs::read_to_string(destination).unwrap(), "keep me");
}

#[test]
fn extracted_markdown_is_self_contained_and_has_no_topic_front_matter() {
    let temp = tempdir().expect("temporary directory should be created");
    let destination = temp.path().join("wavepeek");
    extract(&destination);

    for relative in files_below(&destination)
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
    {
        let markdown = fs::read_to_string(destination.join(&relative)).unwrap();
        assert!(!markdown.contains("](/"), "{}", relative.display());
        if relative.starts_with("references") {
            assert!(!markdown.starts_with("---\n"), "{}", relative.display());
        }
    }
}
