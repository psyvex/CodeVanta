use serde::{Deserialize, Serialize};

/// Severity describes the potential impact if a finding is valid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Confidence describes how strongly the available evidence supports a finding.
/// It is independent from severity: a critical issue can still be only possible.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Definite,
    Likely,
    Possible,
    Suggestion,
}

/// A source location associated with a finding or piece of evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub path: String,
    pub start_line: u32,
    pub start_column: Option<u32>,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// Evidence supporting a finding from deterministic tools, source-flow analysis,
/// model reasoning, or another repository artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub kind: String,
    pub message: Option<String>,
    pub locations: Vec<Location>,
}

/// A structured code-quality finding produced by a deterministic or AI analyzer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: String,
    pub rule_id: String,
    pub category: String,
    pub severity: Severity,
    pub confidence: Confidence,
    pub message: String,
    pub explanation: Option<String>,
    pub location: Option<Location>,
    pub related_locations: Vec<Location>,
    pub evidence: Vec<Evidence>,
    pub analyzer: String,
    pub tags: Vec<String>,
}

impl Finding {
    pub fn new(
        id: impl Into<String>,
        rule_id: impl Into<String>,
        category: impl Into<String>,
        severity: Severity,
        confidence: Confidence,
        message: impl Into<String>,
        analyzer: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            rule_id: rule_id.into(),
            category: category.into(),
            severity,
            confidence,
            message: message.into(),
            explanation: None,
            location: None,
            related_locations: Vec::new(),
            evidence: Vec::new(),
            analyzer: analyzer.into(),
            tags: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_serializes_with_schema_compatible_names() {
        let finding = Finding::new(
            "finding-1",
            "security.sql-injection",
            "security",
            Severity::Critical,
            Confidence::Possible,
            "Potential SQL injection",
            "ast-security",
        );

        let json = serde_json::to_string(&finding).expect("finding should serialize");
        assert!(json.contains("\"severity\":\"critical\""));
        assert!(json.contains("\"confidence\":\"possible\""));
        assert!(json.contains("\"ruleId\""));
    }
}
