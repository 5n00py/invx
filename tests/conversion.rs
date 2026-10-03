use chrono::NaiveDate;
use rust_decimal::Decimal;

use invx::{
    compare::compare,
    conversion::{
        ConversionDiagnostics, ConversionError, ConversionImpact, TargetFormat, analyze, convert,
    },
    domain::Invoice,
    ebinterface::parser as ebinterface_parser,
    ubl::parser as ubl_parser,
};

const UBL_INVOICE: &str = include_str!("fixtures/ubl/simple-invoice.xml");

const EBINTERFACE_INVOICE: &str = include_str!("fixtures/ebinterface/simple-invoice.xml");

fn load_ubl_invoice() -> Invoice {
    let source = ubl_parser::parse_invoice(UBL_INVOICE).expect("UBL should parse");

    Invoice::try_from(source).expect("UBL should map")
}

fn load_ebinterface_invoice() -> Invoice {
    let source =
        ebinterface_parser::parse_invoice(EBINTERFACE_INVOICE).expect("ebInterface should parse");

    Invoice::try_from(source).expect("ebInterface should map")
}

fn load_usd_invoice() -> Invoice {
    let usd_xml = UBL_INVOICE.replace("EUR", "USD");

    let source = ubl_parser::parse_invoice(&usd_xml).expect("USD UBL should parse");

    Invoice::try_from(source).expect("USD UBL should map")
}

fn has_issue(
    diagnostics: &ConversionDiagnostics,
    code: &str,
    path: &str,
    impact: ConversionImpact,
) -> bool {
    diagnostics
        .issues
        .iter()
        .any(|issue| issue.code == code && issue.path == path && issue.impact == impact)
}

#[test]
fn reports_all_known_ebinterface_conversion_issues() {
    let invoice = load_ubl_invoice();

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "EBI-ADJ-001",
            "adjustments[1].base_amount",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );

    assert!(
        diagnostics.issues.iter().any(|issue| {
            issue.code == "EBI-PAY-002" && issue.impact == ConversionImpact::Informational
        }),
        "diagnostics: {:#?}",
        diagnostics.issues
    );

    assert!(
        diagnostics.issues.iter().any(|issue| {
            issue.code == "EBI-PAY-003" && issue.impact == ConversionImpact::Informational
        }),
        "diagnostics: {:#?}",
        diagnostics.issues
    );

    assert!(diagnostics.has_blocking_issues());
}

#[test]
fn conversion_refuses_target_with_blocking_issues() {
    let invoice = load_ubl_invoice();

    let error =
        convert(&invoice, TargetFormat::EbInterface6p1).expect_err("conversion should be blocked");

    match error {
        ConversionError::Incompatible {
            target,
            diagnostics,
        } => {
            assert_eq!(target, TargetFormat::EbInterface6p1);

            assert!(diagnostics.has_blocking_issues());
        }

        other => {
            panic!("unexpected conversion error: {other}");
        }
    }
}

