use rust_decimal::Decimal;

use invx::{
    domain::{Invoice, PaymentMethod},
    ubl::parser::parse_invoice,
    validation::{ValidationProfile, validate},
};

const SIMPLE_INVOICE: &str = include_str!("fixtures/ubl/simple-invoice.xml");

fn load_invoice() -> Invoice {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    Invoice::try_from(ubl).expect("UBL should map")
}

fn load_usd_invoice() -> Invoice {
    let usd_xml = SIMPLE_INVOICE.replace("EUR", "USD");

    let ubl = parse_invoice(&usd_xml).expect("USD UBL should parse");

    Invoice::try_from(ubl).expect("USD UBL should map")
}

fn has_violation(result: &invx::validation::ValidationResult, code: &str) -> bool {
    result
        .violations
        .iter()
        .any(|violation| violation.code == code)
}

#[test]
fn valid_ubl_invoice_passes_core_validation() {
    let invoice = load_invoice();

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(
        result.is_valid(),
        "expected invoice to be valid, got: {:?}",
        result.violations
    );
}

#[test]
fn detects_inconsistent_invoice_total() {
    let mut invoice = load_invoice();

    invoice.totals.gross_amount.amount = Decimal::new(1_130_000, 2);

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(!result.is_valid());
    assert!(has_violation(&result, "CORE-002"));
}

#[test]
fn detects_inconsistent_vat_breakdown_total() {
    let mut invoice = load_invoice();

    invoice.vat_breakdown[0].tax_amount.amount = Decimal::new(180_000, 2);

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(has_violation(&result, "CORE-006"));
}

#[test]
fn valid_line_adjustment_arithmetic_passes_core_validation() {
    let invoice = load_invoice();

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(
        !has_violation(&result, "CORE-011"),
        "expected line arithmetic to be valid, got: {:?}",
        result.violations
    );

    /*
     * First fixture line:
     *
     * 10 × 1000 / 1
     * - 600 allowance
     * + 100 charge
     * = 9500
     */
    let line = &invoice.lines[0];

    assert_eq!(line.net_amount.amount, Decimal::new(950_000, 2));
}

#[test]
fn detects_non_positive_price_base_quantity() {
    let mut invoice = load_invoice();

    let base_quantity = invoice.lines[0]
        .price_base_quantity
        .as_mut()
        .expect("fixture line should contain a price base quantity");

    base_quantity.quantity = Decimal::ZERO;

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(!result.is_valid());

    assert!(
        result.violations.iter().any(|violation| {
            violation.code == "CORE-010"
                && violation.field.as_deref() == Some("lines[0].price_base_quantity.quantity")
        }),
        "violations: {:?}",
        result.violations
    );

    assert!(
        !has_violation(&result, "CORE-011"),
        "invalid base quantity should not also cause CORE-011: {:?}",
        result.violations
    );
}

