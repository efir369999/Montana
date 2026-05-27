// Montana Egress layer — wire codecs.
// spec: Montana Egress v1.0.0 (Egress directory + Control messages).
//
// Application layer above Network. Defines no consensus state; the inner and
// outer transport sessions reuse Noise_PQ XX from the Network layer.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::vec::Vec;

pub mod exit;
pub use exit::{ExitPolicy, ExitSession, ExitSessionError, OpenOutcome};
pub mod client;
pub use client::{select_exit, ExitSelector};
pub mod relay;
pub use relay::{RelayBudget, RelayClass, RelayError, CONSENSUS_RELAY_CAP_BYTES_PER_SEC};

use mt_codec::{write_bytes, write_u16, write_u32, write_u8};
use mt_net::NetError;

/// Egress directory entry size: exit_node_id(32) + country_code(2)
/// + capacity_class(1) + advertised_window(4).
pub const EGRESS_DIRECTORY_ENTRY_SIZE: usize = 39;

/// Concurrent open streams an exit honours per inner session.
pub const MAX_STREAMS_PER_SESSION: u32 = 256;

/// Advisory egress directory bound per node.
pub const MAX_DIRECTORY_ENTRIES: usize = 4096;

fn is_iso_alpha(b: u8) -> bool {
    b.is_ascii_uppercase()
}

/// Advisory directory advertisement for an opt-in exit node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgressDirectoryEntry {
    pub exit_node_id: [u8; 32],
    pub country_code: [u8; 2],
    pub capacity_class: u8, // 0 best-effort, 1 standard, 2 high
    pub advertised_window: u32,
}

impl EgressDirectoryEntry {
    pub fn encode(&self, buf: &mut Vec<u8>) {
        write_bytes(buf, &self.exit_node_id);
        write_bytes(buf, &self.country_code);
        write_u8(buf, self.capacity_class);
        write_u32(buf, self.advertised_window);
    }

    pub fn decode(input: &[u8]) -> Result<Self, NetError> {
        if input.len() != EGRESS_DIRECTORY_ENTRY_SIZE {
            return Err(NetError::PayloadLengthMismatch);
        }
        let mut exit_node_id = [0u8; 32];
        exit_node_id.copy_from_slice(&input[0..32]);
        let mut country_code = [0u8; 2];
        country_code.copy_from_slice(&input[32..34]);
        let capacity_class = input[34];
        let mut win = [0u8; 4];
        win.copy_from_slice(&input[35..39]);
        let advertised_window = u32::from_le_bytes(win);

        if !is_iso_alpha(country_code[0]) || !is_iso_alpha(country_code[1]) {
            return Err(NetError::InvalidPayloadField);
        }
        if capacity_class > 2 {
            return Err(NetError::InvalidPayloadField);
        }
        Ok(EgressDirectoryEntry {
            exit_node_id,
            country_code,
            capacity_class,
            advertised_window,
        })
    }
}

/// Destination address for an egress stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressAddr {
    V4([u8; 4]),
    V6([u8; 16]),
    Host(Vec<u8>), // 1..=255 bytes
}

impl EgressAddr {
    fn addr_type(&self) -> u8 {
        match self {
            EgressAddr::V4(_) => 0,
            EgressAddr::V6(_) => 1,
            EgressAddr::Host(_) => 2,
        }
    }
    fn encode(&self, buf: &mut Vec<u8>) {
        match self {
            EgressAddr::V4(a) => write_bytes(buf, a),
            EgressAddr::V6(a) => write_bytes(buf, a),
            EgressAddr::Host(h) => {
                write_u8(buf, h.len() as u8);
                write_bytes(buf, h);
            },
        }
    }
}

/// Egress control / data message. Travels over the inner end-to-end session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressControl {
    Auth {
        proof: Vec<u8>,
    },
    Open {
        stream_id: u32,
        protocol: u8,
        addr: EgressAddr,
        dest_port: u16,
    },
    OpenAck {
        stream_id: u32,
        status: u8,
    },
    Data {
        stream_id: u32,
        payload: Vec<u8>,
    },
    Close {
        stream_id: u32,
        reason: u8,
    },
    Keepalive,
}

impl EgressControl {
    pub fn msg_type(&self) -> u8 {
        match self {
            EgressControl::Auth { .. } => 0x01,
            EgressControl::Open { .. } => 0x02,
            EgressControl::OpenAck { .. } => 0x03,
            EgressControl::Data { .. } => 0x04,
            EgressControl::Close { .. } => 0x05,
            EgressControl::Keepalive => 0x06,
        }
    }

