//! Этап 1 (второй фронт — децентрализация данных) — архив истории.
//! HistoryBlock = block_seq u64 LE ‖ item_count u32 LE
//!   ‖ [ conv_id 32 ‖ dir 1 ‖ send_time u64 LE ‖ content_len u32 LE ‖ Content ]×item_count
//! sealed = nonce(block_seq_le8 ‖ 0x00000000) ‖ ChaCha20-Poly1305(history_key, nonce, block,
//!                                                                 AD = "mt-history" ‖ 0x00 ‖ account_id).
//! history_key = HKDF-SHA-256(0×32, entropy_32, "mt-history-key", 32) — SSOT (первый фронт, Этап 12).
//! Разбор инвалид-безопасен (Gate 13): любое нарушение → None, НИКОГДА паника.

use crate::kdf::hkdf_sha256;
use crate::ratchet::{open, seal};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const HISTORY_AD: &[u8] = b"mt-history";
pub const HISTORY_KEY_INFO: &[u8] = b"mt-history-key";
pub const CONV_ID_LEN: usize = 32;
pub const DIR_OUT: u8 = 0x00;
pub const DIR_IN: u8 = 0x01;

pub fn history_key(entropy_32: &[u8; 32]) -> [u8; 32] {
    let k = hkdf_sha256(&[0u8; 32], entropy_32, HISTORY_KEY_INFO, 32);
    let mut out = [0u8; 32];
    out.copy_from_slice(&k);
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryItem {
    pub conv_id: [u8; CONV_ID_LEN],
    pub dir: u8,
    pub send_time: u64,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryBlock {
    pub block_seq: u64,
    pub items: Vec<HistoryItem>,
}

pub fn encode_block(b: &HistoryBlock) -> Vec<u8> {
    let mut o = Vec::new();
    o.extend_from_slice(&b.block_seq.to_le_bytes());
    o.extend_from_slice(&(b.items.len() as u32).to_le_bytes());
    for it in &b.items {
        o.extend_from_slice(&it.conv_id);
        o.push(it.dir);
        o.extend_from_slice(&it.send_time.to_le_bytes());
        o.extend_from_slice(&(it.content.len() as u32).to_le_bytes());
        o.extend_from_slice(&it.content);
    }
    o
}

pub fn decode_block(buf: &[u8]) -> Option<HistoryBlock> {
    if buf.len() < 12 {
        return None;
    }
    let block_seq = u64::from_le_bytes(buf[0..8].try_into().ok()?);
    let item_count = u32::from_le_bytes(buf[8..12].try_into().ok()?) as usize;
    let mut off = 12usize;
    let mut items = Vec::new();
    for _ in 0..item_count {
        if off + 32 + 1 + 8 + 4 > buf.len() {
            return None;
        }
        let mut conv_id = [0u8; CONV_ID_LEN];
        conv_id.copy_from_slice(&buf[off..off + 32]);
        off += 32;
        let dir = buf[off];
        off += 1;
        let send_time = u64::from_le_bytes(buf[off..off + 8].try_into().ok()?);
        off += 8;
        let content_len = u32::from_le_bytes(buf[off..off + 4].try_into().ok()?) as usize;
        off += 4;
        if off + content_len > buf.len() {
            return None;
        }
        let content = buf[off..off + content_len].to_vec();
        off += content_len;
        items.push(HistoryItem {
            conv_id,
            dir,
            send_time,
            content,
        });
    }
    if off != buf.len() {
        return None;
    }
    Some(HistoryBlock { block_seq, items })
}

fn block_nonce(block_seq: u64) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[0..8].copy_from_slice(&block_seq.to_le_bytes());
    n
}

fn history_ad(account_id: &[u8; 32]) -> Vec<u8> {
    let mut ad = HISTORY_AD.to_vec();
    ad.push(0x00);
    ad.extend_from_slice(account_id);
    ad
}

pub fn seal_block(history_key: &[u8; 32], account_id: &[u8; 32], b: &HistoryBlock) -> Vec<u8> {
    let nonce = block_nonce(b.block_seq);
    let pt = encode_block(b);
    let ct = seal(history_key, &nonce, &pt, &history_ad(account_id));
    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

pub fn open_block(
    history_key: &[u8; 32],
    account_id: &[u8; 32],
    sealed: &[u8],
) -> Option<HistoryBlock> {
    if sealed.len() < 12 {
        return None;
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&sealed[..12]);
    let pt = open(history_key, &nonce, &sealed[12..], &history_ad(account_id))?;
    let block = decode_block(&pt)?;
    if block_nonce(block.block_seq) != nonce {
        return None;
    }
    Some(block)
}

pub const MEDIA_AD: &[u8] = b"mt-media-vault";

fn media_ad(account_id: &[u8; 32]) -> Vec<u8> {
    let mut ad = MEDIA_AD.to_vec();
    ad.push(0x00);
    ad.extend_from_slice(account_id);
    ad
}

fn media_nonce(blob_id_hex: &str) -> [u8; 12] {
    let h: [u8; 32] = Sha256::digest(blob_id_hex.as_bytes()).into();
    let mut n = [0u8; 12];
    n.copy_from_slice(&h[..12]);
    n
}

pub fn seal_media(history_key: &[u8; 32], account_id: &[u8; 32], blob_id_hex: &str, plaintext: &[u8]) -> Vec<u8> {
    let nonce = media_nonce(blob_id_hex);
    let ct = seal(history_key, &nonce, plaintext, &media_ad(account_id));
    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

pub fn open_media(history_key: &[u8; 32], account_id: &[u8; 32], sealed: &[u8]) -> Option<Vec<u8>> {
    if sealed.len() < 12 {
        return None;
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&sealed[..12]);
    open(history_key, &nonce, &sealed[12..], &media_ad(account_id))
}

pub fn sealed_block_id(sealed: &[u8]) -> [u8; 32] {
    Sha256::digest(sealed).into()
}

// ============ ArchiveStore — локальная иерархия «Монтана/Чаты/<имя чата>/» ============
// Зеркалит структуру приложения: <base>/Чаты/<имя>/переписка.mtlog + <base>/Чаты/<имя>/Медиа/<blob_id>.
// Структура (имена чатов) навигабельна пользователем; содержимое лога — sealed (at-rest, нужен сид).

pub const CHATS_DIR: &str = "Чаты";
pub const MEDIA_DIR: &str = "Медиа";
pub const LOG_FILE: &str = "переписка.mtlog";

pub struct ArchiveStore {
    base: PathBuf,
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '\0' => '_',
            c => c,
        })
        .collect();
    let t = cleaned.trim().trim_matches('.').trim();
    if t.is_empty() {
        "_".to_string()
    } else {
        t.to_string()
    }
}

impl ArchiveStore {
    pub fn open(base: impl AsRef<Path>) -> io::Result<Self> {
        let base = base.as_ref().to_path_buf();
        fs::create_dir_all(base.join(CHATS_DIR))?;
        Ok(Self { base })
    }

    fn chat_dir(&self, chat_name: &str) -> PathBuf {
        self.base.join(CHATS_DIR).join(sanitize(chat_name))
    }

    /// Дописать sealed-блок в лог чата: <base>/Чаты/<имя>/переписка.mtlog (length-prefixed u32 LE ‖ sealed).
    pub fn append_block(&self, chat_name: &str, sealed_block: &[u8]) -> io::Result<()> {
        let dir = self.chat_dir(chat_name);
        fs::create_dir_all(&dir)?;
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(LOG_FILE))?;
        f.write_all(&(sealed_block.len() as u32).to_le_bytes())?;
        f.write_all(sealed_block)?;
        Ok(())
    }

    /// Прочитать все sealed-блоки лога чата по порядку.
    pub fn read_blocks(&self, chat_name: &str) -> io::Result<Vec<Vec<u8>>> {
        let path = self.chat_dir(chat_name).join(LOG_FILE);
        let data = match fs::read(&path) {
            Ok(d) => d,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e),
        };
        let mut out = Vec::new();
        let mut off = 0usize;
        while off + 4 <= data.len() {
            let len = u32::from_le_bytes(data[off..off + 4].try_into().unwrap()) as usize;
            off += 4;
            if off + len > data.len() {
                break;
            }
            out.push(data[off..off + len].to_vec());
            off += len;
        }
        Ok(out)
    }

    /// Положить медиа-блоб: <base>/Чаты/<имя>/Медиа/<blob_id_hex>.
    /// Зашифровать медиа под history_key и положить в <base>/Чаты/<имя>/Медиа/<blob_id>.
    /// Другие приложения видят только шифртекст; расшифровывает только клиент по сиду.
    pub fn put_media(
        &self,
        chat_name: &str,
        blob_id_hex: &str,
        history_key: &[u8; 32],
        account_id: &[u8; 32],
        plaintext: &[u8],
    ) -> io::Result<()> {
        let mdir = self.chat_dir(chat_name).join(MEDIA_DIR);
        fs::create_dir_all(&mdir)?;
        let sealed = seal_media(history_key, account_id, blob_id_hex, plaintext);
        fs::write(mdir.join(sanitize(blob_id_hex)), sealed)
    }

    /// Прочитать и расшифровать медиа. None если нет файла либо расшифровка не прошла.
    pub fn get_media(
        &self,
        chat_name: &str,
        blob_id_hex: &str,
        history_key: &[u8; 32],
        account_id: &[u8; 32],
    ) -> Option<Vec<u8>> {
        let path = self.chat_dir(chat_name).join(MEDIA_DIR).join(sanitize(blob_id_hex));
        let sealed = fs::read(path).ok()?;
        open_media(history_key, account_id, &sealed)
    }

    pub fn media_path(&self, chat_name: &str, blob_id_hex: &str) -> PathBuf {
        self.chat_dir(chat_name)
            .join(MEDIA_DIR)
            .join(sanitize(blob_id_hex))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_block() -> HistoryBlock {
        HistoryBlock {
            block_seq: 0,
            items: vec![HistoryItem {
                conv_id: [0x22u8; 32],
                dir: DIR_OUT,
                send_time: 1000,
                content: b"montana".to_vec(),
            }],
        }
    }

    #[test]
    fn history_key_kat() {
        let hk = history_key(&[0x55u8; 32]);
        assert_eq!(
            hex::encode(hk),
            "e6a7dc51003770589d9f731c1231c1523be7348c7769383875dd34bd6c578def"
        );
    }

    #[test]
    fn block_roundtrip() {
        let b = sample_block();
        let enc = encode_block(&b);
        assert_eq!(decode_block(&enc), Some(b));
    }

    #[test]
    fn decode_trailing_rejected() {
        let mut enc = encode_block(&sample_block());
        enc.push(0xff);
        assert_eq!(decode_block(&enc), None);
    }

    #[test]
    fn decode_truncated_rejected() {
        let enc = encode_block(&sample_block());
        assert_eq!(decode_block(&enc[..enc.len() - 1]), None);
    }

    #[test]
    fn seal_open_roundtrip() {
        let hk = history_key(&[0x55u8; 32]);
        let acct = [0x33u8; 32];
        let b = sample_block();
        let sealed = seal_block(&hk, &acct, &b);
        assert_eq!(open_block(&hk, &acct, &sealed), Some(b));
    }

    #[test]
    fn seal_deterministic() {
        let hk = history_key(&[0x55u8; 32]);
        let acct = [0x33u8; 32];
        let b = sample_block();
        assert_eq!(seal_block(&hk, &acct, &b), seal_block(&hk, &acct, &b));
    }

    #[test]
    fn open_wrong_account_fails() {
        let hk = history_key(&[0x55u8; 32]);
        let b = sample_block();
        let sealed = seal_block(&hk, &[0x33u8; 32], &b);
        assert_eq!(open_block(&hk, &[0x44u8; 32], &sealed), None);
    }

    #[test]
    fn history_block_kat() {
        let hk = history_key(&[0x55u8; 32]);
        let acct = [0x33u8; 32];
        let sealed = seal_block(&hk, &acct, &sample_block());
        // KAT: SHA-256(sealed) детерминирован (nonce из block_seq).
        // block = header(12) + item(conv32+dir1+time8+len4+"montana"7 = 52) = 64; sealed = nonce12 + ct(64+16) = 92
        assert_eq!(sealed.len(), 12 + 64 + 16);
        let id = hex::encode(sealed_block_id(&sealed));
        assert_eq!(
            id,
            "3fed8b1489157d0ec1a2dbb0d291e5c201627d5133f76da40eff12a386b2a17d"
        );
    }

    #[test]
    fn store_hierarchy_roundtrip() {
        let tmp = std::env::temp_dir().join("mt_archive_test_store");
        let _ = fs::remove_dir_all(&tmp);
        let st = ArchiveStore::open(&tmp).unwrap();
        let hk = history_key(&[0x55u8; 32]);
        let acct = [0x33u8; 32];

        let sealed = seal_block(&hk, &acct, &sample_block());
        st.append_block("Алиса", &sealed).unwrap();
        st.put_media("Алиса", "6c385ae2blob", &hk, &acct, b"\x89PNG demo")
            .unwrap();

        // иерархия появилась
        assert!(tmp.join("Чаты").join("Алиса").join(LOG_FILE).exists());
        assert!(tmp
            .join("Чаты")
            .join("Алиса")
            .join(MEDIA_DIR)
            .join("6c385ae2blob")
            .exists());

        // лог читается обратно и расшифровывается
        let blocks = st.read_blocks("Алиса").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(open_block(&hk, &acct, &blocks[0]), Some(sample_block()));

        let _ = fs::remove_dir_all(&tmp);
    }
}
