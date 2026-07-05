//! Этап 5 — машина состояний PQXDH: сборка/разбор InitialHandshake, сторона
//! Алисы (build) и Боба (process). Стенограмма и layout — байт-точно по спеке.

use sha2::{Digest, Sha256};

use mt_crypto::{
    keypair_from_seed_mlkem, mlkem_decapsulate, mlkem_encapsulate, sign as mldsa_sign,
    verify as mldsa_verify, MlkemCiphertext, MlkemPublicKey, MlkemSecretKey, PublicKey, SecretKey,
    Signature,
};

use crate::pqxdh::{confirm_tag, derive_session_keys, SessionKeys, DOMAIN_SIG};

pub const MLDSA_PUBKEY: usize = 1952;
pub const MLDSA_SIG: usize = 3309;
pub const MLKEM_PUBKEY: usize = 1184;
pub const MLKEM_CT: usize = 1088;
const SUITE_MLDSA65_LE: [u8; 2] = [0x01, 0x00];

#[derive(Debug, PartialEq, Eq)]
pub enum E2eError {
    BadLength,
    BadVersion,
    BadOpkFlag,
    Stale,
    BadSignature,
    ConfirmMismatch,
    Crypto,
}

pub fn account_id(account_key_pub: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(mt_codec::domain::ACCOUNT);
    h.update([0u8]);
    h.update(SUITE_MLDSA65_LE);
    h.update(account_key_pub);
    h.finalize().into()
}

/// Связка Боба (публичная часть, из Этапа 4) — вход для стороны Алисы.
pub struct RecipientBundle<'a> {
    pub account_key_pub: &'a [u8; MLDSA_PUBKEY],
    pub app_kem_pub: &'a MlkemPublicKey,
    pub signed_prekey_pub: &'a MlkemPublicKey,
    pub spk_id: u32,
    pub one_time: Option<(u32, &'a MlkemPublicKey)>,
}

fn transcript_bytes(
    account_id_a: &[u8; 32],
    account_id_b: &[u8; 32],
    send_time: u64,
    eph_kem_pub_a: &[u8; MLKEM_PUBKEY],
    app_kem_pub_b: &[u8; MLKEM_PUBKEY],
    signed_prekey_pub_b: &[u8; MLKEM_PUBKEY],
    spk_id_b: u32,
    opk: Option<(u32, &[u8; MLKEM_PUBKEY])>,
    ct_id: &[u8; MLKEM_CT],
    ct_spk: &[u8; MLKEM_CT],
    ct_opk: Option<&[u8; MLKEM_CT]>,
) -> Vec<u8> {
    let mut t = Vec::new();
    t.extend_from_slice(b"mt-pqxdh-v1");
    t.push(0u8);
    t.extend_from_slice(account_id_a);
    t.extend_from_slice(account_id_b);
    t.extend_from_slice(&send_time.to_le_bytes());
    t.extend_from_slice(eph_kem_pub_a);
    t.extend_from_slice(app_kem_pub_b);
    t.extend_from_slice(signed_prekey_pub_b);
    t.extend_from_slice(&spk_id_b.to_le_bytes());
    match opk {
        Some((opk_id, opk_pub)) => {
            t.push(0x01);
            t.extend_from_slice(opk_pub);
            t.extend_from_slice(&opk_id.to_le_bytes());
        },
        None => t.push(0x00),
    }
    t.extend_from_slice(ct_id);
    t.extend_from_slice(ct_spk);
    if let Some(c) = ct_opk {
        t.extend_from_slice(c);
    }
    t
}

pub struct Handshake {
    /// Сериализованный InitialHandshake (кладётся в sealed-конверт Этапа 7).
    pub bytes: Vec<u8>,
    /// Выходы для загрузки храповика Этапа 6.
    pub session: SessionKeys,
    pub eph_kem_pub_a: [u8; MLKEM_PUBKEY],
    pub eph_kem_sk_a: MlkemSecretKey,
    pub signed_prekey_pub_b: [u8; MLKEM_PUBKEY],
    pub transcript_hash: [u8; 32],
}

