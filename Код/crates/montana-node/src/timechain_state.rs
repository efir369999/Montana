use std::fs;
use std::path::{Path, PathBuf};

use mt_codec::domain;
use mt_crypto::{hash, Hash32};
use mt_genesis::genesis_params;

use crate::clock::meta_dir;
use crate::identity::NodeError;

pub const TIMECHAIN_FILE: &str = "timechain.bin";
const TIMECHAIN_MAGIC: &[u8; 4] = b"mttc";
const TIMECHAIN_VERSION: u8 = 1;
const TIMECHAIN_SIZE: usize = 4 + 1 + 3 + 32 + 8 + 8;

pub struct TimeChainState {
    pub t_r: Hash32,
    pub current_d: u64,
    pub last_window: u64,
}

pub fn timechain_path(data_dir: &Path) -> PathBuf {
    meta_dir(data_dir).join(TIMECHAIN_FILE)
}

pub fn genesis_t_r() -> Hash32 {
    hash(domain::TIMECHAIN, &[&[0u8; 32]])
}

pub fn load_or_init_timechain(data_dir: &Path) -> Result<TimeChainState, NodeError> {
    let path = timechain_path(data_dir);
    if !path.exists() {
        let params = genesis_params();
        return Ok(TimeChainState {
            t_r: genesis_t_r(),
            current_d: params.d0,
            last_window: 0,
        });
    }
    let bytes = fs::read(&path)?;
    if bytes.len() != TIMECHAIN_SIZE {
        return Err(NodeError::CorruptedSize {
            expected: TIMECHAIN_SIZE,
            actual: bytes.len(),
        });
    }
    if &bytes[0..4] != TIMECHAIN_MAGIC.as_slice() {
        return Err(NodeError::InvalidMagic);
    }
    if bytes[4] != TIMECHAIN_VERSION {
        return Err(NodeError::UnsupportedVersion(bytes[4]));
    }
    let mut t_r = [0u8; 32];
    t_r.copy_from_slice(&bytes[8..40]);
    let current_d = u64::from_le_bytes(bytes[40..48].try_into().unwrap());
    let last_window = u64::from_le_bytes(bytes[48..56].try_into().unwrap());
    Ok(TimeChainState {
        t_r,
        current_d,
        last_window,
    })
}

pub fn save_timechain(data_dir: &Path, state: &TimeChainState) -> Result<(), NodeError> {
    fs::create_dir_all(meta_dir(data_dir))?;
    let path = timechain_path(data_dir);
    let mut buf = vec![0u8; TIMECHAIN_SIZE];
    buf[0..4].copy_from_slice(TIMECHAIN_MAGIC);
    buf[4] = TIMECHAIN_VERSION;
    buf[8..40].copy_from_slice(&state.t_r);
    buf[40..48].copy_from_slice(&state.current_d.to_le_bytes());
    buf[48..56].copy_from_slice(&state.last_window.to_le_bytes());
    let tmp = path.with_extension("bin.tmp");
    fs::write(&tmp, &buf)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}
