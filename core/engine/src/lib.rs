pub mod analyzer;
pub mod context;
pub mod engine;
pub mod errors;
pub mod finding;

pub use analyzer::{Analyzer, AnalyzerDescriptor};
pub use context::{AnalysisContext, SourceFile};
pub use engine::Engine;
pub use errors::AnalyzerError;
pub use finding::{Confidence, Evidence, Finding, Location, Severity};

#[cfg(test)]
mod tests {
    use super::{Confidence, Finding, Severity};

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
}
