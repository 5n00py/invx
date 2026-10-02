use crate::domain::Invoice;

use super::{ValidationResult, core::validate_core, en16931::validate_en16931_rules};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationProfile {
    Core,
    En16931Subset,
}

pub fn validate(invoice: &Invoice, profile: ValidationProfile) -> ValidationResult {
    let mut result = validate_core(invoice);

    match profile {
        ValidationProfile::Core => {}

        ValidationProfile::En16931Subset => {
            result.extend(validate_en16931_rules(invoice));
        }
    }

    result
}
