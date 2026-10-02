mod diagnostics;

use std::fmt;

use rust_decimal::Decimal;
use thiserror::Error;

use crate::{
    domain::{Invoice, Money, Party, PaymentMethod},
    ebinterface, ubl,
};

pub use diagnostics::{ConversionDiagnostics, ConversionImpact, ConversionIssue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetFormat {
    Ubl,
    EbInterface6p1,
}

impl fmt::Display for TargetFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ubl => {
                write!(f, "UBL 2.1")
            }

            Self::EbInterface6p1 => {
                write!(f, "ebInterface 6.1")
            }
        }
    }
}

#[derive(Debug)]
pub struct ConversionOutput {
    pub xml: String,
    pub diagnostics: ConversionDiagnostics,
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("invoice cannot be represented as {target}")]
    Incompatible {
        target: TargetFormat,
        diagnostics: ConversionDiagnostics,
    },

    #[error("failed to write {target}: {message}")]
    Writer {
        target: TargetFormat,
        message: String,
    },
}

pub fn analyze(invoice: &Invoice, target: TargetFormat) -> ConversionDiagnostics {
    match target {
        TargetFormat::Ubl => analyze_ubl(invoice),

        TargetFormat::EbInterface6p1 => analyze_ebinterface(invoice),
    }
}

pub fn convert(
    invoice: &Invoice,
    target: TargetFormat,
) -> Result<ConversionOutput, ConversionError> {
    let diagnostics = analyze(invoice, target);

    if diagnostics.has_blocking_issues() {
        return Err(ConversionError::Incompatible {
            target,
            diagnostics,
        });
    }

    let xml = match target {
        TargetFormat::Ubl => {
            ubl::writer::write_invoice(invoice).map_err(|error| ConversionError::Writer {
                target,
                message: error.to_string(),
            })?
        }

        TargetFormat::EbInterface6p1 => {
            ebinterface::writer::write_invoice(invoice).map_err(|error| {
                ConversionError::Writer {
                    target,
                    message: error.to_string(),
                }
            })?
        }
    };

    Ok(ConversionOutput { xml, diagnostics })
}

fn analyze_ubl(invoice: &Invoice) -> ConversionDiagnostics {
    let mut diagnostics = ConversionDiagnostics::default();

    if let Some(payment) = &invoice.payment {
        if payment.method == PaymentMethod::Other && payment.means_code.is_none() {
            diagnostics.push(ConversionIssue::new(
                "UBL-PAY-001",
                ConversionImpact::Blocking,
                "payment.means_code",
                concat!(
                    "UBL output requires a ",
                    "PaymentMeansCode for an ",
                    "unclassified payment method"
                ),
            ));
        }

        if payment.method == PaymentMethod::BankTransfer && payment.means_code.is_none() {
            diagnostics.push(ConversionIssue::new(
                "UBL-PAY-002",
                ConversionImpact::Informational,
                "payment.means_code",
                concat!(
                    "source payment has no UBL ",
                    "PaymentMeansCode; writer ",
                    "will emit generic credit ",
                    "transfer code 30"
                ),
            ));
        }
    }

    check_currencies(invoice, TargetFormat::Ubl, &mut diagnostics);

    diagnostics
}

