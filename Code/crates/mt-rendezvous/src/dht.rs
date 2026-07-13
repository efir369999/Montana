//! Mainline BitTorrent DHT-транспорт рандеву (Этап 4): BEP5+BEP44 put/get реальной записи.
//! Обёртка над crate `mainline` (BEP44 mutable). Ядро byte-exact (lib.rs) даёт RendezvousRecord;
//! здесь — сетевой put/get. Testnet — локальная DHT для running-теста без реальной сети.

use mainline::{Dht, MutableItem};

use crate::{dht_pubkey, dht_signing_key, RendezvousRecord, RvError, DHT_SEED_LEN};

pub struct RvDht {
    dht: Dht,
}

impl RvDht {
    /// Клиент на реальную Mainline DHT (публичные bootstrap-ноды BitTorrent).
    pub fn client() -> Result<Self, RvError> {
        let dht = Dht::client().map_err(|e| RvError::Dht(format!("client: {e}")))?;
        Ok(Self { dht })
    }

    /// Клиент на локальную Testnet-DHT (running-тест без реальной сети).
    pub fn from_dht(dht: Dht) -> Self {
        Self { dht }
    }

    /// Положить рандеву-запись в DHT под target=SHA1(dk‖salt). mainline подписывает BEP44
    /// нашим dht_key (ed25519). seq монотонный (BEP44 anti-rollback).
    pub fn put(
        &self,
        dht_seed: &[u8; DHT_SEED_LEN],
        salt: &[u8; crate::SALT_LEN],
        seq: u64,
        record: &RendezvousRecord,
    ) -> Result<(), RvError> {
        let v = record.to_bytes();
        if v.len() > crate::MAX_RECORD_BYTES {
            return Err(RvError::TooLarge(v.len()));
        }
        let signer = dht_signing_key(dht_seed);
        let item = MutableItem::new(signer, &v, seq as i64, Some(salt));
        self.dht
            .put_mutable(item, None)
            .map_err(|e| RvError::Dht(format!("put: {e}")))?;
        Ok(())
    }

    /// Прочитать рандеву-запись из DHT по dk+salt. Возвращает первую валидно
    /// декодируемую запись (mainline уже проверил BEP44-подпись против dk).
    pub fn get(
        &self,
        dk: &[u8; crate::DK_LEN],
        salt: &[u8; crate::SALT_LEN],
    ) -> Option<RendezvousRecord> {
        let items = self.dht.get_mutable(dk, Some(salt), None);
        for item in items {
            if let Ok(r) = RendezvousRecord::decode(item.value()) {
                return Some(r);
            }
        }
        None
    }
}

/// Предподписанная запись (P2): лист подписывает offline своим dht_key, почтальон
/// пере-put'ит её БЕЗ секрета листа (`put_presigned`). Секрет dht_key не покидает лист.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PresignedRecord {
    pub dk: [u8; crate::DK_LEN],
    pub salt: [u8; crate::SALT_LEN],
    pub seq: u64,
    pub value: Vec<u8>, // RendezvousRecord.to_bytes()
    pub sig: [u8; 64],
}

/// Лист (держатель dht_key секрета) предподписывает пачку записей с возрастающим seq
/// и своим valid_until окном — почтальон пере-анонсирует без секрета (стена свежести P2).
pub fn prepare_batch(
    dht_seed: &[u8; DHT_SEED_LEN],
    salt: &[u8; crate::SALT_LEN],
    base_seq: u64,
    mut records: Vec<RendezvousRecord>,
) -> Result<Vec<PresignedRecord>, RvError> {
    let sk = dht_signing_key(dht_seed);
    let dk = dht_pubkey(&sk);
    let mut out = Vec::with_capacity(records.len());
    for (i, rec) in records.iter_mut().enumerate() {
        let seq = base_seq + i as u64;
        rec.seq = seq;
        let sig = crate::sign_record(&sk, salt, seq, rec)?;
        out.push(PresignedRecord {
            dk,
            salt: *salt,
            seq,
            value: rec.to_bytes(),
            sig: sig.to_bytes(),
        });
    }
    Ok(out)
}

impl RvDht {
    /// Почтальон пере-put'ит предподписанную запись листа — БЕЗ его секрета
    /// (mainline new_signed_unchecked с готовой ed25519-подписью; DHT-ноды её проверяют).
    pub fn put_presigned(&self, pre: &PresignedRecord) -> Result<(), RvError> {
        let item = MutableItem::new_signed_unchecked(
            pre.dk,
            pre.sig,
            &pre.value,
            pre.seq as i64,
            Some(&pre.salt),
        );
        self.dht
            .put_mutable(item, None)
            .map_err(|e| RvError::Dht(format!("put_presigned: {e}")))?;
        Ok(())
    }
}
