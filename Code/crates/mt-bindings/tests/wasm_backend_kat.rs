//! Cross-backend KAT: pure-Rust ml-dsa (веб/wasm-бэкенд) и нативный OpenSSL
//! (mt-crypto) обязаны выдавать байт-идентичные ключи, account_id и подписи.
//! Ловит любое расхождение бэкендов на сборке.
#![cfg(not(target_arch = "wasm32"))]

use ml_dsa::{
    EncodedVerifyingKey, Keypair, MlDsa65, Signature as RcSig, Signer, SigningKey, Verifier,
    VerifyingKey, B32,
};
use mt_mnemonic::{entropy_to_mnemonic, mldsa_seed_for_role, mnemonic_to_master_seed};
use sha2::{Digest, Sha256};

fn zero_acc_seed() -> [u8; 32] {
    let mnemonic = entropy_to_mnemonic(&[0u8; 32]);
    let master = mnemonic_to_master_seed(&mnemonic).unwrap();
    mldsa_seed_for_role(&master, mt_codec::domain::ACCOUNT_KEY)
}

#[test]
fn ml_dsa_wasm_backend_matches_openssl_kat() {
    let acc_seed = zero_acc_seed();
    let seed = B32::try_from(&acc_seed[..]).unwrap();
    let sk = SigningKey::<MlDsa65>::from_seed(&seed);
    let pk = sk.verifying_key().encode();
    let mut h = Sha256::new();
    h.update(mt_codec::domain::ACCOUNT);
    h.update([0u8]);
    h.update([0x01u8, 0x00u8]);
    h.update(pk.as_slice());
    let id = hex::encode(h.finalize());
    assert_eq!(
        id,
        "9f199584ed120b987b617ba5bff829e176f23e5465dd70cfac5c141dfb131a21"
    );
}

/// Кросс-бэкенд: OpenSSL (mt-crypto) ↔ pure-Rust (ml-dsa) на одном ξ:
/// (1) публичные ключи байт-идентичны; (2) подписи байт-идентичны
/// (ML-DSA-65 детерминирован, пустой контекст); (3) подпись каждого бэкенда
/// верифицируется другим.
#[test]
fn cross_backend_sign_verify() {
    let acc_seed = zero_acc_seed();
    let msg = b"montana cross-backend kat";

    // OpenSSL backend
    let (pk_o, sk_o) = mt_crypto::keypair_from_seed(&acc_seed).unwrap();
    let sig_o = mt_crypto::sign(&sk_o, msg).unwrap();

    // pure-Rust backend
    let seed = B32::try_from(&acc_seed[..]).unwrap();
    let sk_r = SigningKey::<MlDsa65>::from_seed(&seed);
    let pk_r = sk_r.verifying_key();
    let sig_r: RcSig<MlDsa65> = sk_r.sign(msg);
    let sig_r_bytes = sig_r.encode();

    // (1) pubkeys identical
    assert_eq!(&pk_o.as_bytes()[..], pk_r.encode().as_slice());
    // (2) deterministic signatures identical
    assert_eq!(&sig_o.as_bytes()[..], sig_r_bytes.as_slice());

    // (3a) ml-dsa signature verifies under OpenSSL
    let sig_r_as_o = mt_crypto::Signature::from_slice(sig_r_bytes.as_slice()).unwrap();
    assert!(mt_crypto::verify(&pk_o, msg, &sig_r_as_o));

    // (3b) OpenSSL signature verifies under ml-dsa
    let enc = EncodedVerifyingKey::<MlDsa65>::try_from(&pk_o.as_bytes()[..]).unwrap();
    let vk_r = VerifyingKey::<MlDsa65>::decode(&enc);
    let sig_o_as_r = RcSig::<MlDsa65>::try_from(sig_o.as_bytes().as_slice()).unwrap();
    assert!(vk_r.verify(msg, &sig_o_as_r).is_ok());
}

