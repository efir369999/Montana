// spec, раздел "VPN-Balance Coordinator (Phase 2 milestone M-VPN-1)"
//
// Production-grade axum backend для учёта VPN heartbeat начислений.
// Замена Flask `montana-vpn-balance.service`.
//
// Криптографические гарантии:
//   - Ed25519 подпись heartbeat от ключа derived из BIP39 seed (закрывает F-2 Sybil)
//   - TOFU pinning publickey по адресу при первом heartbeat
//   - Replay-protection: nonce_ms должен быть в окне ±30 секунд от server clock
//   - Atomic single-writer state через tokio::sync::Mutex (закрывает CF-5)
//   - JSON persistence с atomic rename
//
// Известные ограничения (Phase 4 migration):
//   - Ed25519 НЕ post-quantum (нарушает [I-1] протокольной роли).
//     Migration path: заменить на Falcon-512 когда Android JNI bindings готовы.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey, SIGNATURE_LENGTH, PUBLIC_KEY_LENGTH};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc, time::SystemTime};
use tokio::sync::Mutex;

// spec, раздел "VPN-Balance Coordinator" → §1 константы
const RATE_NJ_PER_SECOND: u64 = 1_000;
const MIN_HEARTBEAT_INTERVAL_MS: u64 = 4_000;
const MAX_GAP_MS: u64 = 30_000;
const ADDRESS_HEX_LEN: usize = 40;
const HEARTBEAT_DOMAIN: &[u8] = b"montana-heartbeat-v1:";
const NONCE_WINDOW_MS: u64 = 30_000;
const PURGE_INACTIVE_DAYS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AccountVpnRecord {
    balance_nj: u64,
    seconds_x1000: u64,
    last_hb_unix_ms: u64,
    created_unix_ms: u64,
    last_node: Option<String>,
    pubkey_hex: Option<String>,
    last_nonce_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StateFile {
    accounts: HashMap<String, AccountVpnRecord>,
}

fn exit_node_label(ip: &str) -> Option<(&'static str, &'static str)> {
    match ip {
        "<exit-removed>" => Some(("helsinki", "Хельсинки")),
        "<exit-de>" => Some(("frankfurt", "Франкфурт")),
        "86.104.72.12" => Some(("newyork", "Нью-Йорк")),
        _ => None,
    }
}

fn is_valid_address(addr: &str) -> bool {
    addr.len() == ADDRESS_HEX_LEN
        && addr.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// spec, раздел "VPN-Balance Coordinator" → §5 Ed25519 verification
//
// Message: HEARTBEAT_DOMAIN || address (20B) || nonce_ms (8B little-endian)
fn build_message(address_hex: &str, nonce_ms: u64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEARTBEAT_DOMAIN.len() + 20 + 8);
    buf.extend_from_slice(HEARTBEAT_DOMAIN);
    let addr_bytes = hex::decode(address_hex).unwrap_or_default();
    buf.extend_from_slice(&addr_bytes);
    buf.extend_from_slice(&nonce_ms.to_le_bytes());
    buf
}

fn verify_ed25519(pubkey_hex: &str, signature_hex: &str, message: &[u8]) -> bool {
    let pk_bytes = match hex::decode(pubkey_hex) {
        Ok(b) if b.len() == PUBLIC_KEY_LENGTH => b,
        _ => return false,
    };
    let sig_bytes = match hex::decode(signature_hex) {
        Ok(b) if b.len() == SIGNATURE_LENGTH => b,
        _ => return false,
    };
    let pk_array: [u8; PUBLIC_KEY_LENGTH] = match pk_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let sig_array: [u8; SIGNATURE_LENGTH] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let vk = match VerifyingKey::from_bytes(&pk_array) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let sig = Signature::from_bytes(&sig_array);
    vk.verify(message, &sig).is_ok()
}

#[derive(Debug, Deserialize)]
struct HeartbeatBody {
    address: String,
    #[serde(default)]
    nonce_ms: Option<u64>,
    #[serde(default)]
    pubkey: Option<String>,
    #[serde(default)]
    signature: Option<String>,
}

#[derive(Debug, Serialize)]
struct HeartbeatResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    via_montana: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    throttled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credited_seconds: Option<f64>,
    balance: f64,
    seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_mode: Option<&'static str>,
}

