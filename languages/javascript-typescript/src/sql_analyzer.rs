use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, Confidence, Evidence, Finding, Location,
    Severity,
};

use crate::{has_syntax_errors, parse};

/// Detects a narrow class of potentially unsafe raw SQL construction:
/// a query-like method receives a template string containing interpolation.
#[derive(Debug, Default, Clone, Copy)]
pub struct RawSqlTemplateAnalyzer;

impl Analyzer for RawSqlTemplateAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "javascript.raw-sql-template",
            "JavaScript raw SQL template analyzer",
            "0.1.0",
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
                            let property =
                                function
                                    .child_by_field_name("property")
                                    .and_then(|property| {
                                        property.utf8_text(file.content.as_bytes()).ok()
                                    });

                            let query_like =
                                matches!(property, Some("query") | Some("execute") | Some("raw"));

                            if query_like {
                                if let Some(arguments) = node.child_by_field_name("arguments") {
                                    for argument in arguments.named_children(&mut arguments.walk())
                                    {
                                        if argument.kind() != "template_string"
                                            || argument
                                                .child_by_field_name("string_fragment")
                                                .is_none()
                                        {
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
                                                "javascript.raw-sql-template:{}:{}:{}",
                                                file.path,
                                                start.row + 1,
                                                start.column + 1
                                            ),
                                            rule_id: "javascript.raw-sql-template".into(),
                                            category: "security".into(),
                                            severity: Severity::High,
                                            confidence,
                                            message: "Interpolated template data is passed to a query-like method.".into(),
                                            explanation: Some(
                                                "Review the interpolated value and prefer parameterized queries or the ORM's parameter-binding API. This rule is intentionally conservative and does not prove that the value is attacker-controlled."
                                                    .into(),
                                            ),
                                            location: Some(location.clone()),
                                            related_locations: Vec::new(),
                                            evidence: vec![Evidence {
                                                kind: "ast-template-substitution".into(),
                                                message: Some(
                                                    "A query-like call contains a template string with an interpolation."
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

                nodes.extend(node.children(&mut cursor));
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::RawSqlTemplateAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, Confidence, Severity, SourceFile};

    #[test]
    fn reports_interpolated_query_template() {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/user.ts".into(),
            language: Some("typescript".into()),
            content:
                "const id = input.id;\ndataSource.query(`SELECT * FROM users WHERE id = ${id}`);"
                    .into(),
        }]);

        let findings = RawSqlTemplateAnalyzer
            .analyze(&context)
            .expect("analysis should succeed");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
        assert_eq!(findings[0].confidence, Confidence::Likely);
    }

    #[test]
    fn ignores_non_query_template_strings() {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/user.ts".into(),
            language: Some("typescript".into()),
            content: "logger.raw(`hello ${name}`);".into(),
        }]);

        let findings = RawSqlTemplateAnalyzer
            .analyze(&context)
            .expect("analysis should succeed");

        assert!(findings.is_empty());
    }

    #[test]
    fn does_not_flag_parameterized_placeholder_without_interpolation() {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/user.ts".into(),
            language: Some("typescript".into()),
            content: "dataSource.query(`SELECT * FROM users WHERE id = $1`);".into(),
        }]);

        let findings = RawSqlTemplateAnalyzer
            .analyze(&context)
            .expect("analysis should succeed");

        assert!(findings.is_empty());
    }
}
