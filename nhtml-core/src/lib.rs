pub mod ast;
pub mod parser;
pub mod runtime;

use std::collections::HashMap;
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

// ── FFI (Foreign Function Interface) pour PHP ────────────────────────────────
#[cfg(not(target_arch = "wasm32"))]
pub mod ffi {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    use serde::Serialize;

    #[derive(Serialize)]
    struct FFIResponse<'a> {
        html: &'a str,
        manifest: &'a ast::Manifest,
    }

    /// Fonction C appelée nativement par PHP
    #[unsafe(no_mangle)]
    pub extern "C" fn nhtml_compile(input: *const c_char) -> *mut c_char {
        let c_str = unsafe {
            if input.is_null() { return std::ptr::null_mut(); }
            CStr::from_ptr(input)
        };

        let source = c_str.to_str().unwrap_or("");
        
        // Utilisation du parser pour éviter de re-sérialiser 2 fois
        let mut n_parser = parser::NhtmlParser::new();
        let html = n_parser.parse_document(source);
        
        let response = FFIResponse {
            html: &html,
            manifest: &n_parser.ctx.manifest,
        };

        if let Ok(json_output) = serde_json::to_string(&response) {
            CString::new(json_output).unwrap().into_raw()
        } else {
            std::ptr::null_mut()
        }
    }

    /// Nettoyeur appelé par PHP
    #[unsafe(no_mangle)]
    pub extern "C" fn nhtml_free(s: *mut c_char) {
        unsafe {
            if s.is_null() { return; }
            let _ = CString::from_raw(s);
        }
    }
}

// ── WASM (WebAssembly) pour le Navigateur ────────────────────────────────
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn compile_wasm(source: &str) -> String {
        let result = compile(source);
        serde_json::json!({
            "html": result.html,
            "manifest": serde_json::from_str::<serde_json::Value>(&result.manifest_json).unwrap_or(serde_json::json!({}))
        }).to_string()
    }

    /// Nouveau bridge pour la réactivité "Zéro-JS"
    #[wasm_bindgen]
    pub fn dispatch_wasm(state_json: &str, op_json: &str, manifest_json: &str) -> String {
        let mut state: HashMap<String, serde_json::Value> = serde_json::from_str(state_json).unwrap_or_default();
        let ops: Vec<ast::OpCode> = serde_json::from_str(op_json).unwrap_or_default();
        let manifest: ast::Manifest = serde_json::from_str(manifest_json).unwrap_or(ast::Manifest { 
            state_vars: HashMap::new(), nodes: HashMap::new() 
        });

        let mut rt = runtime::Runtime::new(state);
        for op in ops {
            rt.apply_op(&op);
        }

        // Calcul des mises à jour DOM nécessaires (Diffing)
        let mut updates = HashMap::new();
        for (id, node) in &manifest.nodes {
            match node {
                ast::Node::Text { expr } => {
                    let val = rt.eval(expr, None);
                    updates.insert(id.clone(), val.as_str().map(|s| s.to_string()).unwrap_or(val.to_string()));
                },
                ast::Node::If { condition, group, .. } => {
                    let visible = rt.eval(condition, None).as_bool().unwrap_or(false);
                    updates.insert(id.clone(), serde_json::json!({ "visible": visible, "group": group }).to_string());
                },
                // Note: Each est plus complexe, sera géré par un re-rendu partiel dans une V2
                _ => {}
            }
        }

        serde_json::json!({
            "state": rt.state,
            "updates": updates
        }).to_string()
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
