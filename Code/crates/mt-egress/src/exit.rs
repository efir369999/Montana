// Egress exit-side session state machine + policy gate.
// spec: Montana Egress v1.0.0 (Exit node; Control messages invariants).
//
// Pure control logic, outside consensus state. The IBT level-3 proof in
// EgressAuth is verified by the caller via mt_net::ibt::ibt_online_verify with
// server_node_id = exit_node_id (the inner session terminates at the exit), so
// a proof for one exit is invalid at any other node. This module tracks the
// authenticated flag, the open-stream set, and applies the egress policy.

use alloc::collections::BTreeSet;

use crate::{EgressAddr, MAX_STREAMS_PER_SESSION};

/// Result of an EgressOpen request, mapping to EgressOpenAck.status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenOutcome {
    Open = 0,
    RefusedByPolicy = 1,
    Unreachable = 2,
    RateLimited = 3,
}

impl OpenOutcome {
    pub fn status(self) -> u8 {
        self as u8
    }
}

/// Operator egress policy. `default_allow` sets the base decision; `port_exceptions`
/// inverts it for the listed destination ports.
#[derive(Debug, Clone)]
pub struct ExitPolicy {
    pub default_allow: bool,
    pub port_exceptions: BTreeSet<u16>,
}

impl ExitPolicy {
    pub fn default_allow() -> Self {
        ExitPolicy {
            default_allow: true,
            port_exceptions: BTreeSet::new(),
        }
    }
    pub fn default_deny() -> Self {
        ExitPolicy {
            default_allow: false,
            port_exceptions: BTreeSet::new(),
        }
    }
    pub fn with_exception(mut self, port: u16) -> Self {
        self.port_exceptions.insert(port);
        self
    }
    /// True when the destination port is permitted.
    pub fn allows(&self, _addr: &EgressAddr, port: u16) -> bool {
        let listed = self.port_exceptions.contains(&port);
        if self.default_allow {
            !listed
        } else {
            listed
        }
    }
}

/// Error from applying a control message to the exit session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitSessionError {
    /// Open / Data / Close arrived before a verified EgressAuth (session MUST close).
    NotAuthenticated,
    /// Data / Close referenced an unknown stream_id.
    UnknownStream,
    /// Duplicate stream_id on Open.
    DuplicateStream,
}

/// Exit-side per-inner-session state.
#[derive(Debug)]
pub struct ExitSession {
    authed: bool,
    streams: BTreeSet<u32>,
    policy: ExitPolicy,
}

impl ExitSession {
    pub fn new(policy: ExitPolicy) -> Self {
        ExitSession {
            authed: false,
            streams: BTreeSet::new(),
            policy,
        }
    }

    /// Mark the session authenticated. The caller invokes this only after
    /// mt_net::ibt::ibt_online_verify succeeds against this exit's node_id.
    pub fn authenticate(&mut self) {
        self.authed = true;
    }

    pub fn is_authenticated(&self) -> bool {
        self.authed
    }

    pub fn open_stream_count(&self) -> usize {
        self.streams.len()
    }

    pub fn has_stream(&self, stream_id: u32) -> bool {
        self.streams.contains(&stream_id)
    }

    /// Apply an EgressOpen. Returns the outcome to encode into EgressOpenAck.
    pub fn handle_open(
        &mut self,
        stream_id: u32,
        addr: &EgressAddr,
        dest_port: u16,
    ) -> Result<OpenOutcome, ExitSessionError> {
        if !self.authed {
            return Err(ExitSessionError::NotAuthenticated);
        }
        if self.streams.contains(&stream_id) {
            return Err(ExitSessionError::DuplicateStream);
        }
        if self.streams.len() as u32 >= MAX_STREAMS_PER_SESSION {
            return Ok(OpenOutcome::RateLimited);
        }
        if !self.policy.allows(addr, dest_port) {
            return Ok(OpenOutcome::RefusedByPolicy);
        }
        self.streams.insert(stream_id);
        Ok(OpenOutcome::Open)
    }

    /// Apply an EgressClose. Removes the stream.
    pub fn handle_close(&mut self, stream_id: u32) -> Result<(), ExitSessionError> {
        if !self.authed {
            return Err(ExitSessionError::NotAuthenticated);
        }
        if !self.streams.remove(&stream_id) {
            return Err(ExitSessionError::UnknownStream);
        }
        Ok(())
    }

