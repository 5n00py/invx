mod core;
mod en16931;
mod result;

pub use result::{Severity, ValidationResult, Violation};

pub use core::{ValidationRule, validate_core};

pub use en16931::{CreditTransferRequiresAccount, validate_en16931_subset};
