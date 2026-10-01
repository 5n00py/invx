mod result;
mod rules;

pub use result::{Severity, ValidationResult, Violation};

pub use rules::{ValidationRule, validate_invoice};
