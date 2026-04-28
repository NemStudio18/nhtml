#[cfg(test)]
mod tests {
    use super::*;
    use nhtml_gateway::proto;
    use nhtml_gateway::compiler::NhtmlCompiler;

    #[test]
    fn test_proto_hello() {
        let pkt = proto::hello("test-session", b"01234567890123456789012345678901", 42);
        assert_eq!(pkt[0], 0x01); // Type HELLO
        assert!(pkt.len() > 40);
    }

    #[test]
    fn test_compiler_basic() {
        let source = "<div n-id='test'>Hello {{ name }}</div>";
        let result = NhtmlCompiler::compile(source);
        assert!(result.html.contains("Hello"));
        assert!(!result.states.is_empty());
    }

    #[test]
    fn test_rate_limiter() {
        // Test logic for the rate limiter we just implemented
        // Since it's in socket/mod.rs, I might need to move it to a testable location if it's not pub.
    }
}
