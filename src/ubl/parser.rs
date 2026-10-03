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
                .and_then(|country| { country.identification_code.as_deref() }),
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
                .map(|reference| { reference.id.as_str() }),
            Some("PO-4711")
        );

        assert_eq!(invoice.payment_means.len(), 1);

        let payment = &invoice.payment_means[0];

        assert_eq!(payment.payment_means_code, "58");

        assert_eq!(payment.payment_id.as_deref(), Some("2026-00421"));

        assert_eq!(
            payment
                .payee_financial_account
                .as_ref()
                .and_then(|account| { account.id.as_deref() }),
            Some("AT611904300234573201")
        );

        /*
         * Document-level allowances / charges
         */

        assert_eq!(invoice.allowance_charges.len(), 2);

        let document_allowance = &invoice.allowance_charges[0];

        assert!(!document_allowance.charge_indicator);

        assert_eq!(document_allowance.reasons, vec!["Project discount"]);

        assert_eq!(
            document_allowance.multiplier_factor_numeric.as_deref(),
            Some("5")
        );

        assert_eq!(document_allowance.amount.value, "475.00");

        assert_eq!(
            document_allowance
                .base_amount
                .as_ref()
                .map(|amount| { amount.value.as_str() }),
            Some("9500.00")
        );

        assert_eq!(
            document_allowance
                .tax_category
                .as_ref()
                .and_then(|tax| { tax.id.as_deref() }),
            Some("S")
        );

        let document_charge = &invoice.allowance_charges[1];

        assert!(document_charge.charge_indicator);

        assert_eq!(document_charge.reasons, vec!["Administration fee"]);

        assert_eq!(document_charge.amount.value, "100.00");

        assert_eq!(
            document_charge
                .tax_category
                .as_ref()
                .and_then(|tax| { tax.id.as_deref() }),
            Some("S")
        );

        /*
         * Invoice lines
         */

        assert_eq!(invoice.invoice_lines.len(), 2);

        let first_line = &invoice.invoice_lines[0];

        assert_eq!(first_line.id, "1");

        assert_eq!(first_line.invoiced_quantity.value, "10");

        assert_eq!(
            first_line.invoiced_quantity.unit_code.as_deref(),
            Some("HUR")
        );

        assert_eq!(first_line.line_extension_amount.value, "9500.00");

        /*
         * Line-level allowance / charge
         */

        assert_eq!(first_line.allowance_charges.len(), 2);

        let line_allowance = &first_line.allowance_charges[0];

        assert!(!line_allowance.charge_indicator);

        assert_eq!(line_allowance.reasons, vec!["Volume discount"]);

        assert_eq!(
            line_allowance.multiplier_factor_numeric.as_deref(),
            Some("6")
        );

        assert_eq!(line_allowance.amount.value, "600.00");

        assert_eq!(line_allowance.amount.currency_id, "EUR");

        assert_eq!(
            line_allowance
                .base_amount
                .as_ref()
                .map(|amount| { amount.value.as_str() }),
            Some("10000.00")
        );

        assert_eq!(
            line_allowance
                .base_amount
                .as_ref()
                .map(|amount| { amount.currency_id.as_str() }),
            Some("EUR")
        );

        assert!(line_allowance.tax_category.is_none());

        let line_charge = &first_line.allowance_charges[1];

        assert!(line_charge.charge_indicator);

        assert_eq!(line_charge.reasons, vec!["Line handling surcharge"]);

        assert_eq!(line_charge.multiplier_factor_numeric.as_deref(), Some("1"));

        assert_eq!(line_charge.amount.value, "100.00");

        assert_eq!(line_charge.amount.currency_id, "EUR");

        assert_eq!(
            line_charge
                .base_amount
                .as_ref()
                .map(|amount| { amount.value.as_str() }),
            Some("10000.00")
        );

        assert_eq!(
            line_charge
                .base_amount
                .as_ref()
                .map(|amount| { amount.currency_id.as_str() }),
            Some("EUR")
        );

        assert!(line_charge.tax_category.is_none());

        /*
         * Line item / VAT
         */

        assert_eq!(first_line.item.description, "Integration consulting");

        assert_eq!(
            first_line.item.classified_tax_category.id.as_deref(),
            Some("S")
        );

        assert_eq!(first_line.item.classified_tax_category.percent, "20");

        /*
         * Price + base quantity
         */

        assert_eq!(first_line.price.price_amount.value, "1000.00");

        assert_eq!(first_line.price.price_amount.currency_id, "EUR");

        let base_quantity = first_line
            .price
            .base_quantity
            .as_ref()
            .expect("first line should have a price base quantity");

        assert_eq!(base_quantity.value, "1");

        assert_eq!(base_quantity.unit_code.as_deref(), Some("HUR"));

        /*
         * Second line stays simple
         */

        let second_line = &invoice.invoice_lines[1];

        assert_eq!(second_line.id, "2");

        assert_eq!(second_line.invoiced_quantity.value, "2");

        assert_eq!(
            second_line.invoiced_quantity.unit_code.as_deref(),
            Some("HUR")
        );

        assert_eq!(second_line.line_extension_amount.value, "1000.00");

        assert_eq!(second_line.item.description, "Technical training");

        assert_eq!(
            second_line.item.classified_tax_category.id.as_deref(),
            Some("S")
        );

        assert_eq!(second_line.item.classified_tax_category.percent, "10");

        assert_eq!(second_line.price.price_amount.value, "500.00");

        assert_eq!(second_line.price.price_amount.currency_id, "EUR");

        assert!(second_line.price.base_quantity.is_none());

        assert!(second_line.allowance_charges.is_empty());

        /*
         * VAT totals
         */

        assert_eq!(invoice.tax_total.tax_amount.value, "1925.00");

        assert_eq!(invoice.tax_total.tax_subtotals.len(), 2);

        let first_vat = &invoice.tax_total.tax_subtotals[0];

        assert_eq!(first_vat.taxable_amount.value, "9125.00");

        assert_eq!(first_vat.tax_amount.value, "1825.00");

        assert_eq!(first_vat.tax_category.id.as_deref(), Some("S"));

        assert_eq!(first_vat.tax_category.percent, "20");

        let second_vat = &invoice.tax_total.tax_subtotals[1];

        assert_eq!(second_vat.taxable_amount.value, "1000.00");

        assert_eq!(second_vat.tax_amount.value, "100.00");

        assert_eq!(second_vat.tax_category.id.as_deref(), Some("S"));

        assert_eq!(second_vat.tax_category.percent, "10");

        /*
         * Monetary totals
         */

        assert_eq!(
            invoice.legal_monetary_total.line_extension_amount.value,
            "10500.00"
        );

        assert_eq!(
            invoice
                .legal_monetary_total
                .allowance_total_amount
                .as_ref()
                .map(|amount| { amount.value.as_str() }),
            Some("475.00")
        );

        assert_eq!(
            invoice
                .legal_monetary_total
                .charge_total_amount
                .as_ref()
                .map(|amount| { amount.value.as_str() }),
            Some("100.00")
        );

        assert_eq!(
            invoice.legal_monetary_total.tax_exclusive_amount.value,
            "10125.00"
        );

        assert_eq!(
            invoice.legal_monetary_total.tax_inclusive_amount.value,
            "12050.00"
        );

        assert_eq!(
            invoice.legal_monetary_total.payable_amount.value,
            "12050.00"
        );
    }
}