fn analyze_ebinterface(invoice: &Invoice) -> ConversionDiagnostics {
    let mut diagnostics = ConversionDiagnostics::default();

    analyze_ebinterface_party(&invoice.seller, "seller", &mut diagnostics);

    analyze_ebinterface_party(&invoice.buyer, "buyer", &mut diagnostics);

    for line in &invoice.lines {
        let path = format!("lines[id={}]", line.id);

        if line.unit_code.is_none() {
            diagnostics.push(ConversionIssue::new(
                "EBI-LINE-001",
                ConversionImpact::Blocking,
                format!("{path}.unit_code"),
                concat!("ebInterface 6.1 requires ", "a unit for Quantity"),
            ));
        }

        let valid_position = line.id.parse::<u64>().ok().is_some_and(|value| value > 0);

        if !valid_position {
            diagnostics.push(ConversionIssue::new(
                "EBI-LINE-002",
                ConversionImpact::Blocking,
                format!("{path}.id"),
                concat!(
                    "canonical line ID cannot ",
                    "be represented as an ",
                    "ebInterface PositionNumber"
                ),
            ));
        }

        if line.tax.category_code.is_none() {
            diagnostics.push(ConversionIssue::new(
                "EBI-LINE-003",
                ConversionImpact::Blocking,
                format!("{path}.tax.category_code"),
                concat!("ebInterface output ", "requires a tax category"),
            ));
        }

        check_decimal_scale(
            line.quantity,
            4,
            format!("{path}.quantity"),
            &mut diagnostics,
        );

        check_decimal_scale(
            line.unit_price.amount,
            4,
            format!("{path}.unit_price"),
            &mut diagnostics,
        );

        check_decimal_scale(
            line.net_amount.amount,
            2,
            format!("{path}.net_amount"),
            &mut diagnostics,
        );

        check_decimal_scale(
            line.tax.rate,
            2,
            format!("{path}.tax.rate"),
            &mut diagnostics,
        );
    }

    for (index, adjustment) in invoice.adjustments.iter().enumerate() {
        let path = format!("adjustments[{index}]");

        if adjustment.base_amount.is_none() {
            diagnostics.push(ConversionIssue::new(
                "EBI-ADJ-001",
                ConversionImpact::Blocking,
                format!("{path}.base_amount"),
                concat!(
                    "ebInterface 6.1 ",
                    "reductions and ",
                    "surcharges require ",
                    "BaseAmount"
                ),
            ));
        }

        match &adjustment.tax {
            None => {
                diagnostics.push(ConversionIssue::new(
                    "EBI-ADJ-002",
                    ConversionImpact::Blocking,
                    format!("{path}.tax"),
                    concat!(
                        "ebInterface 6.1 ",
                        "document adjustments ",
                        "require tax information"
                    ),
                ));
            }

            Some(tax) if tax.category_code.is_none() => {
                diagnostics.push(ConversionIssue::new(
                    "EBI-ADJ-003",
                    ConversionImpact::Blocking,
                    format!("{path}.tax.category_code"),
                    concat!(
                        "ebInterface adjustment ",
                        "tax requires a ",
                        "TaxCategoryCode"
                    ),
                ));
            }

            Some(_) => {}
        }

        check_decimal_scale(
            adjustment.amount.amount,
            2,
            format!("{path}.amount"),
            &mut diagnostics,
        );

        if let Some(base_amount) = &adjustment.base_amount {
            check_decimal_scale(
                base_amount.amount,
                2,
                format!("{path}.base_amount"),
                &mut diagnostics,
            );
        }

        if let Some(percentage) = adjustment.percentage {
            check_decimal_scale(
                percentage,
                2,
                format!("{path}.percentage"),
                &mut diagnostics,
            );
        }

        if let Some(tax) = &adjustment.tax {
            check_decimal_scale(tax.rate, 2, format!("{path}.tax.rate"), &mut diagnostics);
        }

        /*
         * ebInterface currently writes all canonical
         * reasons into one Comment value.
         *
         * The business description survives, but the
         * original list structure cannot be recovered.
         */
        if adjustment.reasons.len() > 1 {
            diagnostics.push(ConversionIssue::new(
                "EBI-ADJ-004",
                ConversionImpact::Lossy,
                format!("{path}.reasons"),
                concat!(
                    "multiple adjustment reasons ",
                    "will be combined into one ",
                    "ebInterface Comment"
                ),
            ));
        }
    }

    for (index, vat) in invoice.vat_breakdown.iter().enumerate() {
        let path = format!("vat_breakdown[{index}]");

        if vat.category_code.is_none() {
            diagnostics.push(ConversionIssue::new(
                "EBI-VAT-001",
                ConversionImpact::Blocking,
                format!("{path}.category_code"),
                concat!("ebInterface VAT item ", "requires TaxCategoryCode"),
            ));
        }

        check_decimal_scale(
            vat.taxable_amount.amount,
            2,
            format!("{path}.taxable_amount"),
            &mut diagnostics,
        );

        check_decimal_scale(
            vat.tax_amount.amount,
            2,
            format!("{path}.tax_amount"),
            &mut diagnostics,
        );

        check_decimal_scale(vat.rate, 2, format!("{path}.rate"), &mut diagnostics);
    }

    if let Some(payment) = &invoice.payment {
        if payment.method != PaymentMethod::BankTransfer {
            diagnostics.push(ConversionIssue::new(
                "EBI-PAY-001",
                ConversionImpact::Blocking,
                "payment.method",
                concat!(
                    "current ebInterface ",
                    "writer supports only ",
                    "UniversalBankTransaction"
                ),
            ));
        }

        if payment.means_code.is_some() {
            diagnostics.push(ConversionIssue::new(
                "EBI-PAY-002",
                ConversionImpact::Informational,
                "payment.means_code",
                concat!(
                    "UBL-specific payment ",
                    "means code is not emitted ",
                    "in ebInterface; payment ",
                    "semantics are preserved"
                ),
            ));
        }

        if payment.payee_account.is_some() {
            diagnostics.push(ConversionIssue::new(
                "EBI-PAY-003",
                ConversionImpact::Informational,
                "payment.payee_account",
                concat!(
                    "canonical payment account ",
                    "does not retain its ",
                    "identifier scheme; writer ",
                    "infers whether the value ",
                    "is an IBAN"
                ),
            ));
        }
    }

    check_decimal_scale(
        invoice.totals.gross_amount.amount,
        2,
        "totals.gross_amount",
        &mut diagnostics,
    );

    check_decimal_scale(
        invoice.totals.payable_amount.amount,
        2,
        "totals.payable_amount",
        &mut diagnostics,
    );

    check_currencies(invoice, TargetFormat::EbInterface6p1, &mut diagnostics);

    diagnostics
}

