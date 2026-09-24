use thiserror::Error;

/// Errors produced while running a CodeVanta analyzer.
#[derive(Debug, Error)]
pub enum AnalyzerError {
    #[error("analyzer '{analyzer}' failed: {message}")]
    Failed { analyzer: String, message: String },
}