    pub fn encode(&self, buf: &mut Vec<u8>) {
        write_u8(buf, self.msg_type());
        match self {
            EgressControl::Auth { proof } => write_bytes(buf, proof),
            EgressControl::Open {
                stream_id,
                protocol,
                addr,
                dest_port,
            } => {
                write_u32(buf, *stream_id);
                write_u8(buf, *protocol);
                write_u8(buf, addr.addr_type());
                addr.encode(buf);
                write_u16(buf, *dest_port);
            },
            EgressControl::OpenAck { stream_id, status } => {
                write_u32(buf, *stream_id);
                write_u8(buf, *status);
            },
            EgressControl::Data { stream_id, payload } => {
                write_u32(buf, *stream_id);
                write_bytes(buf, payload);
            },
            EgressControl::Close { stream_id, reason } => {
                write_u32(buf, *stream_id);
                write_u8(buf, *reason);
            },
            EgressControl::Keepalive => {},
        }
    }

    pub fn decode(input: &[u8]) -> Result<Self, NetError> {
        if input.is_empty() {
            return Err(NetError::TruncatedPayload);
        }
        let ty = input[0];
        let body = &input[1..];
        match ty {
            0x01 => {
                if body.is_empty() {
                    return Err(NetError::InvalidPayloadField);
                }
                Ok(EgressControl::Auth {
                    proof: body.to_vec(),
                })
            },
            0x02 => {
                // stream_id(4) protocol(1) addr_type(1) addr(var) dest_port(2)
                if body.len() < 8 {
                    return Err(NetError::TruncatedPayload);
                }
                let stream_id = rd_u32(&body[0..4]);
                let protocol = body[4];
                if protocol > 1 {
                    return Err(NetError::InvalidPayloadField);
                }
                let addr_type = body[5];
                let mut off = 6usize;
                let addr = match addr_type {
                    0 => {
                        if body.len() < off + 4 + 2 {
                            return Err(NetError::TruncatedPayload);
                        }
                        let mut a = [0u8; 4];
                        a.copy_from_slice(&body[off..off + 4]);
                        off += 4;
                        EgressAddr::V4(a)
                    },
                    1 => {
                        if body.len() < off + 16 + 2 {
                            return Err(NetError::TruncatedPayload);
                        }
                        let mut a = [0u8; 16];
                        a.copy_from_slice(&body[off..off + 16]);
                        off += 16;
                        EgressAddr::V6(a)
                    },
                    2 => {
                        if body.len() < off + 1 {
                            return Err(NetError::TruncatedPayload);
                        }
                        let hlen = body[off] as usize;
                        off += 1;
                        if hlen == 0 || body.len() < off + hlen + 2 {
                            return Err(NetError::InvalidPayloadField);
                        }
                        let host = body[off..off + hlen].to_vec();
                        off += hlen;
                        EgressAddr::Host(host)
                    },
                    _ => return Err(NetError::InvalidPayloadField),
                };
                if body.len() != off + 2 {
                    return Err(NetError::PayloadLengthMismatch);
                }
                let dest_port = rd_u16(&body[off..off + 2]);
                Ok(EgressControl::Open {
                    stream_id,
                    protocol,
                    addr,
                    dest_port,
                })
            },
            0x03 => {
                if body.len() != 5 {
                    return Err(NetError::PayloadLengthMismatch);
                }
                let stream_id = rd_u32(&body[0..4]);
                let status = body[4];
                if status > 3 {
                    return Err(NetError::InvalidPayloadField);
                }
                Ok(EgressControl::OpenAck { stream_id, status })
            },
            0x04 => {
                if body.len() < 4 {
                    return Err(NetError::TruncatedPayload);
                }
                let stream_id = rd_u32(&body[0..4]);
                Ok(EgressControl::Data {
                    stream_id,
                    payload: body[4..].to_vec(),
                })
            },
            0x05 => {
                if body.len() != 5 {
                    return Err(NetError::PayloadLengthMismatch);
                }
                let stream_id = rd_u32(&body[0..4]);
                let reason = body[4];
                if reason > 2 {
                    return Err(NetError::InvalidPayloadField);
                }
                Ok(EgressControl::Close { stream_id, reason })
            },
            0x06 => {
                if !body.is_empty() {
                    return Err(NetError::PayloadLengthMismatch);
                }
                Ok(EgressControl::Keepalive)
            },
            other => Err(NetError::InvalidMsgType(other)),
        }
    }
}

