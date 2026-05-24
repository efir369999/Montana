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

struct MeshView: View {
    @ObservedObject var model: MeshModel
    private let fmt: DateFormatter = {
        let f = DateFormatter()
        f.dateFormat = "HH:mm:ss"
        return f
    }()

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header
            Divider()
            if let s = model.doc?.network_summary { summary(s) }
            Divider()
            Text("Узлы Монтаны").font(.subheadline.bold())
            if let nodes = model.doc?.nodes { ForEach(nodes) { NodeRow(node: $0) } }
            if let peers = model.doc?.discovered_peers, !peers.isEmpty {
                Divider()
                Text("Найденные").font(.subheadline.bold())
                ForEach(peers) { PeerRow(peer: $0, isLocal: $0.peer_id == model.localPeerId) }
            }
            if let err = model.lastError {
                Divider()
                Text("ошибка: \(err)").font(.caption).foregroundColor(.red)
            }
            Spacer(minLength: 0)
        }
        .padding(14)
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 0) {
                Text("Сеть Монтана").font(.title3.bold())
                Text("Montana Ядро 0.1").font(.caption2).foregroundColor(.secondary)
            }
            Spacer()
            if let at = model.lastFetchAt {
                Text("обновлено \(fmt.string(from: at))")
                    .font(.caption).foregroundColor(.secondary)
            }
        }
    }

    private func summary(_ s: NetworkSummary) -> some View {
        HStack(spacing: 20) {
            Stat(label: "активны", value: "\(s.active_nodes)/\(s.total_nodes)")
            Stat(label: "найдено", value: "\(s.discovered_peer_count)")
            Stat(label: "окно", value: "\(s.max_window)")
            Stat(label: "supply Ɉ", value: format_supply(s.total_supply_nj))
        }
    }

    private func format_supply(_ nj: UInt64) -> String {
        let coin = Double(nj) / 1_000_000_000_000.0
        return String(format: "%.2f М", coin / 1_000_000.0)
    }
}

struct Stat: View {
    let label: String; let value: String
    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label).font(.caption).foregroundColor(.secondary)
            Text(value).font(.headline.monospacedDigit())
        }
    }
}

struct NodeRow: View {
    let node: GenesisNode
    var body: some View {
        HStack(spacing: 10) {
            Circle().frame(width: 8, height: 8)
                .foregroundColor(node.status == "active" ? .green : .red)
            Text(node.label).bold().frame(width: 90, alignment: .leading)
            Text(node.host).font(.caption).foregroundColor(.secondary).frame(width: 70, alignment: .leading)
            Spacer()
            if let w = node.current_window {
                Text("W \(w)").font(.system(.body, design: .monospaced))
            }
            if let p = node.phase {
                Text(p).font(.caption).foregroundColor(.secondary).frame(width: 100, alignment: .trailing)
            }
        }
    }
}

struct PeerRow: View {
    let peer: DiscoveredPeer
    let isLocal: Bool
    var body: some View {
        HStack(spacing: 10) {
            Circle().frame(width: 8, height: 8)
                .foregroundColor(peer.status == "online" ? .green : .gray)
            Text(peer.label ?? String(peer.peer_id.prefix(12)))
                .frame(width: 140, alignment: .leading)
                .font(isLocal ? .body.bold() : .body)
            if isLocal {
                Text("(вы)").font(.caption).foregroundColor(.blue)
            }
            Spacer()
            if let ago = peer.last_heartbeat_seconds_ago {
                Text("\(ago)s").font(.caption).foregroundColor(.secondary)
            }
        }
    }
}
