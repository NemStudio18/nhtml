use std::collections::HashMap;
use nom::{
    bytes::complete::{tag, take_while, is_not, take_until},
    character::complete::{char, multispace0},
    sequence::{delimited},
    multi::many0,
    IResult,
    branch::alt,
    Parser,
    combinator::opt,
};
use crate::ast::{Manifest, Node, StateVar, OpCode};

/// Contexte pour suivre l'état global pendant le parsing
pub struct ParserContext {
    pub next_id: usize,
    pub manifest: Manifest,
}

impl ParserContext {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            manifest: Manifest {
                state_vars: HashMap::new(),
                nodes: HashMap::new(),
            },
        }
    }

    pub fn gen_id(&mut self) -> String {
        let id = format!("n_{}", self.next_id);
        self.next_id += 1;
        id
    }
}

/// Analyse une paire clé="valeur" ou clé='valeur'
pub fn parse_attribute(input: &str) -> IResult<&str, (String, String)> {
    let (input, _) = multispace0(input)?;
    let (input, key) = take_while(|c: char| c.is_alphanumeric() || c == ':' || c == '-' || c == '.' || c == '_')(input)?;
    let (input, _) = char('=')(input)?;
    
    let (input, value) = alt((
        delimited(char('"'), opt(is_not("\"")), char('"')).map(|v| v.unwrap_or("").to_string()),
        delimited(char('\''), opt(is_not("'")), char('\'')).map(|v| v.unwrap_or("").to_string()),
        take_while(|c: char| !c.is_whitespace() && c != '>') .map(|v: &str| v.to_string()),
    ))(input)?;

    Ok((input, (key.to_string(), value)))
}

