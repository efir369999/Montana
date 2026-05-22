import SwiftUI
import AppKit

// MontanaMonitor — macOS status-bar app that polls https://efir.org/explorer/data.json
// every 15 seconds and renders the four-node Genesis mesh + discovered operators in a
// popover. Designed for the local Mac operator who wants a glance-state view without
// opening a browser. Network reads only — no privileged access required.

@main
struct MontanaMonitorApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    var body: some Scene {
        Settings { EmptyView() }
    }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private var popover: NSPopover!
    private let model = MeshModel()

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let btn = statusItem.button {
            btn.title = "Montana"
            btn.action = #selector(togglePopover(_:))
            btn.target = self
        }
        popover = NSPopover()
        popover.contentSize = NSSize(width: 480, height: 480)
        popover.behavior = .transient
        popover.contentViewController = NSHostingController(rootView: MeshView(model: model))
        model.startPolling()
    }

    @objc func togglePopover(_ sender: AnyObject?) {
        guard let btn = statusItem.button else { return }
        if popover.isShown { popover.performClose(sender) }
        else { popover.show(relativeTo: btn.bounds, of: btn, preferredEdge: .minY) }
    }
}

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
    private var timer: Timer?

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
                if let error = error {
                    self.lastError = error.localizedDescription
                    return
                }
                guard let data = data else { self.lastError = "no data"; return }
                do {
                    self.doc = try JSONDecoder().decode(ExplorerDoc.self, from: data)
                    self.lastError = nil
                } catch {
                    self.lastError = "decode: \(error)"
                }
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
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text("Montana mesh").font(.headline)
                Spacer()
                if let at = model.lastFetchAt {
                    Text("updated \(fmt.string(from: at))").font(.caption).foregroundColor(.secondary)
                }
            }
            Divider()
            if let s = model.doc?.network_summary {
                HStack(spacing: 16) {
                    Stat(label: "active", value: "\(s.active_nodes)/\(s.total_nodes)")
                    Stat(label: "window", value: "\(s.max_window)")
                    Stat(label: "discovered", value: "\(s.discovered_peer_count)")
                    Stat(label: "supply Ɉ", value: format_supply(s.total_supply_nj))
                }
            }
            Divider()
            if let nodes = model.doc?.nodes {
                ForEach(nodes) { n in NodeRow(node: n) }
            }
            if let peers = model.doc?.discovered_peers, !peers.isEmpty {
                Divider()
                Text("Discovered operators").font(.subheadline).bold()
                ForEach(peers) { p in PeerRow(peer: p) }
            }
            if let err = model.lastError {
                Divider()
                Text("error: \(err)").font(.caption).foregroundColor(.red)
            }
            Spacer()
            HStack {
                Button("Open explorer") {
                    NSWorkspace.shared.open(URL(string: "https://efir.org/explorer/")!)
                }
                Spacer()
                Button("Quit") { NSApp.terminate(nil) }
            }
        }
        .padding(14)
        .frame(width: 480)
    }

    private func format_supply(_ nj: UInt64) -> String {
        let coin = Double(nj) / 1_000_000_000_000.0
        return String(format: "%.3f M", coin / 1_000_000.0)
    }
}

struct Stat: View {
    let label: String
    let value: String
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
        HStack {
            Circle().frame(width: 8, height: 8)
                .foregroundColor(node.status == "active" ? .green : .red)
            Text(node.label).bold()
            Text(node.host).font(.caption).foregroundColor(.secondary)
            Spacer()
            if let w = node.current_window {
                Text("W \(w)").font(.system(.body, design: .monospaced))
            }
            if let p = node.phase { Text(p).font(.caption).foregroundColor(.secondary) }
        }
    }
}

struct PeerRow: View {
    let peer: DiscoveredPeer
    var body: some View {
        HStack {
            Circle().frame(width: 8, height: 8)
                .foregroundColor(peer.status == "active" ? .green : .gray)
            Text(peer.label ?? "external").bold()
            Text(String(peer.peer_id.prefix(14)) + "…").font(.caption.monospaced()).foregroundColor(.secondary)
            Spacer()
            if let hb = peer.last_heartbeat_seconds_ago { Text("hb \(hb)s").font(.caption) }
            if let up = peer.uptime_seconds { Text("up \(format_uptime(up))").font(.caption).foregroundColor(.secondary) }
        }
    }

    private func format_uptime(_ s: Int) -> String {
        if s < 60 { return "\(s)s" }
        if s < 3600 { return "\(s/60)m" }
        if s < 86400 { return "\(s/3600)h" }
        return "\(s/86400)d"
    }
}
