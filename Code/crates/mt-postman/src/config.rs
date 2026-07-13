//! QUIC-конфиги стенда (Этап 1). Транспортный хоп QUIC-TLS = admission A-3 ([I-16]):
//! self-signed cert почтальона на захардкоженном адресе стенда (спека §393),
//! клиент его принимает без PKI. Подлинность участников несёт ML-DSA-регистрация
//! (RegProof) + E2E-конверт, НЕ транспортный серт.

use std::sync::Arc;

use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use thiserror::Error;

pub const STAND_SNI: &str = "montana-postman";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("self-signed cert: {0}")]
    Cert(String),
    #[error("rustls: {0}")]
    Rustls(#[from] rustls::Error),
    #[error("quic crypto config: no initial cipher suite")]
    NoInitialCipher,
}

fn ring_provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// Серверный QUIC-конфиг почтальона стенда: self-signed cert, только TLS 1.3.
pub fn stand_server_config() -> Result<quinn::ServerConfig, ConfigError> {
    let cert = rcgen::generate_simple_self_signed(vec![STAND_SNI.to_string()])
        .map_err(|e| ConfigError::Cert(e.to_string()))?;
    let cert_der: CertificateDer<'static> = cert.cert.der().clone();
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der()));
    let rustls_sc = rustls::ServerConfig::builder_with_provider(ring_provider())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)?;
    let quic_sc =
        QuicServerConfig::try_from(rustls_sc).map_err(|_| ConfigError::NoInitialCipher)?;
    Ok(quinn::ServerConfig::with_crypto(Arc::new(quic_sc)))
}

/// Клиентский QUIC-конфiг стенда: принимает self-signed почтальона (skip transport-PKI).
/// [I-16]: транспортный серт — не security; подлинность — ML-DSA-регистрация + E2E.
pub fn stand_client_config() -> Result<quinn::ClientConfig, ConfigError> {
    let p = ring_provider();
    let rustls_cc = rustls::ClientConfig::builder_with_provider(p.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipTransportPki(p)))
        .with_no_client_auth();
    let quic_cc =
        QuicClientConfig::try_from(rustls_cc).map_err(|_| ConfigError::NoInitialCipher)?;
    Ok(quinn::ClientConfig::new(Arc::new(quic_cc)))
}

#[derive(Debug)]
struct SkipTransportPki(Arc<CryptoProvider>);

impl ServerCertVerifier for SkipTransportPki {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}
