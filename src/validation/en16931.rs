use rust_decimal::Decimal;

use crate::domain::{Invoice, PaymentMethod};

use super::{Severity, ValidationResult, Violation, core::ValidationRule};

pub struct CreditTransferRequiresAccount;

impl ValidationRule for CreditTransferRequiresAccount {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let Some(payment) = &invoice.payment else {
            return Vec::new();
        };

        let requires_account = payment.method == PaymentMethod::BankTransfer;

        if requires_account && payment.payee_account.is_none() {
            return vec![Violation {
                code: "BR-61".to_string(),

                severity: Severity::Error,

                message: "Credit transfer requires a payment account identifier".to_string(),

                field: Some("payment.payee_account".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct InvoiceLineRequiresVatCategory;

impl ValidationRule for InvoiceLineRequiresVatCategory {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, line) in invoice.lines.iter().enumerate() {
            if line.tax.category_code.is_none() {
                violations.push(Violation {
                    code: "BR-CO-04".to_string(),

                    severity: Severity::Error,

                    message: "Invoice line must have a VAT category code".to_string(),

                    field: Some(format!("lines[{index}].tax.category_code")),
                });
            }
        }

        violations
    }
}

pub struct StandardRatedLineRequiresPositiveRate;

impl ValidationRule for StandardRatedLineRequiresPositiveRate {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, line) in invoice.lines.iter().enumerate() {
            if line.tax.category_code.as_deref() == Some("S") && line.tax.rate <= Decimal::ZERO {
                violations.push(Violation {
                    code: "BR-S-05".to_string(),

                    severity: Severity::Error,

                    message: "Standard-rated invoice line must have a VAT rate greater than zero"
                        .to_string(),

                    field: Some(format!("lines[{index}].tax.rate")),
                });
            }
        }

        violations
    }
}

pub struct StandardRatedLineRequiresVatBreakdown;

impl ValidationRule for StandardRatedLineRequiresVatBreakdown {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let has_standard_rated_line = invoice
            .lines
            .iter()
            .any(|line| line.tax.category_code.as_deref() == Some("S"));

        let has_standard_rated_breakdown = invoice
            .vat_breakdown
            .iter()
            .any(|vat| vat.category_code.as_deref() == Some("S"));

        if has_standard_rated_line && !has_standard_rated_breakdown {
            return vec![Violation {
                code: "BR-S-01".to_string(),

                severity: Severity::Error,

                message: "Standard-rated invoice lines require a standard-rated VAT breakdown"
                    .to_string(),

                field: Some("vat_breakdown".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct VatBreakdownTaxAmountMatchesRate;

impl ValidationRule for VatBreakdownTaxAmountMatchesRate {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, vat) in invoice.vat_breakdown.iter().enumerate() {
            let expected =
                (vat.taxable_amount.amount * vat.rate / Decimal::new(100, 0)).round_dp(2);

            if vat.tax_amount.amount != expected {
                violations.push(
                    Violation {
                        code:
                        "BR-CO-17"
                            .to_string(),

                        severity:
                        Severity::Error,

                        message:
                        format!(
                            "VAT breakdown tax amount must equal taxable amount multiplied by VAT rate; expected {expected}, found {}",
                            vat.tax_amount.amount
                        ),

                        field:
                        Some(
                            format!(
                                "vat_breakdown[{index}].tax_amount"
                            ),
                        ),
                    },
                );
            }
        }

        violations
    }
}

pub struct ItemNetPriceMustNotBeNegative;

impl ValidationRule for ItemNetPriceMustNotBeNegative {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, line) in invoice.lines.iter().enumerate() {
            if line.unit_price.amount < Decimal::ZERO {
                violations.push(Violation {
                    code: "BR-27".to_string(),

                    severity: Severity::Error,

                    message: "Item net price must not be negative".to_string(),

                    field: Some(format!("lines[{index}].unit_price")),
                });
            }
        }

        violations
    }
}

pub struct PriceBaseQuantityUnitMustMatchLineUnit;

impl ValidationRule for PriceBaseQuantityUnitMustMatchLineUnit {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, line) in invoice.lines.iter().enumerate() {
            let Some(base_quantity) = &line.price_base_quantity else {
                continue;
            };

            let Some(base_unit) = &base_quantity.unit_code else {
                continue;
            };

            if line.unit_code.as_ref() != Some(base_unit) {
                violations.push(Violation {
                    code: "PEPPOL-EN16931-R130".to_string(),

                    severity: Severity::Error,

                    message:
                        "Price base quantity unit code must equal the invoiced quantity unit code"
                            .to_string(),

                    field: Some(format!("lines[{index}].price_base_quantity.unit_code")),
                });
            }
        }

        violations
    }
}

pub struct AdjustmentPercentageRequiresBaseAmount;

