use invx::{
    domain::Invoice,
    ubl::{parser::parse_invoice, writer::write_invoice},
};

const SIMPLE_INVOICE: &str = include_str!("fixtures/ubl/simple-invoice.xml");

#[test]
fn canonical_invoice_can_be_written_as_ubl_and_read_again() {
    let source = parse_invoice(SIMPLE_INVOICE).expect("source UBL should parse");

    let original = Invoice::try_from(source).expect("source UBL should map");

    let xml = write_invoice(&original).expect("canonical invoice should write as UBL");

    let written = parse_invoice(&xml).expect("written UBL should parse");

    let roundtrip = Invoice::try_from(written).expect("written UBL should map");

    assert_eq!(roundtrip, original);
}
