use std::path::PathBuf;

use crate::identity::{default_data_dir, save_identity, Identity, NodeError, IDENTITY_FILE_SIZE};

pub struct InitArgs {
    pub data_dir: Option<PathBuf>,
    pub mnemonic: Option<String>,
    pub entropy_hex: Option<String>,
    pub force: bool,
}

pub fn run(args: InitArgs) -> Result<(), NodeError> {
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);

    let identity = match (args.mnemonic.as_deref(), args.entropy_hex.as_deref()) {
        (Some(_), Some(_)) => {
            return Err(NodeError::InvalidArguments(
                "укажите либо --mnemonic, либо --entropy, не оба сразу".into(),
            ))
        },
        (Some(m), None) => Identity::from_mnemonic(m.trim())?,
        (None, Some(hex)) => {
            let entropy = parse_entropy_hex(hex)?;
            Identity::from_entropy(&entropy)?
        },
        (None, None) => {
            let mut entropy = [0u8; 32];
            getrandom::getrandom(&mut entropy)
                .map_err(|e| NodeError::InvalidArguments(format!("getrandom: {e}")))?;
            Identity::from_entropy(&entropy)?
        },
    };

    let path = save_identity(&data_dir, &identity, args.force)?;

    println!("=== montana-node init ===");
    println!();
    println!("data-dir         : {}", data_dir.display());
    println!("identity         : {}", path.display());
    println!("file size        : {IDENTITY_FILE_SIZE} bytes");
    println!("mode             : 0600 (только владелец)");
    println!();
    println!("--- мнемоника (24 слова — запишите в надёжное место) ---");
    print_mnemonic_grid(&identity.mnemonic);
    println!();
    println!("--- терминальные идентификаторы ---");
    println!("account_id       : {}", hex_lower(&identity.account_id()));
    println!("node_id          : {}", hex_lower(&identity.node_id()));
    println!(
        "master_seed_fp   : {} (8-байтный отпечаток, не секрет)",
        hex_lower(&identity.master_seed_fingerprint())
    );
    println!();
    println!("Секретные ключи (account_sk/node_sk/mlkem_sk) сохранены в identity.bin");
    println!("и не выводятся на экран. Для backup используйте мнемонику выше.");

    Ok(())
}

fn parse_entropy_hex(s: &str) -> Result<[u8; 32], NodeError> {
    let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if clean.len() != 64 {
        return Err(NodeError::InvalidEntropyHex);
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let byte = u8::from_str_radix(&clean[i * 2..i * 2 + 2], 16)
            .map_err(|_| NodeError::InvalidEntropyHex)?;
        out[i] = byte;
    }
    Ok(out)
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn print_mnemonic_grid(mnemonic: &str) {
    let words: Vec<&str> = mnemonic.split(' ').collect();
    for (i, w) in words.iter().enumerate() {
        print!("  [{:>2}] {:<10}", i + 1, w);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }
    if words.len() % 4 != 0 {
        println!();
    }
}
