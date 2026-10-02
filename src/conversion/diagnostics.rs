#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionImpact {
    /// The target format cannot faithfully represent the invoice.
    Blocking,

    /// The target can be written, but some information is lost.
    Lossy,

    /// The representation changes, but business meaning is preserved.
    Informational,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionIssue {
    pub code: String,
    pub impact: ConversionImpact,
    pub path: String,
    pub message: String,
}

impl ConversionIssue {
    pub fn new(
        code: impl Into<String>,
        impact: ConversionImpact,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            impact,
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversionDiagnostics {
    pub issues: Vec<ConversionIssue>,
}

impl ConversionDiagnostics {
    pub fn push(&mut self, issue: ConversionIssue) {
        self.issues.push(issue);
    }

    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn has_blocking_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.impact == ConversionImpact::Blocking)
    }

    pub fn has_lossy_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.impact == ConversionImpact::Lossy)
    }
}
