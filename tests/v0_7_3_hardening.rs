/// Tests de régression pour les correctifs de sécurité et hardening de la v0.7.3
/// MED-25 : Renforcement de la couverture des tests de sécurité

use nhtml_gateway::compiler::NhtmlCompiler;

// ─── Tests : Compilateur (Profondeur + Overflow) ──────────────────────────────

#[test]
fn test_compiler_empty_source() {
    let result = NhtmlCompiler::compile("");
    // Ne doit pas paniquer, retourner un résultat valide (fallback)
    assert!(!result.html.is_empty(), "Le résultat HTML ne devrait pas être vide sur source vide");
}

#[test]
fn test_compiler_no_nid() {
    let source = "<div>Simple content</div>";
    let result = NhtmlCompiler::compile(source);
    assert!(result.html.contains("Simple content"), "Le contenu doit être conservé");
    assert!(result.bind_packets.is_empty(), "Aucun BIND sur un élément sans n-id");
}

#[test]
fn test_compiler_basic_nid() {
    let source = "<div n-id='compteur' n-click='increment'>0</div>";
    let result = NhtmlCompiler::compile(source);
    assert!(result.html.contains("n-id=\"compteur\""), "Le n-id doit être présent dans le HTML");
    assert!(!result.bind_packets.is_empty(), "Un paquet BIND doit être généré");
    assert_eq!(result.bind_packets[0][0], 0x04, "Le paquet doit être de type BIND (0x04)");
}

#[test]
fn test_compiler_max_depth_does_not_panic() {
    // Génère un HTML imbriqué à 100 niveaux (sécuritaire pour scraper)
    let mut deep = String::from("<div>");
    for _ in 0..100 {
        deep.push_str("<div>");
    }
    deep.push_str("content");
    for _ in 0..101 {
        deep.push_str("</div>");
    }
    // Ce test doit se terminer sans panic
    let result = NhtmlCompiler::compile(&deep);
    assert!(!result.html.is_empty());
}

#[test]
fn test_compiler_xss_escaping_in_attributes() {
    let source = r#"<div n-id="test" title="&lt;script&gt;">Safe</div>"#;
    let result = NhtmlCompiler::compile(source);
    assert!(!result.html.contains("<script>"), "Les balises script ne doivent pas apparaître brutes");
}

// ─── Tests : B-TREE Builder (Troncature sécurisée) ────────────────────────────

#[test]
fn test_btree_long_tag_does_not_overflow() {
    // Un tag très long doit être tronqué proprement (pas de panic)
    let long_tag = "a".repeat(300);
    let source = format!("<{} n-id='test'>content</{}>", long_tag, long_tag);
    // Ne doit pas paniquer
    let _ = NhtmlCompiler::compile(&source);
}

#[test]
fn test_btree_serialization_integrity() {
    let source = "<div n-id='root'><span n-id='child'>text</span></div>";
    let result = NhtmlCompiler::compile(source);
    // Les bytes du B-TREE ne doivent pas être vides
    assert!(!result.btree_bytes.is_empty(), "Le B-TREE binaire doit être rempli");
}

// ─── Tests : Proto (Paquets binaires) ─────────────────────────────────────────

#[test]
fn test_proto_hello_packet_structure() {
    use nhtml_gateway::proto;
    let secret = b"01234567890123456789012345678901"; // 32 bytes
    let pkt = proto::hello("session-abc", secret, 0);
    assert_eq!(pkt[0], 0x01, "OpCode HELLO doit être 0x01");
    assert!(pkt.len() > 40, "Le paquet HELLO doit inclure le secret (>40 bytes)");
}

#[test]
fn test_proto_patch_set_text() {
    use nhtml_gateway::proto::{patch, PatchOp};
    let pkt = patch(&[PatchOp::set_text(42, 1, "hello world")]);
    assert_eq!(pkt[0], 0x03, "OpCode PATCH doit être 0x03");
}

#[test]
fn test_proto_bind_packet() {
    use nhtml_gateway::proto::{bind, BindParams};
    let pkt = bind(BindParams {
        node_id: 1,
        nid: "test",
        selector: "[n-id=test]",
        listen_mask: 0x01,
        behavior_flags: 0,
        debounce_100ms: 0,
        handler: "on_click",
        n_model: "",
        n_text: "",
        local_actions: vec![],
    });
    assert_eq!(pkt[0], 0x04, "OpCode BIND doit être 0x04");
    assert!(pkt.len() > 10, "Le paquet BIND doit avoir une longueur suffisante");
}
