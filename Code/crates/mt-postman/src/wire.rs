//! Чтение/запись по TCP+TLS-потоку (Этап 1; спека §152 — «TCP/TLS-443 обязателен»,
//! операторы режут non-443 UDP). Пролог/фиксированные ответы — read_exact/write_all;
//! переменные сообщения — длина-префикс u32 BE (TCP не несёт per-stream FIN как QUIC,
//! граница сообщения — явная длина). Request-response последователен на одном дуплексном
//! потоке: сторона пишет запрос, затем читает ответ на том же `&mut S`.

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use mt_codec::domain::OVERLAY_CHANNEL_LABEL;
use mt_overlay::frame::MAX_PAYLOAD_LEN;

/// Верхний предел на одно сообщение по проводу: header + payload cap + запас.
pub const MAX_FRAME_WIRE: usize = MAX_PAYLOAD_LEN + 4096;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("frame too large: {0}")]
    TooLarge(usize),
    #[error("stream already closed")]
    Closed,
    #[error("TLS-Exporter (channel_hash) failed")]
    Export,
}

/// Записать фиксированный блок (пролог/тег/фиксированный ответ) + flush.
pub async fn write_fixed<W: AsyncWrite + Unpin + ?Sized>(
    s: &mut W,
    bytes: &[u8],
) -> Result<(), WireError> {
    s.write_all(bytes).await?;
    s.flush().await?;
    Ok(())
}

/// Прочитать ровно `buf.len()` байт (пролог/фиксированный ответ).
pub async fn read_fixed<R: AsyncRead + Unpin + ?Sized>(
    s: &mut R,
    buf: &mut [u8],
) -> Result<(), WireError> {
    s.read_exact(buf).await?;
    Ok(())
}

/// Одно сообщение = [u32 BE len][bytes]. Записать + flush (ответ идёт следом на том же потоке).
pub async fn send_frame<W: AsyncWrite + Unpin + ?Sized>(
    s: &mut W,
    bytes: &[u8],
) -> Result<(), WireError> {
    if bytes.len() > MAX_FRAME_WIRE {
        return Err(WireError::TooLarge(bytes.len()));
    }
    s.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
    s.write_all(bytes).await?;
    s.flush().await?;
    Ok(())
}

/// Прочитать сообщение [u32 BE len][bytes].
pub async fn recv_frame<R: AsyncRead + Unpin + ?Sized>(s: &mut R) -> Result<Vec<u8>, WireError> {
    let mut lb = [0u8; 4];
    s.read_exact(&mut lb).await?;
    let len = u32::from_be_bytes(lb) as usize;
    if len > MAX_FRAME_WIRE {
        return Err(WireError::TooLarge(len));
    }
    let mut buf = vec![0u8; len];
    s.read_exact(&mut buf).await?;
    Ok(buf)
}

/// channel_hash соединения = TLS-Exporter(OVERLAY_CHANNEL_LABEL, no-context, 32).
/// Обе стороны TLS 1.3 выводят одинаковые 32 B → привязка RegProof к каналу (R4);
/// спека 0.8.1 (RFC 8446 §7.5). Аргумент — rustls-соединение (client/server), общий у
/// tokio-rustls через `TlsStream::get_ref().1`.
pub fn channel_hash<D>(conn: &rustls::ConnectionCommon<D>) -> Result<[u8; 32], WireError> {
    conn.export_keying_material([0u8; 32], OVERLAY_CHANNEL_LABEL, None)
        .map_err(|_| WireError::Export)
}