/// Analyse un texte jusqu'à la prochaine expression ou balise
pub fn parse_static_text(input: &str) -> IResult<&str, String> {
    let (input, content) = take_while(|c: char| c != '{' && c != '<')(input)?;
    if content.is_empty() && !input.is_empty() {
        // Si on est bloqué sur un car spéc, on vérifie si c'est une expr
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    Ok((input, content.to_string()))
}

/// Analyse une expression {counter + 1}
pub fn parse_expression(input: &str) -> IResult<&str, String> {
    delimited(char('{'), is_not("}"), char('}'))(input).map(|(i, s)| (i, s.to_string()))
}

/// Utilitaire : retire une paire d'accolades englobantes si présentes
/// Exemples : "{complex_var.items}" → "complex_var.items", "foo" → "foo"
pub fn strip_braces(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with('{') && s.ends_with('}') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Convertit une expression brute Nhtml simple en OpCode spécialisé (évite le Eval générique là où c'est possible)
pub fn parse_action(expr: &str) -> OpCode {
    let expr = expr.trim();
    if expr.ends_with("++") {
        let target = expr[..expr.len() - 2].trim().to_string();
        return OpCode::Increment { target, value: "1".to_string() };
    }
    if expr.ends_with("--") {
        let target = expr[..expr.len() - 2].trim().to_string();
        return OpCode::Increment { target, value: "-1".to_string() };
    }
    if let Some(idx) = expr.find("+=") {
        let target = expr[..idx].trim().to_string();
        let value = expr[idx + 2..].trim().to_string();
        return OpCode::Increment { target, value };
    }
    if let Some(idx) = expr.find("-=") {
        let target = expr[..idx].trim().to_string();
        let value = expr[idx + 2..].trim().to_string();
        return OpCode::Increment { target, value: format!("-({})", value) };
    }
    // Set (après += et -=)
    if let Some(idx) = expr.find('=') {
        let target = expr[..idx].trim().to_string();
        let value = expr[idx + 1..].trim().to_string();
        return OpCode::Set { target, value };
    }
    // Call
    if expr.ends_with(')') {
        if let Some(idx) = expr.find('(') {
            let fn_path = expr[..idx].trim().to_string();
            let args_part = expr[idx + 1..expr.len() - 1].trim();
            let args = if args_part.is_empty() {
                Vec::new()
            } else {
                args_part.split(',').map(|s| s.trim().to_string()).collect()
            };
            return OpCode::Call { fn_path, args };
        }
    }
    // Fallback dynamique
    OpCode::Eval { expr: expr.to_string() }
}

/// Structure principale pour coordonner le parsing
pub struct NhtmlParser {
    pub ctx: ParserContext,
}

impl NhtmlParser {
    pub fn new() -> Self {
        Self { ctx: ParserContext::new() }
    }

    /// Analyse un bloc de contenu jusqu'à une balise de fermeture spécifique
    pub fn parse_content_until<'a>(&mut self, input: &'a str, closing_tag: &str) -> IResult<&'a str, String> {
        let mut result = String::new();
        let mut current = input;
        let close_pattern = format!("</{}>", closing_tag);

        while !current.is_empty() && !current.starts_with(&close_pattern) {
            let (next_input, captured) = parse_static_text(current)?;
            result.push_str(&captured);
            current = next_input;
            
            if current.starts_with('{') {
                 if let Ok((rem, expr)) = parse_expression(current) {
                     let id = self.ctx.gen_id();
                     self.ctx.manifest.nodes.insert(id.clone(), Node::Text { expr });
                     result.push_str(&format!("<span id=\"{}\"></span>", id));
                     current = rem;
                 }
            } else if !current.is_empty() && !current.starts_with(&close_pattern) {
                let mut chars = current.chars();
                result.push(chars.next().unwrap());
                current = chars.as_str();
            }
        }
        
        let (current, _) = tag(close_pattern.as_str())(current)?;
        Ok((current, result))
    }

    pub fn parse_document(&mut self, input: &str) -> String {
        let mut final_html = String::new();
        let mut current_input = input;

        while !current_input.is_empty() {
            // 0. Ignorer les directives <!nhtml ...> ou commentaires <!-- ... -->
            if current_input.starts_with("<!") {
                let res: IResult<&str, &str> = take_until(">")(current_input);
                if let Ok((rem, _)) = res {
                    let res_final: IResult<&str, &str> = tag(">")(rem);
                    if let Ok((rem_final, _)) = res_final {
                        current_input = rem_final; continue;
                    }
                }
            }

            // 1. Balise <var>
            if current_input.starts_with("<var") {
                 if let Ok((rem, attrs)) = parse_var_tag_to_map(current_input) {
                    current_input = rem;
                    let persist = attrs.contains_key("persist") && attrs.get("persist").unwrap() == "true";
                    for (k, v) in attrs {
                        if k == "persist" { continue; }
                        
                        let v_trimmed = v.trim();
                        // Désérialisation native complète !
                        let value = if (v_trimmed.starts_with('{') && v_trimmed.ends_with('}')) || 
                                       (v_trimmed.starts_with('[') && v_trimmed.ends_with(']')) {
                            // On tente de parser le JSON (objets complexes)
                            serde_json::from_str(v_trimmed).unwrap_or_else(|_| serde_json::Value::String(v.clone()))
                        } else if v_trimmed == "true" { 
                            serde_json::Value::Bool(true) 
                        } else if v_trimmed == "false" { 
                            serde_json::Value::Bool(false) 
                        } else if let Ok(n) = v_trimmed.parse::<i64>() { 
                            serde_json::Value::Number(n.into()) 
                        } else if let Ok(f) = v_trimmed.parse::<f64>() {
                            if let Some(n) = serde_json::Number::from_f64(f) {
                                serde_json::Value::Number(n)
                            } else {
                                serde_json::Value::String(v.clone())
                            }
                        } else { 
                            serde_json::Value::String(v.clone()) 
                        };
                        
                        self.ctx.manifest.state_vars.insert(k, StateVar {
                            initial_value: value,
                            persist,
                        });
                    }
                    continue;
                }
            }
            
            // 2. Balise <if>
            if current_input.starts_with("<if") {
                if let Ok((rem, attrs)) = parse_control_tag_open(current_input, "if") {
                    let condition = attrs.get("condition").cloned().unwrap_or_default();
                    let (rem_after, content) = self.parse_content_until(rem, "if").expect("Failed to parse if body");
                    
                    let id = self.ctx.gen_id();
                    let group = format!("{}_group", id);
                    self.ctx.manifest.nodes.insert(id.clone(), Node::If { condition, group, role: "if".to_string() });
                    final_html.push_str(&format!("<div id=\"{}\">{}</div>", id, content));
                    current_input = rem_after; 
                    continue;
                }
            }

            // 3. Balise <each>
            if current_input.starts_with("<each") {
                if let Ok((rem, attrs)) = parse_control_tag_open(current_input, "each") {
                    // Correction #4 : Supprimer les {} de l'expression source
                    let source = attrs.get("in").cloned().unwrap_or_default();
                    let source = source.trim_matches(|c| c == '{' || c == '}').to_string();
                    let item = attrs.get("as").cloned().unwrap_or_else(|| "item".to_string());
                    let index = attrs.get("index").cloned();
                    let filter = attrs.get("filter").cloned();
                    let tag_name = attrs.get("tag").cloned().unwrap_or_else(|| "div".to_string());

                    let (rem_after, template) = self.parse_content_until(rem, "each").expect("Failed to parse each body");
                    
                    let id = self.ctx.gen_id();
                    self.ctx.manifest.nodes.insert(id.clone(), Node::Each {
                        source, item, template, index, filter, tag: tag_name,
                    });
                    final_html.push_str(&format!("<div id=\"{}\"></div>", id));
                    current_input = rem_after; continue;
                }
            }

            // 3.5. Balise <empty> (alias logique vers <if condition="!(arr && arr.length > 0)">)
            if current_input.starts_with("<empty") {
                if let Ok((rem, attrs)) = parse_control_tag_open(current_input, "empty") {
                    let source = attrs.get("for").cloned().unwrap_or_default();
                    let source = source.trim_matches(|c| c == '{' || c == '}').to_string();
                    let condition = format!("!({var} && {var}.length > 0)", var=source);
                    
                    let (rem_after, content) = self.parse_content_until(rem, "empty").expect("Failed to parse empty body");
                    
                    let id = self.ctx.gen_id();
                    let group = format!("{}_group", id);
                    self.ctx.manifest.nodes.insert(id.clone(), Node::If { condition, group, role: "if".to_string() });
                    final_html.push_str(&format!("<div id=\"{}\">{}</div>", id, content));
                    current_input = rem_after; 
                    continue;
                }
            }

            // 4. Balises opaques (style, script) - On ne parse PAS l'intérieur pour éviter les conflits { }
            if current_input.starts_with("<style") || current_input.starts_with("<script") {
                let tag_name = if current_input.starts_with("<style") { "style" } else { "script" };
                if let Ok((rem, (name, attrs, _))) = parse_any_tag(current_input) {
                    let attr_str = attrs.iter().map(|(k,v)| format!("{}=\"{}\"", k, v)).collect::<Vec<_>>().join(" ");
                    final_html.push_str(&format!("<{} {}>", name, attr_str));
                    let end_tag = format!("</{}>", tag_name);
                    let res: IResult<&str, &str> = take_until(end_tag.as_str())(rem);
                    if let Ok((rem_after, content)) = res {
                        final_html.push_str(content);
                        final_html.push_str(&end_tag);
                        let res_final: IResult<&str, &str> = tag(end_tag.as_str())(rem_after);
                        if let Ok((rem_final, _)) = res_final {
                            current_input = rem_final; continue;
                        }
                    }
                }
            }

            // 5. Balise HTML standard avec attributs réactifs
            if current_input.starts_with('<') && !current_input.starts_with("</") {
                if let Ok((rem, (tag_name, attrs, self_closing))) = parse_any_tag(current_input) {
                    let mut reactive_bindings = HashMap::new();
                    let mut reactive_events = HashMap::new();
                    let mut static_attrs = Vec::new();

                    for (k, v) in attrs {
                        if k.starts_with("bind:") {
                            let attr_name = k[5..].to_string();
                            reactive_bindings.insert(attr_name.clone(), v.clone());
                            // Two-way binding : génération de l'event montant (DOM → state)
                            let var_name = strip_braces(&v).to_string();
                            // Événement selon la propriété liée
                            let event_name = match attr_name.as_str() {
                                "checked" => "change",
                                _ => "input",
                            };
                            reactive_events
                                .entry(event_name.to_string())
                                .or_insert_with(Vec::new)
                                .push(OpCode::Set { target: var_name, value: "this.value".to_string() });
                            static_attrs.push(format!("{}=\"{}\"", k, v));
                        } else if k.starts_with("on:") {
                            // Phase 4.5 résolue : Parsing sémantique des actions
                            let opcode = parse_action(&v);
                            reactive_events.insert(k[3..].to_string(), vec![opcode]);
                            static_attrs.push(format!("{}=\"{}\"", k, v));
                        } else {
                            static_attrs.push(format!("{}=\"{}\"", k, v));
                        }
                    }

                    if !reactive_bindings.is_empty() || !reactive_events.is_empty() {
                        let id = self.ctx.gen_id();
                        self.ctx.manifest.nodes.insert(id.clone(), Node::Attrs {
                            bindings: if reactive_bindings.is_empty() { None } else { Some(reactive_bindings) },
                            events: if reactive_events.is_empty() { None } else { Some(reactive_events) },
                        });
                        static_attrs.push(format!("id=\"{}\"", id));
                    }

                    let attr_str = if static_attrs.is_empty() { "".to_string() } else { format!(" {}", static_attrs.join(" ")) };
                    final_html.push_str(&format!("<{}{}{}>", tag_name, attr_str, if self_closing { "/" } else { "" }));
                    current_input = rem; continue;
                }
            }

            // 5. Expressions {expr}
            if current_input.starts_with('{') {
                if let Ok((rem, expr)) = parse_expression(current_input) {
                    let id = self.ctx.gen_id();
                    self.ctx.manifest.nodes.insert(id.clone(), Node::Text { expr });
                    final_html.push_str(&format!("<span id=\"{}\"></span>", id));
                    current_input = rem;
                    continue;
                }
            }

            // 6. Texte Statique
            if let Ok((rem, text)) = parse_static_text(current_input) {
                final_html.push_str(&text);
                current_input = rem;
            } else if !current_input.is_empty() {
                let mut chars = current_input.chars();
                final_html.push(chars.next().unwrap());
                current_input = chars.as_str();
            }
        }
        final_html
    }
}

fn parse_control_tag_open<'a>(input: &'a str, name: &str) -> IResult<&'a str, HashMap<String, String>> {
    let (input, _) = tag("<")(input)?;
    let (input, _) = tag(name)(input)?;
    let (input, attrs_vec) = many0(parse_attribute)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(">")(input)?;
    let mut map = HashMap::new();
    for (k, v) in attrs_vec { map.insert(k, v); }
    Ok((input, map))
}

