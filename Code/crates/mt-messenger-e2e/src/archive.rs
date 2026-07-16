//! Этап 1 (второй фронт — децентрализация данных) — архив истории.
//! HistoryBlock = block_seq u64 LE ‖ item_count u32 LE
//!   ‖ [ conv_id 32 ‖ dir 1 ‖ send_time u64 LE ‖ content_len u32 LE ‖ Content ]×item_count
//! sealed = nonce(block_seq_le8 ‖ 0x00000000) ‖ ChaCha20-Poly1305(history_key, nonce, block,
//!                                                                 AD = "mt-history" ‖ 0x00 ‖ account_id).
//! history_key = HKDF-SHA-256(0×32, entropy_32, "mt-history-key", 32) — SSOT (первый фронт, Этап 12).
//! media_key   = HKDF-SHA-256(0×32, entropy_32, "mt-media-key", 32)  — отдельная ветвь сида (≠ history_key).
//! block_seq — сквозной счётчик per-личность (seq.bin в base), не per-чат: иначе повтор nonce (spec s.2 v0.3.1).
//! Разбор инвалид-безопасен (Gate 13): любое нарушение → None, НИКОГДА паника.

use crate::kdf::hkdf_sha256;
use crate::ratchet::{open, seal};
use mt_codec::domain::{MSG_HISTORY, MSG_HISTORY_KEY, MSG_MEDIA_KEY, MSG_MEDIA_VAULT};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const CONV_ID_LEN: usize = 32;
pub const DIR_OUT: u8 = 0x00;
pub const DIR_IN: u8 = 0x01;
pub const SEQ_FILE: &str = "seq.bin";

pub fn history_key(entropy_32: &[u8; 32]) -> [u8; 32] {
    let k = hkdf_sha256(&[0u8; 32], entropy_32, MSG_HISTORY_KEY, 32);
    let mut out = [0u8; 32];
    out.copy_from_slice(&k);
    out
}

