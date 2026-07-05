//! Этап 6 — машина состояний двойного храповика (KEM-храповик на ML-KEM-768).
//! RatchetEncrypt/Decrypt поверх примитивов ratchet.rs. Приём — на пробной логике:
//! состояние меняется только при успехе AEAD (поддельное сообщение не двигает храповик).

use mt_crypto::{
    keypair_from_seed_mlkem, mlkem_decapsulate, mlkem_encapsulate, MlkemCiphertext, MlkemPublicKey,
    MlkemSecretKey,
};

use crate::ratchet::{ad_bytes, kdf_ck, kdf_rk, msg_key, open, seal, MLKEM_PUBKEY_SIZE};

pub const MLKEM_CT_SIZE: usize = 1088;
pub const MAX_SKIP: u32 = 1000;
pub const MAX_MKSKIPPED: usize = 2000;
pub const MAX_PLAINTEXT: usize = 1_048_576;

#[derive(Debug, PartialEq, Eq)]
pub enum RatchetError {
    BadFormat,
    TooManySkipped,
    TooLarge,
    Decrypt,
    Crypto,
}

pub struct SessionState {
    session_id: [u8; 32],
    rk: [u8; 32],
    dhs_pub: [u8; MLKEM_PUBKEY_SIZE],
    dhs_sk: MlkemSecretKey,
    dhr: Option<[u8; MLKEM_PUBKEY_SIZE]>,
    cks: Option<[u8; 32]>,
    send_ct: Option<[u8; MLKEM_CT_SIZE]>,
    ckr: Option<[u8; 32]>,
    ns: u32,
    nr: u32,
    pn: u32,
    ratchet_pending: bool,
    mkskipped: Vec<(Vec<u8>, u32, [u8; 32])>,
}

fn to_pub(pk: &MlkemPublicKey) -> [u8; MLKEM_PUBKEY_SIZE] {
    pk.as_bytes().to_owned()
}

impl SessionState {
    /// Инициатор (Алиса) из выходов Этапа 5.
    pub fn init_initiator(
        session_id: [u8; 32],
        initial_root_key: [u8; 32],
        initial_sending_chain_key: [u8; 32],
        eph_kem_pub_a: [u8; MLKEM_PUBKEY_SIZE],
        eph_kem_sk_a: MlkemSecretKey,
        signed_prekey_pub_b: [u8; MLKEM_PUBKEY_SIZE],
    ) -> Self {
        Self {
            session_id,
            rk: initial_root_key,
            dhs_pub: eph_kem_pub_a,
            dhs_sk: eph_kem_sk_a,
            dhr: Some(signed_prekey_pub_b),
            cks: Some(initial_sending_chain_key),
            send_ct: None,
            ckr: None,
            ns: 0,
            nr: 0,
            pn: 0,
            ratchet_pending: false,
            mkskipped: Vec::new(),
        }
    }

    /// Получатель (Боб) из выходов Этапа 5.
    pub fn init_responder(
        session_id: [u8; 32],
        initial_root_key: [u8; 32],
        initial_sending_chain_key: [u8; 32],
        eph_kem_pub_a: [u8; MLKEM_PUBKEY_SIZE],
        signed_prekey_pub_b: [u8; MLKEM_PUBKEY_SIZE],
        signed_prekey_sk_b: MlkemSecretKey,
    ) -> Self {
        Self {
            session_id,
            rk: initial_root_key,
            dhs_pub: signed_prekey_pub_b,
            dhs_sk: signed_prekey_sk_b,
            dhr: Some(eph_kem_pub_a),
            cks: None,
            send_ct: None,
            ckr: Some(initial_sending_chain_key),
            ns: 0,
            nr: 0,
            pn: 0,
            ratchet_pending: true,
            mkskipped: Vec::new(),
        }
    }

