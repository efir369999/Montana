//! Montana Unlinkable Queues (MUQ, Этап 2) — несвязываемая адресация ящика.
//! Спека: Montana P2P Network v0.9.0, Этап 2. Раздельные случайные send_id/recv_id
//! (не из account_id), двуххоповый депозит. Механика хранения (Reed-Solomon/TTL/drop)
//! переиспользуется из inbox_store (ключ буфера = recv_id).

use mt_codec::{write_bytes, write_u16, write_u32, write_u8, CanonicalEncode};
use mt_crypto::{PUBLIC_KEY_SIZE, SIGNATURE_SIZE};

use crate::challenge::{ChannelHash, Nonce, NONCE_SIZE};
use crate::frame::{FrameError, MsgId, MAX_PAYLOAD_LEN, MSG_ID_SIZE};

pub const QUEUE_ID_SIZE: usize = 32;
pub type QueueId = [u8; QUEUE_ID_SIZE]; // send_id либо recv_id

/// Объект очереди на хосте (off-chain [P2P-1]). recv_id/send_id независимы и случайны.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Queue {
    pub recv_id: QueueId,
    pub send_id: QueueId,
    pub recv_pubkey: [u8; PUBLIC_KEY_SIZE],
    pub send_pubkey: Option<[u8; PUBLIC_KEY_SIZE]>, // None = unsecured
    pub rotation_epoch: u64,
    pub quota: u32,
}

impl Queue {
    /// Генерация независимых случайных id (host-side, OS CSPRNG). Тесты используют from_parts.
    pub fn generate(
        recv_pubkey: [u8; PUBLIC_KEY_SIZE],
        send_pubkey: Option<[u8; PUBLIC_KEY_SIZE]>,
        rotation_epoch: u64,
        quota: u32,
    ) -> Result<Self, FrameError> {
        let mut recv_id = [0u8; QUEUE_ID_SIZE];
        let mut send_id = [0u8; QUEUE_ID_SIZE];
        getrandom::getrandom(&mut recv_id).map_err(|_| FrameError::Truncated)?;
        getrandom::getrandom(&mut send_id).map_err(|_| FrameError::Truncated)?;
        Ok(Self {
            recv_id,
            send_id,
            recv_pubkey,
            send_pubkey,
            rotation_epoch,
            quota,
        })
    }
}

/// Депозит хосту (внутри ProxyForward, запечатан Noise_PQ XX к host).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HostDeposit {
    pub send_id: QueueId,
    pub msg_id: MsgId,
    pub ttl_windows: u32,
    pub shard_index: u8,
    pub shard_total: u8,
    pub nonce: Nonce,
    pub ct: Vec<u8>,
    pub sig: Vec<u8>, // ML-DSA-65 send_key (secured) либо пусто (unsecured)
}

// send_id 32 + msg_id 16 + ttl 4 + idx 1 + total 1 + nonce 16 + ct_len 4
const HD_HEADER: usize = QUEUE_ID_SIZE + MSG_ID_SIZE + 4 + 1 + 1 + NONCE_SIZE + 4;

impl CanonicalEncode for HostDeposit {
    fn encode(&self, buf: &mut Vec<u8>) {
        write_bytes(buf, &self.send_id);
        write_bytes(buf, &self.msg_id);
        write_u32(buf, self.ttl_windows);
        write_u8(buf, self.shard_index);
        write_u8(buf, self.shard_total);
        write_bytes(buf, &self.nonce);
        write_u32(buf, self.ct.len() as u32);
        write_bytes(buf, &self.ct);
        write_u32(buf, self.sig.len() as u32);
        write_bytes(buf, &self.sig);
    }
}

