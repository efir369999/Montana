//! Store-and-forward инбокс (Этап 2): вращающийся epoch_tag, депозит/фетч осколков,
//! padding-бакеты. Спека: Montana P2P Network, Этап 2.
//! `window` — параметр (SSOT window_index = labels::window_index у вызывающего).

use mt_codec::{write_bytes, write_u16, write_u32, write_u8, CanonicalEncode};

use crate::frame::{FrameError, MsgId, MSG_ID_SIZE};

pub const EPOCH_TAG_SIZE: usize = 16;
pub type EpochTag = [u8; EPOCH_TAG_SIZE];

// spec: epoch_tag = SHA-256("mt-inbox-tag" || 0x00 || account_id || window_8B_LE)[0..16]
pub fn epoch_tag(account_id: &[u8; 32], window: u64) -> EpochTag {
    let full = mt_crypto::hash(
        mt_codec::domain::INBOX_TAG,
        &[account_id, &window.to_le_bytes()],
    );
    let mut tag = [0u8; EPOCH_TAG_SIZE];
    tag.copy_from_slice(&full[..EPOCH_TAG_SIZE]);
    tag
}

// spec: padding-бакеты {256, 1024, 4096, 16384, 65536, 262144, 1048576} (степени, ×4).
pub const PADDING_BUCKETS: [usize; 7] = [256, 1024, 4096, 16384, 65536, 262144, 1_048_576];

/// Наименьший бакет ≥ n. None если n превышает верхний бакет (1 MiB = MAX_PLAINTEXT).
pub fn bucket_len(n: usize) -> Option<usize> {
    PADDING_BUCKETS.iter().copied().find(|&b| b >= n)
}

/// spec R6/N_FETCH: epoch_tag принадлежит инбоксу account_id, если совпадает с
/// epoch_tag(account_id, w) для некоторого w в [lo, hi] (окно-диапазон, поглощает
/// async депозит/фетч и разбег UTC-часов). Почтальон так фильтрует «свои» теги.
pub const N_FETCH: u64 = 240; // = 4 часа при WINDOW_SECONDS=60

pub fn epoch_tag_belongs(account_id: &[u8; 32], tag: &EpochTag, lo: u64, hi: u64) -> bool {
    (lo..=hi).any(|w| &epoch_tag(account_id, w) == tag)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Deposit {
    pub epoch_tag: EpochTag,
    pub msg_id: MsgId,
    pub ttl_windows: u32,
    pub shard_index: u8,
    pub shard_total: u8,
    pub ct: Vec<u8>, // осколок, дополненный до бакета
}

impl CanonicalEncode for Deposit {
    fn encode(&self, buf: &mut Vec<u8>) {
        write_bytes(buf, &self.epoch_tag);
        write_bytes(buf, &self.msg_id);
        write_u32(buf, self.ttl_windows);
        write_u8(buf, self.shard_index);
        write_u8(buf, self.shard_total);
        write_u32(buf, self.ct.len() as u32);
        write_bytes(buf, &self.ct);
    }
}

// epoch_tag 16 + msg_id 16 + ttl 4 + shard_index 1 + shard_total 1 + ct_len 4
const DEPOSIT_HEADER: usize = EPOCH_TAG_SIZE + MSG_ID_SIZE + 4 + 1 + 1 + 4;

impl Deposit {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(DEPOSIT_HEADER + self.ct.len());
        self.encode(&mut b);
        b
    }

    pub fn decode(input: &[u8]) -> Result<Self, FrameError> {
        if input.len() < DEPOSIT_HEADER {
            return Err(FrameError::Truncated);
        }
        let mut o = 0;
        let mut epoch_tag = [0u8; EPOCH_TAG_SIZE];
        epoch_tag.copy_from_slice(&input[o..o + EPOCH_TAG_SIZE]);
        o += EPOCH_TAG_SIZE;
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
        let ct_len = u32::from_le_bytes(
            input[o..o + 4]
                .try_into()
                .map_err(|_| FrameError::Truncated)?,
        );
        o += 4;
        if ct_len as usize > crate::frame::MAX_PAYLOAD_LEN {
            return Err(FrameError::PayloadTooLong(ct_len));
        }
        if input.len() - o != ct_len as usize {
            return Err(FrameError::LengthMismatch);
        }
        Ok(Self {
            epoch_tag,
            msg_id,
            ttl_windows,
            shard_index,
            shard_total,
            ct: input[o..].to_vec(),
        })
    }
}