/// Кросс-бэкенд ML-KEM + подпись identity-KEM: app_kem_pub из сида байт-идентичен
/// (OpenSSL vs ml-kem), app_kem_sig (ML-DSA над "mt-idkem"||0x00||app_kem_pub)
/// валидна. Печатает SHA-256-отпечатки для binding-векторов Этапа 3.
#[test]
fn cross_backend_app_kem_and_idkem_sig() {
    use ml_kem::{EncodedSizeUser, KemCore, MlKem768, B32 as KemB32};

    let mnemonic = entropy_to_mnemonic(&[0u8; 32]);
    let master = mnemonic_to_master_seed(&mnemonic).unwrap();
    let app_kem_seed =
        mt_mnemonic::mlkem_seed_for_role(&master, mt_codec::domain::APP_ENCRYPTION_KEY);

    // OpenSSL ML-KEM
    let (pk_o, _sk_o) = mt_crypto::keypair_from_seed_mlkem(&app_kem_seed).unwrap();
    // pure-Rust ML-KEM (d = first 32, z = last 32)
    let d = KemB32::try_from(&app_kem_seed[..32]).unwrap();
    let z = KemB32::try_from(&app_kem_seed[32..]).unwrap();
    let (_dk, ek) = MlKem768::generate_deterministic(&d, &z);
    // (1) cross-backend app_kem_pub identical
    assert_eq!(&pk_o.as_bytes()[..], ek.as_bytes().as_slice());

    let app_kem_pub = pk_o.as_bytes();

    // app_kem_sig = ML-DSA sign(account_key, "mt-idkem" || 0x00 || app_kem_pub)
    let acc_seed = zero_acc_seed();
    let (pk_acc, sk_acc) = mt_crypto::keypair_from_seed(&acc_seed).unwrap();
    let mut msg = Vec::with_capacity(8 + 1 + 1184);
    msg.extend_from_slice(b"mt-idkem");
    msg.push(0u8);
    msg.extend_from_slice(app_kem_pub);
    let sig = mt_crypto::sign(&sk_acc, &msg).unwrap();
    // (2) app_kem_sig verifies under account_key
    assert!(mt_crypto::verify(&pk_acc, &msg, &sig));

    // baked binding-векторы Этапа 3 (нулевая мнемоника)
    assert_eq!(
        hex::encode(Sha256::digest(app_kem_pub)),
        "b827d37b2b225907c835f25a8652c215af69f8f52bd6a7ef0ae31955d63fd1c4"
    );
    assert_eq!(
        hex::encode(Sha256::digest(sig.as_bytes())),
        "316e908176df3d7e17b5a4cec8d0292ab2f0bdeefa3f51da3eb2bf57df80d595"
    );
}

// ---- Этап 4: PQXDH (чистый ML-KEM-768) ----

fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut hi = Sha256::new();
    hi.update(ipad);
    hi.update(msg);
    let inner = hi.finalize();
    let mut ho = Sha256::new();
    ho.update(opad);
    ho.update(inner);
    ho.finalize().into()
}

fn hkdf_sha256(salt: &[u8], ikm: &[u8], info: &[u8], l: usize) -> Vec<u8> {
    let prk = hmac_sha256(salt, ikm);
    let mut okm = Vec::new();
    let mut t: Vec<u8> = Vec::new();
    let mut i = 1u8;
    while okm.len() < l {
        let mut m = t.clone();
        m.extend_from_slice(info);
        m.push(i);
        t = hmac_sha256(&prk, &m).to_vec();
        okm.extend_from_slice(&t);
        i += 1;
    }
    okm.truncate(l);
    okm
}