/// Сторона Алисы. `eph_seed` (64B) — клиентская случайность для эфемерной пары.
#[allow(clippy::too_many_arguments)]
pub fn build_handshake(
    alice_account_pub: &[u8; MLDSA_PUBKEY],
    alice_account_sk: &SecretKey,
    bob: &RecipientBundle,
    eph_seed: &[u8; 64],
    send_time: u64,
) -> Result<Handshake, E2eError> {
    let (eph_pub, eph_sk) = keypair_from_seed_mlkem(eph_seed).map_err(|_| E2eError::Crypto)?;
    let (ct_id, ss_id) = mlkem_encapsulate(bob.app_kem_pub).map_err(|_| E2eError::Crypto)?;
    let (ct_spk, ss_spk) =
        mlkem_encapsulate(bob.signed_prekey_pub).map_err(|_| E2eError::Crypto)?;
    let opk_enc = match bob.one_time {
        Some((id, pk)) => {
            let (ct, ss) = mlkem_encapsulate(pk).map_err(|_| E2eError::Crypto)?;
            Some((id, ct, ss))
        },
        None => None,
    };

    let id_a = account_id(alice_account_pub);
    let id_b = account_id(bob.account_key_pub);
    let opk_pub_ref = bob.one_time.map(|(id, pk)| (id, pk.as_bytes()));
    let ct_opk_ref = opk_enc.as_ref().map(|(_, ct, _)| ct.as_bytes());

    let tb = transcript_bytes(
        &id_a,
        &id_b,
        send_time,
        eph_pub.as_bytes(),
        bob.app_kem_pub.as_bytes(),
        bob.signed_prekey_pub.as_bytes(),
        bob.spk_id,
        opk_pub_ref,
        ct_id.as_bytes(),
        ct_spk.as_bytes(),
        ct_opk_ref,
    );
    let transcript_hash: [u8; 32] = Sha256::digest(&tb).into();

    let mut sig_msg = DOMAIN_SIG.to_vec();
    sig_msg.push(0u8);
    sig_msg.extend_from_slice(&transcript_hash);
    let sig = mldsa_sign(alice_account_sk, &sig_msg).map_err(|_| E2eError::Crypto)?;

    let ss_opk_ref = opk_enc.as_ref().map(|(_, _, ss)| {
        let mut a = [0u8; 32];
        a.copy_from_slice(ss.as_bytes());
        a
    });
    let mut ss_id_a = [0u8; 32];
    ss_id_a.copy_from_slice(ss_id.as_bytes());
    let mut ss_spk_a = [0u8; 32];
    ss_spk_a.copy_from_slice(ss_spk.as_bytes());
    let session = derive_session_keys(&ss_id_a, &ss_spk_a, ss_opk_ref.as_ref(), &transcript_hash);
    let tag = confirm_tag(&session.confirm_key, &transcript_hash);

    // Сериализация InitialHandshake (layout спеки).
    let mut out = Vec::new();
    out.push(0x01);
    out.extend_from_slice(alice_account_pub);
    out.extend_from_slice(eph_pub.as_bytes());
    out.extend_from_slice(&send_time.to_le_bytes());
    out.extend_from_slice(&bob.spk_id.to_le_bytes());
    match (bob.one_time, &opk_enc) {
        (Some((opk_id, _)), Some((_, ct_opk, _))) => {
            out.push(0x01);
            out.extend_from_slice(&opk_id.to_le_bytes());
            out.extend_from_slice(ct_id.as_bytes());
            out.extend_from_slice(ct_spk.as_bytes());
            out.extend_from_slice(ct_opk.as_bytes());
        },
        _ => {
            out.push(0x00);
            out.extend_from_slice(ct_id.as_bytes());
            out.extend_from_slice(ct_spk.as_bytes());
        },
    }
    out.extend_from_slice(sig.as_bytes());
    out.extend_from_slice(&tag);

    let mut eph_pub_arr = [0u8; MLKEM_PUBKEY];
    eph_pub_arr.copy_from_slice(eph_pub.as_bytes());
    let mut spk_pub_arr = [0u8; MLKEM_PUBKEY];
    spk_pub_arr.copy_from_slice(bob.signed_prekey_pub.as_bytes());

    Ok(Handshake {
        bytes: out,
        session,
        eph_kem_pub_a: eph_pub_arr,
        eph_kem_sk_a: eph_sk,
        signed_prekey_pub_b: spk_pub_arr,
        transcript_hash,
    })
}

pub struct Processed {
    pub session: SessionKeys,
    pub eph_kem_pub_a: [u8; MLKEM_PUBKEY],
    pub transcript_hash: [u8; 32],
    pub opk_consumed: Option<u32>,
}

