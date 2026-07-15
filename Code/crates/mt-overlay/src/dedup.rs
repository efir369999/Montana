//! Дедуп msg_id скользящим окном (Этап 1, механика шаг 4).
//! Локальный транспортный кеш, не consensus state.

use std::collections::{HashSet, VecDeque};

use crate::frame::MsgId;

pub const DEDUP_WINDOW_CAP: usize = 4096;

pub struct DedupWindow {
    seen: HashSet<MsgId>,
    order: VecDeque<MsgId>,
    cap: usize,
}

impl Default for DedupWindow {
    fn default() -> Self {
        Self::with_capacity(DEDUP_WINDOW_CAP)
    }
}

impl DedupWindow {
    pub fn with_capacity(cap: usize) -> Self {
        // DEV-050(b): lazy — seen/order растут по мере вставки, НЕ eager-prealloc cap
        // элементов. Иначе каждый первый relay-subscribe новой очереди аллоцировал бы
        // ~160KB (HashSet+VecDeque на 4096) → node-DoS ×32 amplifier при массовой
        // регистрации. cap остаётся логическим потолком скользящего окна.
        Self {
            seen: HashSet::new(),
            order: VecDeque::new(),
            cap: cap.max(1),
        }
    }

    /// true — msg_id новый (принять); false — дубликат (отбросить).
    pub fn check_and_insert(&mut self, id: &MsgId) -> bool {
        if self.seen.contains(id) {
            return false;
        }
        if self.order.len() == self.cap {
            if let Some(old) = self.order.pop_front() {
                self.seen.remove(&old);
            }
        }
        self.order.push_back(*id);
        self.seen.insert(*id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_rejected_fresh_accepted() {
        let mut w = DedupWindow::with_capacity(8);
        assert!(w.check_and_insert(&[1; 16]));
        assert!(!w.check_and_insert(&[1; 16]));
        assert!(w.check_and_insert(&[2; 16]));
    }

    #[test]
    fn window_slides_and_evicts_oldest() {
        let mut w = DedupWindow::with_capacity(2);
        assert!(w.check_and_insert(&[1; 16]));
        assert!(w.check_and_insert(&[2; 16]));
        assert!(w.check_and_insert(&[3; 16])); // вытесняет [1]
        assert!(w.check_and_insert(&[1; 16])); // снова новый после вытеснения
        assert!(!w.check_and_insert(&[3; 16]));
    }
}
