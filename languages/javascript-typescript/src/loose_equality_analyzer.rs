use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct LooseEqualityAnalyzer;

impl Analyzer for LooseEqualityAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "javascript.loose-equality",
            "JavaScript loose equality analyzer",
            "0.1.0",
        )
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, AnalyzerError> {
        let mut findings = Vec::new();

        for file in context.files() {
            if !matches!(
                file.language.as_deref(),
                Some("javascript") | Some("typescript")
            ) {
                continue;
            }

            let tree =
                parse(&file.path, &file.content).map_err(|message| AnalyzerError::Failed {
                    analyzer: self.descriptor().id,
                    message,
                })?;

            let confidence = if has_syntax_errors(&tree) {
                Confidence::Possible
            } else {
                Confidence::Likely
            };

            let mut cursor = tree.walk();
            let mut nodes = vec![tree.root_node()];
            while let Some(node) = nodes.pop() {
                if node.kind() == "binary_expression" {
                    let operator = node
                        .children(&mut node.walk())
                        .find(|child| matches!(child.kind(), "==" | "!="));

                    if let Some(operator) = operator {
                        // `== null` / `!= null` is a common, intentional idiom for
                        // matching both null and undefined; flagging it produces
                        // false positives without a meaningful safety benefit.
                        let compares_to_null = node
                            .child_by_field_name("left")
                            .is_some_and(|node| node.kind() == "null")
                            || node
                                .child_by_field_name("right")
                                .is_some_and(|node| node.kind() == "null");

                        if !compares_to_null {
                            let start = node.start_position();
                            let end = node.end_position();
                            let location = Location {
                                path: file.path.clone(),
                                start_line: start.row as u32 + 1,
                                start_column: Some(start.column as u32 + 1),
                                end_line: Some(end.row as u32 + 1),
                                end_column: Some(end.column as u32 + 1),
                            };
                            let operator_text =
                                operator.utf8_text(file.content.as_bytes()).unwrap_or("==");

                            findings.push(Finding {
                                id: format!(
                                    "javascript.loose-equality:{}:{}:{}",
                                    file.path,
                                    start.row + 1,
                                    start.column + 1
                                ),
                                rule_id: "javascript.loose-equality".into(),
                                category: "maintainability".into(),
                                severity: Severity::Low,
                                confidence,
                                message: format!(
                                    "Loose equality ({operator_text}) triggers type coercion; prefer strict equality."
                                ),
                                explanation: Some(
                                    "Use === or !== unless coercion between types is explicitly intended."
                                        .into(),
                                ),
                                location: Some(location.clone()),
                                related_locations: Vec::new(),
                                evidence: vec![Evidence {
                                    kind: "ast-binary-expression".into(),
                                    message: Some(format!(
                                        "The TypeScript/JavaScript AST contains a binary expression using the {operator_text} operator."
                                    )),
                                    locations: vec![location],
                                }],
                                analyzer: self.descriptor().id,
                                tags: vec!["javascript".into(), "typescript".into(), "maintainability".into()],
                            });
                        }
                    }
                }

                nodes.extend(node.named_children(&mut cursor));
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::LooseEqualityAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.ts".into(),
            language: Some("typescript".into()),
            content: content.into(),
        }]);

        LooseEqualityAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_loose_equality_and_inequality() {
        let findings = analyze("if (a == b) {}\nif (a != b) {}\n");
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn ignores_strict_equality_and_null_checks() {
        let findings = analyze("if (a === b) {}\nif (a == null) {}\nif (null == a) {}\n");
        assert!(findings.is_empty());
    }
}
