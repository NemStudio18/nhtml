use crate::ast::{OpCode};
use serde_json::Value;
use std::collections::HashMap;

/// Évaluateur d'expressions minimaliste pour Nhtml (Native Rust)
pub struct Runtime {
    pub state: HashMap<String, Value>,
}

impl Runtime {
    pub fn new(initial_state: HashMap<String, Value>) -> Self {
        Self { state: initial_state }
    }

    /// Évalue une expression contre l'état actuel et un contexte local optionnel (ex: boucle each)
    pub fn eval(&self, expr: &str, local_context: Option<&HashMap<String, Value>>) -> Value {
        let expr = expr.trim();
        if expr.is_empty() { return Value::Null; }

        // 1. Essayer de parser comme un nombre
        if let Ok(n) = expr.parse::<i64>() {
            return Value::Number(n.into());
        }
        if let Ok(f) = expr.parse::<f64>() {
            if let Some(n) = serde_json::Number::from_f64(f) {
                return Value::Number(n);
            }
        }

        // 2. Booléens
        if expr == "true" { return Value::Bool(true); }
        if expr == "false" { return Value::Bool(false); }

        // 3. Chaînes littérales (entre guillemets)
        if (expr.starts_with('"') && expr.ends_with('"')) || (expr.starts_with('\'') && expr.ends_with('\'')) {
            return Value::String(expr[1..expr.len()-1].to_string());
        }

        // 4. Opérations logiques/comparaison simples (priorité basse pour l'instant)
        if let Some(idx) = expr.find("==") {
            let lhs = self.eval(expr[..idx].trim(), local_context);
            let rhs = self.eval(expr[idx+2..].trim(), local_context);
            return Value::Bool(lhs == rhs);
        }
        if let Some(idx) = expr.find('>') {
            let lhs = self.eval(expr[..idx].trim(), local_context);
            let rhs = self.eval(expr[idx+1..].trim(), local_context);
            if let (Some(l), Some(r)) = (lhs.as_f64(), rhs.as_f64()) {
                return Value::Bool(l > r);
            }
        }
        if let Some(idx) = expr.find('<') {
            let lhs = self.eval(expr[..idx].trim(), local_context);
            let rhs = self.eval(expr[idx+1..].trim(), local_context);
            if let (Some(l), Some(r)) = (lhs.as_f64(), rhs.as_f64()) {
                return Value::Bool(l < r);
            }
        }

        // 5. Résolution de variable (Locale puis Globale)
        self.resolve_variable(expr, local_context)
    }

    /// Applique une opération à l'état
    pub fn apply_op(&mut self, op: &OpCode) {
        match op {
            OpCode::Set { target, value } => {
                let val = self.eval(value, None);
                self.state.insert(target.clone(), val);
            },
            OpCode::Increment { target, value } => {
                let current_val = self.state.get(target).cloned().unwrap_or(Value::Number(0.into()));
                let inc_val = self.eval(value, None);
                
                if let (Some(c), Some(i)) = (current_val.as_f64(), inc_val.as_f64()) {
                    self.state.insert(target.clone(), Value::Number(serde_json::Number::from_f64(c + i).unwrap()));
                }
            },
            _ => {} // Call et Eval non gérés nativement pour l'instant
        }
    }

    fn resolve_variable(&self, path: &str, local_context: Option<&HashMap<String, Value>>) -> Value {
        let parts: Vec<&str> = path.split('.').collect();
        let root = parts[0];

        // Recherche dans le contexte local (ex: variable 'item' d'une boucle)
        let mut current = if let Some(local) = local_context {
            local.get(root).cloned()
        } else {
            None
        };

        // Recherche dans l'état global si non trouvé en local
        if current.is_none() {
            current = self.state.get(root).cloned();
        }

        // Si on a trouvé la racine, on descend dans les propriétés
        if let Some(mut val) = current {
            for part in &parts[1..] {
                // Gestion spéciale pour .length sur les tableaux et chaînes
                if *part == "length" {
                    if let Some(arr) = val.as_array() {
                        return Value::Number(arr.len().into());
                    }
                    if let Some(s) = val.as_str() {
                        return Value::Number(s.len().into());
                    }
                }
                
                if let Some(next) = val.get(part) {
                    val = next.clone();
                } else {
                    return Value::Null;
                }
            }
            val
        } else {
            Value::Null
        }
    }
}
