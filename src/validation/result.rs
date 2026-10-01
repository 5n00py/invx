#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationResult {
    pub violations: Vec<Violation>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        !self
            .violations
            .iter()
            .any(|violation| violation.severity == Severity::Error)
    }

    pub fn push(&mut self, violation: Violation) {
        self.violations.push(violation);
    }
}
