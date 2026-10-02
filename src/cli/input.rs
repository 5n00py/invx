use std::{error::Error, fs, path::Path};

use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
use thiserror::Error;

use invx::{domain::Invoice, ebinterface::parser as ebinterface_parser, ubl::parser as ubl_parser};

const UBL_INVOICE_NAMESPACE: &str = "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2";

const EBINTERFACE_6P1_NAMESPACE: &str = "http://www.ebinterface.at/schema/6p1/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceFormat {
    Ubl,
    EbInterface6p1,
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error("invalid XML while detecting invoice format: {0}")]
    InvalidXml(#[from] quick_xml::Error),

    #[error("XML document contains no root element")]
    MissingRootElement,

    #[error("root element uses undeclared namespace prefix '{prefix}'")]
    UndeclaredNamespacePrefix { prefix: String },

    #[error("unsupported invoice format: root element '{root}' in namespace '{namespace}'")]
    UnsupportedFormat { root: String, namespace: String },
}

pub fn load_invoice(path: &Path) -> Result<Invoice, Box<dyn Error>> {
    let xml = fs::read_to_string(path)?;

    let format = detect_invoice_format(&xml)?;

    match format {
        InvoiceFormat::Ubl => {
            let source = ubl_parser::parse_invoice(&xml)?;

            Ok(Invoice::try_from(source)?)
        }

        InvoiceFormat::EbInterface6p1 => {
            let source = ebinterface_parser::parse_invoice(&xml)?;

            Ok(Invoice::try_from(source)?)
        }
    }
}

pub fn detect_invoice_format(xml: &str) -> Result<InvoiceFormat, InputError> {
    let mut reader = NsReader::from_str(xml);

    loop {
        let (namespace, event) = reader.read_resolved_event()?;

        match event {
            Event::Start(element) | Event::Empty(element) => {
                let root = element.local_name().as_ref().to_string();

                let namespace = match namespace {
                    ResolveResult::Bound(namespace) => namespace.as_ref().to_string(),

                    ResolveResult::Unbound => String::new(),

                    ResolveResult::Unknown(prefix) => {
                        return Err(InputError::UndeclaredNamespacePrefix { prefix });
                    }
                };

                return match (root.as_str(), namespace.as_str()) {
                    ("Invoice", UBL_INVOICE_NAMESPACE) => Ok(InvoiceFormat::Ubl),

                    ("Invoice", EBINTERFACE_6P1_NAMESPACE) => Ok(InvoiceFormat::EbInterface6p1),

                    _ => Err(InputError::UnsupportedFormat { root, namespace }),
                };
            }

            Event::Eof => {
                return Err(InputError::MissingRootElement);
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ubl_invoice() {
        let xml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <Invoice
                xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2">
            </Invoice>
        "#;

        let format = detect_invoice_format(xml).expect("format should be detected");

        assert_eq!(format, InvoiceFormat::Ubl);
    }

    #[test]
    fn detects_ebinterface_6p1_invoice() {
        let xml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <Invoice
                xmlns="http://www.ebinterface.at/schema/6p1/">
            </Invoice>
        "#;

        let format = detect_invoice_format(xml).expect("format should be detected");

        assert_eq!(format, InvoiceFormat::EbInterface6p1);
    }

    #[test]
    fn detects_prefixed_ebinterface_invoice() {
        let xml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <eb:Invoice
                xmlns:eb="http://www.ebinterface.at/schema/6p1/">
            </eb:Invoice>
        "#;

        let format = detect_invoice_format(xml).expect("format should be detected");

        assert_eq!(format, InvoiceFormat::EbInterface6p1);
    }

    #[test]
    fn rejects_unknown_invoice_namespace() {
        let xml = r#"
            <Invoice xmlns="https://example.com/invoice">
            </Invoice>
        "#;

        let error = detect_invoice_format(xml).expect_err("unknown namespace should fail");

        assert!(matches!(error, InputError::UnsupportedFormat { .. }));
    }

    #[test]
    fn rejects_document_without_root_element() {
        let xml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <!-- nothing here -->
        "#;

        let error = detect_invoice_format(xml).expect_err("document should have a root element");

        assert!(matches!(error, InputError::MissingRootElement));
    }
}
