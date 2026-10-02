use crate::domain::Invoice;

use super::{Severity, ValidationResult, Violation, core::ValidationRule};

pub struct CreditTransferRequiresAccount;

impl ValidationRule for CreditTransferRequiresAccount {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let Some(payment) = &invoice.payment else {
            return Vec::new();
        };

        let requires_account = matches!(payment.means_code.as_str(), "30" | "58");

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

pub fn validate_en16931_subset(invoice: &Invoice) -> ValidationResult {
    let rules: Vec<Box<dyn ValidationRule>> = vec![Box::new(CreditTransferRequiresAccount)];

    let mut result = ValidationResult::default();

    for rule in rules {
        result.violations.extend(rule.validate(invoice));
    }

    result
}
