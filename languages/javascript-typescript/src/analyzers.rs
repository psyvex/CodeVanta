use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, Confidence, Evidence, Finding, Location, Severity,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ConsoleLogAnalyzer;

impl Analyzer for ConsoleLogAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "javascript.console-log",
            "JavaScript console.log analyzer",
            "0.1.0",
        )
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, codevanta_engine::AnalyzerError> {
        let mut findings = Vec::new();

        for file in context.files() {
            if !matches!(file.language.as_deref(), Some("javascript") | Some("typescript")) {
                continue;
            }

            for (index, line) in file.content.lines().enumerate() {
                if !line.contains("console.log(") {
                    continue;
                }

                let line_number = index as u32 + 1;
                let location = Location {
                    path: file.path.clone(),
                    start_line: line_number,
                    start_column: line.find("console.log").map(|column| column as u32 + 1),
                    end_line: Some(line_number),
                    end_column: None,
                };

                findings.push(Finding {
                    id: format!("javascript.console-log:{}:{}", file.path, line_number),
                    rule_id: "javascript.console-log".into(),
                    category: "maintainability".into(),
                    severity: Severity::Low,
                    confidence: Confidence::Likely,
                    message: "console.log is present in application source.".into(),
                    explanation: Some(
                        "Prefer the project's structured logging mechanism or remove debug output before production use."
                            .into(),
                    ),
                    location: Some(location.clone()),
                    related_locations: Vec::new(),
                    evidence: vec![Evidence {
                        kind: "source-pattern".into(),
                        message: Some("Matched console.log( in a JavaScript/TypeScript source file.".into()),
                        locations: vec![location],
                    }],
                    analyzer: self.descriptor().id,
                    tags: vec!["javascript".into(), "typescript".into(), "logging".into()],
                });
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::ConsoleLogAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, Confidence, SourceFile};

    #[test]
    fn reports_console_log_with_source_location() {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.ts".into(),
            language: Some("typescript".into()),
            content: "const value = 1;\nconsole.log(value);\n".into(),
        }]);

        let findings = ConsoleLogAnalyzer
            .analyze(&context)
            .expect("analysis should succeed");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_ref().unwrap().start_line, 2);
        assert_eq!(findings[0].confidence, Confidence::Likely);
    }
}