    /// Validate an EgressData stream reference (payload forwarding is I/O glue).
    pub fn check_data(&self, stream_id: u32) -> Result<(), ExitSessionError> {
        if !self.authed {
            return Err(ExitSessionError::NotAuthenticated);
        }
        if !self.streams.contains(&stream_id) {
            return Err(ExitSessionError::UnknownStream);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_before_auth_rejected() {
        let mut s = ExitSession::new(ExitPolicy::default_allow());
        let r = s.handle_open(1, &EgressAddr::V4([1, 1, 1, 1]), 443);
        assert_eq!(r, Err(ExitSessionError::NotAuthenticated));
    }

    #[test]
    fn auth_then_open_allowed_port() {
        let mut s = ExitSession::new(ExitPolicy::default_allow());
        s.authenticate();
        assert_eq!(
            s.handle_open(1, &EgressAddr::V4([1, 1, 1, 1]), 443),
            Ok(OpenOutcome::Open)
        );
        assert!(s.has_stream(1));
        assert_eq!(s.open_stream_count(), 1);
    }

    #[test]
    fn policy_default_allow_denies_listed_port() {
        let mut s = ExitSession::new(ExitPolicy::default_allow().with_exception(25));
        s.authenticate();
        assert_eq!(
            s.handle_open(1, &EgressAddr::V4([1, 1, 1, 1]), 25),
            Ok(OpenOutcome::RefusedByPolicy)
        );
        assert!(!s.has_stream(1));
        assert_eq!(
            s.handle_open(2, &EgressAddr::V4([1, 1, 1, 1]), 443),
            Ok(OpenOutcome::Open)
        );
    }

    #[test]
    fn policy_default_deny_allows_only_listed() {
        let mut s = ExitSession::new(ExitPolicy::default_deny().with_exception(443));
        s.authenticate();
        assert_eq!(
            s.handle_open(1, &EgressAddr::V4([1, 1, 1, 1]), 80),
            Ok(OpenOutcome::RefusedByPolicy)
        );
        assert_eq!(
            s.handle_open(2, &EgressAddr::V4([1, 1, 1, 1]), 443),
            Ok(OpenOutcome::Open)
        );
    }

    #[test]
    fn max_streams_rate_limited() {
        let mut s = ExitSession::new(ExitPolicy::default_allow());
        s.authenticate();
        for i in 0..MAX_STREAMS_PER_SESSION {
            assert_eq!(
                s.handle_open(i, &EgressAddr::V4([1, 1, 1, 1]), 443),
                Ok(OpenOutcome::Open)
            );
        }
        assert_eq!(s.open_stream_count(), MAX_STREAMS_PER_SESSION as usize);
        assert_eq!(
            s.handle_open(MAX_STREAMS_PER_SESSION, &EgressAddr::V4([1, 1, 1, 1]), 443),
            Ok(OpenOutcome::RateLimited)
        );
    }

    #[test]
    fn close_removes_and_unknown_errors() {
        let mut s = ExitSession::new(ExitPolicy::default_allow());
        s.authenticate();
        s.handle_open(1, &EgressAddr::V4([1, 1, 1, 1]), 443)
            .unwrap();
        assert!(s.check_data(1).is_ok());
        assert_eq!(s.handle_close(1), Ok(()));
        assert!(!s.has_stream(1));
        assert_eq!(s.handle_close(1), Err(ExitSessionError::UnknownStream));
        assert_eq!(s.check_data(1), Err(ExitSessionError::UnknownStream));
    }

    #[test]
    fn duplicate_stream_rejected() {
        let mut s = ExitSession::new(ExitPolicy::default_allow());
        s.authenticate();
        s.handle_open(1, &EgressAddr::V4([1, 1, 1, 1]), 443)
            .unwrap();
        assert_eq!(
            s.handle_open(1, &EgressAddr::V4([2, 2, 2, 2]), 80),
            Err(ExitSessionError::DuplicateStream)
        );
    }

    #[test]
    fn open_outcome_status_codes() {
        assert_eq!(OpenOutcome::Open.status(), 0);
        assert_eq!(OpenOutcome::RefusedByPolicy.status(), 1);
        assert_eq!(OpenOutcome::Unreachable.status(), 2);
        assert_eq!(OpenOutcome::RateLimited.status(), 3);
    }
}