#[test]
fn detects_negative_price_base_quantity() {
    let mut invoice = load_invoice();

    let base_quantity = invoice.lines[0]
        .price_base_quantity
        .as_mut()
        .expect("fixture line should contain a price base quantity");

    base_quantity.quantity = Decimal::NEGATIVE_ONE;

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(
        has_violation(&result, "CORE-010"),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn detects_inconsistent_line_net_after_adjustment_change() {
    let mut invoice = load_invoice();

    assert!(
        !invoice.lines[0].adjustments.is_empty(),
        "fixture should contain line adjustments"
    );

    invoice.lines[0].adjustments[0].amount.amount += Decimal::ONE;

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(!result.is_valid());

    assert!(
        result.violations.iter().any(|violation| {
            violation.code == "CORE-011"
                && violation.field.as_deref() == Some("lines[0].net_amount")
        }),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn detects_line_adjustment_amount_currency_mismatch() {
    let mut invoice = load_invoice();
    let usd_invoice = load_usd_invoice();

    invoice.lines[0].adjustments[0].amount.currency = usd_invoice.currency.clone();

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(
        result.violations.iter().any(|violation| {
            violation.code == "CORE-005"
                && violation.field.as_deref() == Some("lines[0].adjustments[0].amount")
        }),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn detects_line_adjustment_base_amount_currency_mismatch() {
    let mut invoice = load_invoice();
    let usd_invoice = load_usd_invoice();

    let base_amount = invoice.lines[0].adjustments[0]
        .base_amount
        .as_mut()
        .expect("fixture line adjustment should contain a base amount");

    base_amount.currency = usd_invoice.currency.clone();

    let result = validate(&invoice, ValidationProfile::Core);

    assert!(
        result.violations.iter().any(|violation| {
            violation.code == "CORE-005"
                && violation.field.as_deref() == Some("lines[0].adjustments[0].base_amount")
        }),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn credit_transfer_requires_payment_account() {
    let mut invoice = load_invoice();

    let payment = invoice
        .payment
        .as_mut()
        .expect("invoice should have payment information");

    payment.payee_account = None;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(!result.is_valid());
    assert!(has_violation(&result, "BR-61"));
}

#[test]
fn credit_transfer_with_payment_account_is_valid() {
    let invoice = load_invoice();

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        result.is_valid(),
        "expected no violations, got: {:?}",
        result.violations
    );
}

#[test]
fn non_bank_transfer_does_not_require_payment_account() {
    let mut invoice = load_invoice();

    let payment = invoice
        .payment
        .as_mut()
        .expect("invoice should have payment information");

    payment.method = PaymentMethod::Other;
    payment.means_code = Some("10".to_string());
    payment.payee_account = None;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(result.is_valid());
}

#[test]
fn realistic_invoice_passes_en16931_subset() {
    let invoice = load_invoice();

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        result.is_valid(),
        "expected valid invoice, got: {:?}",
        result.violations
    );
}

#[test]
fn invoice_line_requires_vat_category() {
    let mut invoice = load_invoice();

    invoice.lines[0].tax.category_code = None;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(has_violation(&result, "BR-CO-04"));
}

#[test]
fn standard_rated_line_requires_positive_rate() {
    let mut invoice = load_invoice();

    invoice.lines[0].tax.rate = Decimal::ZERO;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(has_violation(&result, "BR-S-05"));
}

#[test]
fn vat_breakdown_tax_amount_must_match_rate() {
    let mut invoice = load_invoice();

    invoice.vat_breakdown[0].tax_amount.amount = Decimal::new(180_000, 2);

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(has_violation(&result, "BR-CO-17"));
}

#[test]
fn item_net_price_must_not_be_negative() {
    let mut invoice = load_invoice();

    invoice.lines[0].unit_price.amount = -Decimal::ONE;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        has_violation(&result, "BR-27"),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn price_base_quantity_unit_must_match_line_unit() {
    let mut invoice = load_invoice();

    let base_quantity = invoice.lines[0]
        .price_base_quantity
        .as_mut()
        .expect("fixture line should contain a price base quantity");

    base_quantity.unit_code = Some("DAY".to_string());

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        result.violations.iter().any(|violation| {
            violation.code == "PEPPOL-EN16931-R130"
                && violation.field.as_deref() == Some("lines[0].price_base_quantity.unit_code")
        }),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn adjustment_percentage_requires_base_amount() {
    let mut invoice = load_invoice();

    let adjustment = &mut invoice.lines[0].adjustments[0];

    assert!(adjustment.percentage.is_some());

    adjustment.base_amount = None;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        result.violations.iter().any(|violation| {
            violation.code == "PEPPOL-EN16931-R041"
                && violation.field.as_deref() == Some("lines[0].adjustments[0].base_amount")
        }),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn adjustment_base_amount_requires_percentage() {
    let mut invoice = load_invoice();

    let adjustment = &mut invoice.lines[0].adjustments[0];

    assert!(adjustment.base_amount.is_some());

    adjustment.percentage = None;

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        result.violations.iter().any(|violation| {
            violation.code == "PEPPOL-EN16931-R042"
                && violation.field.as_deref() == Some("lines[0].adjustments[0].percentage")
        }),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn adjustment_amount_must_match_base_and_percentage() {
    let mut invoice = load_invoice();

    /*
     * Fixture allowance:
     *
     * 10000 × 6 / 100 = 600
     *
     * Change the declared amount to 600.03,
     * which is outside the 0.02 tolerance.
     */
    invoice.lines[0].adjustments[0].amount.amount += Decimal::new(3, 2);

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        result.violations.iter().any(|violation| {
            violation.code == "PEPPOL-EN16931-R040"
                && violation.field.as_deref() == Some("lines[0].adjustments[0].amount")
        }),
        "violations: {:?}",
        result.violations
    );
}

#[test]
fn adjustment_amount_within_tolerance_is_accepted() {
    let mut invoice = load_invoice();

    /*
     * R040 allows a difference of exactly 0.02.
     */
    invoice.lines[0].adjustments[0].amount.amount += Decimal::new(2, 2);

    let result = validate(&invoice, ValidationProfile::En16931Subset);

    assert!(
        !has_violation(&result, "PEPPOL-EN16931-R040"),
        "expected R040 tolerance to accept 0.02 difference, got: {:?}",
        result.violations
    );
}
