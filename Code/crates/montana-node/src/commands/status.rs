use std::path::PathBuf;

use mt_account::supply_moneta;
use mt_genesis::genesis_params;

use crate::clock::load_current_window;
use crate::identity::{default_data_dir, load_identity, NodeError};
use crate::node_lifecycle::load_or_init_lifecycle;
use crate::state::LocalState;
use crate::timechain_state::load_or_init_timechain;

pub struct StatusArgs {
    pub data_dir: Option<PathBuf>,
}

pub fn run(args: StatusArgs) -> Result<(), NodeError> {
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    let identity = load_identity(&data_dir)?;
    let params = genesis_params();
    let state = LocalState::load_or_bootstrap(&data_dir, &identity, params, &[])?;
    let current_window = load_current_window(&data_dir)?;
    let lifecycle = load_or_init_lifecycle(&data_dir, &identity, params)?;
    let timechain = load_or_init_timechain(&data_dir)?;

    let my_account = identity.account_id();
    let my_node = identity.node_id();

    println!("=== montana-node status ===");
    println!();
    println!("data-dir              : {}", data_dir.display());
    println!("current_window        : {current_window}");
    println!("phase                 : {:?}", lifecycle.phase);
    println!("D (current)           : {}", timechain.current_d);
    if matches!(
        lifecycle.phase,
        crate::node_lifecycle::NodePhase::CandidateVdf
    ) {
        println!(
            "candidate VDF         : {}/{} (осталось {} окон)",
            lifecycle.candidate_progress,
            lifecycle.target_chain_length,
            lifecycle
                .target_chain_length
                .saturating_sub(lifecycle.candidate_progress)
        );
    }
    if matches!(
        lifecycle.phase,
        crate::node_lifecycle::NodePhase::Registered | crate::node_lifecycle::NodePhase::Active
    ) {
        println!("registration_window   : {}", lifecycle.registration_window);
    }
    println!();
    println!("--- ваша identity ---");
    println!("account_id            : {}", hex_lower(&my_account));
    println!("node_id               : {}", hex_lower(&my_node));
    println!(
        "node_pubkey_hex       : {}",
        hex_lower(identity.node_pk.as_bytes())
    );
    println!(
        "account_pubkey_hex    : {}",
        hex_lower(identity.account_pk.as_bytes())
    );
    println!();
    println!("--- размеры таблиц локального state ---");
    println!("AccountTable          : {} записей", state.accounts.len());
    println!(
        "NodeTable             : {} записей (активные узлы)",
        state.nodes.len()
    );
    println!(
        "CandidatePool         : {} записей (ожидают selection event)",
        state.candidates.len()
    );
    println!();

    let in_node_table = state.nodes.contains(&my_node);
    let in_candidate_pool = state.candidates.contains(&my_node);
    let operator_record = state.accounts.get(&my_account);

    println!("--- ваш статус ---");
    if let Some(rec) = operator_record {
        println!("operator account     : найден в AccountTable");
        println!("balance               : {} nɈ", rec.balance);
        println!("is_node_operator      : {}", rec.is_node_operator);
        println!("account_chain_length  : {}", rec.account_chain_length);
    } else {
        println!("operator account      : НЕ НАЙДЕН (state не bootstrapped)");
    }

    if in_node_table {
        println!("узел в Node Table     : ДА — активен");
        let n = state.nodes.get(&my_node).unwrap();
        println!("  start_window         : {}", n.start_window);
        println!("  chain_length         : {}", n.chain_length);
    } else if in_candidate_pool {
        println!("узел в Candidate Pool : ДА — ожидает selection event");
        let c = state.candidates.get(&my_node).unwrap();
        println!("  w_start              : {}", c.w_start);
        println!("  vdf_chain_length     : {}", c.vdf_chain_length);
        println!("  registration_window  : {}", c.registration_window);
        println!("  expires (window)     : {}", c.expires);
        println!("  proof_endpoint       : {}", hex_lower(&c.proof_endpoint));
    } else {
        println!("узел                  : НЕ ЗАРЕГИСТРИРОВАН");
        println!("                        запустите «start» — узел пройдёт через");
        println!("                        candidate VDF (τ₂ окон) → registered → active");
    }
    println!();

    println!("--- параметры протокола ---");
    println!("τ₂ (окон в эпохе)    : {}", params.tau2_windows);
    println!(
        "selection interval   : каждые {} окон",
        params.selection_interval
    );
    println!(
        "admission divisor    : {} (1/{} активных узлов / selection)",
        params.admission_divisor, params.admission_divisor
    );
    println!(
        "candidate expiry     : {} окон от регистрации",
        params.candidate_expiry_windows
    );
    println!(
        "emission per window  : {} nɈ ({} Ɉ)",
        params.emission_moneta,
        params.emission_moneta / 1_000_000_000
    );
    let total_supply = supply_moneta(current_window, params);
    println!(
        "supply (closed-form) : {} nɈ ({} Ɉ) = emission × (W+1)",
        total_supply,
        total_supply / 1_000_000_000
    );

    let total_balances: u128 = state.accounts.iter().map(|a| a.balance).sum();
    println!();
    println!("--- балансы AccountTable ---");
    println!(
        "Σ balances           : {} nɈ ({} Ɉ)",
        total_balances,
        total_balances / 1_000_000_000
    );

    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