#[test]
fn detects_ebinterface_price_base_quantity_unit_mismatch() {
    let mut invoice = load_ebinterface_invoice();

    let base_quantity = invoice.lines[0]
        .price_base_quantity
        .as_mut()
        .expect("fixture should contain a price base quantity");

    base_quantity.unit_code = Some("DAY".to_string());

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "EBI-LINE-004",
            "lines[id=1].price_base_quantity.unit_code",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn detects_missing_ebinterface_line_adjustment_base_amount() {
    let mut invoice = load_ebinterface_invoice();

    invoice.lines[0].adjustments[0].base_amount = None;

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "EBI-LINE-ADJ-001",
            "lines[id=1].adjustments[0].base_amount",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn missing_line_adjustment_base_amount_blocks_conversion_before_writer() {
    let mut invoice = load_ebinterface_invoice();

    invoice.lines[0].adjustments[0].base_amount = None;

    let error = convert(&invoice, TargetFormat::EbInterface6p1)
        .expect_err("conversion should be blocked by preflight");

    match error {
        ConversionError::Incompatible {
            target,
            diagnostics,
        } => {
            assert_eq!(target, TargetFormat::EbInterface6p1);

            assert!(
                has_issue(
                    &diagnostics,
                    "EBI-LINE-ADJ-001",
                    "lines[id=1].adjustments[0].base_amount",
                    ConversionImpact::Blocking,
                ),
                "diagnostics: {:#?}",
                diagnostics.issues
            );
        }

        other => {
            panic!("expected incompatibility error, got: {other}");
        }
    }
}

#[test]
fn detects_ebinterface_price_base_quantity_precision_overflow() {
    let mut invoice = load_ebinterface_invoice();

    let base_quantity = invoice.lines[0]
        .price_base_quantity
        .as_mut()
        .expect("fixture should contain a price base quantity");

    /*
     * 1.00001 requires five fractional
     * decimal places.
     */
    base_quantity.quantity = Decimal::new(100_001, 5);

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "EBI-NUM-001",
            "lines[id=1].price_base_quantity.quantity",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn detects_ebinterface_line_adjustment_amount_precision_overflow() {
    let mut invoice = load_ebinterface_invoice();

    /*
     * 600.001 cannot be represented by the
     * ebInterface writer without rounding.
     */
    invoice.lines[0].adjustments[0].amount.amount = Decimal::new(600_001, 3);

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "EBI-NUM-001",
            "lines[id=1].adjustments[0].amount",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn detects_ebinterface_line_adjustment_base_amount_precision_overflow() {
    let mut invoice = load_ebinterface_invoice();

    let base_amount = invoice.lines[0].adjustments[0]
        .base_amount
        .as_mut()
        .expect("fixture adjustment should contain a base amount");

    base_amount.amount = Decimal::new(10_000_001, 3);

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "EBI-NUM-001",
            "lines[id=1].adjustments[0].base_amount",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn detects_ebinterface_line_adjustment_percentage_precision_overflow() {
    let mut invoice = load_ebinterface_invoice();

    invoice.lines[0].adjustments[0].percentage = Some(Decimal::new(6_001, 3));

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "EBI-NUM-001",
            "lines[id=1].adjustments[0].percentage",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn reports_multiple_line_adjustment_reasons_as_lossy_for_ebinterface() {
    let mut invoice = load_ebinterface_invoice();

    invoice.lines[0].adjustments[0]
        .reasons
        .push("Additional negotiated discount".to_string());

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "EBI-LINE-ADJ-002",
            "lines[id=1].adjustments[0].reasons",
            ConversionImpact::Lossy,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );

    assert!(diagnostics.has_lossy_issues());

    assert!(
        !diagnostics.has_blocking_issues(),
        "unexpected blocking diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn detects_line_adjustment_amount_currency_mismatch_for_ubl() {
    let mut invoice = load_ebinterface_invoice();

    let usd_invoice = load_usd_invoice();

    invoice.lines[0].adjustments[0].amount.currency = usd_invoice.currency.clone();

    /*
     * Currency checking is shared by both
     * target analyzers, so verify it through
     * UBL rather than only ebInterface.
     */
    let diagnostics = analyze(&invoice, TargetFormat::Ubl);

    assert!(
        has_issue(
            &diagnostics,
            "CONV-CUR-001",
            "lines[id=1].adjustments[0].amount",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn detects_line_adjustment_base_amount_currency_mismatch() {
    let mut invoice = load_ebinterface_invoice();

    let usd_invoice = load_usd_invoice();

    let base_amount = invoice.lines[0].adjustments[0]
        .base_amount
        .as_mut()
        .expect("fixture adjustment should contain a base amount");

    base_amount.currency = usd_invoice.currency.clone();

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        has_issue(
            &diagnostics,
            "CONV-CUR-001",
            "lines[id=1].adjustments[0].base_amount",
            ConversionImpact::Blocking,
        ),
        "diagnostics: {:#?}",
        diagnostics.issues
    );
}

#[test]
fn ebinterface_to_ubl_conversion_succeeds_with_information() {
    let invoice = load_ebinterface_invoice();

    let output =
        convert(&invoice, TargetFormat::Ubl).expect("ebInterface invoice should convert to UBL");

    assert!(
        output.diagnostics.issues.iter().any(|issue| {
            issue.code == "UBL-PAY-002" && issue.impact == ConversionImpact::Informational
        }),
        "diagnostics: {:#?}",
        output.diagnostics.issues
    );

    assert!(
        output
            .xml
            .contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2")
    );

    let written = ubl_parser::parse_invoice(&output.xml).expect("generated UBL should parse");

    let roundtrip = Invoice::try_from(written).expect("generated UBL should map");

    /*
     * This now verifies more than just totals:
     *
     * - due date
     * - price base quantity
     * - line allowances / charges
     * - reason codes / reasons
     * - line tax
     * - VAT breakdown
     * - parties
     * - totals
     * - payment business semantics
     *
     * The comparer intentionally ignores the
     * syntax-specific payment means code.
     */
    let comparison = compare(&invoice, &roundtrip);

    assert!(
        comparison.is_equal(),
        "semantic differences after ebInterface -> UBL conversion: {:#?}",
        comparison.differences
    );
}

#[test]
fn fixtures_map_due_date() {
    let expected = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();

    assert_eq!(load_ubl_invoice().due_date, Some(expected));
    assert_eq!(load_ebinterface_invoice().due_date, Some(expected));
}
