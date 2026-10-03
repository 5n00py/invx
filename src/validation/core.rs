use rust_decimal::Decimal;

use crate::domain::{AdjustmentKind, Invoice};

use super::{Severity, ValidationResult, Violation};

pub trait ValidationRule {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation>;
}

pub struct InvoiceHasLines;

impl ValidationRule for InvoiceHasLines {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        if invoice.lines.is_empty() {
            vec![Violation {
                code: "CORE-001".to_string(),

                severity: Severity::Error,

                message: "Invoice must contain at least one line".to_string(),

                field: Some("lines".to_string()),
            }]
        } else {
            Vec::new()
        }
    }
}

pub struct NetPlusTaxEqualsGross;

impl ValidationRule for NetPlusTaxEqualsGross {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let totals = &invoice.totals;

        if totals.net_amount.amount + totals.tax_amount.amount != totals.gross_amount.amount {
            return vec![Violation {
                code: "CORE-002".to_string(),

                severity: Severity::Error,

                message: "Net amount plus tax amount must equal gross amount".to_string(),

                field: Some("totals.gross_amount".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct GrossEqualsPayable;

impl ValidationRule for GrossEqualsPayable {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let totals = &invoice.totals;

        if totals.gross_amount.amount != totals.payable_amount.amount {
            return vec![Violation {
                code: "CORE-003".to_string(),

                severity: Severity::Error,

                message: "Gross amount must equal payable amount".to_string(),

                field: Some("totals.payable_amount".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct LineNetAmountsEqualLineNetTotal;

impl ValidationRule for LineNetAmountsEqualLineNetTotal {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let line_total: Decimal = invoice
            .lines
            .iter()
            .map(|line| line.net_amount.amount)
            .sum();

        if line_total != invoice.totals.line_net_amount.amount {
            return vec![Violation {
                code: "CORE-004".to_string(),

                severity: Severity::Error,

                message: "Sum of invoice line net amounts must equal line net total".to_string(),

                field: Some("totals.line_net_amount".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct CurrencyConsistency;

impl ValidationRule for CurrencyConsistency {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (line_index, line) in invoice.lines.iter().enumerate() {
            if line.unit_price.currency != invoice.currency {
                violations.push(Violation {
                    code: "CORE-005".to_string(),

                    severity: Severity::Error,

                    message: "Invoice line unit price currency differs from invoice currency"
                        .to_string(),

                    field: Some(format!("lines[{line_index}].unit_price")),
                });
            }

            if line.net_amount.currency != invoice.currency {
                violations.push(Violation {
                    code: "CORE-005".to_string(),

                    severity: Severity::Error,

                    message: "Invoice line net amount currency differs from invoice currency"
                        .to_string(),

                    field: Some(format!("lines[{line_index}].net_amount")),
                });
            }

            for (adjustment_index, adjustment) in line.adjustments.iter().enumerate() {
                if adjustment.amount.currency != invoice.currency {
                    violations.push(Violation {
                        code: "CORE-005".to_string(),

                        severity: Severity::Error,

                        message:
                            "Invoice line adjustment amount currency differs from invoice currency"
                                .to_string(),

                        field: Some(format!(
                            "lines[{line_index}].adjustments[{adjustment_index}].amount"
                        )),
                    });
                }

                if let Some(base_amount) = &adjustment.base_amount
                    && base_amount.currency != invoice.currency
                {
                    violations.push(
                        Violation {
                            code:
                            "CORE-005"
                                .to_string(),

                            severity:
                            Severity::Error,

                            message:
                            "Invoice line adjustment base amount currency differs from invoice currency"
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

pub struct VatBreakdownEqualsTaxTotal;

impl ValidationRule for VatBreakdownEqualsTaxTotal {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let vat_total: Decimal = invoice
            .vat_breakdown
            .iter()
            .map(|vat| vat.tax_amount.amount)
            .sum();

        if vat_total != invoice.totals.tax_amount.amount {
            return vec![Violation {
                code: "CORE-006".to_string(),

                severity: Severity::Error,

                message: "Sum of VAT breakdown amounts must equal total tax amount".to_string(),

                field: Some("totals.tax_amount".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct AdjustedNetAmountIsConsistent;

impl ValidationRule for AdjustedNetAmountIsConsistent {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let expected = invoice.totals.line_net_amount.amount
            - invoice.totals.allowance_amount.amount
            + invoice.totals.charge_amount.amount;

        if expected != invoice.totals.net_amount.amount {
            return vec![Violation {
                code: "CORE-009".to_string(),

                severity: Severity::Error,

                message:
                    "Line net total minus allowances plus charges must equal invoice net amount"
                        .to_string(),

                field: Some("totals.net_amount".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct PriceBaseQuantityMustBePositive;

impl ValidationRule for PriceBaseQuantityMustBePositive {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, line) in invoice.lines.iter().enumerate() {
            let Some(base_quantity) = &line.price_base_quantity else {
                continue;
            };

            if base_quantity.quantity <= Decimal::ZERO {
                violations.push(Violation {
                    code: "CORE-010".to_string(),

                    severity: Severity::Error,

                    message: "Invoice line price base quantity must be greater than zero"
                        .to_string(),

                    field: Some(format!("lines[{index}].price_base_quantity.quantity")),
                });
            }
        }

        violations
    }
}

pub struct LineNetAmountIsConsistent;

impl ValidationRule for LineNetAmountIsConsistent {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, line) in invoice.lines.iter().enumerate() {
            let base_quantity = line
                .price_base_quantity
                .as_ref()
                .map(|base| base.quantity)
                .unwrap_or(Decimal::ONE);

            /*
             * CORE-010 reports invalid base
             * quantities.
             *
             * Do not attempt the calculation
             * here if division would be invalid
             * or semantically meaningless.
             */
            if base_quantity <= Decimal::ZERO {
                continue;
            }

            let base_line_amount = line.quantity * line.unit_price.amount / base_quantity;

            let allowance_total: Decimal = line
                .adjustments
                .iter()
                .filter(|adjustment| adjustment.kind == AdjustmentKind::Allowance)
                .map(|adjustment| adjustment.amount.amount)
                .sum();

            let charge_total: Decimal = line
                .adjustments
                .iter()
                .filter(|adjustment| adjustment.kind == AdjustmentKind::Charge)
                .map(|adjustment| adjustment.amount.amount)
                .sum();

            let expected = base_line_amount - allowance_total + charge_total;

            if expected != line.net_amount.amount {
                violations.push(
                    Violation {
                        code:
                        "CORE-011"
                            .to_string(),

                        severity:
                        Severity::Error,

                        message:
                        format!(
                            "Invoice line net amount must equal quantity times unit price divided by price base quantity minus line allowances plus line charges; expected {expected}, found {}",
                            line.net_amount.amount
                        ),

                        field:
                        Some(
                            format!(
                                "lines[{index}].net_amount"
                            ),
                        ),
                    },
                );
            }
        }

        violations
    }
}

pub(crate) fn validate_core(invoice: &Invoice) -> ValidationResult {
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(InvoiceHasLines),
        Box::new(NetPlusTaxEqualsGross),
        Box::new(GrossEqualsPayable),
        Box::new(LineNetAmountsEqualLineNetTotal),
        Box::new(AdjustedNetAmountIsConsistent),
        Box::new(CurrencyConsistency),
        Box::new(VatBreakdownEqualsTaxTotal),
        Box::new(PriceBaseQuantityMustBePositive),
        Box::new(LineNetAmountIsConsistent),
    ];

    let mut result = ValidationResult::default();

    for rule in rules {
        result.violations.extend(rule.validate(invoice));
    }

    result
}
