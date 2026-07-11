//! Reed-Solomon эрейжер-код (Этап 2): бакетированный шифртекст → n осколков,
//! восстановление из любых k. Спека: Montana P2P Network, Этап 2 «Erasure-код»
//! (RS(k,n) над GF(2⁸); продакшн RS(2,4), стенд RS(2,3)).

use reed_solomon_erasure::galois_8::ReedSolomon;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErasureError {
    BadParams,
    Encode,
    Reconstruct,
    NotEnoughShards,
}

/// Разбить `data` на `n` осколков равного размера так, что любые `k` восстанавливают.
/// data паддится нулями до кратности k (длина хранится вызывающим для точного usreassembly —
/// в Montana реальную длину несёт AEAD-конверт внутри шифртекста).
pub fn rs_split(data: &[u8], k: usize, n: usize) -> Result<Vec<Vec<u8>>, ErasureError> {
    if k == 0 || n <= k {
        return Err(ErasureError::BadParams);
    }
    let shard_len = ((data.len() + k - 1) / k).max(1);
    let mut shards: Vec<Vec<u8>> = Vec::with_capacity(n);
    for i in 0..k {
        let start = i * shard_len;
        let mut sh = vec![0u8; shard_len];
        if start < data.len() {
            let end = (start + shard_len).min(data.len());
            sh[..end - start].copy_from_slice(&data[start..end]);
        }
        shards.push(sh);
    }
    for _ in k..n {
        shards.push(vec![0u8; shard_len]);
    }
    let rs = ReedSolomon::new(k, n - k).map_err(|_| ErasureError::BadParams)?;
    rs.encode(&mut shards).map_err(|_| ErasureError::Encode)?;
    Ok(shards)
}

/// Восстановить исходные `k*shard_len` байт из осколков (None = потерянный).
/// Возвращает конкатенацию k data-осколков (с нулевым паддингом хвоста).
pub fn rs_reconstruct(
    mut shards: Vec<Option<Vec<u8>>>,
    k: usize,
    n: usize,
) -> Result<Vec<u8>, ErasureError> {
    if k == 0 || n <= k || shards.len() != n {
        return Err(ErasureError::BadParams);
    }
    if shards.iter().filter(|s| s.is_some()).count() < k {
        return Err(ErasureError::NotEnoughShards);
    }
    let rs = ReedSolomon::new(k, n - k).map_err(|_| ErasureError::BadParams)?;
    rs.reconstruct(&mut shards)
        .map_err(|_| ErasureError::Reconstruct)?;
    let mut out = Vec::new();
    for sh in shards.into_iter().take(k) {
        out.extend_from_slice(&sh.ok_or(ErasureError::Reconstruct)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn padded(data: &[u8], k: usize) -> Vec<u8> {
        let mut d = data.to_vec();
        let sl = ((data.len() + k - 1) / k).max(1);
        d.resize(sl * k, 0);
        d
    }

    #[test]
    fn split_reconstruct_full() {
        let data = b"montana-store-and-forward-shard-test-payload".to_vec();
        let (k, n) = (2, 4);
        let shards = rs_split(&data, k, n).unwrap();
        assert_eq!(shards.len(), n);
        let opt: Vec<Option<Vec<u8>>> = shards.iter().cloned().map(Some).collect();
        assert_eq!(rs_reconstruct(opt, k, n).unwrap(), padded(&data, k));
    }

    #[test]
    fn reconstruct_from_any_k_of_n_prod_2_4() {
        let data = vec![0x5Au8; 500];
        let (k, n) = (2, 4);
        let shards = rs_split(&data, k, n).unwrap();
        // теряем любые 2 из 4 — восстанавливаем из оставшихся 2
        for lost in [(0usize, 1usize), (0, 2), (1, 3), (2, 3)] {
            let mut opt: Vec<Option<Vec<u8>>> = shards.iter().cloned().map(Some).collect();
            opt[lost.0] = None;
            opt[lost.1] = None;
            assert_eq!(rs_reconstruct(opt, k, n).unwrap(), padded(&data, k));
        }
    }

    #[test]
    fn stand_2_3_survives_one_loss() {
        let data = vec![0x33u8; 300];
        let (k, n) = (2, 3);
        let shards = rs_split(&data, k, n).unwrap();
        for lost in 0..n {
            let mut opt: Vec<Option<Vec<u8>>> = shards.iter().cloned().map(Some).collect();
            opt[lost] = None;
            assert_eq!(rs_reconstruct(opt, k, n).unwrap(), padded(&data, k));
        }
    }

    #[test]
    fn too_many_losses_fails() {
        let data = vec![1u8; 100];
        let shards = rs_split(&data, 2, 4).unwrap();
        let mut opt: Vec<Option<Vec<u8>>> = shards.iter().cloned().map(Some).collect();
        opt[0] = None;
        opt[1] = None;
        opt[2] = None; // потеряли 3 из 4, k=2 не хватает
        assert_eq!(
            rs_reconstruct(opt, 2, 4),
            Err(ErasureError::NotEnoughShards)
        );
    }
}