fn analyze_ebinterface_party(
    party: &Party,
    path: &'static str,
    diagnostics: &mut ConversionDiagnostics,
) {
    if party.vat_id.is_none() {
        diagnostics.push(ConversionIssue::new(
            "EBI-PARTY-001",
            ConversionImpact::Blocking,
            format!("{path}.vat_id"),
            concat!("ebInterface 6.1 requires ", "VATIdentificationNumber"),
        ));
    }

    let Some(address) = &party.address else {
        diagnostics.push(ConversionIssue::new(
            "EBI-PARTY-002",
            ConversionImpact::Blocking,
            format!("{path}.address"),
            concat!("ebInterface 6.1 requires ", "an address"),
        ));

        return;
    };

    if address.city.is_none() {
        diagnostics.push(ConversionIssue::new(
            "EBI-PARTY-003",
            ConversionImpact::Blocking,
            format!("{path}.address.city"),
            concat!("ebInterface address ", "requires Town"),
        ));
    }

    if address.postal_code.is_none() {
        diagnostics.push(ConversionIssue::new(
            "EBI-PARTY-004",
            ConversionImpact::Blocking,
            format!("{path}.address.postal_code"),
            concat!("ebInterface address ", "requires ZIP"),
        ));
    }

    if address.country_code.is_none() {
        diagnostics.push(ConversionIssue::new(
            "EBI-PARTY-005",
            ConversionImpact::Blocking,
            format!("{path}.address.country_code"),
            concat!("ebInterface address ", "requires Country"),
        ));
    }
}

fn check_currencies(
    invoice: &Invoice,
    target: TargetFormat,
    diagnostics: &mut ConversionDiagnostics,
) {
    let expected = &invoice.currency;

    let mut check = |path: String, money: &Money| {
        if money.currency != *expected {
            diagnostics.push(ConversionIssue::new(
                "CONV-CUR-001",
                ConversionImpact::Blocking,
                path,
                format!(
                    "{target} output uses invoice currency {}, but value uses {}",
                    expected.as_str(),
                    money.currency.as_str(),
                ),
            ));
        }
    };

    for line in &invoice.lines {
        check(
            format!("lines[id={}].unit_price", line.id),
            &line.unit_price,
        );

        check(
            format!("lines[id={}].net_amount", line.id),
            &line.net_amount,
        );
    }

    for (index, adjustment) in invoice.adjustments.iter().enumerate() {
        check(format!("adjustments[{index}].amount"), &adjustment.amount);

        if let Some(base_amount) = &adjustment.base_amount {
            check(format!("adjustments[{index}].base_amount"), base_amount);
        }
    }

    for (index, vat) in invoice.vat_breakdown.iter().enumerate() {
        check(
            format!("vat_breakdown[{index}].taxable_amount"),
            &vat.taxable_amount,
        );

        check(
            format!("vat_breakdown[{index}].tax_amount"),
            &vat.tax_amount,
        );
    }

    check(
        "totals.line_net_amount".to_string(),
        &invoice.totals.line_net_amount,
    );

    check(
        "totals.allowance_amount".to_string(),
        &invoice.totals.allowance_amount,
    );

    check(
        "totals.charge_amount".to_string(),
        &invoice.totals.charge_amount,
    );

    check("totals.net_amount".to_string(), &invoice.totals.net_amount);

    check("totals.tax_amount".to_string(), &invoice.totals.tax_amount);

    check(
        "totals.gross_amount".to_string(),
        &invoice.totals.gross_amount,
    );

    check(
        "totals.payable_amount".to_string(),
        &invoice.totals.payable_amount,
    );
}

fn check_decimal_scale(
    value: Decimal,
    max_scale: u32,
    path: impl Into<String>,
    diagnostics: &mut ConversionDiagnostics,
) {
    let normalized = value.normalize();

    if normalized.scale() <= max_scale {
        return;
    }

    diagnostics.push(
        ConversionIssue::new(
            "EBI-NUM-001",
            ConversionImpact::Blocking,
            path,
            format!(
                "value {} has more than {max_scale} decimal places and cannot be represented without rounding",
                value
            ),
        ),
    );
}
