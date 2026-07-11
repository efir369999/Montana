//! [C-4] End-to-End Observable Closure для P2P Network Этап 2 (store-and-forward).
//! Терминал: A депонирует осколки в инбокс спящего B → B фетчит с доказательством
//! владения → восстанавливает сообщение из ≥k осколков (даже при потере n-k) → drop.

use mt_crypto::{keypair_from_seed, PublicKey, SecretKey, PUBLIC_KEY_SIZE};
use mt_overlay::challenge::{sign_fetch, verify_fetch, ChannelHash, Nonce, NONCE_SIZE};
use mt_overlay::erasure::{rs_reconstruct, rs_split};
use mt_overlay::inbox::{bucket_len, epoch_tag, epoch_tag_belongs, N_FETCH};
use mt_overlay::inbox_store::InboxStore;
use mt_state::{derive_account_id, SUITE_MLDSA65};

fn ident(seed: u8) -> ([u8; PUBLIC_KEY_SIZE], SecretKey, [u8; 32]) {
    let (pk, sk): (PublicKey, SecretKey) = keypair_from_seed(&[seed; 32]).unwrap();
    let pkb = *pk.as_bytes();
    let acc = derive_account_id(SUITE_MLDSA65, &pkb);
    (pkb, sk, acc)
}

#[test]
fn e2e_offline_deposit_fetch_reassemble_with_shard_loss() {
    let (_apk, _ask, _aacc) = ident(0xA1);
    let (bpk, bsk, bacc) = ident(0xB2);
    let mut postman = InboxStore::new();

    // Окно депозита. B офлайн; фетчит позже в окне-диапазоне.
    let w_dep = 1000u64;
    let tag = epoch_tag(&bacc, w_dep);

    // A: сообщение → padding до бакета → RS(2,4) split → 4 осколка.
    let sealed = b"opaque-e2e-envelope-for-sleeping-bob-store-and-forward".to_vec();
    let bucket = bucket_len(sealed.len()).expect("<=1MiB");
    let mut padded = sealed.clone();
    padded.resize(bucket, 0);
    let (k, n) = (2usize, 4usize);
    let shards = rs_split(&padded, k, n).unwrap();

    // A депонирует 4 осколка почтальону B (тег принадлежит инбоксу B).
    assert!(epoch_tag_belongs(&bacc, &tag, w_dep, w_dep));
    for (i, sh) in shards.iter().enumerate() {
        postman
            .deposit(
                true,
                tag,
                w_dep,
                [0x5A; 16],
                i as u8,
                n as u8,
                240,
                sh.clone(),
            )
            .expect("deposit");
    }

    // B просыпается позже, фетчит с доказательством владения (challenge Этапа 1 op=fetch).
    let w_fetch = w_dep + 5; // разбег окон в пределах N_FETCH
    let nonce: Nonce = [0x07; NONCE_SIZE];
    let ch: ChannelHash = [0x0C; 32];
    let sig = sign_fetch(&bsk, &tag, &nonce, &ch).unwrap();
    // почтальон: verify подпись + принадлежность тега инбоксу B за окно-диапазон.
    assert!(verify_fetch(&bpk, &tag, &nonce, &ch, &sig));
    assert!(epoch_tag_belongs(
        &bacc,
        &tag,
        w_fetch.saturating_sub(N_FETCH),
        w_fetch
    ));

    let stored = postman.fetch(&tag);
    assert_eq!(stored.len(), 4);

    // ТЕРМИНАЛ: выключаем 2 из 4 почтальонов (теряем 2 осколка) → всё равно восстанавливаем.
    let mut opt: Vec<Option<Vec<u8>>> = vec![None; n];
    opt[1] = Some(stored[1].ct.clone());
    opt[3] = Some(stored[3].ct.clone());
    let recovered = rs_reconstruct(opt, k, n).unwrap();
    assert_eq!(
        &recovered[..sealed.len()],
        &sealed[..],
        "B восстановил конверт из 2 из 4 осколков"
    );

    // drop-on-delivery.
    postman.drop_delivered(&tag, &[0x5A; 16]);
    assert_eq!(postman.fetch(&tag).len(), 0);
}

#[test]
fn e2e_foreign_tag_and_bad_sig_rejected() {
    let (_bpk, bsk, bacc) = ident(0xB2);
    let (epk, _esk, _eacc) = ident(0xE3); // чужак
    let mut postman = InboxStore::new();
    let tag = epoch_tag(&bacc, 1000);

    // Депозит на чужой (не свой) тег — reject.
    assert!(postman
        .deposit(false, tag, 1000, [1; 16], 0, 1, 240, vec![1; 64])
        .is_err());

    // Подпись fetch ключом B, но проверка против pubkey чужака — не сойдётся.
    let nonce: Nonce = [0x07; NONCE_SIZE];
    let ch: ChannelHash = [0x0C; 32];
    let sig = sign_fetch(&bsk, &tag, &nonce, &ch).unwrap();
    assert!(!verify_fetch(&epk, &tag, &nonce, &ch, &sig));
}