/// Детерминированное ключевое расписание PQXDH (Этап 4, Шаг 3). Фиксированные
/// секреты + transcript_hash → запечённые initial_root_key / chain (с одноразовым
/// и без). Чистый HKDF-SHA-256 → кросс-платформенно идентично.
#[test]
fn pqxdh_key_schedule_kat() {
    let ss_id = [0x11u8; 32];
    let ss_spk = [0x22u8; 32];
    let ss_opk = [0x33u8; 32];
    let transcript_hash = [0xAAu8; 32];
    let salt = [0u8; 32];
    let mut info = b"mt-pqxdh-root".to_vec();
    info.push(0u8);
    info.extend_from_slice(&transcript_hash);

    // с одноразовым: IKM = ss_id || ss_spk || ss_opk
    let mut ikm = Vec::new();
    ikm.extend_from_slice(&ss_id);
    ikm.extend_from_slice(&ss_spk);
    ikm.extend_from_slice(&ss_opk);
    let okm = hkdf_sha256(&salt, &ikm, &info, 96);
    assert_eq!(
        hex::encode(&okm[..32]),
        "d1d0a8699658a49099eddf5eafa58cf9da1d8ff02ce00f7218245b3bee0efcd1"
    );
    assert_eq!(
        hex::encode(&okm[32..64]),
        "082046319cc79abbfa129a7699607dd55fe989ca9f1822ab5af53692788a27b2"
    );
    assert_eq!(
        hex::encode(&okm[64..96]),
        "872152f9fcef01639bda5890534901b1ed2c206334b64eeb46c62532ffeed5b9"
    );
    let mut ci = b"mt-pqxdh-confirm".to_vec();
    ci.push(0u8);
    ci.extend_from_slice(&transcript_hash);
    assert_eq!(
        hex::encode(hmac_sha256(&okm[64..96], &ci)),
        "6f5d00d0a49c7a231819863706eb93bc859071ee2b7919e9e0db5c58af538dbf"
    );

    // без одноразового: IKM = ss_id || ss_spk
    let mut ikm2 = Vec::new();
    ikm2.extend_from_slice(&ss_id);
    ikm2.extend_from_slice(&ss_spk);
    let okm2 = hkdf_sha256(&salt, &ikm2, &info, 96);
    assert_eq!(
        hex::encode(&okm2[..32]),
        "38fa29cc640c4a87e554ece7cb1168bf3d18bd0e4b6ee5683336091c433ca4ca"
    );
    assert_eq!(
        hex::encode(&okm2[32..64]),
        "6697d2bb86b5306ff82a86e9213655328bde8b3056226f5d3b1c89b769a76098"
    );
    assert_eq!(
        hex::encode(&okm2[64..96]),
        "19defc490566c6523a96b36610ade231fb73ca9418eeaba9d6fa724bf7ff375b"
    );
    assert_eq!(
        hex::encode(hmac_sha256(&okm2[64..96], &ci)),
        "441e93d5283d8af4d053a16a4a3601342fbae0550c501e700d9062ce5d98bf56"
    );
}

/// Согласие сторон PQXDH (Этап 4). Алиса инкапсулирует к трём реальным ML-KEM
/// ключам Боба (OpenSSL), Боб декапсулирует — общие секреты и выведенный корень
/// совпадают байт-в-байт. Проверяет весь поток установления сессии.
#[test]
fn pqxdh_agreement() {
    // ключи Боба
    let mnemonic = entropy_to_mnemonic(&[0u8; 32]);
    let master = mnemonic_to_master_seed(&mnemonic).unwrap();
    let app_seed = mt_mnemonic::mlkem_seed_for_role(&master, mt_codec::domain::APP_ENCRYPTION_KEY);
    let (app_pk, app_sk) = mt_crypto::keypair_from_seed_mlkem(&app_seed).unwrap();
    let (spk_pk, spk_sk) = mt_crypto::keypair_from_seed_mlkem(&[0x55u8; 64]).unwrap();
    let (opk_pk, opk_sk) = mt_crypto::keypair_from_seed_mlkem(&[0x66u8; 64]).unwrap();

    // Алиса инкапсулирует
    let (ct_id, ss_id_a) = mt_crypto::mlkem_encapsulate(&app_pk).unwrap();
    let (ct_spk, ss_spk_a) = mt_crypto::mlkem_encapsulate(&spk_pk).unwrap();
    let (ct_opk, ss_opk_a) = mt_crypto::mlkem_encapsulate(&opk_pk).unwrap();

    // Боб декапсулирует
    let ss_id_b = mt_crypto::mlkem_decapsulate(&app_sk, &ct_id).unwrap();
    let ss_spk_b = mt_crypto::mlkem_decapsulate(&spk_sk, &ct_spk).unwrap();
    let ss_opk_b = mt_crypto::mlkem_decapsulate(&opk_sk, &ct_opk).unwrap();

    assert_eq!(ss_id_a.as_bytes(), ss_id_b.as_bytes());
    assert_eq!(ss_spk_a.as_bytes(), ss_spk_b.as_bytes());
    assert_eq!(ss_opk_a.as_bytes(), ss_opk_b.as_bytes());

    // обе стороны: одно и то же ключевое расписание → равный корень
    let salt = [0u8; 32];
    let mut info = b"mt-pqxdh-root".to_vec();
    info.push(0u8);
    info.extend_from_slice(&[0xCCu8; 32]); // фиксированный transcript_hash (одинаков у обеих сторон)

    let ikm = |a: &[u8; 32], b: &[u8; 32], c: &[u8; 32]| {
        let mut v = Vec::new();
        v.extend_from_slice(a);
        v.extend_from_slice(b);
        v.extend_from_slice(c);
        v
    };
    let root_a = hkdf_sha256(
        &salt,
        &ikm(ss_id_a.as_bytes(), ss_spk_a.as_bytes(), ss_opk_a.as_bytes()),
        &info,
        64,
    );
    let root_b = hkdf_sha256(
        &salt,
        &ikm(ss_id_b.as_bytes(), ss_spk_b.as_bytes(), ss_opk_b.as_bytes()),
        &info,
        64,
    );
    assert_eq!(root_a, root_b);
    assert_eq!(root_a.len(), 64);
}

