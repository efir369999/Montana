//! Этап 8 — арбитр выбора пути (прямой vs релей). Классификация достижимости —
//! результат libp2p AutoNAT v2 + Identify observed-address; правило выбора (§6):
//! прямой путь возможен, когда хотя бы один конец на не-симметричном NAT (домашний
//! Wi-Fi / IPv6); двойной симметричный CGNAT → релей через почтальона (Этап 1).
//! DCUtR-пробивка и AutoNAT-зондирование — libp2p-машинерия поверх этого решения;
//! собственных wire-форматов этап не вводит.

/// Класс достижимости узла (результат AutoNAT v2 + Identify observed-address).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NatClass {
    /// Достижимость не определена — AutoNAT ещё не завершил зондирование.
    Unknown,
    /// Публичный адрес (IPv4 без NAT / IPv6 global) — dial-in напрямую.
    Public,
    /// Cone-NAT (full / restricted / port-restricted) — DCUtR hole-punch работает.
    Cone,
    /// Симметричный NAT (CGNAT) — внешний порт непредсказуем, hole-punch ≈невозможен.
    Symmetric,
}

impl NatClass {
    /// Может ли конец участвовать в прямой пробивке (не-симметричный и определён).
    pub fn punchable(&self) -> bool {
        matches!(self, NatClass::Public | NatClass::Cone)
    }
}

/// Выбор пути соединения.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PathChoice {
    /// Прямое соединение (DCUtR hole-punch / публичный dial); почтальон вне контура.
    Direct,
    /// Релей через почтальона (Этап 1) — прямая пробивка недостижима.
    Relay,
}

/// Арбитр §6: прямой путь достижим, когда хотя бы один конец не-симметричный.
/// Двойной симметричный CGNAT → релей. Unknown консервативно не считается punchable
/// (не рискуем прямым, пока AutoNAT не подтвердил достижимость хотя бы одного конца).
pub fn select_path(local: NatClass, peer: NatClass) -> PathChoice {
    if local.punchable() || peer.punchable() {
        PathChoice::Direct
    } else {
        PathChoice::Relay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punchable_matrix() {
        assert!(NatClass::Public.punchable());
        assert!(NatClass::Cone.punchable());
        assert!(!NatClass::Symmetric.punchable());
        assert!(!NatClass::Unknown.punchable());
    }

    #[test]
    fn direct_when_one_end_punchable() {
        assert_eq!(
            select_path(NatClass::Public, NatClass::Symmetric),
            PathChoice::Direct
        );
        assert_eq!(
            select_path(NatClass::Symmetric, NatClass::Cone),
            PathChoice::Direct
        );
        assert_eq!(
            select_path(NatClass::Public, NatClass::Public),
            PathChoice::Direct
        );
        assert_eq!(
            select_path(NatClass::Unknown, NatClass::Public),
            PathChoice::Direct
        );
    }

    #[test]
    fn relay_on_double_symmetric_cgnat() {
        // Два кармана в сотовой сети — прямая пробивка ≈невозможна (§6).
        assert_eq!(
            select_path(NatClass::Symmetric, NatClass::Symmetric),
            PathChoice::Relay
        );
    }

    #[test]
    fn relay_when_unknown_unresolved() {
        assert_eq!(
            select_path(NatClass::Unknown, NatClass::Unknown),
            PathChoice::Relay
        );
        assert_eq!(
            select_path(NatClass::Unknown, NatClass::Symmetric),
            PathChoice::Relay
        );
    }
}
