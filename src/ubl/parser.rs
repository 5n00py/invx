use quick_xml::de::from_str;

use super::model::UblInvoice;

pub fn parse_invoice(xml: &str) -> Result<UblInvoice, quick_xml::DeError> {
    from_str(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_INVOICE: &str = include_str!("../../tests/fixtures/ubl/simple-invoice.xml");

    #[test]
    fn parses_simple_ubl_invoice() {
        let invoice = parse_invoice(SIMPLE_INVOICE).expect("UBL invoice should parse");

        assert_eq!(invoice.id, "2026-00421");
        assert_eq!(invoice.issue_date, "2026-09-30");
        assert_eq!(invoice.document_currency_code, "EUR");

        assert_eq!(
            invoice.accounting_supplier_party.party.party_name.name,
            "Example Supplier GmbH"
        );

        assert_eq!(
            invoice.accounting_customer_party.party.party_name.name,
            "Example Logistics GmbH"
        );

        assert_eq!(invoice.invoice_lines.len(), 1);

        let line = &invoice.invoice_lines[0];

        assert_eq!(line.id, "1");
        assert_eq!(line.invoiced_quantity.value, "10");
        assert_eq!(line.invoiced_quantity.unit_code.as_deref(), Some("HUR"));

        assert_eq!(line.item.description, "Integration consulting");

        assert_eq!(line.price.price_amount.value, "950.00");
        assert_eq!(line.price.price_amount.currency_id, "EUR");

        assert_eq!(
            invoice.legal_monetary_total.payable_amount.value,
            "11400.00"
        );

        let seller = &invoice.accounting_supplier_party.party;

        let address = seller
            .postal_address
            .as_ref()
            .expect("seller should have an address");

        assert_eq!(address.street_name.as_deref(), Some("Supplier Street 1"));

        assert_eq!(address.city_name.as_deref(), Some("Vienna"));

        assert_eq!(address.postal_zone.as_deref(), Some("1010"));

        assert_eq!(
            address
                .country
                .as_ref()
                .and_then(|country| country.identification_code.as_deref()),
            Some("AT")
        );

        assert_eq!(seller.party_tax_schemes.len(), 1);

        assert_eq!(
            seller.party_tax_schemes[0].company_id.as_deref(),
            Some("ATU12345678")
        );

        assert_eq!(
            invoice
                .order_reference
                .as_ref()
                .map(|reference| reference.id.as_str()),
            Some("PO-4711")
        );
    }
}
