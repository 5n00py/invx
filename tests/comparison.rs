use invx::{
    compare::compare,
    domain::Invoice,
    ebinterface::parser as ebinterface_parser,
    ubl::{parser as ubl_parser, writer as ubl_writer},
};

const EBINTERFACE_INVOICE: &str = include_str!("fixtures/ebinterface/simple-invoice.xml");

#[test]
fn semantic_comparison_ignores_source_format() {
    let source =
        ebinterface_parser::parse_invoice(EBINTERFACE_INVOICE).expect("ebInterface should parse");

    let original = Invoice::try_from(source).expect("ebInterface should map");

    let ubl_xml =
        ubl_writer::write_invoice(&original).expect("canonical invoice should write as UBL");

    let ubl = ubl_parser::parse_invoice(&ubl_xml).expect("generated UBL should parse");

    let converted = Invoice::try_from(ubl).expect("generated UBL should map");

    let result = compare(&original, &converted);

    assert!(result.is_equal(), "differences: {:#?}", result.differences);
}

#[test]
fn detects_changed_payable_amount() {
    let source =
        ebinterface_parser::parse_invoice(EBINTERFACE_INVOICE).expect("ebInterface should parse");

    let original = Invoice::try_from(source).expect("ebInterface should map");

    let mut changed = original.clone();

    changed.totals.payable_amount.amount += rust_decimal::Decimal::ONE;

    let result = compare(&original, &changed);

    assert!(!result.is_equal());

    assert!(
        result
            .differences
            .iter()
            .any(|difference| { difference.path == "totals.payable_amount" })
    );
}
