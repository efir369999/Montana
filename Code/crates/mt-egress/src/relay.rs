// Egress relay tier — bandwidth accounting for the front/relay role.
// spec: Montana Egress v1.0.0 (Exit node bandwidth tier) + Montana VPN Alliance
// v1.0.0 (front load model) + Montana Network (Circuit Relay v2).
//
// The egress relay carries the inner Noise_PQ XX session as opaque ciphertext
// (no decryption at the front) and accounts the forwarded bytes against a
// distinct, operator-configured high-bandwidth cap — separate from the
// consensus-relay baseline (1 KB/s, Network spec). This separation lets a relay
// apply the correct cap per traffic class and keeps the crypto/egress load on
// the chosen exit, not the front. Transport-layer, outside consensus state.

/// Consensus-relay baseline: 1 KB/s (Network spec, Circuit Relay v2 limits).
pub const CONSENSUS_RELAY_CAP_BYTES_PER_SEC: u64 = 1024;

/// Traffic class carried by a relayed connection. The egress tier is signalled
/// out of band (distinct protocol id / reservation flag) so the relay applies
/// the egress cap rather than the consensus baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayClass {
    Consensus,
    Egress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayError {
    /// The relayed bytes in this accounting window exceed the class cap.
    CapExceeded,
}

/// Per-connection relay byte budget for one accounting window.
#[derive(Debug, Clone)]
pub struct RelayBudget {
    class: RelayClass,
    cap_bytes_per_window: u64,
    used: u64,
}

impl RelayBudget {
    /// Consensus-relay budget: the fixed baseline cap.
    pub fn consensus(window_seconds: u64) -> Self {
        RelayBudget {
            class: RelayClass::Consensus,
            cap_bytes_per_window: CONSENSUS_RELAY_CAP_BYTES_PER_SEC.saturating_mul(window_seconds),
            used: 0,
        }
    }

    /// Egress-relay budget: operator-configured high-bandwidth cap for the window.
    pub fn egress(cap_bytes_per_window: u64) -> Self {
        RelayBudget {
            class: RelayClass::Egress,
            cap_bytes_per_window,
            used: 0,
        }
    }

    pub fn class(&self) -> RelayClass {
        self.class
    }

    pub fn remaining(&self) -> u64 {
        self.cap_bytes_per_window.saturating_sub(self.used)
    }

    /// Account `n` forwarded (ciphertext) bytes against the budget. The relay
    /// never inspects the bytes; it only counts them. Returns CapExceeded when
    /// the window cap would be passed; the relay then backpressures the stream.
    pub fn account(&mut self, n: u64) -> Result<(), RelayError> {
        let next = self.used.saturating_add(n);
        if next > self.cap_bytes_per_window {
            return Err(RelayError::CapExceeded);
        }
        self.used = next;
        Ok(())
    }

    /// Reset at the accounting-window boundary.
    pub fn reset_window(&mut self) {
        self.used = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consensus_baseline_cap() {
        let b = RelayBudget::consensus(60);
        assert_eq!(b.class(), RelayClass::Consensus);
        assert_eq!(b.remaining(), 1024 * 60); // 1 KB/s * 60s
    }

    #[test]
    fn egress_tier_distinct_and_higher() {
        let cap = 10 * 1024 * 1024 * 60; // 10 MB/s * 60s
        let b = RelayBudget::egress(cap);
        assert_eq!(b.class(), RelayClass::Egress);
        assert_eq!(b.remaining(), cap);
        assert!(b.remaining() > RelayBudget::consensus(60).remaining());
    }

    #[test]
    fn account_under_cap_ok() {
        let mut b = RelayBudget::egress(1000);
        assert!(b.account(400).is_ok());
        assert!(b.account(600).is_ok());
        assert_eq!(b.remaining(), 0);
    }

    #[test]
    fn account_over_cap_rejected() {
        let mut b = RelayBudget::egress(1000);
        assert!(b.account(700).is_ok());
        assert_eq!(b.account(400), Err(RelayError::CapExceeded));
        assert_eq!(b.remaining(), 300); // rejected bytes not counted
    }

    #[test]
    fn reset_window_restores_budget() {
        let mut b = RelayBudget::egress(1000);
        b.account(1000).unwrap();
        assert_eq!(b.remaining(), 0);
        b.reset_window();
        assert_eq!(b.remaining(), 1000);
    }

    #[test]
    fn consensus_relay_caps_egress_traffic_low() {
        // a consensus-class budget cannot absorb a video-rate burst
        let mut b = RelayBudget::consensus(1); // 1024 bytes for the second
        assert_eq!(b.account(64 * 1024), Err(RelayError::CapExceeded));
    }
}