pub const NONCE_SIZE: usize = 16;
pub const FETCH_SIG_SIZE: usize = 3309;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FetchReq {
    pub epoch_tag: EpochTag,
    pub nonce: [u8; NONCE_SIZE],
    pub sig: Vec<u8>, // ML-DSA-65, 3309 B
}

impl FetchReq {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(EPOCH_TAG_SIZE + NONCE_SIZE + FETCH_SIG_SIZE);
        b.extend_from_slice(&self.epoch_tag);
        b.extend_from_slice(&self.nonce);
        b.extend_from_slice(&self.sig);
        b
    }

    pub fn decode(input: &[u8]) -> Result<Self, FrameError> {
        if input.len() != EPOCH_TAG_SIZE + NONCE_SIZE + FETCH_SIG_SIZE {
            return Err(FrameError::Truncated);
        }
        let mut epoch_tag = [0u8; EPOCH_TAG_SIZE];
        epoch_tag.copy_from_slice(&input[..EPOCH_TAG_SIZE]);
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&input[EPOCH_TAG_SIZE..EPOCH_TAG_SIZE + NONCE_SIZE]);
        Ok(Self {
            epoch_tag,
            nonce,
            sig: input[EPOCH_TAG_SIZE + NONCE_SIZE..].to_vec(),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FetchItem {
    pub msg_id: MsgId,
    pub shard_index: u8,
    pub shard_total: u8,
    pub ct: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FetchResp {
    pub items: Vec<FetchItem>,
}

impl FetchResp {
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
        // E-4: cap pre-alloc — минимальный item = msg_id16+idx1+total1+ct_len4 = 22 B.
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
            if ct_len > crate::frame::MAX_PAYLOAD_LEN || input.len() < o + ct_len {
                return Err(FrameError::LengthMismatch);
            }
            items.push(FetchItem {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_tag_rotates_by_window_stable_per_window() {
        let acc = [0x11u8; 32];
        assert_eq!(epoch_tag(&acc, 100), epoch_tag(&acc, 100));
        assert_ne!(epoch_tag(&acc, 100), epoch_tag(&acc, 101));
        assert_ne!(epoch_tag(&acc, 100), epoch_tag(&[0x12u8; 32], 100));
        assert_eq!(epoch_tag(&acc, 100).len(), 16);
    }

    #[test]
    fn bucket_len_rounds_up_to_power_of_four() {
        assert_eq!(bucket_len(0), Some(256));
        assert_eq!(bucket_len(256), Some(256));
        assert_eq!(bucket_len(257), Some(1024));
        assert_eq!(bucket_len(1_048_576), Some(1_048_576));
        assert_eq!(bucket_len(1_048_577), None);
    }

    #[test]
    fn deposit_roundtrip() {
        let d = Deposit {
            epoch_tag: [0xAA; 16],
            msg_id: [0xBB; 16],
            ttl_windows: 240,
            shard_index: 1,
            shard_total: 4,
            ct: vec![0xCC; 256],
        };
        assert_eq!(Deposit::decode(&d.to_bytes()).unwrap(), d);
    }

    #[test]
    fn fetchreq_roundtrip_and_size() {
        let r = FetchReq {
            epoch_tag: [0x01; 16],
            nonce: [0x02; 16],
            sig: vec![0x03; FETCH_SIG_SIZE],
        };
        let b = r.to_bytes();
        assert_eq!(b.len(), 16 + 16 + 3309);
        assert_eq!(FetchReq::decode(&b).unwrap(), r);
        assert!(FetchReq::decode(&b[..b.len() - 1]).is_err());
    }

    #[test]
    fn fetchresp_roundtrip_multi_item() {
        let resp = FetchResp {
            items: vec![
                FetchItem {
                    msg_id: [1; 16],
                    shard_index: 0,
                    shard_total: 4,
                    ct: vec![9; 256],
                },
                FetchItem {
                    msg_id: [2; 16],
                    shard_index: 1,
                    shard_total: 4,
                    ct: vec![8; 1024],
                },
            ],
        };
        assert_eq!(FetchResp::decode(&resp.to_bytes()).unwrap(), resp);
    }

    #[test]
    fn deposit_rejects_length_mismatch() {
        let d = Deposit {
            epoch_tag: [0; 16],
            msg_id: [0; 16],
            ttl_windows: 1,
            shard_index: 0,
            shard_total: 1,
            ct: vec![7; 64],
        };
        let mut b = d.to_bytes();
        b.pop();
        assert!(matches!(
            Deposit::decode(&b),
            Err(FrameError::LengthMismatch)
        ));
    }
}