    /// RatchetEncrypt. `rng_seed` (64B) — клиентская случайность для нового DHs
    /// (используется только при выполнении KEM-шага).
    pub fn encrypt(
        &mut self,
        plaintext: &[u8],
        rng_seed: &[u8; 64],
    ) -> Result<Vec<u8>, RatchetError> {
        if plaintext.len() > MAX_PLAINTEXT {
            return Err(RatchetError::TooLarge);
        }
        if self.ratchet_pending {
            self.pn = self.ns;
            self.ns = 0;
            let (new_pub, new_sk) =
                keypair_from_seed_mlkem(rng_seed).map_err(|_| RatchetError::Crypto)?;
            let dhr = self.dhr.ok_or(RatchetError::BadFormat)?;
            let dhr_pk = MlkemPublicKey::from_slice(&dhr).ok_or(RatchetError::BadFormat)?;
            let (ct, ss) = mlkem_encapsulate(&dhr_pk).map_err(|_| RatchetError::Crypto)?;
            let mut ss_arr = [0u8; 32];
            ss_arr.copy_from_slice(ss.as_bytes());
            let (new_rk, new_cks) = kdf_rk(&self.rk, &ss_arr);
            self.rk = new_rk;
            self.cks = Some(new_cks);
            self.dhs_pub = to_pub(&new_pub);
            self.dhs_sk = new_sk;
            self.send_ct = Some(ct.as_bytes().to_owned());
            self.ratchet_pending = false;
        }
        let cks = self.cks.ok_or(RatchetError::BadFormat)?;
        let (mk, next_cks) = kdf_ck(&cks);
        self.cks = Some(next_cks);
        let (enc_key, nonce) = msg_key(&mk);
        let ad = ad_bytes(&self.session_id, self.pn, self.ns, &self.dhs_pub);
        let body = seal(&enc_key, &nonce, plaintext, &ad);

        let has_ct = self.send_ct.is_some();
        let mut out = Vec::new();
        out.extend_from_slice(&self.dhs_pub);
        out.push(if has_ct { 0x01 } else { 0x00 });
        if let Some(ct) = &self.send_ct {
            out.extend_from_slice(ct);
        }
        out.extend_from_slice(&self.pn.to_le_bytes());
        out.extend_from_slice(&self.ns.to_le_bytes());
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&body);
        self.ns += 1;
        Ok(out)
    }

    /// RatchetDecrypt. Состояние меняется только при успехе AEAD.
    pub fn decrypt(&mut self, msg: &[u8]) -> Result<Vec<u8>, RatchetError> {
        // Разбор
        let mut p = 0;
        if msg.len() < MLKEM_PUBKEY_SIZE + 1 {
            return Err(RatchetError::BadFormat);
        }
        let ratchet_pub = &msg[p..p + MLKEM_PUBKEY_SIZE];
        p += MLKEM_PUBKEY_SIZE;
        let has_ct = match msg[p] {
            0x00 => false,
            0x01 => true,
            _ => return Err(RatchetError::BadFormat),
        };
        p += 1;
        let ct = if has_ct {
            if msg.len() < p + MLKEM_CT_SIZE {
                return Err(RatchetError::BadFormat);
            }
            let c = &msg[p..p + MLKEM_CT_SIZE];
            p += MLKEM_CT_SIZE;
            Some(c)
        } else {
            None
        };
        if msg.len() < p + 12 {
            return Err(RatchetError::BadFormat);
        }
        let m_pn = u32::from_le_bytes(msg[p..p + 4].try_into().unwrap());
        p += 4;
        let m_ns = u32::from_le_bytes(msg[p..p + 4].try_into().unwrap());
        p += 4;
        let body_len = u32::from_le_bytes(msg[p..p + 4].try_into().unwrap()) as usize;
        p += 4;
        if body_len > MAX_PLAINTEXT + 16 || msg.len() != p + body_len {
            return Err(RatchetError::BadFormat);
        }
        let body = &msg[p..p + body_len];
        let ad = {
            let mut rp = [0u8; MLKEM_PUBKEY_SIZE];
            rp.copy_from_slice(ratchet_pub);
            ad_bytes(&self.session_id, m_pn, m_ns, &rp)
        };

        // (1) пропущенный ключ?
        if let Some(idx) = self
            .mkskipped
            .iter()
            .position(|(rp, n, _)| rp.as_slice() == ratchet_pub && *n == m_ns)
        {
            let mk = self.mkskipped[idx].2;
            let (enc_key, nonce) = msg_key(&mk);
            let pt = open(&enc_key, &nonce, body, &ad).ok_or(RatchetError::Decrypt)?;
            self.mkskipped.remove(idx); // расходуется только при успехе
            return Ok(pt);
        }

        // Рабочая копия скалярного состояния (secret key только читается).
        let is_kem_step = match &self.dhr {
            Some(d) => &d[..] != ratchet_pub,
            None => true,
        };
        let mut rk = self.rk;
        let mut ckr = self.ckr;
        let mut nr = self.nr;
        let mut dhr = self.dhr;
        let mut pending = self.ratchet_pending;
        let mut new_skipped: Vec<(Vec<u8>, u32, [u8; 32])> = Vec::new();

        if is_kem_step {
            if !has_ct {
                return Err(RatchetError::BadFormat);
            }
            // досохранить пропущенные текущей приёмной цепочки до m_pn
            if let (Some(dr), Some(mut c)) = (dhr, ckr) {
                if m_pn.saturating_sub(nr) > MAX_SKIP {
                    return Err(RatchetError::TooManySkipped);
                }
                while nr < m_pn {
                    let (mk, nc) = kdf_ck(&c);
                    new_skipped.push((dr.to_vec(), nr, mk));
                    c = nc;
                    nr += 1;
                }
            }
            let ctv = MlkemCiphertext::from_slice(ct.unwrap()).ok_or(RatchetError::BadFormat)?;
            let ss = mlkem_decapsulate(&self.dhs_sk, &ctv).map_err(|_| RatchetError::Crypto)?;
            let mut ss_arr = [0u8; 32];
            ss_arr.copy_from_slice(ss.as_bytes());
            let (new_rk, new_ckr) = kdf_rk(&rk, &ss_arr);
            rk = new_rk;
            ckr = Some(new_ckr);
            let mut ndr = [0u8; MLKEM_PUBKEY_SIZE];
            ndr.copy_from_slice(ratchet_pub);
            dhr = Some(ndr);
            nr = 0;
            pending = true;
        }

        // пропустить в текущей цепочке до m_ns
        let mut c = ckr.ok_or(RatchetError::BadFormat)?;
        if m_ns.saturating_sub(nr) > MAX_SKIP {
            return Err(RatchetError::TooManySkipped);
        }
        let dr = dhr.ok_or(RatchetError::BadFormat)?;
        while nr < m_ns {
            let (mk, nc) = kdf_ck(&c);
            new_skipped.push((dr.to_vec(), nr, mk));
            c = nc;
            nr += 1;
        }
        let (mk, next_c) = kdf_ck(&c);
        let (enc_key, nonce) = msg_key(&mk);
        let pt = open(&enc_key, &nonce, body, &ad).ok_or(RatchetError::Decrypt)?;

        // Успех AEAD -> коммит рабочей копии.
        self.rk = rk;
        self.ckr = Some(next_c);
        self.dhr = dhr;
        self.nr = nr + 1;
        self.ratchet_pending = pending;
        for s in new_skipped {
            self.mkskipped.push(s);
        }
        while self.mkskipped.len() > MAX_MKSKIPPED {
            self.mkskipped.remove(0);
        }
        Ok(pt)
    }
}
