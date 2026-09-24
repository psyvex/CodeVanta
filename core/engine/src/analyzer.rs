use crate::errors::AnalyzerError;
use crate::{AnalysisContext, Finding};

/// Stable metadata describing an analyzer implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzerDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
}

impl AnalyzerDescriptor {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Common contract for deterministic and AI-backed analyzers.
pub trait Analyzer: Send + Sync {
    fn descriptor(&self) -> AnalyzerDescriptor;
    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, AnalyzerError>;
}
