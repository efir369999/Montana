use std::path::PathBuf;

use crate::identity::{default_data_dir, identity_path, load_identity, NodeError};

pub struct InspectArgs {
    pub data_dir: Option<PathBuf>,
    pub reveal_master_seed: bool,
}

pub fn run(args: InspectArgs) -> Result<(), NodeError> {
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    let identity = load_identity(&data_dir)?;

    println!("=== montana-node inspect ===");
    println!();
    println!("data-dir         : {}", data_dir.display());
    println!("identity         : {}", identity_path(&data_dir).display());
    println!(
        "suite            : ML-DSA-65 (suite_id = {})",
        identity.suite_id as u16
    );
    println!();
    println!("account_id       : {}", hex_lower(&identity.account_id()));
    println!("node_id          : {}", hex_lower(&identity.node_id()));
    println!(
        "master_seed_fp   : {}",
        hex_lower(&identity.master_seed_fingerprint())
    );
    println!();
    println!(
        "account_pk       : {} bytes",
        identity.account_pk.as_bytes().len()
    );
    println!(
        "account_pk[..16] : {}",
        hex_lower(&identity.account_pk.as_bytes()[..16])
    );
    println!(
        "node_pk          : {} bytes",
        identity.node_pk.as_bytes().len()
    );
    println!(
        "node_pk[..16]    : {}",
        hex_lower(&identity.node_pk.as_bytes()[..16])
    );
    println!(
        "mlkem_pk         : {} bytes",
        identity.mlkem_pk.as_bytes().len()
    );
    println!(
        "mlkem_pk[..16]   : {}",
        hex_lower(&identity.mlkem_pk.as_bytes()[..16])
    );
    println!();
    println!("--- libp2p transport identity (M8 cross-machine) ---");
    match mt_net_transport::derive_peer_id(&identity.node_pk) {
        Ok(pid) => println!(
            "network_peer_id  : {pid}  (Noise_PQ XX — укажите в genesis-manifest)"
        ),
        Err(e) => println!("network_peer_id  : <ошибка вывода: {e}>"),
    }
    println!(
        "libp2p_peer_id   : {}  (legacy Ed25519, транспортом Noise_PQ XX не используется)",
        identity.libp2p_peer_id()
    );

    if args.reveal_master_seed {
        println!();
        println!("--- master_seed (полный, 64 байта) ---");
        println!("{}", hex_lower(&identity.master_seed));
    }

    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
