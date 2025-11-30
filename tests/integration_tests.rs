use std::{
    collections::HashSet,
    process::{Command, Output, Stdio},
};

use pretty_assertions::assert_eq;
use serde_json::json;

use crate::utils::TestTree;

mod utils;

fn run_and_expect(args: &[&str], expected_exit_code: i32) -> Output {
    println!("Running command: leave {}", args.join(" "));
    let output = Command::new(env!("CARGO_BIN_EXE_leave"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let actual_exit_code = output.status.code().unwrap();
    assert_eq!(
        expected_exit_code, actual_exit_code,
        "Expected exit code {expected_exit_code}, got {actual_exit_code}"
    );
    output
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

#[test]
pub fn bail_on_nested_file() {
    let tt = TestTree::new(json!({
        "dir": {
            "file": null
        }
    }));
    tt.cd_into();
    let output = run_and_expect(&["dir/file"], 1);
    assert_eq!(set(["dir"]), tt.contents());
    let stderr = str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        "Error: dir/file is not in the current directory; it would be removed anyways. This is likely a mistake. To continue anyways, use -f/--force.\n",
        stderr
    );
}
