pub mod ast;
pub mod parser;

use serde::Serialize;

#[derive(Serialize)]
pub struct CompilationResult {
    pub html: String,
    pub manifest_json: String,
}

/// Fonction pivot : Compile une chaîne Nhtml en (HTML hydraté + Manifeste JSON)
pub fn compile(source: &str) -> CompilationResult {
    let mut n_parser = parser::NhtmlParser::new();
    let html = n_parser.parse_document(source);
    
    // Sérialisation du manifeste pour le transport (JSON)
    let manifest_json = serde_json::to_string_pretty(&n_parser.ctx.manifest)
        .expect("Failed to serialize manifest");

    CompilationResult {
        html,
        manifest_json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_compilation_poc() {
        let nhtml = r#"
            <var name="test" value="42">
            <div on:click="counter++">
                Titre : {title}
            </div>
        "#;
        
        let result = compile(nhtml);
        
        // Vérifications
        assert!(result.html.contains("<div on:click=\"counter++\" id=\"n_"));
        assert!(result.manifest_json.contains("\"test\""));
    }
}