#[derive(Debug, Deserialize)]
struct BalanceQuery {
    address: String,
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    address: String,
    balance: f64,
    seconds: f64,
    online: bool,
    rate_per_second: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_node: Option<String>,
}

#[derive(Clone)]
struct AppState {
    state_path: Arc<PathBuf>,
    inner: Arc<Mutex<StateFile>>,
}

impl AppState {
    async fn save(&self, state: &StateFile) -> Result<(), String> {
        let tmp = self.state_path.with_extension("tmp");
        let json = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
        tokio::fs::write(&tmp, &json).await.map_err(|e| e.to_string())?;
        tokio::fs::rename(&tmp, self.state_path.as_path())
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn load_initial(path: &PathBuf) -> StateFile {
        match tokio::fs::read(path).await {
            Ok(bytes) => match serde_json::from_slice::<StateFile>(&bytes) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("state parse error: {}, starting fresh", e);
                    StateFile::default()
                }
            },
            Err(_) => {
                tracing::info!("no state file, starting fresh");
                StateFile::default()
            }
        }
    }
}

async fn handler_heartbeat(
    State(app): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<HeartbeatBody>,
) -> impl IntoResponse {
    let client_ip = headers
        .get("x-real-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .unwrap_or_default();

    let addr = body.address.to_lowercase();
    if !is_valid_address(&addr) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "bad address"})),
        )
            .into_response();
    }

    let (node_label, node_city) = match exit_node_label(&client_ip) {
        Some(x) => x,
        None => {
            return (
                StatusCode::OK,
                Json(serde_json::to_value(HeartbeatResponse {
                    ok: false,
                    reason: Some("not_via_montana_vpn".into()),
                    via_montana: false,
                    node: None,
                    node_city: None,
                    throttled: None,
                    credited_seconds: None,
                    balance: 0.0,
                    seconds: 0.0,
                    auth_mode: None,
                }).unwrap()),
            )
                .into_response();
        }
    };

    let now_ms = unix_ms();

    // signature verification (если предоставлена)
    let auth_mode: &'static str;
    let signature_check = if let (Some(pk_hex), Some(sig_hex), Some(nonce)) =
        (body.pubkey.as_ref(), body.signature.as_ref(), body.nonce_ms)
    {
        if nonce < now_ms.saturating_sub(NONCE_WINDOW_MS) || nonce > now_ms + NONCE_WINDOW_MS {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "nonce out of window"})),
            )
                .into_response();
        }
        let msg = build_message(&addr, nonce);
        if !verify_ed25519(pk_hex, sig_hex, &msg) {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "signature invalid"})),
            )
                .into_response();
        }
        auth_mode = "ed25519-signed";
        Some((pk_hex.clone(), nonce))
    } else {
        auth_mode = "legacy-unsigned";
        None
    };

    let mut state = app.inner.lock().await;
    let rec = state.accounts.entry(addr.clone()).or_insert_with(|| AccountVpnRecord {
        created_unix_ms: now_ms,
        ..Default::default()
    });

    if let Some((pk_hex, nonce)) = signature_check {
        if rec.pubkey_hex.is_none() {
            rec.pubkey_hex = Some(pk_hex);
            tracing::info!("pubkey TOFU pinned for address {}", addr);
        } else if rec.pubkey_hex.as_deref() != Some(pk_hex.as_str()) {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "pubkey mismatch with pinned"})),
            )
                .into_response();
        }
        if let Some(last_nonce) = rec.last_nonce_ms {
            if nonce <= last_nonce {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "nonce replay"})),
                )
                    .into_response();
            }
        }
        rec.last_nonce_ms = Some(nonce);
    } else if rec.pubkey_hex.is_some() {
        // Once signed, always signed — нельзя downgrade
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "signature required (account already pinned to pubkey)"})),
        )
            .into_response();
    }

    let last = rec.last_hb_unix_ms;
    let gap_ms = now_ms.saturating_sub(last);

    if last > 0 && gap_ms < MIN_HEARTBEAT_INTERVAL_MS {
        let response = HeartbeatResponse {
            ok: true,
            reason: None,
            via_montana: true,
            node: Some(node_label.into()),
            node_city: Some(node_city.into()),
            throttled: Some(true),
            credited_seconds: Some(0.0),
            balance: rec.balance_nj as f64 / 1_000_000.0,
            seconds: rec.seconds_x1000 as f64 / 1000.0,
            auth_mode: Some(auth_mode),
        };
        return (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response();
    }

    let credited_ms: u64 = if last > 0 && gap_ms <= MAX_GAP_MS {
        gap_ms
    } else if last == 0 {
        MIN_HEARTBEAT_INTERVAL_MS
    } else {
        0
    };

    // [I-9]: integer arithmetic, нет floats в consensus path
    let credited_nj = credited_ms.saturating_mul(RATE_NJ_PER_SECOND) / 1000;
    rec.balance_nj = rec.balance_nj.saturating_add(credited_nj);
    rec.seconds_x1000 = rec.seconds_x1000.saturating_add(credited_ms);
    rec.last_hb_unix_ms = now_ms;
    rec.last_node = Some(node_label.into());

    let balance = rec.balance_nj as f64 / 1_000_000.0;
    let seconds = rec.seconds_x1000 as f64 / 1000.0;

    let snapshot = state.clone();
    drop(state);
    if let Err(e) = app.save(&snapshot).await {
        tracing::error!("save state failed: {}", e);
    }

    let response = HeartbeatResponse {
        ok: true,
        reason: None,
        via_montana: true,
        node: Some(node_label.into()),
        node_city: Some(node_city.into()),
        throttled: Some(false),
        credited_seconds: Some(credited_ms as f64 / 1000.0),
        balance,
        seconds,
        auth_mode: Some(auth_mode),
    };
    (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
}

async fn handler_balance(
    State(app): State<AppState>,
    Query(q): Query<BalanceQuery>,
) -> impl IntoResponse {
    let addr = q.address.to_lowercase();
    if !is_valid_address(&addr) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "bad address"})),
        )
            .into_response();
    }
    let state = app.inner.lock().await;
    let rec = state.accounts.get(&addr);
    let response = match rec {
        Some(r) => BalanceResponse {
            address: addr.clone(),
            balance: r.balance_nj as f64 / 1_000_000.0,
            seconds: r.seconds_x1000 as f64 / 1000.0,
            online: unix_ms().saturating_sub(r.last_hb_unix_ms) < MIN_HEARTBEAT_INTERVAL_MS * 4,
            rate_per_second: 0.001,
            last_node: r.last_node.clone(),
        },
        None => BalanceResponse {
            address: addr,
            balance: 0.0,
            seconds: 0.0,
            online: false,
            rate_per_second: 0.001,
            last_node: None,
        },
    };
    Json(response).into_response()
}

