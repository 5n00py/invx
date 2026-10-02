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

#[test]
fn inspect_prints_invoice_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_invx"))
        .arg("inspect")
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

    assert!(stdout.contains("Integration consulting"));
    assert!(stdout.contains("Technical training"));

    assert!(stdout.contains("Adjustments"));
    assert!(stdout.contains("Project discount"));
    assert!(stdout.contains("Administration fee"));

    assert!(stdout.contains("475.00"));
    assert!(stdout.contains("100.00"));

    assert!(stdout.contains("10125.00"));
    assert!(stdout.contains("1925.00"));
    assert!(stdout.contains("12050.00"));

    assert!(stdout.contains("9125.00"));
    assert!(stdout.contains("1825.00"));
}

#[test]
fn validate_accepts_en16931_subset_profile() {
    let output = Command::new(env!("CARGO_BIN_EXE_invx"))
        .arg("validate")
        .arg("--profile")
        .arg("en16931-subset")
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
    assert!(stdout.contains("en16931-subset"));
}

#[test]
fn to_json_prints_canonical_invoice() {
    let output = Command::new(env!("CARGO_BIN_EXE_invx"))
        .arg("to-json")
        .arg(fixture("simple-invoice.xml"))
        .output()
        .expect("invx should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should contain valid JSON");

    assert_eq!(json["id"], "2026-00421");

    assert_eq!(json["currency"], "EUR");

    assert_eq!(json["seller"]["name"], "Example Supplier GmbH");

    assert_eq!(json["buyer"]["name"], "Example Logistics GmbH");

    assert_eq!(json["lines"].as_array().unwrap().len(), 2);

    assert_eq!(json["adjustments"].as_array().unwrap().len(), 2);
}
