use std::{path::PathBuf, process::Command};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("ubl")
        .join(name)
}

#[test]
fn validate_succeeds_for_valid_invoice() {
    let output = Command::new(env!("CARGO_BIN_EXE_invx"))
        .arg("validate")
        .arg(fixture("simple-invoice.xml"))
        .output()
        .expect("invx should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("VALID"));
}

#[test]
fn validate_fails_for_invalid_invoice() {
    let output = Command::new(env!("CARGO_BIN_EXE_invx"))
        .arg("validate")
        .arg(fixture("invalid-total.xml"))
        .output()
        .expect("invx should run");

    assert_eq!(output.status.code(), Some(1));

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("INVALID"));
    assert!(stdout.contains("CORE-002"));
}