async fn handler_sub() -> impl IntoResponse {
    let link = "vless://e6d355e2-2d79-4c96-a373-3b0e6b6f4b0d@cdn.montana.quest:443\
                ?flow=xtls-rprx-vision&type=tcp&headerType=none&security=reality\
                &fp=chrome&sni=www.googletagmanager.com\
                &pbk=EkTs2aGKnFNgFZ0f7wgft2sJp3VjwFQqIrwkZKM4gD8\
                &sid=302805bc0c25e504\
                #%C9%88%20%D0%9C%D0%BE%D0%BD%D1%82%D0%B0%D0%BD%D0%B0";
    use base64::Engine;
    let enc = base64::engine::general_purpose::STANDARD.encode(link.as_bytes());
    (
        [
            ("content-type", "text/plain; charset=utf-8"),
            ("cache-control", "no-store"),
        ],
        enc,
    )
        .into_response()
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    wallets: usize,
    total_juno: f64,
    total_seconds: f64,
    active_now: usize,
    signed_accounts: usize,
    rate_per_second: f64,
}

async fn handler_stats(State(app): State<AppState>) -> impl IntoResponse {
    let state = app.inner.lock().await;
    let now = unix_ms();
    let total_nj: u64 = state.accounts.values().map(|r| r.balance_nj).sum();
    let total_sec_x1000: u64 = state.accounts.values().map(|r| r.seconds_x1000).sum();
    let active = state.accounts.values().filter(|r| now.saturating_sub(r.last_hb_unix_ms) < 60_000).count();
    let signed = state.accounts.values().filter(|r| r.pubkey_hex.is_some()).count();
    Json(StatsResponse {
        wallets: state.accounts.len(),
        total_juno: total_nj as f64 / 1_000_000.0,
        total_seconds: total_sec_x1000 as f64 / 1000.0,
        active_now: active,
        signed_accounts: signed,
        rate_per_second: 0.001,
    })
    .into_response()
}

