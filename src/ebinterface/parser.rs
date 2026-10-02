use quick_xml::de::from_str;

use super::model::EbInterfaceInvoice;

pub fn parse_invoice(xml: &str) -> Result<EbInterfaceInvoice, quick_xml::DeError> {
    from_str(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_INVOICE: &str =
        include_str!("../../tests/fixtures/ebinterface/simple-invoice.xml");

    #[test]
    fn parses_simple_ebinterface_invoice() {
        let invoice = parse_invoice(SIMPLE_INVOICE).expect("ebInterface invoice should parse");

        // Root metadata

        assert_eq!(invoice.generating_system, "invx");

        assert_eq!(invoice.document_type, "Invoice");

        assert_eq!(invoice.invoice_currency, "EUR");

        assert_eq!(invoice.invoice_number, "2026-00421");

        assert_eq!(invoice.invoice_date, "2026-09-30");

        // Seller

        assert_eq!(invoice.biller.vat_identification_number, "ATU12345678");

        let seller_address = invoice
            .biller
            .address
            .as_ref()
            .expect("biller should have an address");

        assert_eq!(seller_address.name, "Example Supplier GmbH");

        assert_eq!(seller_address.street.as_deref(), Some("Supplier Street 1"));

        assert_eq!(seller_address.town, "Vienna");

        assert_eq!(seller_address.zip, "1010");

        assert_eq!(seller_address.country.country_code.as_deref(), Some("AT"));

        assert_eq!(seller_address.country.value, "Austria");

        // Buyer

        assert_eq!(
            invoice.invoice_recipient.vat_identification_number,
            "ATU87654321"
        );

        let buyer_address = invoice
            .invoice_recipient
            .address
            .as_ref()
            .expect("invoice recipient should have an address");

        assert_eq!(buyer_address.name, "Example Logistics GmbH");

        assert_eq!(buyer_address.street.as_deref(), Some("Customer Street 10"));

        assert_eq!(buyer_address.town, "Vienna");

        assert_eq!(buyer_address.zip, "1020");

        assert_eq!(buyer_address.country.country_code.as_deref(), Some("AT"));

        // Order reference

        let order_reference = invoice
            .invoice_recipient
            .order_reference
            .as_ref()
            .expect("invoice recipient should have an order reference");

        assert_eq!(order_reference.order_id, "PO-4711");

        // Details

        assert_eq!(invoice.details.item_lists.len(), 1);

        let item_list = &invoice.details.item_lists[0];

        assert_eq!(item_list.line_items.len(), 1);

        let line = &item_list.line_items[0];

        assert_eq!(line.position_number.as_deref(), Some("1"));

        assert_eq!(line.descriptions, vec!["Integration consulting"]);

        assert_eq!(line.quantity.value, "10");

        assert_eq!(line.quantity.unit, "HUR");

        assert_eq!(line.unit_price.value, "950.00");

        assert_eq!(line.line_item_amount, "9500.00");

        // Line tax

        assert_eq!(line.tax_item.taxable_amount, "9500.00");

        assert_eq!(line.tax_item.tax_percent.value, "20.00");

        assert_eq!(line.tax_item.tax_percent.tax_category_code, "S");

        assert_eq!(line.tax_item.tax_amount.as_deref(), Some("1900.00"));

        // Invoice VAT breakdown

        assert_eq!(invoice.tax.tax_items.len(), 1);

        let tax = &invoice.tax.tax_items[0];

        assert_eq!(tax.taxable_amount, "9500.00");

        assert_eq!(tax.tax_percent.value, "20.00");

        assert_eq!(tax.tax_percent.tax_category_code, "S");

        assert_eq!(tax.tax_amount.as_deref(), Some("1900.00"));

        // Totals

        assert_eq!(invoice.total_gross_amount, "11400.00");

        assert_eq!(invoice.payable_amount, "11400.00");

        // Payment

        let payment_method = invoice
            .payment_method
            .as_ref()
            .expect("invoice should contain a payment method");

        let bank_transfer = payment_method
            .universal_bank_transaction
            .as_ref()
            .expect("payment method should be a universal bank transaction");

        assert_eq!(bank_transfer.beneficiary_accounts.len(), 1);

        assert_eq!(
            bank_transfer.beneficiary_accounts[0].iban.as_deref(),
            Some("AT611904300234573201")
        );

        assert_eq!(
            bank_transfer
                .payment_reference
                .as_ref()
                .map(|reference| reference.value.as_str()),
            Some("2026-00421")
        );
    }
}