// ---- Этап 5: двойной храповик ----

fn kdf_ck(ck: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    (hmac_sha256(ck, &[0x01]), hmac_sha256(ck, &[0x02]))
}

#[test]
fn ratchet_aead_kat() {
    let mk = kdf_ck(&[0x42u8; 32]).0;
    let (enc_key, nonce) = msg_key(&mk);
    assert_eq!(
        hex::encode(enc_key),
        "7bb31482d13db3ad12d8529dc53aa5ba4f47b490b29f13fa275d6f56de4e8ed4"
    );
    assert_eq!(hex::encode(nonce), "00f4b713e2453c6ace58189c");
    let mut ad = vec![0xAAu8; 32];
    ad.extend_from_slice(&0u32.to_le_bytes());
    ad.extend_from_slice(&0u32.to_le_bytes());
    ad.extend_from_slice(&[0x07u8; 1184]);
    let body = seal(&enc_key, &nonce, b"montana", &ad);
    assert_eq!(
        hex::encode(&body),
        "5f43ddbc831a09fab69467ec81e97c2b10e2ba06b1f287"
    );
    assert_eq!(open(&enc_key, &nonce, &body, &ad).unwrap(), b"montana");
}

fn kdf_rk(rk: &[u8; 32], ss: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let okm = hkdf_sha256(rk, ss, b"mt-ratchet-rk", 64);
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    a.copy_from_slice(&okm[..32]);
    b.copy_from_slice(&okm[32..]);
    (a, b)
}

fn msg_key(mk: &[u8; 32]) -> ([u8; 32], [u8; 12]) {
    let okm = hkdf_sha256(&[0u8; 32], mk, b"mt-ratchet-msg", 44);
    let mut k = [0u8; 32];
    let mut n = [0u8; 12];
    k.copy_from_slice(&okm[..32]);
    n.copy_from_slice(&okm[32..44]);
    (k, n)
}

fn ad_bytes(sid: &[u8; 32], pn: u32, ns: u32, rpub: &[u8; 1184]) -> Vec<u8> {
    let mut ad = Vec::with_capacity(32 + 8 + 1184);
    ad.extend_from_slice(sid);
    ad.extend_from_slice(&pn.to_le_bytes());
    ad.extend_from_slice(&ns.to_le_bytes());
    ad.extend_from_slice(rpub);
    ad
}

fn seal(k: &[u8; 32], n: &[u8; 12], pt: &[u8], ad: &[u8]) -> Vec<u8> {
    use chacha20poly1305::aead::{Aead, Payload};
    use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
    ChaCha20Poly1305::new_from_slice(k)
        .unwrap()
        .encrypt(Nonce::from_slice(n), Payload { msg: pt, aad: ad })
        .unwrap()
}

fn open(k: &[u8; 32], n: &[u8; 12], ctb: &[u8], ad: &[u8]) -> Option<Vec<u8>> {
    use chacha20poly1305::aead::{Aead, Payload};
    use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
    ChaCha20Poly1305::new_from_slice(k)
        .unwrap()
        .decrypt(Nonce::from_slice(n), Payload { msg: ctb, aad: ad })
        .ok()
}