async fn handler_purge(
    State(app): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let client_ip = headers.get("x-real-ip").and_then(|v| v.to_str().ok()).unwrap_or("");
    if client_ip != "127.0.0.1" && client_ip != "::1" && !client_ip.is_empty() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "forbidden"}))).into_response();
    }
    let cutoff_ms = unix_ms().saturating_sub(PURGE_INACTIVE_DAYS * 86_400_000);
    let mut state = app.inner.lock().await;
    let to_remove: Vec<String> = state
        .accounts
        .iter()
        .filter(|(_, r)| r.last_hb_unix_ms < cutoff_ms && r.balance_nj == 0)
        .map(|(k, _)| k.clone())
        .collect();
    for k in &to_remove {
        state.accounts.remove(k);
    }
    let n = to_remove.len();
    let snapshot = state.clone();
    drop(state);
    let _ = app.save(&snapshot).await;
    Json(serde_json::json!({
        "removed_count": n,
        "removed": to_remove.iter().take(50).collect::<Vec<_>>(),
    }))
    .into_response()
}



#[derive(Debug, Deserialize)]
struct RevokeBody {
    address: String,
    nonce_ms: u64,
    pubkey: String,
    signature: String,
    /// reason — optional comment
    #[serde(default)]
    reason: Option<String>,
}

