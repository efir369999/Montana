import SwiftUI
import AppKit

struct ExplorerDoc: Decodable {
    let updated: String
    let nodes: [GenesisNode]
    let discovered_peers: [DiscoveredPeer]
    let network_summary: NetworkSummary
}
struct GenesisNode: Decodable, Identifiable {
    let label: String
    let host: String
    let status: String
    let current_window: UInt64?
    let phase: String?
    let supply_nj: UInt64?
    var id: String { label }
}
struct DiscoveredPeer: Decodable, Identifiable {
    let peer_id: String
    let label: String?
    let remote_ip: String
    let witnessed_by: [String]
    let last_heartbeat_seconds_ago: Int?
    let uptime_seconds: Int?
    let status: String
    var id: String { peer_id }
}
struct NetworkSummary: Decodable {
    let active_nodes: Int
    let total_nodes: Int
    let discovered_peer_count: Int
    let max_window: UInt64
    let total_supply_nj: UInt64
}

@MainActor final class MeshModel: ObservableObject {
    @Published var doc: ExplorerDoc?
    @Published var lastError: String?
    @Published var lastFetchAt: Date?
    @Published var localPeerId: String? = nil
    private var timer: Timer?
    var onUpdate: ((Int, Int) -> Void)?

    init() {
        let logPath = NSHomeDirectory() + "/Applications/Montana/data/logs/montana.err.log"
        if let data = try? String(contentsOfFile: logPath) {
            for line in data.split(separator: "\n").reversed() {
                if let r = line.range(of: "local_peer_id=") {
                    let after = line[r.upperBound...]
                    localPeerId = String(after.prefix { $0.isLetter || $0.isNumber })
                    break
                }
            }
        }
    }

    func startPolling() {
        fetch()
        timer = Timer.scheduledTimer(withTimeInterval: 15, repeats: true) { _ in
            Task { @MainActor in self.fetch() }
        }
    }

    private func fetch() {
        guard let url = URL(string: "https://efir.org/explorer/data.json") else { return }
        var req = URLRequest(url: url)
        req.cachePolicy = .reloadIgnoringLocalAndRemoteCacheData
        req.timeoutInterval = 8
        URLSession.shared.dataTask(with: req) { [weak self] data, _, error in
            DispatchQueue.main.async {
                guard let self else { return }
                self.lastFetchAt = Date()
                if let error = error { self.lastError = error.localizedDescription; return }
                guard let data = data else { self.lastError = "пустой ответ"; return }
                do {
                    let doc = try JSONDecoder().decode(ExplorerDoc.self, from: data)
                    self.doc = doc
                    self.lastError = nil
                    self.onUpdate?(doc.network_summary.active_nodes, doc.network_summary.total_nodes)
                } catch { self.lastError = "decode: \(error)" }
            }
        }.resume()
    }
}

