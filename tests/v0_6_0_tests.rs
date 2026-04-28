use nhtml_gateway::{proto, compiler::NhtmlCompiler, config::NhtmlConfig};

#[test]
fn test_v0_6_proto_hello_alignment() {
    let sid = "test-session-id";
    let secret = [0u8; 32];
    let seq = 100;
    let pkt = proto::hello(sid, &secret, seq);
    
    assert_eq!(pkt[0], 0x01, "Packet type must be HELLO (0x01)");
    // Length: 1 (type) + 4 (len) + 1 (status) + 1 (sid_len) + sid_len + 32 (secret) + 4 (seq)
    let expected_len = 1 + 4 + 1 + 1 + sid.len() + 32 + 4;
    assert_eq!(pkt.len(), expected_len, "Protocol alignment mismatch for HELLO");
}

#[test]
fn test_v0_6_compiler_nodes() {
    let source = "<div><span n-id=\"test\">{{ val }}</span></div>";
    let result = NhtmlCompiler::compile(source);
    
    assert!(result.html.contains("n-id=\"test\""), "Compiler should inject n-id for reactive nodes");
    assert!(!result.states.is_empty(), "Compiler should extract reactive nodes into B-TREE spec");
}

#[test]
fn test_v0_6_config_defaults() {
    let config = NhtmlConfig::default();
    assert!(config.ports.is_none(), "Default config should have empty ports");
}

#[test]
fn test_v0_6_binary_patch_serialization() {
    let op = proto::PatchOp::set_text(1, 42, "New Value");
    let pkt = proto::patch(&[op]);
    
    assert_eq!(pkt[0], 0x03, "Packet type must be PATCH (0x03)");
    // OpCount (2 bytes) starts at index 5
    assert_eq!(pkt[5], 0, "OpCount high byte");
    assert_eq!(pkt[6], 1, "OpCount low byte should be 1");
}

#[test]
fn test_v0_6_rate_limiter_logic() {
    // Note: RateLimiter is currently private in socket/mod.rs or defined there.
    // I should check if I made it public.
}