/// Отдельная ветвь сида для медиа at-rest — не пересекается с history_key (разные nonce-пространства).
pub fn media_key(entropy_32: &[u8; 32]) -> [u8; 32] {
    let k = hkdf_sha256(&[0u8; 32], entropy_32, MSG_MEDIA_KEY, 32);
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
    let mut ad = MSG_HISTORY.to_vec();
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

fn media_ad(account_id: &[u8; 32]) -> Vec<u8> {
    let mut ad = MSG_MEDIA_VAULT.to_vec();
    ad.push(0x00);
    ad.extend_from_slice(account_id);
    ad
}

fn media_nonce(blob_ref: &str) -> [u8; 12] {
    let h: [u8; 32] = Sha256::digest(blob_ref.as_bytes()).into();
    let mut n = [0u8; 12];
    n.copy_from_slice(&h[..12]);
    n
}

/// Запечатать медиа под media_key (≠ history_key). blob_ref — уникальная метка блоба → уникальный nonce.
pub fn seal_media(
    media_key: &[u8; 32],
    account_id: &[u8; 32],
    blob_ref: &str,
    plaintext: &[u8],
) -> Vec<u8> {
    let nonce = media_nonce(blob_ref);
    let ct = seal(media_key, &nonce, plaintext, &media_ad(account_id));
    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

pub fn open_media(media_key: &[u8; 32], account_id: &[u8; 32], sealed: &[u8]) -> Option<Vec<u8>> {
    if sealed.len() < 12 {
        return None;
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&sealed[..12]);
    open(media_key, &nonce, &sealed[12..], &media_ad(account_id))
}

pub fn sealed_block_id(sealed: &[u8]) -> [u8; 32] {
    Sha256::digest(sealed).into()
}

// ============ ArchiveStore — локальная иерархия «Монтана/Чаты/<имя чата>/» ============
// Зеркалит структуру приложения: <base>/Чаты/<ярлык>/переписка.mtlog + <base>/Чаты/<ярлык>/Медиа/<blob_ref>.
// Ярлык (имя папки) навигабелен пользователем; содержимое лога — sealed (at-rest, нужен сид).
// Сквозной счётчик block_seq хранится в <base>/seq.bin (per-личность, не per-чат).

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

    /// Назначить следующий сквозной block_seq (per-личность): читает seq.bin, возвращает текущее,
    /// сохраняет current+1 атомарно (write tmp → rename). Первый вызов → 0.
    pub fn next_block_seq(&self) -> io::Result<u64> {
        let path = self.base.join(SEQ_FILE);
        let cur = match fs::read(&path) {
            Ok(d) if d.len() >= 8 => u64::from_le_bytes(d[..8].try_into().unwrap()),
            _ => 0,
        };
        let next = cur
            .checked_add(1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "block_seq overflow"))?;
        let tmp = self.base.join("seq.bin.tmp");
        fs::write(&tmp, next.to_le_bytes())?;
        fs::rename(&tmp, &path)?;
        Ok(cur)
    }

    /// Единая точка записи одного элемента в архив: ядро назначает block_seq (SSOT счётчика),
    /// строит одноэлементный HistoryBlock, seal под history_key, дописывает в лог чата.
    /// Возвращает назначенный block_seq.
    #[allow(clippy::too_many_arguments)]
    pub fn append_item(
        &self,
        chat_name: &str,
        history_key: &[u8; 32],
        account_id: &[u8; 32],
        conv_id: &[u8; 32],
        dir: u8,
        send_time: u64,
        content: &[u8],
    ) -> io::Result<u64> {
        let seq = self.next_block_seq()?;
        let block = HistoryBlock {
            block_seq: seq,
            items: vec![HistoryItem {
                conv_id: *conv_id,
                dir,
                send_time,
                content: content.to_vec(),
            }],
        };
        let sealed = seal_block(history_key, account_id, &block);
        self.append_block(chat_name, &sealed)?;
        Ok(seq)
    }

    /// Дописать sealed-блок в лог чата: <base>/Чаты/<ярлык>/переписка.mtlog (length-prefixed u32 LE ‖ sealed).
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

    /// Зашифровать медиа под media_key и положить в <base>/Чаты/<ярлык>/Медиа/<blob_ref>.
    /// Другие приложения видят только шифртекст; расшифровывает только клиент по сиду.
    pub fn put_media(
        &self,
        chat_name: &str,
        blob_ref: &str,
        media_key: &[u8; 32],
        account_id: &[u8; 32],
        plaintext: &[u8],
    ) -> io::Result<()> {
        let mdir = self.chat_dir(chat_name).join(MEDIA_DIR);
        fs::create_dir_all(&mdir)?;
        let sealed = seal_media(media_key, account_id, blob_ref, plaintext);
        fs::write(mdir.join(sanitize(blob_ref)), sealed)
    }

    /// Прочитать и расшифровать медиа. None если нет файла либо расшифровка не прошла.
    pub fn get_media(
        &self,
        chat_name: &str,
        blob_ref: &str,
        media_key: &[u8; 32],
        account_id: &[u8; 32],
    ) -> Option<Vec<u8>> {
        let path = self
            .chat_dir(chat_name)
            .join(MEDIA_DIR)
            .join(sanitize(blob_ref));
        let sealed = fs::read(path).ok()?;
        open_media(media_key, account_id, &sealed)
    }

    pub fn media_path(&self, chat_name: &str, blob_ref: &str) -> PathBuf {
        self.chat_dir(chat_name)
            .join(MEDIA_DIR)
            .join(sanitize(blob_ref))
    }

    /// Миграция папки чата при переименовании ярлыка (идентичность переписки = conv_address, не ярлык).
    pub fn rename_chat(&self, old_label: &str, new_label: &str) -> io::Result<()> {
        let so = sanitize(old_label);
        let sn = sanitize(new_label);
        if so == sn {
            return Ok(());
        }
        let src = self.base.join(CHATS_DIR).join(&so);
        let dst = self.base.join(CHATS_DIR).join(&sn);
        if src.exists() && !dst.exists() {
            fs::rename(&src, &dst)?;
        }
        Ok(())
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
    fn media_key_differs_from_history_key() {
        let ent = [0x55u8; 32];
        assert_ne!(
            history_key(&ent),
            media_key(&ent),
            "ветви ключей обязаны различаться"
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
    fn media_seal_open_roundtrip() {
        let mk = media_key(&[0x55u8; 32]);
        let acct = [0x33u8; 32];
        let sealed = seal_media(&mk, &acct, "voice_abc", b"\x89PNG demo");
        assert_eq!(
            open_media(&mk, &acct, &sealed),
            Some(b"\x89PNG demo".to_vec())
        );
    }

    #[test]
    fn media_wrong_key_fails() {
        let mk = media_key(&[0x55u8; 32]);
        let hk = history_key(&[0x55u8; 32]);
        let acct = [0x33u8; 32];
        let sealed = seal_media(&mk, &acct, "voice_abc", b"secret");
        // history_key НЕ открывает media (раздельные ветви)
        assert_eq!(open_media(&hk, &acct, &sealed), None);
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
    fn seq_counter_monotonic_across_chats() {
        let tmp = std::env::temp_dir().join("mt_archive_seq_test");
        let _ = fs::remove_dir_all(&tmp);
        let st = ArchiveStore::open(&tmp).unwrap();
        // сквозной счётчик: разные чаты не сбрасывают seq
        assert_eq!(st.next_block_seq().unwrap(), 0);
        assert_eq!(st.next_block_seq().unwrap(), 1);
        assert_eq!(st.next_block_seq().unwrap(), 2);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn append_item_assigns_global_seq() {
        let tmp = std::env::temp_dir().join("mt_archive_append_item_test");
        let _ = fs::remove_dir_all(&tmp);
        let st = ArchiveStore::open(&tmp).unwrap();
        let hk = history_key(&[0x55u8; 32]);
        let acct = [0x33u8; 32];
        let conv_a = [0x01u8; 32];
        let conv_b = [0x02u8; 32];
        // чат A первое сообщение → seq 0; чат B первое → seq 1 (НЕ 0 — нет повтора nonce)
        let s0 = st
            .append_item("Алиса", &hk, &acct, &conv_a, DIR_OUT, 1000, b"hi")
            .unwrap();
        let s1 = st
            .append_item("Боб", &hk, &acct, &conv_b, DIR_OUT, 1001, b"yo")
            .unwrap();
        assert_eq!(s0, 0);
        assert_eq!(s1, 1);
        // прочитать блок чата B — seq 1
        let blocks_b = st.read_blocks("Боб").unwrap();
        assert_eq!(blocks_b.len(), 1);
        let blk = open_block(&hk, &acct, &blocks_b[0]).unwrap();
        assert_eq!(blk.block_seq, 1);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn store_hierarchy_roundtrip() {
        let tmp = std::env::temp_dir().join("mt_archive_test_store");
        let _ = fs::remove_dir_all(&tmp);
        let st = ArchiveStore::open(&tmp).unwrap();
        let hk = history_key(&[0x55u8; 32]);
        let mk = media_key(&[0x55u8; 32]);
        let acct = [0x33u8; 32];

        let sealed = seal_block(&hk, &acct, &sample_block());
        st.append_block("Алиса", &sealed).unwrap();
        st.put_media("Алиса", "voice_ref", &mk, &acct, b"\x89PNG demo")
            .unwrap();

        // иерархия появилась
        assert!(tmp.join("Чаты").join("Алиса").join(LOG_FILE).exists());
        assert!(tmp
            .join("Чаты")
            .join("Алиса")
            .join(MEDIA_DIR)
            .join("voice_ref")
            .exists());

        // лог читается обратно и расшифровывается
        let blocks = st.read_blocks("Алиса").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(open_block(&hk, &acct, &blocks[0]), Some(sample_block()));

        // медиа читается под media_key
        assert_eq!(
            st.get_media("Алиса", "voice_ref", &mk, &acct),
            Some(b"\x89PNG demo".to_vec())
        );

        // миграция папки при переименовании
        st.rename_chat("Алиса", "Алиса Смит").unwrap();
        assert!(tmp.join("Чаты").join("Алиса Смит").join(LOG_FILE).exists());
        assert_eq!(
            st.get_media("Алиса Смит", "voice_ref", &mk, &acct),
            Some(b"\x89PNG demo".to_vec())
        );

        let _ = fs::remove_dir_all(&tmp);
    }
}
