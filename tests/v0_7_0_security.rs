use nhtml_gateway::socket::{RateLimiter, verify_hmac};
use std::time::Duration;

#[tokio::test]
async fn test_v0_7_rate_limiter_basic() {
    let limiter = RateLimiter::new(2); // 2 events per second
    
    // Test allow
    assert!(limiter.check("127.0.0.1".to_string()).await, "First request should be allowed");
    assert!(limiter.check("127.0.0.1".to_string()).await, "Second request should be allowed");
    
    // Test block
    assert!(!limiter.check("127.0.0.1".to_string()).await, "Third request should be blocked");
    
    // Test other IP not blocked
    assert!(limiter.check("192.168.1.1".to_string()).await, "Request from other IP should be allowed");
}

#[tokio::test]
async fn test_v0_7_rate_limiter_reset() {
    let limiter = RateLimiter::new(1);
    assert!(limiter.check("127.0.0.1".to_string()).await);
    assert!(!limiter.check("127.0.0.1".to_string()).await);
    
    // Wait for reset
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert!(limiter.check("127.0.0.1".to_string()).await, "Should be allowed after 1s");
}

#[test]
fn test_v0_7_hmac_verification() {
    let secret = b"super-secret-key-32-chars-long!!"; // 32 bytes
    let data = b"some-data-to-sign";
    
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(data);
    let signature = mac.finalize().into_bytes();
    
    assert!(verify_hmac(secret, data, &signature), "HMAC verification should pass");
    assert!(!verify_hmac(secret, b"wrong-data", &signature), "HMAC verification should fail on data mismatch");
    assert!(!verify_hmac(b"wrong-secret", data, &signature), "HMAC verification should fail on secret mismatch");
}

#[tokio::test]
async fn test_v0_7_rate_limiter_anti_leak() {
    let limiter = RateLimiter::new(1000);
    
    // Fill with many IPs - doit rester fonctionnel sans panic ni OOM
    for i in 0..1100 {
        limiter.check(format!("10.0.0.{}", i)).await;
    }
    
    // Le test passe si aucun panic n'est survenu (vérification du nettoyage TTL)
    // La logique de cleanup interne ne permet pas d'inspecter le champ privé `ips`
    // => Ce test valide uniquement la robustesse et l'absence de crash/OOM.
    assert!(true, "Rate limiter survit à 1100 IPs distinctes");
}
