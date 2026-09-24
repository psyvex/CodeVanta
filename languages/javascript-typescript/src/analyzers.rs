use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, Confidence, Evidence, Finding, Location,
    Severity,
};

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct ConsoleLogAnalyzer;

impl Analyzer for ConsoleLogAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "javascript.console-log",
            "JavaScript console.log analyzer",
            "0.2.0",
        )
    }

    fn analyze(
        &self,
        context: &AnalysisContext,
    ) -> Result<Vec<Finding>, codevanta_engine::AnalyzerError> {
        let mut findings = Vec::new();

        for file in context.files() {
            if !matches!(
                file.language.as_deref(),
                Some("javascript") | Some("typescript")
            ) {
                continue;
            }

            let tree = parse(&file.path, &file.content).map_err(|message| {
                codevanta_engine::AnalyzerError::Failed {
                    analyzer: self.descriptor().id.clone(),
                    message,
                }
            })?;
            let confidence = if has_syntax_errors(&tree) {
                Confidence::Possible
            } else {
                Confidence::Likely
            };

            let mut cursor = tree.walk();
            let mut nodes = vec![tree.root_node()];
            while let Some(node) = nodes.pop() {
                if node.kind() == "call_expression" {
                    if let Some(function) = node.child_by_field_name("function") {
                        if function.kind() == "member_expression" {
                            let object = function.child_by_field_name("object");
                            let property = function.child_by_field_name("property");
                            let matches_console_log = object
                                .and_then(|node| node.utf8_text(file.content.as_bytes()).ok())
                                == Some("console")
                                && property
                                    .and_then(|node| node.utf8_text(file.content.as_bytes()).ok())
                                    == Some("log");

                            if matches_console_log {
                                let start = function.start_position();
                                let end = function.end_position();
                                let location = Location {
                                    path: file.path.clone(),
                                    start_line: start.row as u32 + 1,
                                    start_column: Some(start.column as u32 + 1),
                                    end_line: Some(end.row as u32 + 1),
                                    end_column: Some(end.column as u32 + 1),
                                };

                                findings.push(Finding {
                                    id: format!(
                                        "javascript.console-log:{}:{}:{}",
                                        file.path,
                                        start.row + 1,
                                        start.column + 1
                                    ),
                                    rule_id: "javascript.console-log".into(),
                                    category: "maintainability".into(),
                                    severity: Severity::Low,
                                    confidence,
                                    message: "console.log is present in application source.".into(),
                                    explanation: Some(
                                        "Prefer the project's structured logging mechanism or remove debug output before production use."
                                            .into(),
                                    ),
                                    location: Some(location.clone()),
                                    related_locations: Vec::new(),
                                    evidence: vec![Evidence {
                                        kind: "ast-call-expression".into(),
                                        message: Some(
                                            "The TypeScript/JavaScript AST contains a console.log call expression."
                                                .into(),
                                        ),
                                        locations: vec![location],
                                    }],
                                    analyzer: self.descriptor().id,
                                    tags: vec!["javascript".into(), "typescript".into(), "logging".into()],
                                });
                            }
                        }
                    }
                }

                nodes.extend(node.children(&mut cursor));
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::ConsoleLogAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, Confidence, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.ts".into(),
            language: Some("typescript".into()),
            content: content.into(),
        }]);

        ConsoleLogAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn reports_ast_console_log_with_source_location() {
        let findings = analyze("const value = 1;\nconsole.log(value);\n");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_ref().unwrap().start_line, 2);
        assert_eq!(findings[0].confidence, Confidence::Likely);
    }

    #[test]
    fn ignores_strings_and_comments() {
        let findings = analyze(
            "const text = 'console.log(value)';\n// console.log(value)\nconst value = console.log('real');\n",
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_ref().unwrap().start_line, 3);
    }

    #[test]
    fn supports_tsx() {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/App.tsx".into(),
            language: Some("typescript".into()),
            content: "export const App = () => <button onClick={() => console.log('clicked')} />;"
                .into(),
        }]);

        let findings = ConsoleLogAnalyzer
            .analyze(&context)
            .expect("analysis should succeed");

        assert_eq!(findings.len(), 1);
    }
}