#[test]
fn ratchet_kdf_kat() {
    let (mk, ckn) = kdf_ck(&[0x42u8; 32]);
    assert_eq!(
        hex::encode(mk),
        "0b175bca3524cc7301c33946d7e00d3f008cb14632b72855b3442a7365403893"
    );
    assert_eq!(
        hex::encode(ckn),
        "4fa923f5d122080142716bf80fec4930203815c6b10199d1a871e09fe0a3c720"
    );
    let (rkn, ck) = kdf_rk(&[0x01u8; 32], &[0x02u8; 32]);
    assert_eq!(
        hex::encode(rkn),
        "37e663dfa28c8d5f87228aec91eb191a0536f5348254bc47a0617e3a0f4af9d5"
    );
    assert_eq!(
        hex::encode(ck),
        "1b0350c65ad26f14bb52e21d8b3f73c778b17f4efa7d9d096f7d18e5ef4fb3bb"
    );
}

type KemKp = (mt_crypto::MlkemPublicKey, mt_crypto::MlkemSecretKey);
type SkipEntry = (([u8; 1184], u32), [u8; 32]);

struct Msg {
    ratchet_pub: [u8; 1184],
    ct: Option<[u8; 1088]>,
    pn: u32,
    ns: u32,
    body: Vec<u8>,
}

struct Rat {
    sid: [u8; 32],
    rk: [u8; 32],
    dhs: KemKp,
    dhr: Option<mt_crypto::MlkemPublicKey>,
    cks: Option<[u8; 32]>,
    ckr: Option<[u8; 32]>,
    send_ct: Option<[u8; 1088]>,
    ns: u32,
    nr: u32,
    pn: u32,
    skipped: Vec<SkipEntry>,
    pending: bool,
    seed_base: u8,
    seed_ctr: u8,
}

impl Rat {
    fn new_kp(&mut self) -> KemKp {
        let mut seed = [self.seed_base; 64];
        seed[0] = self.seed_ctr;
        self.seed_ctr = self.seed_ctr.wrapping_add(1);
        mt_crypto::keypair_from_seed_mlkem(&seed).unwrap()
    }
}

fn encrypt(r: &mut Rat, pt: &[u8]) -> Msg {
    if r.pending {
        r.pn = r.ns;
        r.ns = 0;
        let (pk, sk) = r.new_kp();
        let (ct, ss) = mt_crypto::mlkem_encapsulate(r.dhr.as_ref().unwrap()).unwrap();
        let (rk2, ck) = kdf_rk(&r.rk, ss.as_bytes());
        r.rk = rk2;
        r.cks = Some(ck);
        r.send_ct = Some(*ct.as_bytes());
        r.dhs = (pk, sk);
        r.pending = false;
    }
    let (mk, ck2) = kdf_ck(&r.cks.unwrap());
    r.cks = Some(ck2);
    let (ek, no) = msg_key(&mk);
    let rpub = *r.dhs.0.as_bytes();
    let ad = ad_bytes(&r.sid, r.pn, r.ns, &rpub);
    let body = seal(&ek, &no, pt, &ad);
    let m = Msg {
        ratchet_pub: rpub,
        ct: r.send_ct,
        pn: r.pn,
        ns: r.ns,
        body,
    };
    r.ns += 1;
    m
}

