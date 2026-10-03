use rust_decimal::Decimal;

use invx::{
    compare::compare,
    domain::{Invoice, PriceBaseQuantity},
    ebinterface::parser as ebinterface_parser,
    ubl::{parser as ubl_parser, writer as ubl_writer},
};

const EBINTERFACE_INVOICE: &str = include_str!("fixtures/ebinterface/simple-invoice.xml");

fn load_invoice() -> Invoice {
    let source =
        ebinterface_parser::parse_invoice(EBINTERFACE_INVOICE).expect("ebInterface should parse");

    Invoice::try_from(source).expect("ebInterface should map")
}

#[test]
fn semantic_comparison_ignores_source_format() {
    let original = load_invoice();

    let ubl_xml =
        ubl_writer::write_invoice(&original).expect("canonical invoice should write as UBL");

    let ubl = ubl_parser::parse_invoice(&ubl_xml).expect("generated UBL should parse");

    let converted = Invoice::try_from(ubl).expect("generated UBL should map");

    let result = compare(&original, &converted);

    assert!(result.is_equal(), "differences: {:#?}", result.differences);
}

#[test]
fn detects_changed_payable_amount() {
    let original = load_invoice();

    let mut changed = original.clone();

    changed.totals.payable_amount.amount += Decimal::ONE;

    let result = compare(&original, &changed);

    assert!(!result.is_equal());

    assert!(
        result
            .differences
            .iter()
            .any(|difference| { difference.path == "totals.payable_amount" })
    );
}

#[test]
fn detects_changed_line_adjustment_amount() {
    let original = load_invoice();

    let mut changed = original.clone();

    changed.lines[0].adjustments[0].amount.amount += Decimal::ONE;

    let result = compare(&original, &changed);

    assert!(!result.is_equal());

    assert!(
        result
            .differences
            .iter()
            .any(|difference| { difference.path == "lines[id=1].adjustments[0].amount" }),
        "differences: {:#?}",
        result.differences
    );
}

#[test]
fn implicit_and_explicit_price_base_quantity_one_are_equal() {
    let mut implicit = load_invoice();

    implicit.lines[0].price_base_quantity = None;

    let mut explicit = implicit.clone();

    explicit.lines[0].price_base_quantity = Some(PriceBaseQuantity {
        quantity: Decimal::ONE,

        unit_code: Some("HUR".to_string()),
    });

    let result = compare(&implicit, &explicit);

    assert!(result.is_equal(), "differences: {:#?}", result.differences);
}

#[test]
fn detects_changed_price_base_quantity() {
    let original = load_invoice();

    let mut changed = original.clone();

    changed.lines[0].price_base_quantity = Some(PriceBaseQuantity {
        quantity: Decimal::new(10, 0),

        unit_code: Some("HUR".to_string()),
    });

    let result = compare(&original, &changed);

    assert!(!result.is_equal());

    assert!(
        result
            .differences
            .iter()
            .any(|difference| { difference.path == "lines[id=1].price_base_quantity.quantity" }),
        "differences: {:#?}",
        result.differences
    );
}
