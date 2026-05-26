// Egress client-side exit selection.
// spec: Montana Egress v1.0.0 (Session establishment, step 2 — Exit selection).
//
// Manual: the directory entry whose country_code equals the chosen country.
// Auto:   the reachable exit ranked highest by the reachability map for the
//         client's vantage (Network -> Reachability sensing). Exit selection is
//         performed by the client, never dictated by the entry; the chosen exit
//         is confirmed by a direct IBT handshake before any egress.

use alloc::vec::Vec;

use mt_net::ReachabilityMap;

use crate::EgressDirectoryEntry;

/// Exit selection mode requested by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitSelector {
    /// Manual: exit in the given ISO-3166-1 alpha-2 jurisdiction.
    Manual([u8; 2]),
    /// Auto: highest-reachability exit for the client's vantage.
    Auto,
}

/// Select an exit node_id from the advisory directory.
///
/// Manual mode picks the highest-capacity entry whose country matches.
/// Auto mode ranks the directory's exits by the reachability map for the
/// client's (country_code, asn) vantage and picks the top; with no actionable
/// map data it falls back to the highest-capacity entry. Returns None when no
/// candidate satisfies the request.
pub fn select_exit(
    directory: &[EgressDirectoryEntry],
    selector: &ExitSelector,
    map: &ReachabilityMap,
    vantage_country: [u8; 2],
    vantage_asn: u32,
) -> Option<[u8; 32]> {
    match selector {
        ExitSelector::Manual(country) => directory
            .iter()
            .filter(|e| &e.country_code == country)
            .max_by_key(|e| e.capacity_class)
            .map(|e| e.exit_node_id),
        ExitSelector::Auto => {
            let candidates: Vec<[u8; 32]> = directory.iter().map(|e| e.exit_node_id).collect();
            if candidates.is_empty() {
                return None;
            }
            let ranked = map.steer(&candidates, vantage_country, vantage_asn);
            // steer() returns reachability-ranked first, then unranked in input
            // order; if the map had any actionable data the head is the best
            // reachable exit. Fall back to highest capacity when the head is an
            // unranked candidate (map empty).
            let head = ranked.first().copied();
            match head {
                Some(id)
                    if directory
                        .iter()
                        .any(|e| e.exit_node_id == id) =>
                {
                    // prefer reachability head; if map was empty steer preserves
                    // input order, so refine to highest capacity for determinism
                    if map.is_empty() {
                        directory.iter().max_by_key(|e| e.capacity_class).map(|e| e.exit_node_id)
                    } else {
                        Some(id)
                    }
                },
                _ => directory.iter().max_by_key(|e| e.capacity_class).map(|e| e.exit_node_id),
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: u8, cc: &[u8; 2], cap: u8) -> EgressDirectoryEntry {
        EgressDirectoryEntry {
            exit_node_id: [id; 32],
            country_code: *cc,
            capacity_class: cap,
            advertised_window: 20160,
        }
    }

    #[test]
    fn manual_picks_matching_country_highest_capacity() {
        let dir = [
            entry(1, b"FR", 1),
            entry(2, b"DE", 2),
            entry(3, b"FR", 2), // higher capacity FR
        ];
        let map = ReachabilityMap::new();
        let pick = select_exit(&dir, &ExitSelector::Manual(*b"FR"), &map, *b"AM", 1);
        assert_eq!(pick, Some([3u8; 32]));
    }

    #[test]
    fn manual_none_when_country_absent() {
        let dir = [entry(1, b"FR", 1)];
        let map = ReachabilityMap::new();
        assert_eq!(select_exit(&dir, &ExitSelector::Manual(*b"US"), &map, *b"AM", 1), None);
    }

    #[test]
    fn auto_empty_directory_none() {
        let dir: [EgressDirectoryEntry; 0] = [];
        let map = ReachabilityMap::new();
        assert_eq!(select_exit(&dir, &ExitSelector::Auto, &map, *b"AM", 1), None);
    }

    #[test]
    fn auto_falls_back_to_capacity_when_map_empty() {
        let dir = [entry(1, b"FR", 0), entry(2, b"DE", 2), entry(3, b"US", 1)];
        let map = ReachabilityMap::new();
        // map empty -> highest capacity (id 2)
        assert_eq!(select_exit(&dir, &ExitSelector::Auto, &map, *b"AM", 1), Some([2u8; 32]));
    }

    #[test]
    fn auto_prefers_reachability_head() {
        use mt_net::ReachabilityAdvert;
        let dir = [entry(1, b"FR", 2), entry(2, b"DE", 0)];
        let mut map = ReachabilityMap::new();
        // make exit id 2 highly reachable from vantage AM/asn=1 across 3 /16
        let mk = |target: [u8; 32], n: u16, d: u16| ReachabilityAdvert {
            country_code: *b"AM",
            asn: 1,
            target_ref: target,
            profile: 0,
            reachable_num: n,
            reachable_den: d,
            observed_window: 20160,
        };
        for src in [[1u8, 0], [2, 0], [3, 0]] {
            map.ingest(mk([2u8; 32], 9, 10), src, 20160, 100); // DE exit, 0.9
        }
        // Auto should pick id 2 (reachable) over higher-capacity id 1 (no map data)
        let pick = select_exit(&dir, &ExitSelector::Auto, &map, *b"AM", 1);
        assert_eq!(pick, Some([2u8; 32]));
    }
}