fn decrypt(r: &mut Rat, m: &Msg) -> Option<Vec<u8>> {
    if let Some(pos) = r
        .skipped
        .iter()
        .position(|((rp, n), _)| *rp == m.ratchet_pub && *n == m.ns)
    {
        let mk = r.skipped.remove(pos).1;
        let (ek, no) = msg_key(&mk);
        let ad = ad_bytes(&r.sid, m.pn, m.ns, &m.ratchet_pub);
        return open(&ek, &no, &m.body, &ad);
    }
    let is_step = match &r.dhr {
        Some(d) => d.as_bytes() != &m.ratchet_pub,
        None => true,
    };
    let mut rk = r.rk;
    let mut ckr = r.ckr;
    let mut nr = r.nr;
    let mut dhr_bytes: Option<[u8; 1184]> = r.dhr.as_ref().map(|d| *d.as_bytes());
    let mut new_skipped: Vec<SkipEntry> = Vec::new();
    if is_step {
        let ct = m.ct?;
        if let (Some(cur), Some(mut c)) = (dhr_bytes, ckr) {
            while nr < m.pn {
                let (mk, cn) = kdf_ck(&c);
                new_skipped.push(((cur, nr), mk));
                c = cn;
                nr += 1;
            }
        }
        let ss =
            mt_crypto::mlkem_decapsulate(&r.dhs.1, &mt_crypto::MlkemCiphertext::from_array(ct))
                .unwrap();
        let (rk2, ck) = kdf_rk(&rk, ss.as_bytes());
        rk = rk2;
        ckr = Some(ck);
        dhr_bytes = Some(m.ratchet_pub);
        nr = 0;
    }
    let cur = dhr_bytes.unwrap();
    let mut c = ckr.unwrap();
    while nr < m.ns {
        let (mk, cn) = kdf_ck(&c);
        new_skipped.push(((cur, nr), mk));
        c = cn;
        nr += 1;
    }
    let (mk, cn) = kdf_ck(&c);
    c = cn;
    nr += 1;
    let (ek, no) = msg_key(&mk);
    let ad = ad_bytes(&r.sid, m.pn, m.ns, &m.ratchet_pub);
    let pt = open(&ek, &no, &m.body, &ad)?;
    r.rk = rk;
    r.ckr = Some(c);
    r.nr = nr;
    if is_step {
        r.dhr = Some(mt_crypto::MlkemPublicKey::from_array(m.ratchet_pub));
        r.pending = true;
    }
    r.skipped.extend(new_skipped);
    Some(pt)
}

/// Полный поток двойного храповика: два симметричных сообщения, ответ с KEM-шагом
/// (реальный ML-KEM Encaps/Decaps), доставка вне порядка через MKSKIPPED.
#[test]
fn ratchet_session_roundtrip() {
    let sid = [0xABu8; 32];
    let rk0 = [0x01u8; 32];
    let init_chain = [0x02u8; 32];
    let (eph_pub, eph_sk) = mt_crypto::keypair_from_seed_mlkem(&[0x99u8; 64]).unwrap();
    let (spk_pub, spk_sk) = mt_crypto::keypair_from_seed_mlkem(&[0x55u8; 64]).unwrap();
    let eph_pub_bytes = *eph_pub.as_bytes();
    let spk_pub_bytes = *spk_pub.as_bytes();

    let mut alice = Rat {
        sid,
        rk: rk0,
        dhs: (eph_pub, eph_sk),
        dhr: Some(mt_crypto::MlkemPublicKey::from_array(spk_pub_bytes)),
        cks: Some(init_chain),
        ckr: None,
        send_ct: None,
        ns: 0,
        nr: 0,
        pn: 0,
        skipped: Vec::new(),
        pending: false,
        seed_base: 0xA0,
        seed_ctr: 0,
    };
    let mut bob = Rat {
        sid,
        rk: rk0,
        dhs: (spk_pub, spk_sk),
        dhr: Some(mt_crypto::MlkemPublicKey::from_array(eph_pub_bytes)),
        cks: None,
        ckr: Some(init_chain),
        send_ct: None,
        ns: 0,
        nr: 0,
        pn: 0,
        skipped: Vec::new(),
        pending: true,
        seed_base: 0xB0,
        seed_ctr: 0,
    };

    // симметричный храповик Алиса -> Боб
    let m1 = encrypt(&mut alice, b"m1");
    assert_eq!(decrypt(&mut bob, &m1).unwrap(), b"m1");
    let m2 = encrypt(&mut alice, b"m2");
    assert_eq!(decrypt(&mut bob, &m2).unwrap(), b"m2");

    // KEM-шаг: ответ Боба
    let r1 = encrypt(&mut bob, b"r1");
    assert!(r1.ct.is_some());
    assert_eq!(decrypt(&mut alice, &r1).unwrap(), b"r1");

    // Алиса делает свой KEM-шаг, два сообщения, доставка вне порядка (m4 до m3)
    let m3 = encrypt(&mut alice, b"m3");
    let m4 = encrypt(&mut alice, b"m4");
    assert!(m3.ct.is_some() && m4.ct.is_some()); // ct повторяется на цепочке
    assert_eq!(decrypt(&mut bob, &m4).unwrap(), b"m4");
    assert_eq!(decrypt(&mut bob, &m3).unwrap(), b"m3"); // из MKSKIPPED
}
