use invx::{domain::Invoice, ubl::parser::parse_invoice, validation::validate_invoice};

const SIMPLE_INVOICE: &str = include_str!("fixtures/ubl/simple-invoice.xml");

#[test]
fn valid_ubl_invoice_passes_core_validation() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let invoice = Invoice::try_from(ubl).expect("UBL should map");

    let result = validate_invoice(&invoice);

    assert!(
        result.is_valid(),
        "expected invoice to be valid, got: {:?}",
        result.violations
    );
}

use rust_decimal::Decimal;

#[test]
fn detects_inconsistent_invoice_total() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let mut invoice = Invoice::try_from(ubl).expect("UBL should map");

    invoice.totals.gross_amount.amount = Decimal::new(1_130_000, 2);

    let result = validate_invoice(&invoice);

    assert!(!result.is_valid());

    assert!(result
        .violations
        .iter()
        .any(|violation| violation.code == "CORE-002"));
}
