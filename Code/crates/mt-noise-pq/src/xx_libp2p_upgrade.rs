//! Async drive functions for the Noise_PQ XX-pattern handshake over any
//! `AsyncRead + AsyncWrite` socket. Library-only — no libp2p types.
//! libp2p UpgradeInfo / Inbound / OutboundConnectionUpgrade trait impls live
//! in `mt-net-transport/src/xx_noise_pq_upgrade.rs` to keep libp2p deps out
//! of mt-noise-pq.

use crate::stream::NoisePqStream;
use crate::xx_handshake::{
    initiator_receive_msg2, initiator_send_msg1, initiator_send_msg3, responder_receive_msg1,
    responder_receive_msg3, responder_send_msg2, XxSession, XX_MSG1_SIZE, XX_MSG2_SIZE,
    XX_MSG3_SIZE,
};
use crate::NoisePqError;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use futures::{AsyncRead, AsyncWrite};
use mt_crypto::{PublicKey, SecretKey};

pub const XX_PROTOCOL_NAME: &str = "/montana/noise-pq-xx/1.0.0";

#[derive(Debug)]
pub enum XxUpgradeError {
    Io(std::io::Error),
    Handshake(NoisePqError),
}

impl std::fmt::Display for XxUpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XxUpgradeError::Io(e) => write!(f, "io: {e}"),
            XxUpgradeError::Handshake(e) => write!(f, "handshake: {e:?}"),
        }
    }
}

impl std::error::Error for XxUpgradeError {}

impl From<std::io::Error> for XxUpgradeError {
    fn from(e: std::io::Error) -> Self {
        XxUpgradeError::Io(e)
    }
}

impl From<NoisePqError> for XxUpgradeError {
    fn from(e: NoisePqError) -> Self {
        XxUpgradeError::Handshake(e)
    }
}

/// Drive initiator side of XX handshake: send msg1, recv msg2, send msg3.
/// Returns the final session and the AEAD-wrapped stream.
pub async fn xx_initiator_drive<C>(
    mut socket: C,
    id_pk: PublicKey,
    id_sk: SecretKey,
) -> Result<(XxSession, NoisePqStream<C>), XxUpgradeError>
where
    C: AsyncRead + AsyncWrite + Unpin,
{
    let (msg1, init_after_msg1) = initiator_send_msg1(id_sk, id_pk)?;
    socket.write_all(&msg1).await?;
    socket.flush().await?;

    let mut msg2 = vec![0u8; XX_MSG2_SIZE];
    socket.read_exact(&mut msg2).await?;
    let init_after_msg2 = initiator_receive_msg2(&msg2, init_after_msg1)?;

    let (msg3, session) = initiator_send_msg3(init_after_msg2)?;
    socket.write_all(&msg3).await?;
    socket.flush().await?;

    let sk_tx = session.sk_i_to_r;
    let sk_rx = session.sk_r_to_i;
    let stream = NoisePqStream::new(socket, sk_tx, sk_rx);
    Ok((session, stream))
}

/// Drive responder side of XX handshake: recv msg1, send msg2, recv msg3.
/// Returns the final session and the AEAD-wrapped stream.
pub async fn xx_responder_drive<C>(
    mut socket: C,
    id_pk: PublicKey,
    id_sk: SecretKey,
) -> Result<(XxSession, NoisePqStream<C>), XxUpgradeError>
where
    C: AsyncRead + AsyncWrite + Unpin,
{
    let mut msg1 = vec![0u8; XX_MSG1_SIZE];
    socket.read_exact(&mut msg1).await?;
    let resp_after_msg1 = responder_receive_msg1(&msg1, id_sk, id_pk)?;

    let (msg2, resp_after_msg2) = responder_send_msg2(resp_after_msg1)?;
    socket.write_all(&msg2).await?;
    socket.flush().await?;

    let mut msg3 = vec![0u8; XX_MSG3_SIZE];
    socket.read_exact(&mut msg3).await?;
    let session = responder_receive_msg3(&msg3, resp_after_msg2)?;

    let sk_tx = session.sk_r_to_i;
    let sk_rx = session.sk_i_to_r;
    let stream = NoisePqStream::new(socket, sk_tx, sk_rx);
    Ok((session, stream))
}
