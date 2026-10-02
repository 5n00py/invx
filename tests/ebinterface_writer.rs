use invx::{
    domain::Invoice,
    ebinterface::{
        parser::parse_invoice,
        writer::write_invoice,
    },
};

const SIMPLE_INVOICE: &str =
    include_str!(
        "fixtures/ebinterface/simple-invoice.xml"
    );

#[test]
fn canonical_invoice_can_be_written_as_ebinterface_and_read_again() {
    let source =
        parse_invoice(SIMPLE_INVOICE)
            .expect(
                "source ebInterface should parse",
            );

    let original =
        Invoice::try_from(source)
            .expect(
                "source ebInterface should map",
            );

    let xml =
        write_invoice(&original)
            .expect(
                "canonical invoice should write as ebInterface",
            );

    let written =
        parse_invoice(&xml)
            .expect(
                "written ebInterface should parse",
            );

    let roundtrip =
        Invoice::try_from(written)
            .expect(
                "written ebInterface should map",
            );

    assert_eq!(
        roundtrip,
        original
    );
}