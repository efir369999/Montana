//! Этап 12 — медиа: запечатанный блоб (content-addressed) + MediaRef (Content 0x05).
//! Крупный контент не влезает в храповик (MAX_PLAINTEXT), поэтому шифруется отдельным
//! блобом (сервер видит только шифр), а в храповик едет компактная ссылка MediaRef.

use sha2::{Digest, Sha256};

use crate::ratchet::{open, seal};

/// AD блоба: "mt-media" (8 B) || 0x00.
pub const MEDIA_AD: &[u8] = b"mt-media\x00";

pub const MAX_BLOB: u64 = 2_147_483_648; // 2 GiB — логический потолок файла
pub const MAX_BLOB_CHUNK: usize = 16_777_216; // 16 MiB — единица заливки/аллокации/PoT
pub const THUMB_MAX: usize = 16_384; // 16 KiB
pub const BLOB_POT_STEPS: u32 = 1_048_576;

pub const MEDIA_IMAGE: u8 = 0x01;
pub const MEDIA_VIDEO: u8 = 0x02;
pub const MEDIA_FILE: u8 = 0x03;
pub const MEDIA_AUDIO: u8 = 0x04;
pub const MEDIA_STICKER: u8 = 0x05;

/// pad_len(n): скрывает точный размер (overhead < 1/16). bit_length — позиция старшего бита.
pub fn pad_len(n: usize) -> usize {
    if n < 256 {
        return 256;
    }
    let bl = usize::BITS - n.leading_zeros(); // bit_length(n)
    let step = 1usize << (bl - 5);
    // ceiling division (n + step - 1) / step — MSRV 1.70 (div_ceil стабилен лишь с 1.73);
    // переполнение невозможно: n ≤ MAX_PLAINTEXT (1 MiB) ≪ usize::MAX - step
    ((n + step - 1) / step) * step
}

/// sealed_blob = nonce || ChaCha20-Poly1305.Seal(blob_key, nonce, input, AD="mt-media"||0x00).
/// `input` — уже финальный вход (в production паддится через pad_len до вызова; в KAT — сырой).
pub fn seal_blob(blob_key: &[u8; 32], nonce: &[u8; 12], input: &[u8]) -> Vec<u8> {
    let body = seal(blob_key, nonce, input, MEDIA_AD);
    let mut out = Vec::with_capacity(12 + body.len());
    out.extend_from_slice(nonce);
    out.extend_from_slice(&body);
    out
}

/// blob_id = SHA-256(sealed_blob) — контент-адресация (целостность).
pub fn blob_id(sealed_blob: &[u8]) -> [u8; 32] {
    Sha256::digest(sealed_blob).into()
}

/// Расшифровать блоб -> padded plaintext (вызывающий усекает до MediaRef.size).
pub fn open_blob(blob_key: &[u8; 32], sealed_blob: &[u8]) -> Option<Vec<u8>> {
    if sealed_blob.len() < 12 {
        return None;
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&sealed_blob[..12]);
    open(blob_key, &nonce, &sealed_blob[12..], MEDIA_AD)
}

/// MediaRef — тело Content типа 0x05 (едет внутри храповика, E2E).
/// content_type(0x05) || msg_id(16) || sent_at(u64 LE) || media_kind(1) || blob_id(32) ||
/// blob_key(32) || size(u64 LE) || mime_len(1) || mime || name_len(1) || name || thumb_len(u16 LE) || thumb.
#[allow(clippy::too_many_arguments)]
pub fn encode_media_content(
    msg_id: &[u8; 16],
    sent_at: u64,
    media_kind: u8,
    blob_id: &[u8; 32],
    blob_key: &[u8; 32],
    size: u64,
    mime: &[u8],
    name: &[u8],
    thumb: &[u8],
) -> Vec<u8> {
    let mut v = Vec::with_capacity(
        1 + 16 + 8 + 1 + 32 + 32 + 8 + 1 + mime.len() + 1 + name.len() + 2 + thumb.len(),
    );
    v.push(0x05);
    v.extend_from_slice(msg_id);
    v.extend_from_slice(&sent_at.to_le_bytes());
    v.push(media_kind);
    v.extend_from_slice(blob_id);
    v.extend_from_slice(blob_key);
    v.extend_from_slice(&size.to_le_bytes());
    v.push(mime.len() as u8);
    v.extend_from_slice(mime);
    v.push(name.len() as u8);
    v.extend_from_slice(name);
    v.extend_from_slice(&(thumb.len() as u16).to_le_bytes());
    v.extend_from_slice(thumb);
    v
}

pub struct MediaRef {
    pub msg_id: [u8; 16],
    pub sent_at: u64,
    pub media_kind: u8,
    pub blob_id: [u8; 32],
    pub blob_key: [u8; 32],
    pub size: u64,
    pub mime: Vec<u8>,
    pub name: Vec<u8>,
    pub thumb: Vec<u8>,
}

