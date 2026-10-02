use rust_decimal::Decimal;

use invx::{domain::Invoice, ubl::parser::parse_invoice};

const SIMPLE_INVOICE: &str = include_str!("fixtures/ubl/simple-invoice.xml");

#[test]
fn maps_ubl_invoice_into_canonical_invoice() {
    let ubl = parse_invoice(SIMPLE_INVOICE).expect("UBL invoice should parse");

    let invoice = Invoice::try_from(ubl).expect("UBL invoice should map");

    assert_eq!(invoice.id.as_str(), "2026-00421");
    assert_eq!(invoice.currency.as_str(), "EUR");

    assert_eq!(invoice.seller.name, "Example Supplier GmbH");

    assert_eq!(invoice.buyer.name, "Example Logistics GmbH");

    assert_eq!(invoice.lines.len(), 2);

    let line = &invoice.lines[0];

    assert_eq!(line.description, "Integration consulting");

    assert_eq!(line.quantity, Decimal::new(10, 0));

    assert_eq!(line.unit_price.amount, Decimal::new(95000, 2));

    assert_eq!(line.net_amount.amount, Decimal::new(950000, 2));

    assert_eq!(line.tax.rate, Decimal::new(20, 0));

    assert_eq!(invoice.vat_breakdown.len(), 2);

    assert_eq!(invoice.totals.net_amount.amount, Decimal::new(1012500, 2));

    assert_eq!(invoice.totals.tax_amount.amount, Decimal::new(192500, 2));

    assert_eq!(invoice.totals.gross_amount.amount, Decimal::new(1205000, 2));

    assert!(invoice.totals.is_arithmetically_consistent());

    assert_eq!(invoice.seller.vat_id.as_deref(), Some("ATU12345678"));

    let seller_address = invoice
        .seller
        .address
        .as_ref()
        .expect("seller should have an address");

    assert_eq!(seller_address.street.as_deref(), Some("Supplier Street 1"));

    assert_eq!(seller_address.postal_code.as_deref(), Some("1010"));

    assert_eq!(seller_address.city.as_deref(), Some("Vienna"));

    assert_eq!(seller_address.country_code.as_deref(), Some("AT"));

    assert_eq!(invoice.buyer.vat_id.as_deref(), Some("ATU87654321"));

    assert_eq!(invoice.order_reference.as_deref(), Some("PO-4711"));

    let payment = invoice
        .payment
        .as_ref()
        .expect("invoice should contain payment information");

    assert_eq!(payment.means_code.as_deref(), Some("58"));

    assert_eq!(payment.reference.as_deref(), Some("2026-00421"));

    let account = payment
        .payee_account
        .as_ref()
        .expect("payment should contain a payee account");

    assert_eq!(account.identifier, "AT611904300234573201");

    assert_eq!(invoice.lines.len(), 2);

    let first = &invoice.lines[0];

    assert_eq!(first.tax.category_code.as_deref(), Some("S"));

    assert_eq!(first.tax.rate, Decimal::new(20, 0));

    let second = &invoice.lines[1];

    assert_eq!(second.description, "Technical training");

    assert_eq!(second.quantity, Decimal::new(2, 0));

    assert_eq!(second.unit_price.amount, Decimal::new(50_000, 2));

    assert_eq!(second.net_amount.amount, Decimal::new(100_000, 2));

    assert_eq!(second.tax.category_code.as_deref(), Some("S"));

    assert_eq!(second.tax.rate, Decimal::new(10, 0));

    assert_eq!(invoice.vat_breakdown.len(), 2);

    assert_eq!(invoice.totals.net_amount.amount, Decimal::new(1_012_500, 2));

    assert_eq!(invoice.totals.tax_amount.amount, Decimal::new(192_500, 2));

    assert_eq!(
        invoice.totals.gross_amount.amount,
        Decimal::new(1_205_000, 2)
    );
}
