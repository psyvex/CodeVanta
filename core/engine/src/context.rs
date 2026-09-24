use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A source file made available to an analyzer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub language: Option<String>,
    pub content: String,
}

/// Confidence attached to repository ecosystem detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectionConfidence {
    Definite,
    Likely,
    Possible,
}

/// Evidence supporting a detected ecosystem technology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnologyEvidence {
    pub kind: String,
    pub source: String,
    pub detail: String,
}

/// A detected technology and the evidence supporting it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnologyContext {
    pub id: String,
    pub confidence: DetectionConfidence,
    pub evidence: Vec<TechnologyEvidence>,
}

/// Repository-scoped ecosystem information shared by analyzers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EcosystemContext {
    pub runtime: Option<String>,
    pub technologies: Vec<TechnologyContext>,
}

/// Repository-scoped information shared by analyzers.
#[derive(Debug, Default)]
pub struct AnalysisContext {
    files: Vec<SourceFile>,
    metadata: BTreeMap<String, String>,
    ecosystem: Option<EcosystemContext>,
}

impl AnalysisContext {
    pub fn new(files: Vec<SourceFile>) -> Self {
        Self {
            files,
            metadata: BTreeMap::new(),
            ecosystem: None,
        }
    }

    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    pub fn ecosystem(&self) -> Option<&EcosystemContext> {
        self.ecosystem.as_ref()
    }

    pub fn set_ecosystem(&mut self, ecosystem: EcosystemContext) {
        self.ecosystem = Some(ecosystem);
    }

    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_technology_detection_provenance() {
        let ecosystem = EcosystemContext {
            runtime: Some("nodejs".into()),
            technologies: vec![TechnologyContext {
                id: "typeorm".into(),
                confidence: DetectionConfidence::Definite,
                evidence: vec![TechnologyEvidence {
                    kind: "manifest".into(),
                    source: "package.json".into(),
                    detail: "dependency: typeorm".into(),
                }],
            }],
        };

        let json = serde_json::to_string(&ecosystem).expect("ecosystem should serialize");
        assert!(json.contains("\"typeorm\""));
        assert!(json.contains("\"package.json\""));
        assert!(json.contains("\"definite\""));
    }
}