/// Приватный материал Боба для обработки рукопожатия.
pub struct RecipientKeys<'a> {
    pub account_id: &'a [u8; 32],
    pub app_kem_pub: &'a [u8; MLKEM_PUBKEY],
    pub app_kem_sk: &'a MlkemSecretKey,
    pub signed_prekey_pub: &'a [u8; MLKEM_PUBKEY],
    pub signed_prekey_sk: &'a MlkemSecretKey,
    pub one_time: Option<(&'a [u8; MLKEM_PUBKEY], &'a MlkemSecretKey)>,
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut d = 0u8;
    for i in 0..a.len() {
        d |= a[i] ^ b[i];
    }
    d == 0
}

fn decaps32(sk: &MlkemSecretKey, ct: &[u8]) -> Result<[u8; 32], E2eError> {
    let c = MlkemCiphertext::from_slice(ct).ok_or(E2eError::BadLength)?;
    let ss = mlkem_decapsulate(sk, &c).map_err(|_| E2eError::Crypto)?;
    let mut out = [0u8; 32];
    out.copy_from_slice(ss.as_bytes());
    Ok(out)
}

/// Сторона Боба: разбор, проверка свежести/подписи, три декапсуляции, вывод корня,
/// сверка confirm_tag. Любой сбой → отклонение без изменения состояния.
pub fn process_handshake(
    hs: &[u8],
    bob: &RecipientKeys,
    now: u64,
    accept_skew: u64,
) -> Result<Processed, E2eError> {
    let base = 1 + MLDSA_PUBKEY + MLKEM_PUBKEY + 8 + 4;
    if hs.len() < base + 1 {
        return Err(E2eError::BadLength);
    }
    if hs[0] != 0x01 {
        return Err(E2eError::BadVersion);
    }
    let mut p = 1;
    let account_key_pub_a = &hs[p..p + MLDSA_PUBKEY];
    p += MLDSA_PUBKEY;
    let eph_kem_pub_a = &hs[p..p + MLKEM_PUBKEY];
    p += MLKEM_PUBKEY;
    let send_time = u64::from_le_bytes(hs[p..p + 8].try_into().unwrap());
    p += 8;
    let spk_id_b = u32::from_le_bytes(hs[p..p + 4].try_into().unwrap());
    p += 4;
    let opk_flag = hs[p];
    p += 1;
    if opk_flag != 0x00 && opk_flag != 0x01 {
        return Err(E2eError::BadOpkFlag);
    }
    let has_opk = opk_flag == 0x01;
    if now.abs_diff(send_time) > accept_skew {
        return Err(E2eError::Stale);
    }
    let expect = base
        + 1
        + if has_opk {
            4 + 3 * MLKEM_CT
        } else {
            2 * MLKEM_CT
        }
        + MLDSA_SIG
        + 32;
    if hs.len() != expect {
        return Err(E2eError::BadLength);
    }
    if has_opk && bob.one_time.is_none() {
        return Err(E2eError::BadOpkFlag);
    }
    let opk_id_b = if has_opk {
        let v = u32::from_le_bytes(hs[p..p + 4].try_into().unwrap());
        p += 4;
        Some(v)
    } else {
        None
    };
    let ct_id = &hs[p..p + MLKEM_CT];
    p += MLKEM_CT;
    let ct_spk = &hs[p..p + MLKEM_CT];
    p += MLKEM_CT;
    let ct_opk = if has_opk {
        let c = &hs[p..p + MLKEM_CT];
        p += MLKEM_CT;
        Some(c)
    } else {
        None
    };
    let sig = &hs[p..p + MLDSA_SIG];
    p += MLDSA_SIG;
    let tag = &hs[p..p + 32];

    let id_a = account_id(account_key_pub_a);
    let eph_arr: [u8; MLKEM_PUBKEY] = eph_kem_pub_a.try_into().unwrap();
    let opk_for_transcript = match (has_opk, opk_id_b, bob.one_time) {
        (true, Some(id), Some((pk, _))) => Some((id, pk)),
        _ => None,
    };

    let tb = transcript_bytes(
        &id_a,
        bob.account_id,
        send_time,
        &eph_arr,
        bob.app_kem_pub,
        bob.signed_prekey_pub,
        spk_id_b,
        opk_for_transcript.map(|(id, pk)| (id, pk)),
        ct_id.try_into().unwrap(),
        ct_spk.try_into().unwrap(),
        ct_opk.map(|c| c.try_into().unwrap()),
    );
    let transcript_hash: [u8; 32] = Sha256::digest(&tb).into();

    // Проверка подписи Алисы над стенограммой.
    let mut sig_msg = DOMAIN_SIG.to_vec();
    sig_msg.push(0u8);
    sig_msg.extend_from_slice(&transcript_hash);
    let pk_a = PublicKey::from_slice(account_key_pub_a).ok_or(E2eError::BadLength)?;
    let sig_a = Signature::from_slice(sig).ok_or(E2eError::BadLength)?;
    if !mldsa_verify(&pk_a, &sig_msg, &sig_a) {
        return Err(E2eError::BadSignature);
    }

    // Три декапсуляции.
    let ss_id = decaps32(bob.app_kem_sk, ct_id)?;
    let ss_spk = decaps32(bob.signed_prekey_sk, ct_spk)?;
    let ss_opk = match (ct_opk, bob.one_time) {
        (Some(c), Some((_, sk))) => Some(decaps32(sk, c)?),
        _ => None,
    };

    let session = derive_session_keys(&ss_id, &ss_spk, ss_opk.as_ref(), &transcript_hash);
    let expected_tag = confirm_tag(&session.confirm_key, &transcript_hash);
    if !ct_eq(&expected_tag, tag) {
        return Err(E2eError::ConfirmMismatch);
    }

    Ok(Processed {
        session,
        eph_kem_pub_a: eph_arr,
        transcript_hash,
        opk_consumed: opk_id_b,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mldsa(seed: &[u8; 32]) -> ([u8; MLDSA_PUBKEY], SecretKey) {
        let (pk, sk) = mt_crypto::keypair_from_seed(seed).unwrap();
        (pk.as_bytes().to_owned(), sk)
    }

    fn kem(seed: u8) -> (MlkemPublicKey, MlkemSecretKey, [u8; MLKEM_PUBKEY]) {
        let (pk, sk) = keypair_from_seed_mlkem(&[seed; 64]).unwrap();
        let arr = pk.as_bytes().to_owned();
        (pk, sk, arr)
    }

    #[test]
    fn handshake_agreement_with_opk() {
        let (app_pk, app_sk, app_arr) = kem(0x11);
        let (spk_pk, spk_sk, spk_arr) = kem(0x22);
        let (opk_pk, opk_sk, opk_arr) = kem(0x33);
        let (bob_pub, _bob_sk) = mldsa(&[0x44; 32]);
        let bob_id = account_id(&bob_pub);
        let (alice_pub, alice_sk) = mldsa(&[0x55; 32]);

        let bundle = RecipientBundle {
            account_key_pub: &bob_pub,
            app_kem_pub: &app_pk,
            signed_prekey_pub: &spk_pk,
            spk_id: 7,
            one_time: Some((99, &opk_pk)),
        };
        let hs = build_handshake(&alice_pub, &alice_sk, &bundle, &[0x66; 64], 1000).unwrap();

        let keys = RecipientKeys {
            account_id: &bob_id,
            app_kem_pub: &app_arr,
            app_kem_sk: &app_sk,
            signed_prekey_pub: &spk_arr,
            signed_prekey_sk: &spk_sk,
            one_time: Some((&opk_arr, &opk_sk)),
        };
        let proc = process_handshake(&hs.bytes, &keys, 1001, 604800).unwrap();
        assert_eq!(hs.session.root_key, proc.session.root_key);
        assert_eq!(hs.session.sending_chain_key, proc.session.sending_chain_key);
        assert_eq!(hs.transcript_hash, proc.transcript_hash);
        assert_eq!(proc.opk_consumed, Some(99));
    }

    #[test]
    fn handshake_agreement_without_opk() {
        let (app_pk, app_sk, app_arr) = kem(0x11);
        let (spk_pk, spk_sk, spk_arr) = kem(0x22);
        let (bob_pub, _) = mldsa(&[0x44; 32]);
        let bob_id = account_id(&bob_pub);
        let (alice_pub, alice_sk) = mldsa(&[0x55; 32]);

        let bundle = RecipientBundle {
            account_key_pub: &bob_pub,
            app_kem_pub: &app_pk,
            signed_prekey_pub: &spk_pk,
            spk_id: 7,
            one_time: None,
        };
        let hs = build_handshake(&alice_pub, &alice_sk, &bundle, &[0x66; 64], 1000).unwrap();
        let keys = RecipientKeys {
            account_id: &bob_id,
            app_kem_pub: &app_arr,
            app_kem_sk: &app_sk,
            signed_prekey_pub: &spk_arr,
            signed_prekey_sk: &spk_sk,
            one_time: None,
        };
        let proc = process_handshake(&hs.bytes, &keys, 1001, 604800).unwrap();
        assert_eq!(hs.session.root_key, proc.session.root_key);
        assert_eq!(proc.opk_consumed, None);
    }

    #[test]
    fn tampered_ct_rejected() {
        let (app_pk, app_sk, app_arr) = kem(0x11);
        let (spk_pk, spk_sk, spk_arr) = kem(0x22);
        let (bob_pub, _) = mldsa(&[0x44; 32]);
        let bob_id = account_id(&bob_pub);
        let (alice_pub, alice_sk) = mldsa(&[0x55; 32]);

        let bundle = RecipientBundle {
            account_key_pub: &bob_pub,
            app_kem_pub: &app_pk,
            signed_prekey_pub: &spk_pk,
            spk_id: 7,
            one_time: None,
        };
        let mut hs = build_handshake(&alice_pub, &alice_sk, &bundle, &[0x66; 64], 1000).unwrap();
        let ct_off = 1 + MLDSA_PUBKEY + MLKEM_PUBKEY + 8 + 4 + 1;
        hs.bytes[ct_off] ^= 1;
        let keys = RecipientKeys {
            account_id: &bob_id,
            app_kem_pub: &app_arr,
            app_kem_sk: &app_sk,
            signed_prekey_pub: &spk_arr,
            signed_prekey_sk: &spk_sk,
            one_time: None,
        };
        // ct входит в подписанную стенограмму -> ломается подпись (внешняя целостность).
        let r = process_handshake(&hs.bytes, &keys, 1001, 604800);
        assert!(matches!(r, Err(E2eError::BadSignature)));
    }

    #[test]
    fn tampered_confirm_tag_rejected() {
        let (app_pk, app_sk, app_arr) = kem(0x11);
        let (spk_pk, spk_sk, spk_arr) = kem(0x22);
        let (bob_pub, _) = mldsa(&[0x44; 32]);
        let bob_id = account_id(&bob_pub);
        let (alice_pub, alice_sk) = mldsa(&[0x55; 32]);
        let bundle = RecipientBundle {
            account_key_pub: &bob_pub,
            app_kem_pub: &app_pk,
            signed_prekey_pub: &spk_pk,
            spk_id: 7,
            one_time: None,
        };
        let mut hs = build_handshake(&alice_pub, &alice_sk, &bundle, &[0x66; 64], 1000).unwrap();
        // confirm_tag (последние 32 байта) НЕ в стенограмме -> подпись валидна, тег не сойдётся.
        let n = hs.bytes.len();
        hs.bytes[n - 1] ^= 1;
        let keys = RecipientKeys {
            account_id: &bob_id,
            app_kem_pub: &app_arr,
            app_kem_sk: &app_sk,
            signed_prekey_pub: &spk_arr,
            signed_prekey_sk: &spk_sk,
            one_time: None,
        };
        let r = process_handshake(&hs.bytes, &keys, 1001, 604800);
        assert!(matches!(r, Err(E2eError::ConfirmMismatch)));
    }

    #[test]
    fn stale_rejected() {
        let (app_pk, app_sk, app_arr) = kem(0x11);
        let (spk_pk, spk_sk, spk_arr) = kem(0x22);
        let (bob_pub, _) = mldsa(&[0x44; 32]);
        let bob_id = account_id(&bob_pub);
        let (alice_pub, alice_sk) = mldsa(&[0x55; 32]);
        let bundle = RecipientBundle {
            account_key_pub: &bob_pub,
            app_kem_pub: &app_pk,
            signed_prekey_pub: &spk_pk,
            spk_id: 7,
            one_time: None,
        };
        let hs = build_handshake(&alice_pub, &alice_sk, &bundle, &[0x66; 64], 1000).unwrap();
        let keys = RecipientKeys {
            account_id: &bob_id,
            app_kem_pub: &app_arr,
            app_kem_sk: &app_sk,
            signed_prekey_pub: &spk_arr,
            signed_prekey_sk: &spk_sk,
            one_time: None,
        };
        let r = process_handshake(&hs.bytes, &keys, 1_000_000, 604800);
        assert!(matches!(r, Err(E2eError::Stale)));
    }
}