impl HostDeposit {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(HD_HEADER + self.ct.len() + self.sig.len());
        self.encode(&mut b);
        b
    }

    pub fn decode(input: &[u8]) -> Result<Self, FrameError> {
        if input.len() < HD_HEADER {
            return Err(FrameError::Truncated);
        }
        let mut o = 0;
        let mut send_id = [0u8; QUEUE_ID_SIZE];
        send_id.copy_from_slice(&input[o..o + QUEUE_ID_SIZE]);
        o += QUEUE_ID_SIZE;
        let mut msg_id = [0u8; MSG_ID_SIZE];
        msg_id.copy_from_slice(&input[o..o + MSG_ID_SIZE]);
        o += MSG_ID_SIZE;
        let ttl_windows = u32::from_le_bytes(
            input[o..o + 4]
                .try_into()
                .map_err(|_| FrameError::Truncated)?,
        );
        o += 4;
        let shard_index = input[o];
        o += 1;
        let shard_total = input[o];
        o += 1;
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&input[o..o + NONCE_SIZE]);
        o += NONCE_SIZE;
        let ct_len = u32::from_le_bytes(
            input[o..o + 4]
                .try_into()
                .map_err(|_| FrameError::Truncated)?,
        ) as usize;
        o += 4;
        if ct_len > MAX_PAYLOAD_LEN || input.len() < o + ct_len + 4 {
            return Err(FrameError::LengthMismatch);
        }
        let ct = input[o..o + ct_len].to_vec();
        o += ct_len;
        let sig_len = u32::from_le_bytes(
            input[o..o + 4]
                .try_into()
                .map_err(|_| FrameError::Truncated)?,
        ) as usize;
        o += 4;
        if input.len() - o != sig_len {
            return Err(FrameError::LengthMismatch);
        }
        Ok(Self {
            send_id,
            msg_id,
            ttl_windows,
            shard_index,
            shard_total,
            nonce,
            ct,
            sig: input[o..].to_vec(),
        })
    }
}

/// Оборачивание депозита для entry-proxy (proxy не расшифрует sealed).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProxyForward {
    pub host_addr: crate::OverlayAddr, // overlay-адрес queue-host
    pub sealed: Vec<u8>,               // HostDeposit запечатан Noise_PQ XX для host
}

impl ProxyForward {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(32 + 4 + self.sealed.len());
        b.extend_from_slice(&self.host_addr);
        b.extend_from_slice(&(self.sealed.len() as u32).to_le_bytes());
        b.extend_from_slice(&self.sealed);
        b
    }

    pub fn decode(input: &[u8]) -> Result<Self, FrameError> {
        if input.len() < 36 {
            return Err(FrameError::Truncated);
        }
        let mut host_addr = [0u8; 32];
        host_addr.copy_from_slice(&input[..32]);
        let sealed_len = u32::from_le_bytes(
            input[32..36]
                .try_into()
                .map_err(|_| FrameError::Truncated)?,
        ) as usize;
        if input.len() - 36 != sealed_len {
            return Err(FrameError::LengthMismatch);
        }
        Ok(Self {
            host_addr,
            sealed: input[36..].to_vec(),
        })
    }
}

/// Выборка получателем (прямое соединение B↔host, подпись привязана к каналу — F3/R4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueueSubscribe {
    pub recv_id: QueueId,
    pub nonce: Nonce,
    pub sig: [u8; SIGNATURE_SIZE],
}

impl QueueSubscribe {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(QUEUE_ID_SIZE + NONCE_SIZE + SIGNATURE_SIZE);
        b.extend_from_slice(&self.recv_id);
        b.extend_from_slice(&self.nonce);
        b.extend_from_slice(&self.sig);
        b
    }

    pub fn decode(input: &[u8]) -> Result<Self, FrameError> {
        if input.len() != QUEUE_ID_SIZE + NONCE_SIZE + SIGNATURE_SIZE {
            return Err(FrameError::Truncated);
        }
        let mut recv_id = [0u8; QUEUE_ID_SIZE];
        recv_id.copy_from_slice(&input[..QUEUE_ID_SIZE]);
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&input[QUEUE_ID_SIZE..QUEUE_ID_SIZE + NONCE_SIZE]);
        let mut sig = [0u8; SIGNATURE_SIZE];
        sig.copy_from_slice(&input[QUEUE_ID_SIZE + NONCE_SIZE..]);
        Ok(Self {
            recv_id,
            nonce,
            sig,
        })
    }
}

/// Ответ хоста получателю (осколки из буфера recv_id).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueueResp {
    pub items: Vec<QueueItem>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueueItem {
    pub msg_id: MsgId,
    pub shard_index: u8,
    pub shard_total: u8,
    pub ct: Vec<u8>,
}

