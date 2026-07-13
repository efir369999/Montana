//! DHT-рандеву (Montana P2P Network, Этап 4): byte-exact ядро поиска эндпоинта через
//! Mainline BitTorrent DHT (BEP44 mutable). `dht_key` — ed25519 admission-token ([P2P-5]/
//! A-2 [I-16]): его квантовая подделка даёт лишь DoS-редирект, НЕ вскрытие; overlay/контент/
//! личность — PQ (Этап 1). Целостность доставки ловит E2E-квитанция (R1), не подпись рандеву.
//! Mainline DHT-транспорт (BEP5 put/get) — отдельный сетевой слой поверх этого ядра.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use mt_codec::{write_bytes, write_u8, CanonicalEncode};
use sha1::{Digest, Sha1};
use thiserror::Error;

pub const DHT_SEED_LEN: usize = 32;
pub const DK_LEN: usize = 32; // ed25519 pubkey
pub const SALT_LEN: usize = 20;
pub const TARGET_LEN: usize = 20; // SHA1
pub const OVERLAY_ADDR_LEN: usize = 32;
pub const PQ_HINT_LEN: usize = 32;
/// BEP44 verdict: тело `v` ≤ 1000 B.
pub const MAX_RECORD_BYTES: usize = 1000;

pub mod dht;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RvError {
    #[error("truncated")]
    Truncated,
    #[error("bad endpoint kind {0:#04x}")]
    BadKind(u8),
    #[error("record exceeds BEP44 1000 B ({0})")]
    TooLarge(usize),
    #[error("length mismatch")]
    LengthMismatch,
    #[error("dht: {0}")]
    Dht(String),
}

// --- dht_key (ed25519 admission, HKDF-ветка master_seed) ---

/// dht_seed = HKDF-Expand(master_seed, info="mt-dht-key", L=32). Секрет не покидает лист.
pub fn derive_dht_seed(master_seed: &[u8]) -> [u8; DHT_SEED_LEN] {
    let okm = mt_mnemonic::hkdf_expand(master_seed, mt_codec::domain::DHT_KEY, DHT_SEED_LEN);
    let mut s = [0u8; DHT_SEED_LEN];
    s.copy_from_slice(&okm);
    s
}

// [I-16] A-2: ed25519 — admission-token BEP44 (BitTorrent мандат, ≤1000 B не вмещает
// ML-DSA 3309 B). Компрометация КК = DoS-редирект, не breach; overlay/контент/личность — PQ.
pub fn dht_signing_key(dht_seed: &[u8; DHT_SEED_LEN]) -> SigningKey {
    SigningKey::from_bytes(dht_seed)
}

pub fn dht_pubkey(sk: &SigningKey) -> [u8; DK_LEN] {
    sk.verifying_key().to_bytes()
}

// --- BEP44 target / salt ---

/// salt = SHA-256("mt-rv-salt" ‖ 0x00 ‖ session_id ‖ epoch_index_8B_LE)[0..20]. Per-pair,
/// per-epoch — вращается, знают только двое (session_id непрозрачен сети).
pub fn derive_salt(session_id: &[u8; 32], epoch_index: u64) -> [u8; SALT_LEN] {
    let h = mt_crypto::hash(
        mt_codec::domain::RV_SALT,
        &[session_id, &epoch_index.to_le_bytes()],
    );
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&h[..SALT_LEN]);
    salt
}

/// target = SHA1(dk ‖ salt). SHA1 — мандат BEP44, не наш выбор (admission-token, не PQ).
pub fn derive_target(dk: &[u8; DK_LEN], salt: &[u8; SALT_LEN]) -> [u8; TARGET_LEN] {
    let mut h = Sha1::new();
    h.update(dk);
    h.update(salt);
    h.finalize().into()
}

// --- RendezvousRecord (тело BEP44 v, byte-exact) ---

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Endpoint {
    pub kind: u8, // 0x01=relay-circuit, 0x02=direct-v6, 0x03=direct-v4
    pub addr: Vec<u8>,
}

pub const EP_RELAY_CIRCUIT: u8 = 0x01;
pub const EP_DIRECT_V6: u8 = 0x02;
pub const EP_DIRECT_V4: u8 = 0x03;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RendezvousRecord {
    pub overlay_addr: [u8; OVERLAY_ADDR_LEN],
    pub endpoints: Vec<Endpoint>, // ep_count = len (u8)
    pub pq_hint: [u8; PQ_HINT_LEN],
    pub seq: u64,
    pub valid_until: u64,
}

impl CanonicalEncode for RendezvousRecord {
    fn encode(&self, buf: &mut Vec<u8>) {
        write_bytes(buf, &self.overlay_addr);
        write_u8(buf, self.endpoints.len() as u8);
        for ep in &self.endpoints {
            write_u8(buf, ep.kind);
            write_u8(buf, ep.addr.len() as u8);
            write_bytes(buf, &ep.addr);
        }
        write_bytes(buf, &self.pq_hint);
        write_bytes(buf, &self.seq.to_le_bytes());
        write_bytes(buf, &self.valid_until.to_le_bytes());
    }
}