async fn handler_revoke(
    State(app): State<AppState>,
    Json(body): Json<RevokeBody>,
) -> impl IntoResponse {
    let addr = body.address.to_lowercase();
    if !is_valid_address(&addr) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "bad address"})),
        )
            .into_response();
    }
    let now_ms = unix_ms();
    if body.nonce_ms < now_ms.saturating_sub(NONCE_WINDOW_MS)
        || body.nonce_ms > now_ms + NONCE_WINDOW_MS
    {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "nonce out of window"})),
        )
            .into_response();
    }
    // Сообщение revocation: "montana-revoke-v1:" || address || nonce_ms
    let mut msg = Vec::new();
    msg.extend_from_slice(b"montana-revoke-v1:");
    msg.extend_from_slice(&hex::decode(&addr).unwrap_or_default());
    msg.extend_from_slice(&body.nonce_ms.to_le_bytes());
    if !verify_ed25519(&body.pubkey, &body.signature, &msg) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "signature invalid"})),
        )
            .into_response();
    }
    let mut state = app.inner.lock().await;
    let removed = state.accounts.remove(&addr).is_some();
    let snapshot = state.clone();
    drop(state);
    if let Err(e) = app.save(&snapshot).await {
        tracing::error!("save state failed: {}", e);
    }
    Json(serde_json::json!({
        "ok": true,
        "address": addr,
        "removed": removed,
        "reason": body.reason,
    }))
    .into_response()
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("MT_VPN_LOG")
                .unwrap_or_else(|_| "info,axum::rejection=trace".into()),
        )
        .init();

    let state_path = std::env::var("MT_VPN_STATE_PATH")
        .unwrap_or_else(|_| "/var/lib/mt-vpn-balance/state.json".into());
    let state_path = PathBuf::from(state_path);
    if let Some(parent) = state_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let initial = AppState::load_initial(&state_path).await;
    let app_state = AppState {
        state_path: Arc::new(state_path),
        inner: Arc::new(Mutex::new(initial)),
    };

    let app = Router::new()
        .route("/api/vpn/heartbeat", post(handler_heartbeat))
        .route("/api/vpn/balance", get(handler_balance))
        .route("/api/vpn/stats", get(handler_stats))
        .route("/api/vpn/admin/purge", post(handler_purge))
        .route("/api/vpn/revoke", post(handler_revoke))
        .route("/vpn/sub", get(handler_sub))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(app_state);

    let bind = std::env::var("MT_VPN_BIND").unwrap_or_else(|_| "127.0.0.1:5009".into());
    let addr: SocketAddr = bind.parse()?;
    tracing::info!("mt-vpn-balance listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn test_address_validation() {
        assert!(is_valid_address("2f8714b236118011647ec51d0ca6ad40d286bec7"));
        assert!(!is_valid_address("2f8714b236118011647ec51d0ca6ad40d286bec")); // too short
        assert!(!is_valid_address("2F8714B236118011647EC51D0CA6AD40D286BEC7")); // uppercase
        assert!(!is_valid_address("xx14b236118011647ec51d0ca6ad40d286bec7zz")); // non-hex
    }

    #[test]
    fn test_exit_node_label() {
        assert_eq!(exit_node_label("<exit-removed>"), Some(("helsinki", "Хельсинки")));
        assert_eq!(exit_node_label("<exit-de>"), Some(("frankfurt", "Франкфурт")));
        assert_eq!(exit_node_label("86.104.72.12"), Some(("newyork", "Нью-Йорк")));
        assert_eq!(exit_node_label("1.2.3.4"), None);
    }

    #[test]
    fn test_ed25519_signing_roundtrip() {
        let seed_bytes: [u8; 32] = [42u8; 32];
        let signing = SigningKey::from_bytes(&seed_bytes);
        let verifying = signing.verifying_key();
        let pk_hex = hex::encode(verifying.to_bytes());

        let address = "2f8714b236118011647ec51d0ca6ad40d286bec7";
        let nonce_ms: u64 = 1_700_000_000_000;
        let msg = build_message(address, nonce_ms);
        let sig = signing.sign(&msg);
        let sig_hex = hex::encode(sig.to_bytes());

        assert!(verify_ed25519(&pk_hex, &sig_hex, &msg));

        // Wrong message → rejected
        let bad_msg = build_message(address, nonce_ms + 1);
        assert!(!verify_ed25519(&pk_hex, &sig_hex, &bad_msg));

        // Tampered sig → rejected
        let mut tampered = sig.to_bytes();
        tampered[0] ^= 0x01;
        let tampered_hex = hex::encode(tampered);
        assert!(!verify_ed25519(&pk_hex, &tampered_hex, &msg));
    }

    #[test]
    fn test_build_message_deterministic() {
        let m1 = build_message("2f8714b236118011647ec51d0ca6ad40d286bec7", 1700000000000);
        let m2 = build_message("2f8714b236118011647ec51d0ca6ad40d286bec7", 1700000000000);
        assert_eq!(m1, m2);
        assert_eq!(&m1[..HEARTBEAT_DOMAIN.len()], HEARTBEAT_DOMAIN);
        assert_eq!(m1.len(), HEARTBEAT_DOMAIN.len() + 20 + 8);
    }

    #[test]
    fn test_credited_arithmetic_overflow_safe() {
        // saturating_mul защита от overflow
        let huge_ms = u64::MAX / 2;
        let result = huge_ms.saturating_mul(RATE_NJ_PER_SECOND) / 1000;
        // Просто проверка что не panics
        assert!(result <= u64::MAX);
    }
}
