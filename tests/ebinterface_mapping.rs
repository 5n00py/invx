use rust_decimal::Decimal;

use invx::{
    domain::{Invoice, PaymentMethod},
    ebinterface::parser::parse_invoice,
};

const SIMPLE_INVOICE: &str = include_str!("fixtures/ebinterface/simple-invoice.xml");

#[test]
fn maps_ebinterface_invoice_into_canonical_invoice() {
    let ebinterface = parse_invoice(SIMPLE_INVOICE).expect("ebInterface invoice should parse");

    let invoice = Invoice::try_from(ebinterface).expect("ebInterface invoice should map");

    assert_eq!(invoice.id.as_str(), "2026-00421");

    assert_eq!(invoice.currency.as_str(), "EUR");

    assert_eq!(invoice.seller.name, "Example Supplier GmbH");

    assert_eq!(invoice.seller.vat_id.as_deref(), Some("ATU12345678"));

    assert_eq!(invoice.buyer.name, "Example Logistics GmbH");

    assert_eq!(invoice.buyer.vat_id.as_deref(), Some("ATU87654321"));

    assert_eq!(invoice.order_reference.as_deref(), Some("PO-4711"));

    assert_eq!(invoice.lines.len(), 1);

    let line = &invoice.lines[0];

    assert_eq!(line.id, "1");

    assert_eq!(line.description, "Integration consulting");

    assert_eq!(line.quantity, Decimal::new(10, 0));

    assert_eq!(line.unit_code.as_deref(), Some("HUR"));

    assert_eq!(line.unit_price.amount, Decimal::new(95_000, 2));

    assert_eq!(line.net_amount.amount, Decimal::new(950_000, 2));

    assert_eq!(line.tax.category_code.as_deref(), Some("S"));

    assert_eq!(line.tax.rate, Decimal::new(2_000, 2));

    assert_eq!(invoice.vat_breakdown.len(), 1);

    assert_eq!(
        invoice.vat_breakdown[0].taxable_amount.amount,
        Decimal::new(950_000, 2)
    );

    assert_eq!(
        invoice.vat_breakdown[0].tax_amount.amount,
        Decimal::new(190_000, 2)
    );

    assert_eq!(
        invoice.totals.line_net_amount.amount,
        Decimal::new(950_000, 2)
    );

    assert_eq!(invoice.totals.net_amount.amount, Decimal::new(950_000, 2));

    assert_eq!(invoice.totals.tax_amount.amount, Decimal::new(190_000, 2));

    assert_eq!(
        invoice.totals.gross_amount.amount,
        Decimal::new(1_140_000, 2)
    );

    assert_eq!(
        invoice.totals.payable_amount.amount,
        Decimal::new(1_140_000, 2)
    );

    assert!(invoice.adjustments.is_empty());

    let payment = invoice
        .payment
        .as_ref()
        .expect("invoice should contain payment information");

    assert_eq!(payment.method, PaymentMethod::BankTransfer);

    assert_eq!(payment.means_code, None);

    assert_eq!(payment.reference.as_deref(), Some("2026-00421"));

    assert_eq!(
        payment
            .payee_account
            .as_ref()
            .map(|account| account.identifier.as_str()),
        Some("AT611904300234573201")
    );

    assert!(invoice.totals.is_arithmetically_consistent());
}