impl RendezvousRecord {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();
        self.encode(&mut b);
        b
    }

    pub fn decode(input: &[u8]) -> Result<Self, RvError> {
        // overlay_addr32 ep_count1 [kind1 addr_len1 addr]* pq_hint32 seq8 valid_until8
        let min = OVERLAY_ADDR_LEN + 1 + PQ_HINT_LEN + 8 + 8;
        if input.len() < min {
            return Err(RvError::Truncated);
        }
        let mut o = 0;
        let mut overlay_addr = [0u8; OVERLAY_ADDR_LEN];
        overlay_addr.copy_from_slice(&input[o..o + OVERLAY_ADDR_LEN]);
        o += OVERLAY_ADDR_LEN;
        let ep_count = input[o] as usize;
        o += 1;
        let mut endpoints = Vec::with_capacity(ep_count);
        for _ in 0..ep_count {
            if input.len() < o + 2 {
                return Err(RvError::Truncated);
            }
            let kind = input[o];
            if !matches!(kind, EP_RELAY_CIRCUIT | EP_DIRECT_V6 | EP_DIRECT_V4) {
                return Err(RvError::BadKind(kind));
            }
            let addr_len = input[o + 1] as usize;
            o += 2;
            if input.len() < o + addr_len {
                return Err(RvError::Truncated);
            }
            endpoints.push(Endpoint {
                kind,
                addr: input[o..o + addr_len].to_vec(),
            });
            o += addr_len;
        }
        if input.len() != o + PQ_HINT_LEN + 8 + 8 {
            return Err(RvError::LengthMismatch);
        }
        let mut pq_hint = [0u8; PQ_HINT_LEN];
        pq_hint.copy_from_slice(&input[o..o + PQ_HINT_LEN]);
        o += PQ_HINT_LEN;
        let seq = u64::from_le_bytes(input[o..o + 8].try_into().map_err(|_| RvError::Truncated)?);
        o += 8;
        let valid_until =
            u64::from_le_bytes(input[o..o + 8].try_into().map_err(|_| RvError::Truncated)?);
        Ok(Self {
            overlay_addr,
            endpoints,
            pq_hint,
            seq,
            valid_until,
        })
    }
}

/// Резолв физического адреса из endpoint DHT-записи (замена захардкоженного адреса, Этап 4).
/// v4 addr = 4B ip ‖ 2B port BE; v6 = 16B ip ‖ 2B port BE. relay-circuit — не сырой SocketAddr.
pub fn resolve_endpoint(ep: &Endpoint) -> Option<std::net::SocketAddr> {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
    match ep.kind {
        EP_DIRECT_V4 if ep.addr.len() == 6 => {
            let ip = Ipv4Addr::new(ep.addr[0], ep.addr[1], ep.addr[2], ep.addr[3]);
            let port = u16::from_be_bytes([ep.addr[4], ep.addr[5]]);
            Some(SocketAddr::new(IpAddr::V4(ip), port))
        },
        EP_DIRECT_V6 if ep.addr.len() == 18 => {
            let mut o = [0u8; 16];
            o.copy_from_slice(&ep.addr[..16]);
            let port = u16::from_be_bytes([ep.addr[16], ep.addr[17]]);
            Some(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(o)), port))
        },
        _ => None, // relay-circuit / некорректная длина — резолв через оверлей-слой, не сокет
    }
}

// --- BEP44 ed25519 подпись (admission-token, [P2P-5]) ---

/// Канонический буфер подписи BEP44 mutable-with-salt: bencoded поля salt/seq/v в
/// фиксированном порядке (BEP44 стандарт). v — RendezvousRecord bytes.
fn bep44_sign_buffer(salt: &[u8], seq: u64, v: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(b"4:salt");
    b.extend_from_slice(format!("{}:", salt.len()).as_bytes());
    b.extend_from_slice(salt);
    b.extend_from_slice(b"3:seqi");
    b.extend_from_slice(seq.to_string().as_bytes());
    b.extend_from_slice(b"e1:v");
    b.extend_from_slice(format!("{}:", v.len()).as_bytes());
    b.extend_from_slice(v);
    b
}

/// Подписать рандеву-запись dht_key (ed25519 над BEP44-буфером salt‖seq‖v).
pub fn sign_record(
    sk: &SigningKey,
    salt: &[u8; SALT_LEN],
    seq: u64,
    record: &RendezvousRecord,
) -> Result<Signature, RvError> {
    let v = record.to_bytes();
    if v.len() > MAX_RECORD_BYTES {
        return Err(RvError::TooLarge(v.len()));
    }
    Ok(sk.sign(&bep44_sign_buffer(salt, seq, &v)))
}

