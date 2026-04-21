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

        // 4. Opérations logiques/comparaison simples
        if let Some(idx) = expr.find("==") {
            let lhs = self.eval(expr[..idx].trim(), local_context);
            let rhs = self.eval(expr[idx+2..].trim(), local_context);
            return Value::Bool(lhs == rhs);
        }
        if let Some(idx) = expr.find('>') {
            let lhs = self.eval(expr[..idx].trim(), local_context);
            let rhs = self.eval(expr[idx+1..].trim(), local_context);
            if let (Some(l), Some(r)) = (self.as_f64(&lhs), self.as_f64(&rhs)) {
                return Value::Bool(l > r);
            }
        }
        if let Some(idx) = expr.find('<') {
            let lhs = self.eval(expr[..idx].trim(), local_context);
            let rhs = self.eval(expr[idx+1..].trim(), local_context);
            if let (Some(l), Some(r)) = (self.as_f64(&lhs), self.as_f64(&rhs)) {
                return Value::Bool(l < r);
            }
        }

        // 4.5 Opérations arithmétiques basiques (basique : une seule opération à la fois pour la POC)
        if let Some(idx) = expr.find('+') {
            let lhs = self.eval(expr[..idx].trim(), local_context);
            let rhs = self.eval(expr[idx+1..].trim(), local_context);
            if let (Some(l), Some(r)) = (self.as_f64(&lhs), self.as_f64(&rhs)) {
                return Value::Number(serde_json::Number::from_f64(l + r).unwrap());
            }
            // Concaténation de chaînes
            if let (Some(l), Some(r)) = (lhs.as_str(), rhs.as_str()) {
                return Value::String(format!("{}{}", l, r));
            }
        }
        if let Some(idx) = expr.find('-') {
            let lhs = self.eval(expr[..idx].trim(), local_context);
            let rhs = self.eval(expr[idx+1..].trim(), local_context);
            if let (Some(l), Some(r)) = (self.as_f64(&lhs), self.as_f64(&rhs)) {
                return Value::Number(serde_json::Number::from_f64(l - r).unwrap());
            }
        }

        // 5. Résolution de variable (Locale puis Globale)
        self.resolve_variable(expr, local_context)
    }

    /// Utilitaire pour forcer l'extraction d'un nombre (gère les chaînes numériques)
    fn as_f64(&self, v: &Value) -> Option<f64> {
        if let Some(f) = v.as_f64() { return Some(f); }
        if let Some(s) = v.as_str() {
            if let Ok(f) = s.parse::<f64>() { return Some(f); }
        }
        None
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
                
                if let (Some(c), Some(i)) = (self.as_f64(&current_val), self.as_f64(&inc_val)) {
                    self.state.insert(target.clone(), Value::Number(serde_json::Number::from_f64(c + i).unwrap()));
                }
            },
            OpCode::Push { target, value } => {
                let val = self.eval(value, None);
                if let Some(current_val) = self.state.get_mut(target) {
                    if let Some(arr) = current_val.as_array_mut() {
                        arr.push(val);
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eval_simple() {
        let rt = Runtime::new(HashMap::new());
        assert_eq!(rt.eval("123", None), json!(123));
        assert_eq!(rt.eval("true", None), json!(true));
        assert_eq!(rt.eval("'hello'", None), json!("hello"));
    }

    #[test]
    fn test_eval_vars() {
        let mut state = HashMap::new();
        state.insert("counter".to_string(), json!(10));
        state.insert("user".to_string(), json!({"name": "Naim"}));
        let rt = Runtime::new(state);

        assert_eq!(rt.eval("counter", None), json!(10));
        assert_eq!(rt.eval("user.name", None), json!("Naim"));
        assert_eq!(rt.eval("counter + 5", None), json!(15.0));
    }

    #[test]
    fn test_eval_conditions() {
        let mut state = HashMap::new();
        state.insert("counter".to_string(), json!(15));
        let rt = Runtime::new(state);

        assert_eq!(rt.eval("counter > 10", None), json!(true));
        assert_eq!(rt.eval("counter == 15", None), json!(true));
    }
    #[test]
    fn test_push_operation() {
        let mut state = HashMap::new();
        state.insert("list".to_string(), serde_json::json!(["a", "b"]));
        let mut rt = Runtime::new(state);
        
        rt.apply_op(&OpCode::Push { target: "list".to_string(), value: "\"c\"".to_string() });
        
        let list = rt.state.get("list").unwrap().as_array().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[2], "c");
    }
}
