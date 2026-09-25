mod bare_except_analyzer;
mod debug_artifact_analyzer;
mod eval_exec_analyzer;
mod mutable_default_argument_analyzer;
mod parser;
mod sql_analyzer;

pub use bare_except_analyzer::BareExceptAnalyzer;
pub use debug_artifact_analyzer::DebugArtifactAnalyzer;
pub use eval_exec_analyzer::EvalExecAnalyzer;
pub use mutable_default_argument_analyzer::MutableDefaultArgumentAnalyzer;
pub use parser::{has_syntax_errors, is_python_path, parse};
pub use sql_analyzer::SqlAnalyzer;

use codevanta_engine::{Language, LanguageDescriptor};

#[derive(Debug, Default, Clone, Copy)]
pub struct Python;

impl Language for Python {
    fn descriptor(&self) -> LanguageDescriptor {
        LanguageDescriptor::new("python", "Python", vec![".py".into(), ".pyi".into()])
    }
}
