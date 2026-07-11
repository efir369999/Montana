//! [C-4] End-to-End Observable Closure для P2P Network Этап 2 (store-and-forward).
//! Терминал: A депонирует осколки в инбокс спящего B → B фетчит с доказательством
//! владения (E-2: ownership инкапсулирован в verify_fetch) → восстанавливает из ≥k
//! осколков (даже при потере n-k) → drop.

use mt_crypto::{keypair_from_seed, PublicKey, SecretKey, PUBLIC_KEY_SIZE};
use mt_overlay::challenge::{sign_fetch, verify_fetch, ChannelHash, Nonce, NONCE_SIZE};
use mt_overlay::erasure::{rs_reconstruct, rs_split};
use mt_overlay::inbox::{bucket_len, epoch_tag};
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
    let (bpk, bsk, bacc) = ident(0xB2);
    let mut postman = InboxStore::new();
    postman.register_own(bacc); // B — юзер этого почтальона

    let w_dep = 1000u64;
    let tag = epoch_tag(&bacc, w_dep);

    // A: сообщение → padding до бакета → RS(2,4) split → 4 осколка.
    let sealed = b"opaque-e2e-envelope-for-sleeping-bob-store-and-forward".to_vec();
    let bucket = bucket_len(sealed.len()).expect("<=1MiB");
    let mut padded = sealed.clone();
    padded.resize(bucket, 0);
    let (k, n) = (2usize, 4usize);
    let shards = rs_split(&padded, k, n).unwrap();

    // A депонирует 4 осколка (store сам проверяет принадлежность тега инбоксу B, E-3).
    for (i, sh) in shards.iter().enumerate() {
        postman
            .deposit(tag, w_dep, [0x5A; 16], i as u8, n as u8, 240, sh.clone())
            .expect("deposit own tag");
    }

    // B просыпается позже, фетчит. verify_fetch инкапсулирует sig + ownership (E-2).
    let w_fetch = w_dep + 5;
    let nonce: Nonce = [0x07; NONCE_SIZE];
    let ch: ChannelHash = [0x0C; 32];
    let sig = sign_fetch(&bsk, &tag, &nonce, &ch).unwrap();
    assert!(verify_fetch(&bpk, &tag, &nonce, &ch, &sig, w_fetch));

    let stored = postman.fetch(&tag).to_vec();
    assert_eq!(stored.len(), 4);

    // ТЕРМИНАЛ: теряем 2 из 4 → восстанавливаем из оставшихся 2.
    let mut opt: Vec<Option<Vec<u8>>> = vec![None; n];
    opt[1] = Some(stored[1].ct.clone());
    opt[3] = Some(stored[3].ct.clone());
    let recovered = rs_reconstruct(opt, k, n).unwrap();
    assert_eq!(
        &recovered[..sealed.len()],
        &sealed[..],
        "B восстановил конверт из 2 из 4 осколков"
    );

    postman.drop_delivered(&tag, &[0x5A; 16]);
    assert_eq!(postman.fetch(&tag).len(), 0);
}

#[test]
fn e2e_cannot_fetch_foreign_inbox() {
    // E-2 регрессия: B подписывает ЧУЖОЙ epoch_tag (инбокс V) — verify_fetch отвергает
    // (ownership: epoch_tag должен принадлежать инбоксу заявителя).
    let (bpk, bsk, _bacc) = ident(0xB2);
    let (_vpk, _vsk, vacc) = ident(0x44); // жертва V
    let w = 1000u64;
    let victim_tag = epoch_tag(&vacc, w);
    let nonce: Nonce = [0x07; NONCE_SIZE];
    let ch: ChannelHash = [0x0C; 32];
    // B честно подписывает чужой тег своим ключом — подпись валидна, но тег не его.
    let sig = sign_fetch(&bsk, &victim_tag, &nonce, &ch).unwrap();
    assert!(
        !verify_fetch(&bpk, &victim_tag, &nonce, &ch, &sig, w),
        "B не может зафетчить чужой инбокс даже с валидной подписью"
    );
}

#[test]
fn e2e_foreign_deposit_rejected() {
    let (_bpk, _bsk, bacc) = ident(0xB2);
    let mut postman = InboxStore::new();
    postman.register_own(bacc);
    // депозит на тег постороннего аккаунта — reject (store не знает такого own).
    let foreign = epoch_tag(&[0x99; 32], 1000);
    assert!(postman
        .deposit(foreign, 1000, [1; 16], 0, 1, 240, vec![1; 64])
        .is_err());
}
