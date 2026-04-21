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
use crate::runtime::Runtime;
use serde_json::{self, Value, from_str};

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
    // Array Push : list.push(item)
    if expr.ends_with(')') {
        if let Some(idx) = expr.find(".push(") {
            let target = expr[..idx].trim().to_string();
            let value_part = expr[idx + 6..expr.len() - 1].trim();
            return OpCode::Push { target, value: value_part.to_string() };
        }
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
    pub runtime: Option<Runtime>, // Runtime optionnel pour le SSR
}

impl NhtmlParser {
    pub fn new() -> Self {
        Self { 
            ctx: ParserContext::new(),
            runtime: None,
        }
    }

    /// Prépare le runtime interne à partir des variables d'état déjà parsées
    pub fn sync_runtime(&mut self) {
        let state: HashMap<String, Value> = self.ctx.manifest.state_vars.iter()
            .map(|(k, v)| (k.clone(), v.initial_value.clone()))
            .collect();
        self.runtime = Some(Runtime::new(state));
    }

    /// Analyse un bloc de contenu jusqu'à une balise de fermeture spécifique, gérant l'imbrication
    pub fn parse_content_until<'a>(&mut self, input: &'a str, tag_name: &str, local_context: Option<&HashMap<String, Value>>) -> IResult<&'a str, String> {
        let mut result = String::new();
        let mut current = input;
        let open_pattern = format!("<{}", tag_name);
        let close_pattern = format!("</{}>", tag_name);
        let mut depth = 1;

        while !current.is_empty() {
            // Est-ce qu'on rencontre une ouverture imbriquée ?
            if current.starts_with(&open_pattern) {
                let followed_by = current[open_pattern.len()..].chars().next();
                if followed_by.is_none() || followed_by.unwrap().is_whitespace() || followed_by.unwrap() == '>' {
                    depth += 1;
                    result.push('<');
                    current = &current[1..];
                    continue;
                }
            }
            
            // Est-ce qu'on rencontre LA fermeture ?
            if current.starts_with(&close_pattern) {
                depth -= 1;
                if depth == 0 {
                    let (rem, _) = tag(close_pattern.as_str())(current)?;
                    return Ok((rem, result));
                } else {
                    // C'est une fermeture imbriquée, on la traite comme du texte et on avance
                    result.push('<');
                    current = &current[1..];
                    continue;
                }
            }

            // Expressions {expr} avec SSR
            if current.starts_with('{') {
                 if let Ok((rem, expr)) = parse_expression(current) {
                     let id = self.ctx.gen_id();
                     self.ctx.manifest.nodes.insert(id.clone(), Node::Text { expr: expr.clone() });
                     
                     // Rendu SSR si le runtime est disponible
                     let initial_val = if let Some(rt) = &self.runtime {
                        rt.eval(&expr, local_context).as_str().map(|s| s.to_string())
                            .unwrap_or_else(|| rt.eval(&expr, local_context).to_string())
                     } else {
                         "".to_string()
                     };

                     result.push_str(&format!("<span id=\"{}\">{}</span>", id, initial_val));
                     current = rem;
                     continue;
                 }
            }

            // Sinon on avance
            if let Ok((rem, captured)) = parse_static_text(current) {
                if !captured.is_empty() {
                    result.push_str(&captured);
                    current = rem;
                } else if !current.is_empty() {
                    let mut chars = current.chars();
                    result.push(chars.next().unwrap());
                    current = chars.as_str();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Err(nom::Err::Error(nom::error::Error::new(current, nom::error::ErrorKind::Tag)))
    }

    pub fn parse_document(&mut self, input: &str) -> String {
        let mut final_html = String::new();
        
        // PRE-PASS : On parse d'abord toutes les balises <var> pour isoler l'état initial avant le rendu
        let mut temp_input = input;
        while !temp_input.is_empty() {
            if temp_input.starts_with("<var") {
                if let Ok((rem, attrs)) = parse_var_tag_to_map(temp_input) {
                    temp_input = rem;
                    for (k, v) in attrs {
                        if k == "persist" { continue; }
                        let v_trimmed = v.trim();
                        let value = if (v_trimmed.starts_with('{') && v_trimmed.ends_with('}')) || (v_trimmed.starts_with('[') && v_trimmed.ends_with(']')) {
                            from_str(v_trimmed).unwrap_or_else(|_| Value::String(v.clone()))
                        } else if v_trimmed == "true" { Value::Bool(true) }
                        else if v_trimmed == "false" { Value::Bool(false) }
                        else if let Ok(n) = v_trimmed.parse::<i64>() { Value::Number(n.into()) }
                        else { Value::String(v.clone()) };
                        self.ctx.manifest.state_vars.insert(k, StateVar { initial_value: value, persist: false });
                    }
                    continue;
                }
            }
            if let Some(c) = temp_input.chars().next() {
                temp_input = &temp_input[c.len_utf8()..];
            } else { break; }
        }
        self.sync_runtime(); // Le runtime est prêt pour le SSR

        // PASS 2 : Rendu véritable
        let mut current_input = input;
        while !current_input.is_empty() {
             // Ignorer les directives <!nhtml ...> ou commentaires <!-- ... -->
            if current_input.starts_with("<!") {
                let res: IResult<&str, &str> = take_until(">")(current_input);
                if let Ok((rem, _)) = res {
                    let res_final: IResult<&str, &str> = tag(">")(rem);
                    if let Ok((rem_final, _)) = res_final {
                        current_input = rem_final; continue;
                    }
                }
            }

            // Ignorer <var> déjà traitées
            if current_input.starts_with("<var") {
                 if let Ok((rem, _)) = parse_var_tag_to_map(current_input) {
                    current_input = rem; continue;
                }
            }

            // 2. Balise <if> avec SSR
            if current_input.starts_with("<if") {
                if let Ok((rem, attrs)) = parse_control_tag_open(current_input, "if") {
                    let condition = attrs.get("condition").cloned().unwrap_or_default();
                    let (rem_after, content) = self.parse_content_until(rem, "if", None).expect("Failed to parse if body");
                    
                    let id = self.ctx.gen_id();
                    let group = format!("{}_group", id);
                    self.ctx.manifest.nodes.insert(id.clone(), Node::If { condition: condition.clone(), group, role: "if".to_string() });
                    
                    // SSR: Masquer si la condition initiale est fausse
                    let is_visible = self.runtime.as_ref().map(|rt| rt.eval(&condition, None).as_bool().unwrap_or(false)).unwrap_or(true);
                    let style = if is_visible { "" } else { " style=\"display: none;\"" };
                    
                    final_html.push_str(&format!("<div id=\"{}\"{}>{}</div>", id, style, content));
                    current_input = rem_after; continue;
                }
            }

            // 3. Balise <each> avec Rendu SSR Natif !
            if current_input.starts_with("<each") {
                if let Ok((rem, attrs)) = parse_control_tag_open(current_input, "each") {
                    let source_expr = attrs.get("in").cloned().unwrap_or_default().trim_matches(|c| c == '{' || c == '}').to_string();
                    let item_var = attrs.get("as").cloned().unwrap_or_else(|| "item".to_string());
                    let tag_name = attrs.get("tag").cloned().unwrap_or_else(|| "div".to_string());

                    let (rem_after, template_raw) = self.parse_content_until(rem, "each", None).expect("Failed to parse each body");
                    
                    let id = self.ctx.gen_id();
                    self.ctx.manifest.nodes.insert(id.clone(), Node::Each {
                        source: source_expr.clone(), item: item_var.clone(), template: template_raw.clone(),
                        index: attrs.get("index").cloned(), filter: None, tag: tag_name.clone(),
                    });

                    // SSR: Rendu immédiat
                    let mut rendered_loop = String::new();
                    if let Some(rt) = &self.runtime {
                        if let Some(list) = rt.eval(&source_expr, None).as_array() {
                            for (idx, val) in list.iter().enumerate() {
                                let mut local = HashMap::new();
                                local.insert(item_var.clone(), val.clone());
                                if let Some(idx_var) = attrs.get("index") {
                                    local.insert(idx_var.clone(), Value::Number(idx.into()));
                                }
                                let mut sub_parser = NhtmlParser::new();
                                sub_parser.ctx.next_id = self.ctx.next_id + (idx * 100); 
                                sub_parser.runtime = Some(Runtime::new(rt.state.clone()));
                                let rendered_item = sub_parser.parse_document_internal(&template_raw, Some(&local));
                                rendered_loop.push_str(&rendered_item);
                            }
                        }
                    }
                    final_html.push_str(&format!("<div id=\"{}\">{}</div>", id, rendered_loop));
                    current_input = rem_after; continue;
                }
            }

            // Expressions {expr}
            if current_input.starts_with('{') {
                if let Ok((rem, expr)) = parse_expression(current_input) {
                    let id = self.ctx.gen_id();
                    self.ctx.manifest.nodes.insert(id.clone(), Node::Text { expr: expr.clone() });
                    let val = self.runtime.as_ref().map(|rt| rt.eval(&expr, None).to_string()).unwrap_or_default();
                    final_html.push_str(&format!("<span id=\"{}\">{}</span>", id, val.trim_matches('"')));
                    current_input = rem; continue;
                }
            }

            // Balise HTML standard
            if current_input.starts_with('<') && !current_input.starts_with("</") {
                if let Ok((rem, (tag_name, attrs, self_closing))) = parse_any_tag(current_input) {
                    let mut reactive_bindings = HashMap::new();
                    let mut reactive_events = HashMap::new();
                    let mut static_attrs = Vec::new();
                    for (k, v) in attrs {
                        if k.starts_with("bind:") { reactive_bindings.insert(k[5..].to_string(), v.clone()); }
                        else if k.starts_with("on:") { reactive_events.insert(k[3..].to_string(), vec![parse_action(&v)]); }
                        else { static_attrs.push(format!("{}=\"{}\"", k, v)); }
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

            // Fallback : avancer d'un caractère pour éviter les boucles infinies
            // (catch-all pour </div>, </p>, etc. et tout texte non géré)
            if let Ok((rem, text)) = parse_static_text(current_input) {
                if !text.is_empty() {
                    final_html.push_str(&text);
                    current_input = rem;
                    continue;
                }
            }
            // Si parse_static_text n'a rien consommé (ex: commence par '<' ou '{'), avancer d'un char
            if let Some(c) = current_input.chars().next() {
                final_html.push(c);
                current_input = &current_input[c.len_utf8()..];
            } else { break; }
        }
        final_html
    }

    pub fn parse_document_internal(&mut self, input: &str, local_context: Option<&HashMap<String, Value>>) -> String {
        let mut result = String::new();
        let mut current = input;
        while !current.is_empty() {
            if current.starts_with('{') {
                if let Ok((rem, expr)) = parse_expression(current) {
                    let id = self.ctx.gen_id();
                    let val = self.runtime.as_ref().map(|rt| rt.eval(&expr, local_context).to_string()).unwrap_or_default();
                    result.push_str(&format!("<span id=\"{}\">{}</span>", id, val.trim_matches('"')));
                    current = rem; continue;
                }
            }
            // Fallback : avancer d'un caractère dans tous les cas
            if let Ok((rem, text)) = parse_static_text(current) {
                if !text.is_empty() {
                    result.push_str(&text);
                    current = rem;
                    continue;
                }
            }
            if let Some(c) = current.chars().next() {
                result.push(c);
                current = &current[c.len_utf8()..];
            } else { break; }
        }
        result
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
