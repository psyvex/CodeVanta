use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct SqlAnalyzer;

impl Analyzer for SqlAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "javascript.sql-injection",
            "JavaScript interpolated SQL analyzer",
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
                if node.kind() == "call_expression" {
                    if let Some(function) = node.child_by_field_name("function") {
                        if function.kind() == "member_expression" {
                            let property =
                                function
                                    .child_by_field_name("property")
                                    .and_then(|property| {
                                        property.utf8_text(file.content.as_bytes()).ok()
                                    });

                            // `raw` is intentionally excluded here because it is a common
                            // generic method name (for example, logger.raw()). Without
                            // receiver/package context, treating every `.raw()` call as SQL
                            // produces false positives. `query` and `execute` are explicit
                            // database-operation names and remain useful conservative signals.
                            let query_like = matches!(property, Some("query") | Some("execute"));

                            if query_like {
                                if let Some(arguments) = node.child_by_field_name("arguments") {
                                    for argument in arguments.named_children(&mut arguments.walk())
                                    {
                                        if argument.kind() != "template_string" {
                                            continue;
                                        }

                                        let has_interpolation = argument
                                            .named_children(&mut argument.walk())
                                            .any(|child| child.kind() == "template_substitution");

                                        if !has_interpolation {
                                            continue;
                                        }

                                        let start = argument.start_position();
                                        let end = argument.end_position();
                                        let location = Location {
                                            path: file.path.clone(),
                                            start_line: start.row as u32 + 1,
                                            start_column: Some(start.column as u32 + 1),
                                            end_line: Some(end.row as u32 + 1),
                                            end_column: Some(end.column as u32 + 1),
                                        };

                                        findings.push(Finding {
                                            id: format!(
                                                "javascript.sql-injection:{}:{}:{}",
                                                file.path,
                                                start.row + 1,
                                                start.column + 1
                                            ),
                                            rule_id: "javascript.sql-injection".into(),
                                            category: "security".into(),
                                            severity: Severity::High,
                                            confidence,
                                            message: "Interpolated SQL query may allow SQL injection; use parameterized queries.".into(),
                                            explanation: None,
                                            location: Some(location.clone()),
                                            related_locations: Vec::new(),
                                            evidence: vec![Evidence {
                                                kind: "ast-call-expression".into(),
                                                message: Some(
                                                    "The TypeScript/JavaScript AST contains an interpolated template string passed to a query-like call."
                                                        .into(),
                                                ),
                                                locations: vec![location],
                                            }],
                                            analyzer: self.descriptor().id,
                                            tags: vec!["javascript".into(), "typescript".into(), "sql".into()],
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

#[cfg(test)]
mod tests {
    use super::SqlAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, Confidence, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/db.ts".into(),
            language: Some("typescript".into()),
            content: content.into(),
        }]);

        SqlAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_interpolated_query_call() {
        let findings = analyze("db.query(`SELECT * FROM users WHERE id = ${id}`);\n");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].confidence, Confidence::Likely);
    }

    #[test]
    fn collects_multiple_findings_in_one_file() {
        let findings = analyze(
            "db.query(`SELECT * FROM a WHERE id = ${a}`);\ndb.execute(`SELECT * FROM b WHERE id = ${b}`);\n",
        );

        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn ignores_static_queries_and_non_query_methods() {
        let findings = analyze("db.query('SELECT * FROM users');\nlogger.raw(`value ${id}`);\n");

        assert!(findings.is_empty());
    }
}
