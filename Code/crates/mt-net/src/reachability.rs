// spec, Network layer -> "Reachability sensing and auto-steering"
// (ReachabilityAdvert layout + invariants).
//
// Transport-layer telemetry, outside consensus state ([I-3] orthogonal).
// The advert propagates over peer exchange; the aggregated map is advisory
// and ranks candidate entry points only. No state-root participation.

use alloc::vec::Vec;

use mt_codec::{write_bytes, write_u16, write_u32, write_u8};

use crate::error::NetError;

/// Wire size of a ReachabilityAdvert: 2 + 4 + 32 + 1 + 2 + 2 + 4.
pub const REACHABILITY_ADVERT_SIZE: usize = 47;

/// Per-vantage retained observation bound (mirrors the IBT online-nonce bound).
pub const MAX_OBSERVATIONS_PER_VANTAGE: usize = 256;

/// Distinct /16 source groups required to act on a reachability triple.
pub const REACHABILITY_QUORUM: usize = 3;

/// Highest transport profile index (T0..T4).
pub const PROFILE_MAX: u8 = 4;

/// Advisory record of one vantage's reachability to one peer on one transport
/// profile. Propagated over peer exchange; aggregated into the reachability map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachabilityAdvert {
    /// ISO-3166-1 alpha-2 of the observing vantage.
    pub country_code: [u8; 2],
    /// Autonomous system of the observing vantage.
    pub asn: u32,
    /// node_id of the observed peer.
    pub target_ref: [u8; 32],
    /// Transport profile observed (T0..T4 = 0..4).
    pub profile: u8,
    /// Corroborating observations with outcome = reachable.
    pub reachable_num: u16,
    /// Total observations for the triple.
    pub reachable_den: u16,
    /// Cached window_index of the latest observation.
    pub observed_window: u32,
}

fn is_iso_alpha(b: u8) -> bool {
    (b'A'..=b'Z').contains(&b)
}

impl ReachabilityAdvert {
    pub fn encode(&self, buf: &mut Vec<u8>) {
        write_bytes(buf, &self.country_code);
        write_u32(buf, self.asn);
        write_bytes(buf, &self.target_ref);
        write_u8(buf, self.profile);
        write_u16(buf, self.reachable_num);
        write_u16(buf, self.reachable_den);
        write_u32(buf, self.observed_window);
    }

    pub fn decode(input: &[u8]) -> Result<Self, NetError> {
        if input.len() != REACHABILITY_ADVERT_SIZE {
            return Err(NetError::PayloadLengthMismatch);
        }
        let mut country_code = [0u8; 2];
        country_code.copy_from_slice(&input[0..2]);
        let mut asn_b = [0u8; 4];
        asn_b.copy_from_slice(&input[2..6]);
        let asn = u32::from_le_bytes(asn_b);
        let mut target_ref = [0u8; 32];
        target_ref.copy_from_slice(&input[6..38]);
        let profile = input[38];
        let mut num_b = [0u8; 2];
        num_b.copy_from_slice(&input[39..41]);
        let reachable_num = u16::from_le_bytes(num_b);
        let mut den_b = [0u8; 2];
        den_b.copy_from_slice(&input[41..43]);
        let reachable_den = u16::from_le_bytes(den_b);
        let mut win_b = [0u8; 4];
        win_b.copy_from_slice(&input[43..47]);
        let observed_window = u32::from_le_bytes(win_b);

        // Invariants ReachabilityAdvert (spec):
        // country_code is two ISO-3166-1 alpha-2 letters.
        if !is_iso_alpha(country_code[0]) || !is_iso_alpha(country_code[1]) {
            return Err(NetError::InvalidPayloadField);
        }
        // profile in T0..T4.
        if profile > PROFILE_MAX {
            return Err(NetError::InvalidPayloadField);
        }
        // reachable_den >= 1 and reachable_num <= reachable_den.
        if reachable_den == 0 || reachable_num > reachable_den {
            return Err(NetError::InvalidPayloadField);
        }

        Ok(ReachabilityAdvert {
            country_code,
            asn,
            target_ref,
            profile,
            reachable_num,
            reachable_den,
            observed_window,
        })
    }

