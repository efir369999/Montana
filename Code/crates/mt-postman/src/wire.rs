//! Чтение/запись по QUIC-потоку (Этап 1). Пролог регистрации — фиксированные размеры
//! (read_exact); OverlayFrame — один uni-поток на фрейм (write_all + finish, read_to_end).

use quinn::{RecvStream, SendStream};
use thiserror::Error;

use mt_codec::domain::OVERLAY_CHANNEL_LABEL;
use mt_overlay::frame::MAX_PAYLOAD_LEN;

/// Верхний предел на один OverlayFrame по проводу: header 86 B + payload cap + запас.
pub const MAX_FRAME_WIRE: usize = MAX_PAYLOAD_LEN + 4096;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("write: {0}")]
    Write(#[from] quinn::WriteError),
    #[error("read_exact: {0}")]
    ReadExact(#[from] quinn::ReadExactError),
    #[error("read_to_end: {0}")]
    ReadToEnd(#[from] quinn::ReadToEndError),
    #[error("stream already closed")]
    Closed,
    #[error("TLS-Exporter (channel_hash) failed")]
    Export,
}

/// Записать фиксированный блок в persistent bi-поток (пролог), без finish.
pub async fn write_fixed(s: &mut SendStream, bytes: &[u8]) -> Result<(), WireError> {
    s.write_all(bytes).await?;
    Ok(())
}

/// Прочитать ровно `buf.len()` байт из persistent bi-потока (пролог).
pub async fn read_fixed(s: &mut RecvStream, buf: &mut [u8]) -> Result<(), WireError> {
    s.read_exact(buf).await?;
    Ok(())
}

/// Один uni-поток = один фрейм: записать и закрыть (FIN).
pub async fn send_frame(s: &mut SendStream, bytes: &[u8]) -> Result<(), WireError> {
    s.write_all(bytes).await?;
    s.finish().map_err(|_| WireError::Closed)?;
    Ok(())
}

/// Прочитать фрейм целиком до FIN (uni-поток).
pub async fn recv_frame(s: &mut RecvStream) -> Result<Vec<u8>, WireError> {
    Ok(s.read_to_end(MAX_FRAME_WIRE).await?)
}

/// channel_hash соединения = TLS-Exporter(OVERLAY_CHANNEL_LABEL, "", 32).
/// Метка — SSOT из mt_codec::domain (не дублируется literal-ом; [C-1]/[I-10]).
/// Обе стороны QUIC выводят одинаковые 32 B → привязка RegProof к каналу (R4);
/// спека 0.8.1 (RFC 8446 §7.5 / RFC 9266-паттерн).
pub fn channel_hash(conn: &quinn::Connection) -> Result<[u8; 32], WireError> {
    let mut ch = [0u8; 32];
    conn.export_keying_material(&mut ch, OVERLAY_CHANNEL_LABEL, b"")
        .map_err(|_| WireError::Export)?;
    Ok(ch)
}
