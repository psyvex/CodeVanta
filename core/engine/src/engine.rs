use crate::{AnalysisContext, Analyzer, AnalyzerError, Finding};

/// Coordinates analyzers without knowing language-specific implementation details.
#[derive(Default)]
pub struct Engine {
    analyzers: Vec<Box<dyn Analyzer>>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_analyzer(mut self, analyzer: impl Analyzer + 'static) -> Self {
        self.add_analyzer(analyzer);
        self
    }

    pub fn add_analyzer(&mut self, analyzer: impl Analyzer + 'static) {
        self.analyzers.push(Box::new(analyzer));
    }

    pub fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, AnalyzerError> {
        let mut findings = Vec::new();

        for analyzer in &self.analyzers {
            findings.extend(analyzer.analyze(context)?);
        }

        Ok(findings)
    }
}
