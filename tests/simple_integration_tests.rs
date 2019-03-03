extern crate checklist;
extern crate rexpect;
#[macro_use]
extern crate assert_cmd;

use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_help() {
    Command::cargo_bin(crate_name!())
        .unwrap()
        .arg("-h")
        .assert()
        .success();
}

#[test]
fn test_nonexistent_flag() {
    Command::cargo_bin(crate_name!())
        .unwrap()
        .arg("--wrong-flag")
        .assert()
        .failure();
}
