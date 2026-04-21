use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub state_vars: HashMap<String, StateVar>,
    pub nodes: HashMap<String, Node>,
}

impl Manifest {
    /// Charge un manifeste depuis une chaîne JSON
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateVar {
    pub initial_value: serde_json::Value,
    pub persist: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Node {
    #[serde(rename = "text")]
    Text { expr: String },
    #[serde(rename = "html")]
    Html { expr: String },
    #[serde(rename = "if")]
    If { 
        condition: String, 
        group: String,
        #[serde(default = "default_role")]
        role: String 
    },
    #[serde(rename = "each")]
    Each {
        #[serde(rename = "expr_in")]
        source: String,
        #[serde(rename = "expr_as")]
        item: String,
        #[serde(rename = "expr_index")]
        index: Option<String>,
        #[serde(rename = "expr_filter")]
        filter: Option<String>,
        template: String,
        #[serde(default = "default_tag")]
        tag: String,
    },
    #[serde(rename = "attrs")]
    Attrs {
        bindings: Option<HashMap<String, String>>,
        events: Option<HashMap<String, Vec<OpCode>>>,
    },
}

fn default_tag() -> String { "div".to_string() }
fn default_role() -> String { "if".to_string() }

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "op")]
pub enum OpCode {
    #[serde(rename = "set")]
    Set { target: String, value: String },
    #[serde(rename = "increment")]
    Increment { target: String, value: String },
    #[serde(rename = "call")]
    Call { 
        #[serde(rename = "fn")]
        fn_path: String, 
        args: Vec<String> 
    },
    #[serde(rename = "eval")]
    Eval { expr: String },
    #[serde(rename = "push")]
    Push { target: String, value: String },
}
