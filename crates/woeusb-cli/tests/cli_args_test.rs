use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("woeusb").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Flash a Windows ISO image"))
        .stdout(predicate::str::contains("--device"))
        .stdout(predicate::str::contains("--partition"));
}

#[test]
fn test_cli_list() {
    let mut cmd = Command::cargo_bin("woeusb").unwrap();
    cmd.arg("list")
        .assert()
        .success();
}

#[test]
fn test_cli_flash_dry_run_subcommand() {
    let dir = tempdir().unwrap();
    let iso = dir.path().join("win.iso");
    fs::write(&iso, "iso data").unwrap();

    let mut cmd = Command::cargo_bin("woeusb").unwrap();
    cmd.args(["flash", "--dry-run", "-y"])
        .arg(&iso)
        .arg("/dev/sdb")
        .assert()
        .success()
        .stdout(predicate::str::contains("Success!"));
}

#[test]
fn test_cli_legacy_flags_dry_run() {
    let dir = tempdir().unwrap();
    let iso = dir.path().join("win_legacy.iso");
    fs::write(&iso, "iso data").unwrap();

    let mut cmd = Command::cargo_bin("woeusb").unwrap();
    cmd.args(["--device", "--dry-run", "-y"])
        .arg(&iso)
        .arg("/dev/sdb")
        .assert()
        .success()
        .stdout(predicate::str::contains("Success!"));
}

#[test]
fn test_cli_completions_bash() {
    let mut cmd = Command::cargo_bin("woeusb").unwrap();
    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -F _woeusb"));

    let mut cmd2 = Command::cargo_bin("woeusb").unwrap();
    cmd2.args(["completions", "bash", "--bin", "woeusb-ng"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -F _woeusb__ng"));
}

#[test]
fn test_cli_completions_fish() {
    let mut cmd = Command::cargo_bin("woeusb").unwrap();
    cmd.args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c woeusb"));

    let mut cmd2 = Command::cargo_bin("woeusb").unwrap();
    cmd2.args(["completions", "fish", "--bin", "woeusb-ng"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c woeusb-ng"));
}

#[test]
fn test_cli_completions_zsh() {
    let mut cmd = Command::cargo_bin("woeusb").unwrap();
    cmd.args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef woeusb"));

    let mut cmd2 = Command::cargo_bin("woeusb").unwrap();
    cmd2.args(["completions", "zsh", "--bin", "woeusb-ng"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef woeusb-ng"));
}
