import SwiftUI
import AppKit
import ServiceManagement

// MontanaMonitor — macOS status-bar app that polls the live mesh
// at https://efir.org/explorer/data.json every 15 seconds. Reads-only,
// no privileged access. The popover surfaces the four Genesis-cohort
// validator nodes, the discovered external operators (including the
// local workstation, if onboarded), and a network-wide summary.

@main
struct MontanaMonitorApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    var body: some Scene { Settings { EmptyView() } }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private var popover: NSPopover!
    private let model = MeshModel()

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        refreshStatusButton(nodes: 0, total: 0)
        statusItem.button?.action = #selector(togglePopover(_:))
        statusItem.button?.target = self
        popover = NSPopover()
        popover.contentSize = NSSize(width: 520, height: 540)
        popover.behavior = .transient
        popover.contentViewController = NSHostingController(rootView: MeshView(model: model))
        model.onUpdate = { [weak self] active, total in
            self?.refreshStatusButton(nodes: active, total: total)
        }
        model.startPolling()
    }

    private func refreshStatusButton(nodes: Int, total: Int) {
        guard let btn = statusItem.button else { return }
        let title = total == 0 ? "Mt …" : "Mt \(nodes)/\(total)"
        let icon = NSImage(systemSymbolName: "circle.grid.3x3.fill",
                           accessibilityDescription: "Montana mesh")
        icon?.isTemplate = true
        btn.image = icon
        btn.title = title
        btn.imagePosition = .imageLeading
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
    @Published var localPeerId: String? = nil
    private var timer: Timer?
    var onUpdate: ((Int, Int) -> Void)?

    init() {
        // Best-effort: read the local node's peer_id from the launchd-managed log.
        let logPath = NSHomeDirectory() + "/Applications/Montana/data/logs/montana.err.log"
        if let data = try? String(contentsOfFile: logPath) {
            let lines = data.split(separator: "\n")
            for line in lines.reversed() {
                if let r = line.range(of: "local_peer_id=") {
                    let after = line[r.upperBound...]
                    let pid = after.prefix { $0.isLetter || $0.isNumber }
                    localPeerId = String(pid)
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
                guard let data = data else { self.lastError = "no data"; return }
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
    @State private var launchAtLogin = false
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
            Text("Genesis cohort").font(.subheadline.bold())
            if let nodes = model.doc?.nodes { ForEach(nodes) { NodeRow(node: $0) } }
            if let peers = model.doc?.discovered_peers, !peers.isEmpty {
                Divider()
                Text("Discovered operators").font(.subheadline.bold())
                ForEach(peers) { PeerRow(peer: $0, isLocal: $0.peer_id == model.localPeerId) }
            }
            if let err = model.lastError {
                Divider()
                Text("error: \(err)").font(.caption).foregroundColor(.red)
            }
            Spacer()
            footer
        }
        .padding(14)
        .frame(width: 520)
        .onAppear { launchAtLogin = SMAppService.mainApp.status == .enabled }
    }

    private var header: some View {
        HStack {
            Text("Montana mesh").font(.title3.bold())
            Spacer()
            if let at = model.lastFetchAt {
                Text("updated \(fmt.string(from: at))")
                    .font(.caption).foregroundColor(.secondary)
            }
        }
    }

    private func summary(_ s: NetworkSummary) -> some View {
        HStack(spacing: 20) {
            Stat(label: "active", value: "\(s.active_nodes)/\(s.total_nodes)")
            Stat(label: "discovered", value: "\(s.discovered_peer_count)")
            Stat(label: "window", value: "\(s.max_window)")
            Stat(label: "supply Ɉ", value: format_supply(s.total_supply_nj))
        }
    }

    private var footer: some View {
        HStack {
            Toggle("Launch at login", isOn: $launchAtLogin).font(.caption)
                .onChange(of: launchAtLogin) { _, new in setLaunchAtLogin(new) }
            Spacer()
            Button("Open explorer") { open_url("https://efir.org/explorer/") }
            Button("Open repo") { open_url("https://github.com/efir369999/Montana") }
            Button("Quit") { NSApp.terminate(nil) }
        }
    }

    private func setLaunchAtLogin(_ enabled: Bool) {
        do {
            if enabled { try SMAppService.mainApp.register() }
            else { try SMAppService.mainApp.unregister() }
        } catch {
            launchAtLogin = SMAppService.mainApp.status == .enabled
        }
    }

    private func open_url(_ s: String) { if let u = URL(string: s) { NSWorkspace.shared.open(u) } }

    private func format_supply(_ nj: UInt64) -> String {
        let coin = Double(nj) / 1_000_000_000_000.0
        return String(format: "%.2f M", coin / 1_000_000.0)
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
                .foregroundColor(peer.status == "active" ? .green : .gray)
            Text(peer.label ?? "external").bold().frame(width: 90, alignment: .leading)
            Text(String(peer.peer_id.prefix(14)) + "…")
                .font(.caption.monospaced()).foregroundColor(.secondary)
                .frame(width: 130, alignment: .leading)
            if isLocal {
                Text("you").font(.caption2).padding(.horizontal, 5).padding(.vertical, 1)
                    .background(Color.accentColor.opacity(0.25)).cornerRadius(3)
            }
            Spacer()
            if let hb = peer.last_heartbeat_seconds_ago {
                Text("hb \(hb)s").font(.caption.monospacedDigit())
            }
            if let up = peer.uptime_seconds {
                Text("up \(format_uptime(up))").font(.caption.monospacedDigit())
                    .foregroundColor(.secondary)
                    .frame(width: 60, alignment: .trailing)
            }
        }
    }
    private func format_uptime(_ s: Int) -> String {
        if s < 60 { return "\(s)s" }
        if s < 3600 { return "\(s/60)m" }
        if s < 86400 { return "\(s/3600)h" }
        return "\(s/86400)d"
    }
}
