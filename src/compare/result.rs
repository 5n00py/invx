#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference {
    pub path: String,
    pub left: Option<String>,
    pub right: Option<String>,
}

impl Difference {
    pub fn new(path: impl Into<String>, left: Option<String>, right: Option<String>) -> Self {
        Self {
            path: path.into(),
            left,
            right,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComparisonResult {
    pub differences: Vec<Difference>,
}

impl ComparisonResult {
    pub fn is_equal(&self) -> bool {
        self.differences.is_empty()
    }

    pub fn push(&mut self, difference: Difference) {
        self.differences.push(difference);
    }
}
