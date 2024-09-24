use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_list_patterns_empty() -> Result<(), Box<dyn std::error::Error>> {
    // Test listing patterns when no patterns exist
    let temp_dir = tempdir()?;
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path()).arg("--list");
    cmd.assert().success().stdout(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_save_pattern_and_list() -> Result<(), Box<dyn std::error::Error>> {
    // Test saving a pattern and then listing it
    let temp_dir = tempdir()?;
    let gf_dir = temp_dir.path().join(".config/gf");
    fs::create_dir_all(&gf_dir)?;

    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--save", "testpattern", "-Hnri", "search-pattern"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path()).arg("--list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("testpattern"));

    Ok(())
}

#[test]
fn test_save_pattern_without_name() -> Result<(), Box<dyn std::error::Error>> {
    // Test saving a pattern without providing a name
    let temp_dir = tempdir()?;
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path()).arg("--save");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Name cannot be empty"));
    Ok(())
}

#[test]
fn test_save_pattern_without_pattern() -> Result<(), Box<dyn std::error::Error>> {
    // Test saving a pattern without providing the pattern string
    let temp_dir = tempdir()?;
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--save", "test pattern"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Pattern cannot be empty"));
    Ok(())
}

#[test]
fn test_use_nonexistent_pattern() -> Result<(), Box<dyn std::error::Error>> {
    // Test using a pattern that doesn't exist
    let temp_dir = tempdir()?;
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path()).arg("nonexistentpattern");
    cmd.assert().failure().stderr(predicate::str::contains(
        "No such pattern 'nonexistentpattern'",
    ));
    Ok(())
}

#[test]
fn test_dump_pattern() -> Result<(), Box<dyn std::error::Error>> {
    // Test dumping a saved pattern
    let temp_dir = tempdir()?;
    let gf_dir = temp_dir.path().join(".config/gf");
    fs::create_dir_all(&gf_dir)?;

    // Save a pattern with an engine and flags
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path()).args([
        "--save",
        "testpattern",
        "--engine",
        "rg",
        "-Hnri",
        "search-pattern",
    ]);
    cmd.assert().success();

    // Dump the pattern
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--dump", "testpattern", "/path/to/files"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "rg -Hnri \"search-pattern\" /path/to/files",
    ));
    Ok(())
}

#[test]
fn test_execute_pattern_with_piped_input() -> Result<(), Box<dyn std::error::Error>> {
    // Test executing a pattern with piped input
    let temp_dir = tempdir()?;
    let gf_dir = temp_dir.path().join(".config/gf");
    fs::create_dir_all(&gf_dir)?;

    // Save a simple pattern
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--save", "testpattern", "-nri", "test"]);
    cmd.assert().success();

    // Create a temporary file to grep
    let temp_file_path = temp_dir.path().join("testfile.txt");
    let mut temp_file = File::create(&temp_file_path)?;
    writeln!(temp_file, "This is a test line")?;
    writeln!(temp_file, "Another line")?;
    drop(temp_file);

    // Use the pattern with piped input
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .arg("testpattern")
        .write_stdin(fs::read_to_string(&temp_file_path)?);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("This is a test line"));
    Ok(())
}

#[test]
fn test_pattern_file_malformed() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior when the pattern file is malformed
    let temp_dir = tempdir()?;
    let gf_dir = temp_dir.path().join(".config/gf");
    fs::create_dir_all(&gf_dir)?;

    // Create a malformed pattern file
    let pattern_file_path = gf_dir.join("malformedpattern.json");
    let mut pattern_file = File::create(&pattern_file_path)?;
    writeln!(pattern_file, "{{ malformed json")?;
    drop(pattern_file);

    // Try to use the malformed pattern
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path()).arg("malformedpattern");
    cmd.assert().failure().stderr(
        predicate::str::contains("Pattern file").and(predicate::str::contains("is malformed")),
    );
    Ok(())
}

#[test]
fn test_pattern_with_no_patterns() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior when the pattern file contains no patterns
    let temp_dir = tempdir()?;
    let gf_dir = temp_dir.path().join(".config/gf");
    fs::create_dir_all(&gf_dir)?;

    // Create a pattern file with empty patterns
    let pattern_file_path = gf_dir.join("emptypattern.json");
    let mut pattern_file = File::create(&pattern_file_path)?;
    writeln!(pattern_file, r#"{{ "flags": "-Hnri" }}"#)?;
    drop(pattern_file);

    // Try to use the pattern
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path()).arg("emptypattern");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("contains no pattern(s)"));
    Ok(())
}

#[test]
fn test_save_pattern_with_existing_name() -> Result<(), Box<dyn std::error::Error>> {
    // Test saving a pattern when a pattern with the same name already exists
    let temp_dir = tempdir()?;
    let gf_dir = temp_dir.path().join(".config/gf");
    fs::create_dir_all(&gf_dir)?;

    // Save the initial pattern
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--save", "testpattern", "-Hnri", "search-pattern"]);
    cmd.assert().success();

    // Attempt to save another pattern with the same name
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--save", "testpattern", "-Hnri", "another-pattern"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to create pattern file"));

    Ok(())
}

#[test]
fn test_dump_pattern_with_no_flags() -> Result<(), Box<dyn std::error::Error>> {
    // Test dumping a pattern that has no flags
    let temp_dir = tempdir()?;
    let gf_dir = temp_dir.path().join(".config/gf");
    fs::create_dir_all(&gf_dir)?;

    // Save a pattern without flags by providing an empty string for flags
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--save", "noflagpattern", "", "search-pattern"]);
    cmd.assert().success();

    // Dump the pattern
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--dump", "noflagpattern", "/path/to/files"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "grep \"search-pattern\" /path/to/files",
    ));
    Ok(())
}

#[test]
fn test_list_patterns_with_multiple_patterns() -> Result<(), Box<dyn std::error::Error>> {
    // Test listing when multiple patterns exist
    let temp_dir = tempdir()?;
    let gf_dir = temp_dir.path().join(".config/gf");
    fs::create_dir_all(&gf_dir)?;

    // Save multiple patterns
    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--save", "pattern1", "-nri", "test1"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path())
        .args(["--save", "pattern2", "-nri", "test2"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("gf")?;
    cmd.env("HOME", temp_dir.path()).arg("--list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pattern1").and(predicate::str::contains("pattern2")));
    Ok(())
}