/// Проверить рандеву-запись против dk (admission-token; НЕ гарантия PQ-целостности).
/// Отдельно подписчик обязан сверить record.overlay_addr с overlay_addr друга (Этап 1)
/// и целостность доставки — E2E-квитанцией (R1).
pub fn verify_record(
    dk: &[u8; DK_LEN],
    salt: &[u8; SALT_LEN],
    seq: u64,
    record: &RendezvousRecord,
    sig: &Signature,
) -> bool {
    let vk = match VerifyingKey::from_bytes(dk) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let v = record.to_bytes();
    vk.verify(&bep44_sign_buffer(salt, seq, &v), sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RendezvousRecord {
        RendezvousRecord {
            overlay_addr: [0xAB; 32],
            endpoints: vec![
                Endpoint {
                    kind: EP_RELAY_CIRCUIT,
                    addr: vec![0x01, 0x02, 0x03],
                },
                Endpoint {
                    kind: EP_DIRECT_V6,
                    addr: vec![0x0A; 16],
                },
            ],
            pq_hint: [0xCD; 32],
            seq: 42,
            valid_until: 1_000_000,
        }
    }

    #[test]
    fn record_roundtrip_and_reject_bad_kind() {
        let r = sample();
        assert_eq!(RendezvousRecord::decode(&r.to_bytes()).unwrap(), r);
        // мусорный kind
        let mut b = r.to_bytes();
        b[33] = 0x09; // первый endpoint kind
        assert!(matches!(
            RendezvousRecord::decode(&b),
            Err(RvError::BadKind(0x09))
        ));
    }

    #[test]
    fn dht_key_deterministic_from_master_seed() {
        let master = [0x42u8; 64];
        let s1 = derive_dht_seed(&master);
        let s2 = derive_dht_seed(&master);
        assert_eq!(s1, s2, "детерминизм из master_seed");
        let sk = dht_signing_key(&s1);
        let dk = dht_pubkey(&sk);
        // другой master → другой ключ
        let dk2 = dht_pubkey(&dht_signing_key(&derive_dht_seed(&[0x43u8; 64])));
        assert_ne!(dk, dk2);
    }

    #[test]
    fn sign_verify_admission_and_tamper_fails() {
        let master = [0x42u8; 64];
        let sk = dht_signing_key(&derive_dht_seed(&master));
        let dk = dht_pubkey(&sk);
        let salt = derive_salt(&[0x33; 32], 7);
        let r = sample();
        let sig = sign_record(&sk, &salt, r.seq, &r).unwrap();
        assert!(verify_record(&dk, &salt, r.seq, &r, &sig));
        // чужой seq — подпись не сойдётся (BEP44 anti-rollback)
        assert!(!verify_record(&dk, &salt, r.seq + 1, &r, &sig));
        // подделка записи
        let mut r2 = r.clone();
        r2.overlay_addr[0] ^= 1;
        assert!(!verify_record(&dk, &salt, r.seq, &r2, &sig));
        // чужой dk
        let dk2 = dht_pubkey(&dht_signing_key(&derive_dht_seed(&[0x99u8; 64])));
        assert!(!verify_record(&dk2, &salt, r.seq, &r, &sig));
    }

    #[test]
    fn resolve_endpoint_v4_v6() {
        let v4 = Endpoint {
            kind: EP_DIRECT_V4,
            addr: vec![203, 0, 113, 5, 0x20, 0xFC],
        };
        assert_eq!(
            resolve_endpoint(&v4).unwrap().to_string(),
            "203.0.113.5:8444"
        );
        let mut a6 = vec![0u8; 16];
        a6[15] = 1;
        a6.extend_from_slice(&8444u16.to_be_bytes());
        let v6 = Endpoint {
            kind: EP_DIRECT_V6,
            addr: a6,
        };
        assert_eq!(resolve_endpoint(&v6).unwrap().to_string(), "[::1]:8444");
        // relay-circuit не резолвится в сырой сокет
        assert!(resolve_endpoint(&Endpoint {
            kind: EP_RELAY_CIRCUIT,
            addr: vec![1, 2, 3]
        })
        .is_none());
        // некорректная длина
        assert!(resolve_endpoint(&Endpoint {
            kind: EP_DIRECT_V4,
            addr: vec![1, 2]
        })
        .is_none());
    }

    #[test]
    fn salt_target_kat_oracle() {
        // Oracle: python hashlib.sha256/sha1 (Проход 25) — byte-exact для cross-client.
        let salt = derive_salt(&[0x33; 32], 7);
        assert_eq!(
            hex::encode(salt),
            "93c7c3fbad00768cee189aedc2ef57738d55ef38"
        );
        let target = derive_target(&[0x11; 32], &salt);
        assert_eq!(
            hex::encode(target),
            "db1c2182fd6a0029df27462bf2d1cfad598567cb"
        );
    }
}
