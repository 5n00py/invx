use invx::{
    conversion::{ConversionError, ConversionImpact, TargetFormat, analyze, convert},
    domain::Invoice,
    ebinterface::parser as ebinterface_parser,
    ubl::parser as ubl_parser,
};

const UBL_INVOICE: &str = include_str!("fixtures/ubl/simple-invoice.xml");

const EBINTERFACE_INVOICE: &str = include_str!("fixtures/ebinterface/simple-invoice.xml");

#[test]
fn reports_all_known_ebinterface_conversion_issues() {
    let source = ubl_parser::parse_invoice(UBL_INVOICE).expect("UBL should parse");

    let invoice = Invoice::try_from(source).expect("UBL should map");

    let diagnostics = analyze(&invoice, TargetFormat::EbInterface6p1);

    assert!(
        diagnostics.issues.iter().any(|issue| {
            issue.code == "EBI-ADJ-001"
                && issue.path == "adjustments[1].base_amount"
                && issue.impact == ConversionImpact::Blocking
        }),
        "diagnostics: {:#?}",
        diagnostics.issues
    );

    assert!(
        diagnostics.issues.iter().any(|issue| {
            issue.code == "EBI-PAY-002" && issue.impact == ConversionImpact::Informational
        }),
        "diagnostics: {:#?}",
        diagnostics.issues
    );

    assert!(
        diagnostics.issues.iter().any(|issue| {
            issue.code == "EBI-PAY-003" && issue.impact == ConversionImpact::Informational
        }),
        "diagnostics: {:#?}",
        diagnostics.issues
    );

    assert!(diagnostics.has_blocking_issues());
}

#[test]
fn conversion_refuses_target_with_blocking_issues() {
    let source = ubl_parser::parse_invoice(UBL_INVOICE).expect("UBL should parse");

    let invoice = Invoice::try_from(source).expect("UBL should map");

    let error =
        convert(&invoice, TargetFormat::EbInterface6p1).expect_err("conversion should be blocked");

    match error {
        ConversionError::Incompatible {
            target,
            diagnostics,
        } => {
            assert_eq!(target, TargetFormat::EbInterface6p1);

            assert!(diagnostics.has_blocking_issues());
        }

        other => {
            panic!("unexpected conversion error: {other}");
        }
    }
}

#[test]
fn ebinterface_to_ubl_conversion_succeeds_with_information() {
    let source =
        ebinterface_parser::parse_invoice(EBINTERFACE_INVOICE).expect("ebInterface should parse");

    let invoice = Invoice::try_from(source).expect("ebInterface should map");

    let output =
        convert(&invoice, TargetFormat::Ubl).expect("ebInterface invoice should convert to UBL");

    assert!(
        output.diagnostics.issues.iter().any(|issue| {
            issue.code == "UBL-PAY-002" && issue.impact == ConversionImpact::Informational
        }),
        "diagnostics: {:#?}",
        output.diagnostics.issues
    );

    assert!(
        output
            .xml
            .contains("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2")
    );

    let written = ubl_parser::parse_invoice(&output.xml).expect("generated UBL should parse");

    let roundtrip = Invoice::try_from(written).expect("generated UBL should map");

    assert_eq!(roundtrip.id, invoice.id);

    assert_eq!(roundtrip.currency, invoice.currency);

    assert_eq!(roundtrip.totals, invoice.totals);
}