fn rd_u32(b: &[u8]) -> u32 {
    let mut a = [0u8; 4];
    a.copy_from_slice(&b[0..4]);
    u32::from_le_bytes(a)
}
fn rd_u16(b: &[u8]) -> u16 {
    let mut a = [0u8; 2];
    a.copy_from_slice(&b[0..2]);
    u16::from_le_bytes(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt(m: &EgressControl) -> EgressControl {
        let mut buf = Vec::new();
        m.encode(&mut buf);
        EgressControl::decode(&buf).unwrap()
    }

    #[test]
    fn directory_roundtrip_and_kat() {
        let e = EgressDirectoryEntry {
            exit_node_id: [0x22u8; 32],
            country_code: *b"FR",
            capacity_class: 2,
            advertised_window: 0x0000_4ec0,
        };
        let mut buf = Vec::new();
        e.encode(&mut buf);
        assert_eq!(buf.len(), EGRESS_DIRECTORY_ENTRY_SIZE);
        assert_eq!(EgressDirectoryEntry::decode(&buf).unwrap(), e);
        let mut expected = Vec::new();
        expected.extend_from_slice(&[0x22u8; 32]);
        expected.extend_from_slice(b"FR");
        expected.push(2);
        expected.extend_from_slice(&[0xc0, 0x4e, 0x00, 0x00]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn directory_rejects() {
        let mut e = EgressDirectoryEntry {
            exit_node_id: [0u8; 32],
            country_code: *b"fr",
            capacity_class: 0,
            advertised_window: 1,
        };
        let mut buf = Vec::new();
        e.encode(&mut buf);
        assert!(matches!(
            EgressDirectoryEntry::decode(&buf),
            Err(NetError::InvalidPayloadField)
        ));
        e.country_code = *b"FR";
        e.capacity_class = 3;
        let mut buf2 = Vec::new();
        e.encode(&mut buf2);
        assert!(matches!(
            EgressDirectoryEntry::decode(&buf2),
            Err(NetError::InvalidPayloadField)
        ));
    }

    #[test]
    fn control_roundtrips() {
        let msgs = [
            EgressControl::Auth {
                proof: alloc::vec![0xAB; 64],
            },
            EgressControl::Open {
                stream_id: 7,
                protocol: 0,
                addr: EgressAddr::V4([1, 2, 3, 4]),
                dest_port: 443,
            },
            EgressControl::Open {
                stream_id: 8,
                protocol: 1,
                addr: EgressAddr::V6([9u8; 16]),
                dest_port: 53,
            },
            EgressControl::Open {
                stream_id: 9,
                protocol: 0,
                addr: EgressAddr::Host(b"example.com".to_vec()),
                dest_port: 80,
            },
            EgressControl::OpenAck {
                stream_id: 7,
                status: 0,
            },
            EgressControl::Data {
                stream_id: 7,
                payload: alloc::vec![1, 2, 3, 4, 5],
            },
            EgressControl::Close {
                stream_id: 7,
                reason: 2,
            },
            EgressControl::Keepalive,
        ];
        for m in &msgs {
            assert_eq!(&rt(m), m);
        }
    }

    #[test]
    fn open_kat_v4() {
        let m = EgressControl::Open {
            stream_id: 7,
            protocol: 0,
            addr: EgressAddr::V4([1, 2, 3, 4]),
            dest_port: 443,
        };
        let mut buf = Vec::new();
        m.encode(&mut buf);
        let mut expected = Vec::new();
        expected.push(0x02); // type
        expected.extend_from_slice(&[0x07, 0, 0, 0]); // stream_id LE
        expected.push(0); // protocol tcp
        expected.push(0); // addr_type v4
        expected.extend_from_slice(&[1, 2, 3, 4]); // addr
        expected.extend_from_slice(&[0xBB, 0x01]); // port 443 LE
        assert_eq!(buf, expected);
    }

    #[test]
    fn control_rejects() {
        // bad protocol
        let m = EgressControl::Open {
            stream_id: 1,
            protocol: 2,
            addr: EgressAddr::V4([0; 4]),
            dest_port: 1,
        };
        let mut b = Vec::new();
        m.encode(&mut b);
        assert!(matches!(
            EgressControl::decode(&b),
            Err(NetError::InvalidPayloadField)
        ));
        // bad status
        let mut b2 = Vec::new();
        EgressControl::OpenAck {
            stream_id: 1,
            status: 9,
        }
        .encode(&mut b2);
        assert!(matches!(
            EgressControl::decode(&b2),
            Err(NetError::InvalidPayloadField)
        ));
        // bad reason
        let mut b3 = Vec::new();
        EgressControl::Close {
            stream_id: 1,
            reason: 9,
        }
        .encode(&mut b3);
        assert!(matches!(
            EgressControl::decode(&b3),
            Err(NetError::InvalidPayloadField)
        ));
        // unknown type
        assert!(matches!(
            EgressControl::decode(&[0x7F, 0, 0]),
            Err(NetError::InvalidMsgType(0x7F))
        ));
        // empty
        assert!(matches!(
            EgressControl::decode(&[]),
            Err(NetError::TruncatedPayload)
        ));
        // auth empty proof
        assert!(matches!(
            EgressControl::decode(&[0x01]),
            Err(NetError::InvalidPayloadField)
        ));
    }
}
