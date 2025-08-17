use blake3::Hasher;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use uuid::Uuid;

/// 결정적 RNG 생성기: global_seed와 네임스페이스 문자열을 해시하여 32바이트 시드를 생성
pub fn rng_for(global_seed: u64, namespace: &str) -> ChaCha20Rng {
    let mut h = Hasher::new();
    h.update(&global_seed.to_le_bytes());
    h.update(namespace.as_bytes());
    let bytes = *h.finalize().as_bytes();
    ChaCha20Rng::from_seed(bytes)
}

/// 결정적 UUID 생성: (global_seed, namespace, index)를 해시해 16바이트로 잘라 UUID로 변환
pub fn uuid_for(global_seed: u64, namespace: &str, index: u64) -> Uuid {
    let mut h = Hasher::new();
    h.update(&global_seed.to_le_bytes());
    h.update(namespace.as_bytes());
    h.update(&index.to_le_bytes());
    let hash = h.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[0..16]);
    Uuid::from_bytes(bytes)
}