/// Разбор Content 0x05 (не паникует; инварианты — из спеки Этапа 12).
pub fn decode_media_content(b: &[u8]) -> Option<MediaRef> {
    let mut i = 0usize;
    let need = |i: usize, n: usize| -> Option<()> { (i + n <= b.len()).then_some(()) };
    need(i, 1)?;
    if b[i] != 0x05 {
        return None;
    }
    i += 1;
    need(i, 16)?;
    let mut msg_id = [0u8; 16];
    msg_id.copy_from_slice(&b[i..i + 16]);
    i += 16;
    need(i, 8)?;
    let sent_at = u64::from_le_bytes(b[i..i + 8].try_into().ok()?);
    i += 8;
    need(i, 1)?;
    let media_kind = b[i];
    i += 1;
    if !(0x01..=0x05).contains(&media_kind) {
        return None;
    }
    need(i, 32)?;
    let mut blob_id = [0u8; 32];
    blob_id.copy_from_slice(&b[i..i + 32]);
    i += 32;
    need(i, 32)?;
    let mut blob_key = [0u8; 32];
    blob_key.copy_from_slice(&b[i..i + 32]);
    i += 32;
    need(i, 8)?;
    let size = u64::from_le_bytes(b[i..i + 8].try_into().ok()?);
    i += 8;
    if size > MAX_BLOB {
        return None;
    }
    need(i, 1)?;
    let mime_len = b[i] as usize;
    i += 1;
    need(i, mime_len)?;
    let mime = b[i..i + mime_len].to_vec();
    i += mime_len;
    need(i, 1)?;
    let name_len = b[i] as usize;
    i += 1;
    need(i, name_len)?;
    let name = b[i..i + name_len].to_vec();
    i += name_len;
    need(i, 2)?;
    let thumb_len = u16::from_le_bytes(b[i..i + 2].try_into().ok()?) as usize;
    i += 2;
    if thumb_len > THUMB_MAX {
        return None;
    }
    need(i, thumb_len)?;
    let thumb = b[i..i + thumb_len].to_vec();
    i += thumb_len;
    if i != b.len() {
        return None;
    }
    Some(MediaRef {
        msg_id,
        sent_at,
        media_kind,
        blob_id,
        blob_key,
        size,
        mime,
        name,
        thumb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    // media_blob_kat: blob_key=66×32, nonce=00×12, plaintext="montana-media", AD="mt-media"||0x00.
    #[test]
    fn media_blob_kat() {
        let sealed = seal_blob(&[0x66; 32], &[0u8; 12], b"montana-media");
        // sealed = nonce(12) || sealed_body(29); проверяем sealed_body и blob_id.
        assert_eq!(
            hex(&sealed[12..]),
            "e26a877f209a12646c4e630e0a6705598d68389e621357aee335b7d636"
        );
        assert_eq!(
            hex(&blob_id(&sealed)),
            "6c385ae2ef1c472b373a77e582c889d7ed2585c5a036c246b580f05f94c7efd3"
        );
        // round-trip
        assert_eq!(open_blob(&[0x66; 32], &sealed).unwrap(), b"montana-media");
    }

    // media_content_kat.
    #[test]
    fn media_content_kat() {
        let blob = [
            0x6cu8, 0x38, 0x5a, 0xe2, 0xef, 0x1c, 0x47, 0x2b, 0x37, 0x3a, 0x77, 0xe5, 0x82, 0xc8,
            0x89, 0xd7, 0xed, 0x25, 0x85, 0xc5, 0xa0, 0x36, 0xc2, 0x46, 0xb5, 0x80, 0xf0, 0x5f,
            0x94, 0xc7, 0xef, 0xd3,
        ];
        let content = encode_media_content(
            &[0x11; 16],
            1000,
            MEDIA_IMAGE,
            &blob,
            &[0x66; 32],
            13,
            b"image/png",
            b"a.png",
            &[],
        );
        assert_eq!(
            hex(&content),
            "0511111111111111111111111111111111e803000000000000016c385ae2ef1c472b373a77e582c889d7ed2585c5a036c246b580f05f94c7efd366666666666666666666666666666666666666666666666666666666666666660d0000000000000009696d6167652f706e6705612e706e670000"
        );
        // round-trip decode
        let r = decode_media_content(&content).unwrap();
        assert_eq!(r.media_kind, MEDIA_IMAGE);
        assert_eq!(r.size, 13);
        assert_eq!(r.mime, b"image/png");
        assert_eq!(r.name, b"a.png");
        assert_eq!(r.blob_id, blob);
    }

    #[test]
    fn pad_len_buckets() {
        assert_eq!(pad_len(0), 256);
        assert_eq!(pad_len(255), 256);
        assert_eq!(pad_len(256), 256); // bit_length(256)=9, step=1<<4=16, ceil(256/16)*16=256
        assert!(pad_len(1000) >= 1000 && pad_len(1000) < 1000 + 1000 / 16 + 64);
    }
}
