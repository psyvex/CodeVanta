pub mod analyzer;
pub mod context;
pub mod engine;
pub mod errors;
pub mod finding;
pub mod language;

pub use analyzer::{Analyzer, AnalyzerDescriptor};
pub use context::{AnalysisContext, SourceFile};
pub use engine::Engine;
pub use errors::AnalyzerError;
pub use finding::{Confidence, Evidence, Finding, Location, Severity};
pub use language::{Language, LanguageDescriptor};

#[cfg(test)]
mod tests {
    use super::{
        AnalysisContext, Analyzer, AnalyzerDescriptor, Confidence, Engine, Finding, Severity,
    };

    struct TestAnalyzer;

    impl Analyzer for TestAnalyzer {
        fn descriptor(&self) -> AnalyzerDescriptor {
            AnalyzerDescriptor::new("test", "Test analyzer", "0.1.0")
        }

        fn analyze(&self, _context: &AnalysisContext) -> Result<Vec<Finding>, super::AnalyzerError> {
            Ok(vec![Finding::new(
                "finding-1",
                "test.rule",
                "correctness",
                Severity::Low,
                Confidence::Definite,
                "Test finding",
                self.descriptor().id,
            )])
        }
    }

    #[test]
    fn finding_round_trips_as_json() {
        let finding = Finding::new(
            "finding-1",
            "security.sql-injection",
            "security",
            Severity::High,
            Confidence::Likely,
            "User-controlled input reaches a SQL sink.",
            "ast-security",
        );

        let encoded = serde_json::to_string(&finding).expect("finding should serialize");
        let decoded: Finding = serde_json::from_str(&encoded).expect("finding should deserialize");

        assert_eq!(decoded, finding);
    }

    #[test]
    fn engine_runs_registered_analyzers() {
        let engine = Engine::new().with_analyzer(TestAnalyzer);
        let context = AnalysisContext::default();

        let findings = engine.analyze(&context).expect("analyzer should succeed");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "test.rule");
    }
}
