// rexpect and assert_cmd generate bad vibes when mixed together

extern crate checklist;
extern crate rexpect;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate assert_cmd;

use assert_cmd::cargo::cargo_bin;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use rexpect::spawn;

const TEST_TIMEOUT_MS: Option<u64> = Some(2000);

fn get_command_path() -> String {
    lazy_static! {
        static ref BIN_PATH: String = cargo_bin(crate_name!())
            .to_path_buf()
            .to_str()
            .unwrap()
            .to_string();
    }
    BIN_PATH.to_string()
}

fn with_checklist(tempdir: &TempDir, contents: &str) -> String {
    let checklist_file = tempdir.child(".checklist.yml");

    checklist_file.write_str(contents).unwrap();

    checklist_file.path().to_str().unwrap().to_string()
}

#[test]
fn test_basic_usage() {
    let t = TempDir::new().unwrap();
    let temp = scopeguard::guard(t, |t| {
        t.close().unwrap();
    });

    let checklist_path = with_checklist(&temp, "committing:\n- test");

    let mut p = spawn(
        &format!("{} --checklist {}", get_command_path(), checklist_path),
        TEST_TIMEOUT_MS,
    )
    .unwrap();

    p.exp_string("test").unwrap();
    p.send_line("y").unwrap();
    p.exp_string("all clear!").unwrap();
    p.exp_eof().unwrap();
}

#[test]
fn test_basic_rejection() {
    let t = TempDir::new().unwrap();
    let temp = scopeguard::guard(t, |t| {
        t.close().unwrap();
    });

    let checklist_path = with_checklist(&temp, "committing:\n- test");

    let mut p = spawn(
        &format!("{} --checklist {}", get_command_path(), checklist_path),
        TEST_TIMEOUT_MS,
    )
    .unwrap();

    p.exp_string("test").unwrap();
    p.send_line("n").unwrap();
    p.exp_string("please fix and start again").unwrap();
    p.exp_eof().unwrap();
}

#[test]
fn test_multi_success() {
    let t = TempDir::new().unwrap();
    let temp = scopeguard::guard(t, |t| {
        t.close().unwrap();
    });

    let checklist_path = with_checklist(&temp, "committing:\n- test\n- test2");

    let mut p = spawn(
        &format!("{} --checklist {}", get_command_path(), checklist_path),
        TEST_TIMEOUT_MS,
    )
    .unwrap();

    p.exp_string("test").unwrap();
    p.send_line("y").unwrap();
    p.exp_string("Great! Continuing...").unwrap();
    p.send_line("y").unwrap();
    p.exp_string("all clear!").unwrap();
    p.exp_eof().unwrap();
}

#[test]
fn test_multi_rejection() {
    let t = TempDir::new().unwrap();
    let temp = scopeguard::guard(t, |t| {
        t.close().unwrap();
    });

    let checklist_path = with_checklist(&temp, "committing:\n- test\n- test2");

    let mut p = spawn(
        &format!("{} --checklist {}", get_command_path(), checklist_path),
        TEST_TIMEOUT_MS,
    )
    .unwrap();

    p.exp_string("test").unwrap();
    p.send_line("y").unwrap();
    p.exp_string("Great! Continuing...").unwrap();
    p.send_line("n").unwrap();
    p.exp_string("please fix and start again").unwrap();
    p.exp_eof().unwrap();
}
