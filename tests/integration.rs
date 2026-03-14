use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_help_shows_description() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("mwvm simulation"));
}

#[test]
fn cli_version_works() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("morpheum"));
}

#[test]
fn tx_subcommand_shows_help() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("tx")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn query_subcommand_shows_help() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("query")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn mwvm_subcommand_shows_help() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("mwvm")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn mcp_subcommand_shows_help() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("mcp")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn a2a_subcommand_shows_help() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("a2a")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn keys_subcommand_shows_help() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("keys")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn config_show_runs() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("config")
        .arg("show")
        .assert()
        .success();
}

#[test]
fn config_path_runs() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("config")
        .arg("path")
        .assert()
        .success();
}

#[test]
fn status_runs() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("status")
        .assert()
        .success();
}

#[test]
fn mwvm_infer_runs() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("mwvm")
        .arg("infer")
        .arg("--model")
        .arg("llama-3.1-8b-q4")
        .arg("--prompt")
        .arg("Hello from integration test")
        .assert()
        .success();
}

#[test]
fn mwvm_status_runs() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("mwvm")
        .arg("status")
        .assert()
        .success();
}

#[test]
fn unknown_subcommand_fails() {
    Command::cargo_bin("morpheum")
        .unwrap()
        .arg("nonexistent")
        .assert()
        .failure();
}
