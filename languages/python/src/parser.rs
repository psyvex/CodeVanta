use std::path::Path;

use tree_sitter::{Language, Parser, Tree};
use tree_sitter_python::LANGUAGE;

pub fn is_python_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("py") | Some("pyi")
    )
}

pub fn parse(path: &str, source: &str) -> Result<Tree, String> {
    if !is_python_path(Path::new(path)) {
        return Err(format!("unsupported Python parser input: {path}"));
    }

    let language: Language = LANGUAGE.into();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|error| format!("failed to configure parser: {error}"))?;

    parser
        .parse(source, None)
        .ok_or_else(|| "parser returned no syntax tree".to_string())
}

pub fn has_syntax_errors(tree: &Tree) -> bool {
    tree.root_node().has_error()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{has_syntax_errors, is_python_path, parse};

    #[test]
    fn recognizes_python_extensions() {
        assert!(is_python_path(Path::new("src/app.py")));
        assert!(is_python_path(Path::new("src/app.pyi")));
        assert!(!is_python_path(Path::new("src/app.ts")));
    }

    #[test]
    fn parses_python_without_syntax_errors() {
        let tree = parse("src/app.py", "value: int = 1\n").expect("should parse");
        assert!(!has_syntax_errors(&tree));
    }

    #[test]
    fn rejects_non_python_paths() {
        assert!(parse("src/app.ts", "const a = 1;").is_err());
    }
}
