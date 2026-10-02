use invx::{domain::Invoice, ubl::parser::parse_invoice};

const SIMPLE_INVOICE: &str = include_str!("fixtures/ubl/simple-invoice.xml");

#[test]
fn canonical_invoice_serializes_to_json() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL should parse");

    let invoice = Invoice::try_from(ubl).expect("UBL should map");

    let value = serde_json::to_value(&invoice).expect("invoice should serialize");

    assert_eq!(value["id"], "2026-00421");

    assert_eq!(value["totals"]["line_net_amount"]["amount"], "10500.00");

    assert_eq!(value["totals"]["allowance_amount"]["amount"], "475.00");

    assert_eq!(value["totals"]["charge_amount"]["amount"], "100.00");

    assert_eq!(value["totals"]["payable_amount"]["amount"], "12050.00");
}
