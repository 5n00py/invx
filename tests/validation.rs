use rust_decimal::Decimal;

use invx::{
    domain::Invoice, ubl::parser::parse_invoice, validation::ValidationProfile,
    validation::validate,
};

const SIMPLE_INVOICE: &str = include_str!("fixtures/ubl/simple-invoice.xml");

#[test]
fn valid_ubl_invoice_passes_core_validation() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let invoice = Invoice::try_from(ubl).expect("UBL should map");

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(
        result.is_valid(),
        "expected invoice to be valid, got: {:?}",
        result.violations
    );
}

#[test]
fn detects_inconsistent_invoice_total() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let mut invoice = Invoice::try_from(ubl).expect("UBL should map");

    invoice.totals.gross_amount.amount = Decimal::new(1_130_000, 2);

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(!result.is_valid());

    assert!(
        result
            .violations
            .iter()
            .any(|violation| violation.code == "CORE-002")
    );
}

#[test]
fn credit_transfer_requires_payment_account() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let mut invoice = Invoice::try_from(ubl).expect("UBL should map");

    let payment = invoice
        .payment
        .as_mut()
        .expect("invoice should have payment information");

    payment.payee_account = None;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(!result.is_valid());

    assert!(
        result
            .violations
            .iter()
            .any(|violation| violation.code == "BR-61")
    );
}

#[test]
fn credit_transfer_with_payment_account_is_valid() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let invoice = Invoice::try_from(ubl).expect("UBL should map");

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        result.is_valid(),
        "expected no violations, got: {:?}",
        result.violations
    );
}

#[test]
fn non_credit_transfer_does_not_require_payment_account() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let mut invoice = Invoice::try_from(ubl).expect("UBL should map");

    let payment = invoice
        .payment
        .as_mut()
        .expect("invoice should have payment information");

    payment.means_code = "10".to_string();
    payment.payee_account = None;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(result.is_valid());
}

#[test]
fn detects_inconsistent_vat_breakdown_total() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let mut invoice = Invoice::try_from(ubl).expect("UBL should map");

    invoice.vat_breakdown[0].tax_amount.amount = Decimal::new(180_000, 2);

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(
        result
            .violations
            .iter()
            .any(|violation| violation.code == "CORE-006")
    );
}