    /// Ranking ratio (num, den). Advisory only; forms no consensus state.
    pub fn reachable_fraction(&self) -> (u16, u16) {
        (self.reachable_num, self.reachable_den)
    }

    /// Staleness gate: the advert is fresh when its observed_window lies within
    /// [known_window - staleness_bound, known_window]. The mesh-IBT staleness
    /// bound (7 * tau1) is supplied by the caller.
    pub fn is_fresh(&self, known_window: u32, staleness_bound: u32) -> bool {
        let lo = known_window.saturating_sub(staleness_bound);
        self.observed_window >= lo && self.observed_window <= known_window
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn sample() -> ReachabilityAdvert {
        ReachabilityAdvert {
            country_code: *b"AM",
            asn: 0x0001_0203,
            target_ref: [0x11u8; 32],
            profile: 1,
            reachable_num: 3,
            reachable_den: 4,
            observed_window: 0x0000_4ec0,
        }
    }

    #[test]
    fn roundtrip() {
        let a = sample();
        let mut buf = Vec::new();
        a.encode(&mut buf);
        assert_eq!(buf.len(), REACHABILITY_ADVERT_SIZE);
        let b = ReachabilityAdvert::decode(&buf).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn kat_byte_exact() {
        // Binding KAT: fixed advert -> exact 47-byte little-endian encoding.
        let a = sample();
        let mut buf = Vec::new();
        a.encode(&mut buf);
        let mut expected = Vec::new();
        expected.extend_from_slice(b"AM"); // country_code
        expected.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]); // asn LE
        expected.extend_from_slice(&[0x11u8; 32]); // target_ref
        expected.push(0x01); // profile
        expected.extend_from_slice(&[0x03, 0x00]); // reachable_num LE
        expected.extend_from_slice(&[0x04, 0x00]); // reachable_den LE
        expected.extend_from_slice(&[0xc0, 0x4e, 0x00, 0x00]); // observed_window LE
        assert_eq!(buf, expected);
    }

    #[test]
    fn reject_wrong_length() {
        assert!(matches!(
            ReachabilityAdvert::decode(&[0u8; 46]),
            Err(NetError::PayloadLengthMismatch)
        ));
    }

    #[test]
    fn reject_bad_country() {
        let mut a = sample();
        a.country_code = *b"a1";
        let mut buf = Vec::new();
        a.encode(&mut buf);
        assert!(matches!(
            ReachabilityAdvert::decode(&buf),
            Err(NetError::InvalidPayloadField)
        ));
    }

    #[test]
    fn reject_bad_profile() {
        let mut a = sample();
        a.profile = 5;
        let mut buf = Vec::new();
        a.encode(&mut buf);
        assert!(matches!(
            ReachabilityAdvert::decode(&buf),
            Err(NetError::InvalidPayloadField)
        ));
    }

    #[test]
    fn reject_den_zero_and_num_gt_den() {
        let mut a = sample();
        a.reachable_den = 0;
        let mut buf = Vec::new();
        a.encode(&mut buf);
        assert!(matches!(
            ReachabilityAdvert::decode(&buf),
            Err(NetError::InvalidPayloadField)
        ));

        let mut a2 = sample();
        a2.reachable_num = 5;
        a2.reachable_den = 4;
        let mut buf2 = Vec::new();
        a2.encode(&mut buf2);
        assert!(matches!(
            ReachabilityAdvert::decode(&buf2),
            Err(NetError::InvalidPayloadField)
        ));
    }

    #[test]
    fn freshness_window() {
        let a = sample(); // observed_window = 0x4ec0 = 20160
        assert!(a.is_fresh(20160, 100));
        assert!(a.is_fresh(20200, 100)); // within [20100, 20200]
        assert!(!a.is_fresh(20300, 100)); // below 20200 lower bound
        assert!(!a.is_fresh(20159, 100)); // above known
    }
}