impl ValidationRule for AdjustmentPercentageRequiresBaseAmount {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, adjustment) in invoice.adjustments.iter().enumerate() {
            if adjustment.percentage.is_some() && adjustment.base_amount.is_none() {
                violations.push(
                    Violation {
                        code:
                        "PEPPOL-EN16931-R041"
                            .to_string(),

                        severity:
                        Severity::Error,

                        message:
                        "Allowance or charge base amount must be provided when a percentage is provided"
                            .to_string(),

                        field:
                        Some(
                            format!(
                                "adjustments[{index}].base_amount"
                            ),
                        ),
                    },
                );
            }
        }

        for (line_index, line) in invoice.lines.iter().enumerate() {
            for (adjustment_index, adjustment) in line.adjustments.iter().enumerate() {
                if adjustment.percentage.is_some() && adjustment.base_amount.is_none() {
                    violations.push(
                        Violation {
                            code:
                            "PEPPOL-EN16931-R041"
                                .to_string(),

                            severity:
                            Severity::Error,

                            message:
                            "Allowance or charge base amount must be provided when a percentage is provided"
                                .to_string(),

                            field:
                            Some(
                                format!(
                                    "lines[{line_index}].adjustments[{adjustment_index}].base_amount"
                                ),
                            ),
                        },
                    );
                }
            }
        }

        violations
    }
}

pub struct AdjustmentBaseAmountRequiresPercentage;

impl ValidationRule for AdjustmentBaseAmountRequiresPercentage {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, adjustment) in invoice.adjustments.iter().enumerate() {
            if adjustment.base_amount.is_some() && adjustment.percentage.is_none() {
                violations.push(
                    Violation {
                        code:
                        "PEPPOL-EN16931-R042"
                            .to_string(),

                        severity:
                        Severity::Error,

                        message:
                        "Allowance or charge percentage must be provided when a base amount is provided"
                            .to_string(),

                        field:
                        Some(
                            format!(
                                "adjustments[{index}].percentage"
                            ),
                        ),
                    },
                );
            }
        }

        for (line_index, line) in invoice.lines.iter().enumerate() {
            for (adjustment_index, adjustment) in line.adjustments.iter().enumerate() {
                if adjustment.base_amount.is_some() && adjustment.percentage.is_none() {
                    violations.push(
                        Violation {
                            code:
                            "PEPPOL-EN16931-R042"
                                .to_string(),

                            severity:
                            Severity::Error,

                            message:
                            "Allowance or charge percentage must be provided when a base amount is provided"
                                .to_string(),

                            field:
                            Some(
                                format!(
                                    "lines[{line_index}].adjustments[{adjustment_index}].percentage"
                                ),
                            ),
                        },
                    );
                }
            }
        }

        violations
    }
}

pub struct AdjustmentAmountMatchesBaseAndPercentage;

impl ValidationRule for AdjustmentAmountMatchesBaseAndPercentage {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, adjustment) in invoice.adjustments.iter().enumerate() {
            validate_adjustment_amount(
                &mut violations,
                &format!("adjustments[{index}]"),
                adjustment.amount.amount,
                adjustment.base_amount.as_ref().map(|money| money.amount),
                adjustment.percentage,
            );
        }

        for (line_index, line) in invoice.lines.iter().enumerate() {
            for (adjustment_index, adjustment) in line.adjustments.iter().enumerate() {
                validate_adjustment_amount(
                    &mut violations,
                    &format!("lines[{line_index}].adjustments[{adjustment_index}]"),
                    adjustment.amount.amount,
                    adjustment.base_amount.as_ref().map(|money| money.amount),
                    adjustment.percentage,
                );
            }
        }

        violations
    }
}

fn validate_adjustment_amount(
    violations: &mut Vec<Violation>,
    path: &str,
    amount: Decimal,
    base_amount: Option<Decimal>,
    percentage: Option<Decimal>,
) {
    let (Some(base_amount), Some(percentage)) = (base_amount, percentage) else {
        /*
         * R041 / R042 report missing
         * components separately.
         */
        return;
    };

    let expected = base_amount * percentage / Decimal::new(100, 0);

    /*
     * PEPPOL-EN16931-R040 uses
     * a slack of 0.02.
     */
    let tolerance = Decimal::new(2, 2);

    let difference = (amount - expected).abs();

    if difference > tolerance {
        violations.push(
            Violation {
                code:
                "PEPPOL-EN16931-R040"
                    .to_string(),

                severity:
                Severity::Error,

                message:
                format!(
                    "Allowance or charge amount must equal base amount multiplied by percentage divided by 100; expected {expected}, found {amount}"
                ),

                field:
                Some(
                    format!(
                        "{path}.amount"
                    ),
                ),
            },
        );
    }
}

pub(crate) fn validate_en16931_rules(invoice: &Invoice) -> ValidationResult {
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(CreditTransferRequiresAccount),
        Box::new(InvoiceLineRequiresVatCategory),
        Box::new(StandardRatedLineRequiresPositiveRate),
        Box::new(StandardRatedLineRequiresVatBreakdown),
        Box::new(VatBreakdownTaxAmountMatchesRate),
        Box::new(ItemNetPriceMustNotBeNegative),
        Box::new(PriceBaseQuantityUnitMustMatchLineUnit),
        Box::new(AdjustmentPercentageRequiresBaseAmount),
        Box::new(AdjustmentBaseAmountRequiresPercentage),
        Box::new(AdjustmentAmountMatchesBaseAndPercentage),
    ];

    let mut result = ValidationResult::default();

    for rule in rules {
        result.violations.extend(rule.validate(invoice));
    }

    result
}