impl QueueResp {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();
        write_u16(&mut b, self.items.len() as u16);
        for it in &self.items {
            write_bytes(&mut b, &it.msg_id);
            write_u8(&mut b, it.shard_index);
            write_u8(&mut b, it.shard_total);
            write_u32(&mut b, it.ct.len() as u32);
            write_bytes(&mut b, &it.ct);
        }
        b
    }

    pub fn decode(input: &[u8]) -> Result<Self, FrameError> {
        if input.len() < 2 {
            return Err(FrameError::Truncated);
        }
        let count = u16::from_le_bytes([input[0], input[1]]) as usize;
        let mut o = 2;
        let mut items = Vec::with_capacity(count.min((input.len() - 2) / 22));
        for _ in 0..count {
            if input.len() < o + MSG_ID_SIZE + 1 + 1 + 4 {
                return Err(FrameError::Truncated);
            }
            let mut msg_id = [0u8; MSG_ID_SIZE];
            msg_id.copy_from_slice(&input[o..o + MSG_ID_SIZE]);
            o += MSG_ID_SIZE;
            let shard_index = input[o];
            o += 1;
            let shard_total = input[o];
            o += 1;
            let ct_len = u32::from_le_bytes(
                input[o..o + 4]
                    .try_into()
                    .map_err(|_| FrameError::Truncated)?,
            ) as usize;
            o += 4;
            if ct_len > MAX_PAYLOAD_LEN || input.len() < o + ct_len {
                return Err(FrameError::LengthMismatch);
            }
            items.push(QueueItem {
                msg_id,
                shard_index,
                shard_total,
                ct: input[o..o + ct_len].to_vec(),
            });
            o += ct_len;
        }
        if o != input.len() {
            return Err(FrameError::LengthMismatch);
        }
        Ok(Self { items })
    }
}

// --- подписи (переиспользуют challenge_message из Этапа 1) ---

/// Подпись выборки: sig = ML-DSA-65(recv_key, "mt-queue-sub"‖0x00‖recv_id‖nonce‖channel_hash).
pub fn sign_subscribe(
    recv_sk: &mt_crypto::SecretKey,
    recv_id: &QueueId,
    nonce: &Nonce,
    channel_hash: &ChannelHash,
) -> Result<mt_crypto::Signature, mt_crypto::CryptoError> {
    let msg = crate::challenge::challenge_message(
        mt_codec::domain::QUEUE_SUB,
        recv_id,
        nonce,
        channel_hash,
    );
    mt_crypto::sign(recv_sk, &msg)
}

/// F3/E-2: проверка выборки — подпись против ХРАНИМОГО recv_pubkey очереди + channel_hash.
/// Ownership инкапсулирован (recv_pubkey привязан к очереди на хосте, забыть нельзя).
pub fn verify_subscribe(
    recv_pubkey: &[u8; PUBLIC_KEY_SIZE],
    recv_id: &QueueId,
    nonce: &Nonce,
    channel_hash: &ChannelHash,
    sig: &mt_crypto::Signature,
) -> bool {
    let msg = crate::challenge::challenge_message(
        mt_codec::domain::QUEUE_SUB,
        recv_id,
        nonce,
        channel_hash,
    );
    mt_crypto::verify(&mt_crypto::PublicKey::from_array(*recv_pubkey), &msg, sig)
}

/// Подпись депозита (secured): sig = ML-DSA-65(send_key, "mt-queue-send"‖0x00‖send_id‖msg_id‖nonce).
/// БЕЗ channel_hash — двуххоп, прямого канала sender↔host нет; replay закрыт send_key+nonce.
pub fn sign_deposit(
    send_sk: &mt_crypto::SecretKey,
    send_id: &QueueId,
    msg_id: &MsgId,
    nonce: &Nonce,
) -> Result<mt_crypto::Signature, mt_crypto::CryptoError> {
    let mut resource = Vec::with_capacity(QUEUE_ID_SIZE + MSG_ID_SIZE);
    resource.extend_from_slice(send_id);
    resource.extend_from_slice(msg_id);
    let mut msg = Vec::new();
    msg.extend_from_slice(mt_codec::domain::QUEUE_SEND);
    msg.push(0u8);
    msg.extend_from_slice(&resource);
    msg.extend_from_slice(nonce);
    mt_crypto::sign(send_sk, &msg)
}

