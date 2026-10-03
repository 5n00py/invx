use rust_decimal::Decimal;

use invx::{
    domain::{AdjustmentKind, Invoice, PaymentMethod},
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

    assert_eq!(line.quantity, Decimal::new(10, 0,));

    assert_eq!(line.unit_code.as_deref(), Some("HUR"));

    assert_eq!(line.unit_price.amount, Decimal::new(100_000, 2,));

    // Price base quantity

    let base_quantity = line
        .price_base_quantity
        .as_ref()
        .expect("line should have price base quantity");

    assert_eq!(base_quantity.quantity, Decimal::ONE);

    assert_eq!(base_quantity.unit_code.as_deref(), Some("HUR"));

    // Line-level reductions / surcharges

    assert_eq!(line.adjustments.len(), 2);

    let allowance = &line.adjustments[0];

    assert_eq!(allowance.kind, AdjustmentKind::Allowance);

    assert_eq!(allowance.amount.amount, Decimal::new(60_000, 2,));

    assert_eq!(allowance.amount.currency.as_str(), "EUR");

    assert_eq!(
        allowance.base_amount.as_ref().map(|money| { money.amount }),
        Some(Decimal::new(1_000_000, 2,))
    );

    assert_eq!(allowance.percentage, Some(Decimal::new(6, 0,)));

    assert_eq!(allowance.reason_code.as_deref(), Some("VOLUME_DISCOUNT"));

    assert_eq!(allowance.reasons, vec!["Volume discount"]);

    let charge = &line.adjustments[1];

    assert_eq!(charge.kind, AdjustmentKind::Charge);

    assert_eq!(charge.amount.amount, Decimal::new(10_000, 2,));

    assert_eq!(charge.amount.currency.as_str(), "EUR");

    assert_eq!(
        charge.base_amount.as_ref().map(|money| { money.amount }),
        Some(Decimal::new(1_000_000, 2,))
    );

    assert_eq!(charge.percentage, Some(Decimal::ONE));

    assert_eq!(charge.reason_code.as_deref(), Some("HANDLING"));

    assert_eq!(charge.reasons, vec!["Line handling surcharge"]);

    // Line net amount stays after adjustments

    assert_eq!(line.net_amount.amount, Decimal::new(950_000, 2,));

    // Line VAT

    assert_eq!(line.tax.category_code.as_deref(), Some("S"));

    assert_eq!(line.tax.rate, Decimal::new(2_000, 2,));

    // VAT breakdown

    assert_eq!(invoice.vat_breakdown.len(), 1);

    assert_eq!(
        invoice.vat_breakdown[0].taxable_amount.amount,
        Decimal::new(950_000, 2,)
    );

    assert_eq!(
        invoice.vat_breakdown[0].tax_amount.amount,
        Decimal::new(190_000, 2,)
    );

    // Totals

    assert_eq!(
        invoice.totals.line_net_amount.amount,
        Decimal::new(950_000, 2,)
    );

    assert_eq!(invoice.totals.net_amount.amount, Decimal::new(950_000, 2,));

    assert_eq!(invoice.totals.tax_amount.amount, Decimal::new(190_000, 2,));

    assert_eq!(
        invoice.totals.gross_amount.amount,
        Decimal::new(1_140_000, 2,)
    );

    assert_eq!(
        invoice.totals.payable_amount.amount,
        Decimal::new(1_140_000, 2,)
    );

    /*
     * These are document-level adjustments.
     * The reductions / surcharges above belong
     * to the invoice line, so this remains empty.
     */
    assert!(invoice.adjustments.is_empty());

    // Payment

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
            .map(|account| { account.identifier.as_str() }),
        Some("AT611904300234573201")
    );

    assert!(invoice.totals.is_arithmetically_consistent());
}