pub fn parse_any_tag(input: &str) -> IResult<&str, (String, HashMap<String, String>, bool)> {
    let (input, _) = char('<')(input)?;
    let (input, name) = take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '.')(input)?;
    let (input, attrs_vec) = many0(parse_attribute)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, self_closing) = alt((
        tag("/>").map(|_| true),
        tag(">").map(|_| false),
    ))(input)?;
    
    let mut attrs = HashMap::new();
    for (k, v) in attrs_vec { attrs.insert(k, v); }
    Ok((input, (name.to_string(), attrs, self_closing)))
}

fn parse_var_tag_to_map(input: &str) -> IResult<&str, HashMap<String, String>> {
    let (input, _) = tag("<var")(input)?;
    let (input, attrs_vec) = many0(parse_attribute)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(">")(input)?;
    let mut map = HashMap::new();
    for (k, v) in attrs_vec { map.insert(k, v); }
    Ok((input, map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_parsing() {
        let input = " name=\"test\" persist='true'";
        let (_, attrs) = many0(parse_attribute)(input).unwrap();
        assert_eq!(attrs[0].0, "name");
        assert_eq!(attrs[0].1, "test");
        assert_eq!(attrs[1].0, "persist");
        assert_eq!(attrs[1].1, "true");
    }
}