pub fn verify_deposit(
    send_pubkey: &[u8; PUBLIC_KEY_SIZE],
    send_id: &QueueId,
    msg_id: &MsgId,
    nonce: &Nonce,
    sig: &mt_crypto::Signature,
) -> bool {
    let mut msg = Vec::new();
    msg.extend_from_slice(mt_codec::domain::QUEUE_SEND);
    msg.push(0u8);
    msg.extend_from_slice(send_id);
    msg.extend_from_slice(msg_id);
    msg.extend_from_slice(nonce);
    mt_crypto::verify(&mt_crypto::PublicKey::from_array(*send_pubkey), &msg, sig)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mt_crypto::{keypair_from_seed, PublicKey, SecretKey};

    fn kp(seed: u8) -> ([u8; PUBLIC_KEY_SIZE], SecretKey) {
        let (pk, sk): (PublicKey, SecretKey) = keypair_from_seed(&[seed; 32]).unwrap();
        (*pk.as_bytes(), sk)
    }

    #[test]
    fn queue_ids_independent_random() {
        let (rpk, _) = kp(0x11);
        let q = Queue::generate(rpk, None, 100, 64).unwrap();
        assert_ne!(q.recv_id, q.send_id);
        assert_ne!(q.recv_id, [0u8; 32]);
    }

    #[test]
    fn hostdeposit_roundtrip_secured_and_unsecured() {
        for sig in [vec![0x03; SIGNATURE_SIZE], Vec::new()] {
            let d = HostDeposit {
                send_id: [0xAA; 32],
                msg_id: [0xBB; 16],
                ttl_windows: 240,
                shard_index: 1,
                shard_total: 4,
                nonce: [0x07; 16],
                ct: vec![0xCC; 256],
                sig,
            };
            assert_eq!(HostDeposit::decode(&d.to_bytes()).unwrap(), d);
        }
    }

    #[test]
    fn proxyforward_roundtrip() {
        let pf = ProxyForward {
            host_addr: [0xEE; 32],
            sealed: vec![0x5A; 500],
        };
        assert_eq!(ProxyForward::decode(&pf.to_bytes()).unwrap(), pf);
    }

    #[test]
    fn queuesubscribe_roundtrip_and_size() {
        let s = QueueSubscribe {
            recv_id: [0x01; 32],
            nonce: [0x02; 16],
            sig: [0x03; SIGNATURE_SIZE],
        };
        let b = s.to_bytes();
        assert_eq!(b.len(), 32 + 16 + 3309);
        assert_eq!(QueueSubscribe::decode(&b).unwrap(), s);
    }

    #[test]
    fn queueresp_roundtrip() {
        let r = QueueResp {
            items: vec![
                QueueItem {
                    msg_id: [1; 16],
                    shard_index: 0,
                    shard_total: 4,
                    ct: vec![9; 256],
                },
                QueueItem {
                    msg_id: [2; 16],
                    shard_index: 1,
                    shard_total: 4,
                    ct: vec![8; 1024],
                },
            ],
        };
        assert_eq!(QueueResp::decode(&r.to_bytes()).unwrap(), r);
    }

    #[test]
    fn subscribe_sig_verify_and_channel_binding() {
        let (rpk, rsk) = kp(0x11);
        let recv_id = [0x33; 32];
        let nonce = [0x07; 16];
        let ch = [0x0C; 32];
        let sig = sign_subscribe(&rsk, &recv_id, &nonce, &ch).unwrap();
        assert!(verify_subscribe(&rpk, &recv_id, &nonce, &ch, &sig));
        // другой канал — подпись непереносима (F3)
        assert!(!verify_subscribe(&rpk, &recv_id, &nonce, &[0x0D; 32], &sig));
        // другой recv_pubkey — не сойдётся
        let (rpk2, _) = kp(0x12);
        assert!(!verify_subscribe(&rpk2, &recv_id, &nonce, &ch, &sig));
    }

    #[test]
    fn deposit_sig_verify_send_key() {
        let (spk, ssk) = kp(0x22);
        let send_id = [0x44; 32];
        let msg_id = [0x55; 16];
        let nonce = [0x08; 16];
        let sig = sign_deposit(&ssk, &send_id, &msg_id, &nonce).unwrap();
        assert!(verify_deposit(&spk, &send_id, &msg_id, &nonce, &sig));
        assert!(!verify_deposit(&spk, &send_id, &msg_id, &[0x09; 16], &sig)); // чужой nonce
    }
}
