use codevanta_engine::{AnalysisFile, Analyzer, Confidence, Finding, FindingKind};
use tree_sitter::{Node, Parser};

use crate::parser::parse;

#[derive(Debug, Default)]
pub struct SqlAnalyzer;

impl Analyzer for SqlAnalyzer {
    fn name(&self) -> &'static str {
        "javascript.sql-injection"
    }

    fn analyze(&self, file: &AnalysisFile) -> Vec<Finding> {
        let Some(tree) = parse(file) else {
            return Vec::new();
        };

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
                        let property = function
                            .child_by_field_name("property")
                            .and_then(|property| property.utf8_text(file.content.as_bytes()).ok());

                        // `raw` is intentionally excluded here because it is a common
                        // generic method name (for example, logger.raw()). Without
                        // receiver/package context, treating every `.raw()` call as SQL
                        // produces false positives. `query` and `execute` are explicit
                        // database-operation names and remain useful conservative signals.
                        let query_like = matches!(property, Some("query") | Some("execute"));

                        if query_like {
                            if let Some(arguments) = node.child_by_field_name("arguments") {
                                for argument in arguments.named_children(&mut arguments.walk()) {
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
                                    let message = "Interpolated SQL query may allow SQL injection; use parameterized queries.";
                                    nodes.push(argument);

                                    return vec![Finding::new(
                                        "javascript.sql-injection",
                                        FindingKind::Security,
                                        confidence,
                                        message,
                                        start.row + 1,
                                        start.column + 1,
                                        end.row + 1,
                                        end.column + 1,
                                    )];
                                }
                            }
                        }
                    }
                }
            }

            nodes.extend(node.named_children(&mut cursor));
        }

        Vec::new()
    }
}

fn has_syntax_errors(tree: &tree_sitter::Tree) -> bool {
    let mut cursor = tree.walk();
    let mut nodes = vec![tree.root_node()];
    while let Some(node) = nodes.pop() {
        if node.is_error() || node.is_missing() {
            return true;
        }
        nodes.extend(node.named_children(&mut cursor));
    }
    false
}
