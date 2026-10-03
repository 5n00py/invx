use chrono::NaiveDate;
use rust_decimal::Decimal;

use invx::domain::{
    Address, Currency, Invoice, InvoiceId, InvoiceLine, InvoiceTotals, Money, Party,
    TaxInformation, VatBreakdown,
};

fn eur(amount: i64, scale: u32) -> Money {
    Money::new(Decimal::new(amount, scale), Currency::new("EUR"))
}

#[test]
fn can_build_a_complete_invoice_through_the_public_api() {
    let seller = Party {
        name: "Example Supplier GmbH".to_string(),
        address: Some(Address {
            street: Some("Supplier Street 1".to_string()),
            postal_code: Some("1010".to_string()),
            city: Some("Vienna".to_string()),
            country_code: Some("AT".to_string()),
        }),
        vat_id: Some("ATU12345678".to_string()),
    };

    let buyer = Party {
        name: "Example Logistics GmbH".to_string(),
        address: Some(Address {
            street: Some("Customer Street 10".to_string()),
            postal_code: Some("1020".to_string()),
            city: Some("Vienna".to_string()),
            country_code: Some("AT".to_string()),
        }),
        vat_id: Some("ATU87654321".to_string()),
    };

    let line = InvoiceLine {
        id: "1".to_string(),
        description: "Integration consulting".to_string(),
        quantity: Decimal::new(10, 0),
        unit_code: Some("HUR".to_string()),
        unit_price: eur(95_000, 2),
        net_amount: eur(950_000, 2),
        tax: TaxInformation {
            category_code: Some("S".to_string()),
            rate: Decimal::new(20, 0),
        },
        price_base_quantity: None,
        adjustments: vec![],
    };

    let vat = VatBreakdown {
        category_code: Some("S".to_string()),
        rate: Decimal::new(20, 0),
        taxable_amount: eur(950_000, 2),
        tax_amount: eur(190_000, 2),
    };

    let totals = InvoiceTotals {
        line_net_amount: eur(950_000, 2),
        allowance_amount: eur(0, 2),
        charge_amount: eur(0, 2),

        net_amount: eur(950_000, 2),
        tax_amount: eur(190_000, 2),
        gross_amount: eur(1_140_000, 2),
        payable_amount: eur(1_140_000, 2),
    };

    let invoice = Invoice {
        id: InvoiceId::new("2026-00421"),
        issue_date: NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
        currency: Currency::new("EUR"),

        seller,
        buyer,

        order_reference: Some("PO-4711".to_string()),

        lines: vec![line],
        vat_breakdown: vec![vat],

        payment: None,
        adjustments: vec![],

        totals,
    };

    assert_eq!(invoice.id.as_str(), "2026-00421");
    assert_eq!(invoice.currency.as_str(), "EUR");

    assert_eq!(invoice.seller.name, "Example Supplier GmbH");
    assert_eq!(invoice.buyer.name, "Example Logistics GmbH");

    assert_eq!(invoice.lines.len(), 1);
    assert_eq!(invoice.lines[0].description, "Integration consulting");
    assert_eq!(invoice.lines[0].tax.category_code.as_deref(), Some("S"));

    assert_eq!(invoice.vat_breakdown.len(), 1);
    assert_eq!(invoice.vat_breakdown[0].category_code.as_deref(), Some("S"));

    assert!(invoice.adjustments.is_empty());

    assert!(invoice.totals.is_arithmetically_consistent());
}
