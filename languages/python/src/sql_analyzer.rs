use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};
use tree_sitter::Node;

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct SqlAnalyzer;

impl Analyzer for SqlAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "python.sql-injection",
            "Python interpolated SQL analyzer",
            "0.1.0",
        )
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, AnalyzerError> {
        let mut findings = Vec::new();

        for file in context.files() {
            if file.language.as_deref() != Some("python") {
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
                if node.kind() == "call" {
                    if let Some(function) = node.child_by_field_name("function") {
                        let method_name = if function.kind() == "attribute" {
                            function
                                .child_by_field_name("attribute")
                                .and_then(|property| {
                                    property.utf8_text(file.content.as_bytes()).ok()
                                })
                        } else {
                            None
                        };

                        let query_like =
                            matches!(method_name, Some("execute") | Some("executemany"));

                        if query_like {
                            if let Some(arguments) = node.child_by_field_name("arguments") {
                                for argument in arguments.named_children(&mut arguments.walk()) {
                                    if let Some(location_node) = interpolated_query_node(argument) {
                                        let start = location_node.start_position();
                                        let end = location_node.end_position();
                                        let location = Location {
                                            path: file.path.clone(),
                                            start_line: start.row as u32 + 1,
                                            start_column: Some(start.column as u32 + 1),
                                            end_line: Some(end.row as u32 + 1),
                                            end_column: Some(end.column as u32 + 1),
                                        };

                                        findings.push(Finding {
                                            id: format!(
                                                "python.sql-injection:{}:{}:{}",
                                                file.path,
                                                start.row + 1,
                                                start.column + 1
                                            ),
                                            rule_id: "python.sql-injection".into(),
                                            category: "security".into(),
                                            severity: Severity::High,
                                            confidence,
                                            message: "Interpolated SQL query may allow SQL injection; use parameterized queries.".into(),
                                            explanation: Some(
                                                "Pass parameters as a separate argument to execute()/executemany() instead of interpolating them into the query string."
                                                    .into(),
                                            ),
                                            location: Some(location.clone()),
                                            related_locations: Vec::new(),
                                            evidence: vec![Evidence {
                                                kind: "ast-call".into(),
                                                message: Some(
                                                    "The Python AST contains an interpolated or %-formatted string passed to a query-like call."
                                                        .into(),
                                                ),
                                                locations: vec![location],
                                            }],
                                            analyzer: self.descriptor().id,
                                            tags: vec!["python".into(), "sql".into()],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                nodes.extend(node.named_children(&mut cursor));
            }
        }

        Ok(findings)
    }
}

/// Returns the node to report when `argument` is a query string built by
/// interpolation: an f-string with an `interpolation` child, or a `%`
/// binary operator whose left-hand side is a string.
fn interpolated_query_node(argument: Node) -> Option<Node> {
    if argument.kind() == "string" {
        let has_interpolation = argument
            .named_children(&mut argument.walk())
            .any(|child| child.kind() == "interpolation");
        return has_interpolation.then_some(argument);
    }

    if argument.kind() == "binary_operator" {
        let is_percent_format = argument
            .child_by_field_name("operator")
            .is_some_and(|operator| operator.kind() == "%")
            && argument
                .child_by_field_name("left")
                .is_some_and(|left| left.kind() == "string");
        return is_percent_format.then_some(argument);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::SqlAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, Confidence, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/db.py".into(),
            language: Some("python".into()),
            content: content.into(),
        }]);

        SqlAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_f_string_interpolated_query() {
        let findings = analyze("cursor.execute(f\"SELECT * FROM users WHERE id = {id}\")\n");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].confidence, Confidence::Likely);
    }

    #[test]
    fn flags_percent_formatted_query() {
        let findings = analyze("cursor.executemany(\"SELECT * FROM users WHERE id = %s\" % id)\n");
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn ignores_static_queries_and_non_query_methods() {
        let findings =
            analyze("cursor.execute(\"SELECT * FROM users\")\ncursor.commit(f\"value {x}\")\n");
        assert!(findings.is_empty());
    }
}
