use std::{
    collections::HashSet,
    process::Command,
};

use pretty_assertions::assert_eq;
use serde_json::json;

use crate::utils::TestTree;

mod utils;

fn run_and_expect(args: &[&str], exit_code: i32) {
    println!("Running command: leave {}", args.join(" "));
    let code = Command::new(env!("CARGO_BIN_EXE_leave"))
        .args(args)
        .status()
        .unwrap()
        .code()
        .unwrap();
    assert_eq!(
        exit_code, code,
        "Expected exit code {exit_code}, got {code}"
    );
}

fn set<I, T>(args: I) -> HashSet<String>
where
    I: IntoIterator<Item = T>,
    T: ToString,
{
    HashSet::from_iter(args.into_iter().map(|s| s.to_string()))
}

#[test]
pub fn just_files() {
    let tt = TestTree::new(json!({
        "file1": null,
        "file2": null,
        "file3": null,
    }));
    tt.cd_into();
    run_and_expect(&["file1"], 0);
    assert_eq!(set(["file1"]), tt.contents());
}

#[test]
pub fn chdir() {
    let tt = TestTree::new(json!({
        "file1": null,
        "file2": null,
        "file3": null,
    }));
    run_and_expect(&["-C", tt.path().to_str().unwrap(), "file1"], 0);
    assert_eq!(set(["file1"]), tt.contents());
}

#[test]
pub fn dirs() {
    let tt = TestTree::new(json!({
        "file1": null,
        "file2": null,
        "file3": null,
        "dir1": {},
        "dir2": {},
    }));
    tt.cd_into();
    run_and_expect(&["-d", "file1", "dir2"], 0);
    assert_eq!(set(["file1", "dir2"]), tt.contents());
}

#[test]
pub fn relative_paths() {
    let tt = TestTree::new(json!({
        "file1": null,
        "file2": null,
        "file3": null,
        "dir1": {},
        "dir2": {},
    }));
    tt.cd_into();
    run_and_expect(&["-d", "./file1", "./././dir1"], 0);
    assert_eq!(set(["file1", "dir1"]), tt.contents());
}

#[test]
pub fn recursive() {
    let tt = TestTree::new(json!({
        "file1": null,
        "file2": null,
        "file3": null,
        "dir1": {
            "dir2": {
                "file4": null,
            },
            "link1": "dir2"
        },
    }));
    tt.cd_into();
    run_and_expect(&["-r", "file1"], 0);
    assert_eq!(set(["file1"]), tt.contents());
}

/// Tests that directories aren't recursively removed without -r
#[test]
pub fn recursive_without_flag() {
    let tt = TestTree::new(json!({
        "file1": null,
        "file2": null,
        "file3": null,
        "dir1": {
            "dir2": {
                "file4": null,
            },
            "link1": "dir2"
        },
        "dir3": {}
    }));
    tt.cd_into();
    run_and_expect(&["-d", "file1"], 1);
    assert_eq!(set(["file1", "dir1"]), tt.contents());
}

/// Test that empty directories are not removed when no options are given
#[test]
pub fn dirs_fail() {
    let tt = TestTree::new(json!({
        "file1": null,
        "file2": null,
        "file3": null,
        "dir1": {},
        "dir2": {
            "file4": null
        },
    }));
    tt.cd_into();
    run_and_expect(&["file1", "dir2"], 1);
    let contents = tt.contents();
    assert!(contents.contains("dir1"));
    assert!(contents.contains("dir2"));
}

/// Test that nothing is removed if nonexistent files are given as arguments
#[test]
pub fn nonexistent_args() {
    let tt = TestTree::new(json!({
        "file1": null,
    }));
    tt.cd_into();
    run_and_expect(&["file2"], 1);
    assert_eq!(set(["file1"]), tt.contents());
}

/// Test that the existence check is overridden by -f/--force
#[test]
pub fn nonexistent_args_force() {
    let tt = TestTree::new(json!({
        "file1": null,
    }));
    tt.cd_into();
    run_and_expect(&["-f", "file2"], 0);
    assert!(tt.is_empty());
}

#[test]
pub fn continue_on_error() {
    let tt = TestTree::new(json!({
        "a": null,
        "b": null,
        "c": {},
        "d": null,
        "e": null,
        "f": null,
    }));
    tt.cd_into();
    run_and_expect(&["-f"], 1);
    assert_eq!(set(["c"]), tt.contents());
}
