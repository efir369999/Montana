import SwiftUI
import Foundation
import Network

enum VPNState: String, Equatable, Codable { case disconnected, connecting, connected, error }

struct VPNServer: Identifiable, Hashable, Codable {
    let id: String
    let city: String
    let country: String
    let flag: String
    let host: String
    let port: UInt16
    let uuid: String
    let pbk: String
    let sid: String
    let sni: String
    let flow: String
    let fp: String
    var pingMs: Int?
    var source: ServerSource = .subscription("default")
}

enum ServerSource: Hashable, Codable {
    case subscription(String)   // id of the subscription
    case manual                  // user-added
}

struct Subscription: Identifiable, Codable, Hashable {
    let id: String
    var name: String
    var url: String
    var lastRefreshed: Date?
    var serversCount: Int = 0
}

@MainActor
final class VPNModel: ObservableObject {
    @Published var servers: [VPNServer] = []
    @Published var manualServers: [VPNServer] = []
    @Published var subscriptions: [Subscription] = []
    @Published var selected: VPNServer.ID? = nil
    @Published var state: VPNState = .disconnected
    @Published var statusLine: String = ""
    @Published var bytesUp: UInt64 = 0
    @Published var bytesDown: UInt64 = 0
    @Published var lastError: String? = nil

    var onStateChange: (() -> Void)?
    private let xray = XrayController()
    private var refreshTimer: Timer?
    private var statsTimer: Timer?

    private let subsKey = "vpn.subscriptions"
    private let manualKey = "vpn.manualServers"
    private let selectedKey = "vpn.selected"
    private(set) var subRefreshMin: Int = 30

    var allServers: [VPNServer] { servers + manualServers }
    var currentServer: VPNServer? {
        if let id = selected { return allServers.first(where: { $0.id == id }) ?? allServers.first }
        return allServers.first
    }

    func bootstrap() {
        loadSubs()
        loadManual()
        if let s = UserDefaults.standard.string(forKey: selectedKey) { selected = s }
        refreshAll()
        scheduleAutoRefresh(min: subRefreshMin)
    }

    func setAutoRefreshInterval(_ min: Int) {
        subRefreshMin = min
        scheduleAutoRefresh(min: min)
    }
    private func scheduleAutoRefresh(min: Int) {
        refreshTimer?.invalidate()
        let interval = TimeInterval(max(60, min * 60))
        refreshTimer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { _ in
            Task { @MainActor in self.refreshAll() }
        }
    }

    // MARK: - Subscriptions persistence

    private func loadSubs() {
        if let data = UserDefaults.standard.data(forKey: subsKey),
           let arr = try? JSONDecoder().decode([Subscription].self, from: data),
           !arr.isEmpty {
            subscriptions = arr
        } else {
            subscriptions = [Subscription(id: "default", name: "default",
                                          url: "https://montana.quest/vpn/sub",
                                          lastRefreshed: nil, serversCount: 0)]
            saveSubs()
        }
    }
    private func saveSubs() {
        if let data = try? JSONEncoder().encode(subscriptions) {
            UserDefaults.standard.set(data, forKey: subsKey)
        }
    }
    func addSubscription(url: String, name: String? = nil) {
        let id = UUID().uuidString
        subscriptions.append(Subscription(id: id, name: name ?? "sub-\(subscriptions.count+1)",
                                          url: url, lastRefreshed: nil, serversCount: 0))
        saveSubs()
        refresh(subId: id)
    }
    func removeSubscription(_ sub: Subscription) {
        subscriptions.removeAll { $0.id == sub.id }
        servers.removeAll {
            if case .subscription(let sid) = $0.source { return sid == sub.id }
            return false
        }
        saveSubs()
    }

    // MARK: - Manual servers persistence

    private func loadManual() {
        if let data = UserDefaults.standard.data(forKey: manualKey),
           let arr = try? JSONDecoder().decode([VPNServer].self, from: data) {
            manualServers = arr
            for s in manualServers { pingServer(s) }
        }
    }
    private func saveManual() {
        if let data = try? JSONEncoder().encode(manualServers) {
            UserDefaults.standard.set(data, forKey: manualKey)
        }
    }
    func addManualServer(url: String, customName: String?) -> Bool {
        guard var s = Self.parseVless(url) else { return false }
        s = VPNServer(id: "manual-\(UUID().uuidString.prefix(8))",
                      city: customName?.isEmpty == false ? customName! : s.city,
                      country: s.country, flag: s.flag, host: s.host, port: s.port,
                      uuid: s.uuid, pbk: s.pbk, sid: s.sid, sni: s.sni, flow: s.flow, fp: s.fp,
                      pingMs: nil, source: .manual)
        manualServers.append(s)
        saveManual()
        pingServer(s)
        return true
    }
    func removeManualServer(_ s: VPNServer) {
        manualServers.removeAll { $0.id == s.id }
        saveManual()
    }

    // MARK: - Refresh

    func refreshAll() {
        for sub in subscriptions { refresh(subId: sub.id) }
    }
    func refresh(subId: String) {
        guard let sub = subscriptions.first(where: { $0.id == subId }),
              let url = URL(string: sub.url) else { return }
        var req = URLRequest(url: url)
        req.cachePolicy = .reloadIgnoringLocalAndRemoteCacheData
        req.timeoutInterval = 8
        URLSession.shared.dataTask(with: req) { [weak self] data, _, _ in
            guard let data, let text = String(data: data, encoding: .utf8) else { return }
            let decoded = Self.decodeBase64Sub(text) ?? text
            let parsed = decoded.split(separator: "\n")
                .compactMap { Self.parseVless(String($0)) }
                .map { srv in
                    var s = srv
                    s = VPNServer(id: "\(subId)-\(s.host)", city: s.city, country: s.country,
                                  flag: s.flag, host: s.host, port: s.port, uuid: s.uuid,
                                  pbk: s.pbk, sid: s.sid, sni: s.sni, flow: s.flow, fp: s.fp,
                                  pingMs: nil, source: .subscription(subId))
                    return s
                }
            Task { @MainActor in self?.apply(subId: subId, list: parsed) }
        }.resume()
    }
    private func apply(subId: String, list: [VPNServer]) {
        servers.removeAll {
            if case .subscription(let sid) = $0.source { return sid == subId }
            return false
        }
        servers.append(contentsOf: list)
        if let idx = subscriptions.firstIndex(where: { $0.id == subId }) {
            subscriptions[idx].lastRefreshed = Date()
            subscriptions[idx].serversCount = list.count
            saveSubs()
        }
        if selected == nil { selected = allServers.first?.id }
        for s in list { pingServer(s) }
    }
    func pingAll() {
        for s in allServers { pingServer(s) }
    }
    private func pingServer(_ s: VPNServer) {
        let host = NWEndpoint.Host(s.host)
        guard let port = NWEndpoint.Port(rawValue: s.port) else { return }
        let conn = NWConnection(host: host, port: port, using: .tcp)
        let started = Date()
        conn.stateUpdateHandler = { [weak self, weak conn] st in
            switch st {
            case .ready:
                let ms = Int(Date().timeIntervalSince(started) * 1000)
                Task { @MainActor in self?.setPing(s.id, ms: ms) }
                conn?.cancel()
            case .failed, .cancelled:
                conn?.cancel()
            default: break
            }
        }
        conn.start(queue: .global(qos: .utility))
        DispatchQueue.global().asyncAfter(deadline: .now() + 4) { conn.cancel() }
    }
    private func setPing(_ id: VPNServer.ID, ms: Int) {
        if let i = servers.firstIndex(where: { $0.id == id }) { servers[i].pingMs = ms }
        if let i = manualServers.firstIndex(where: { $0.id == id }) {
            manualServers[i].pingMs = ms; saveManual()
        }
    }

    // MARK: - Connect

    func select(_ id: VPNServer.ID) {
        selected = id
        UserDefaults.standard.set(id, forKey: selectedKey)
    }

    /// View устанавливает это: показать Montana-пояснение перед первым privileged-запросом.
    /// Возвращает true если пользователь разрешил.
    var confirmFirstSetup: (() -> Bool)?

    func connect() {
        guard let s = currentServer else { return }
        if !XrayController.helperInstalled {
            let allowed = confirmFirstSetup?() ?? true
            if !allowed { return }
        }
        state = .connecting
        statusLine = "→ \(s.city)"
        onStateChange?()
        Task.detached { [weak self] in
            do {
                try await self?.xray.start(server: s)
                try await XrayController.setSystemProxy(enable: true)
                await MainActor.run {
                    guard let self else { return }
                    self.state = .connected
                    self.statusLine = "\(s.flag) \(s.city)"
                    self.lastError = nil
                    self.startStatsPolling()
                    self.onStateChange?()
                }
            } catch let e as XrayError where e == .portBusy {
                await MainActor.run {
                    self?.state = .error
                    self?.lastError = "PORT_BUSY"
                    self?.statusLine = "error"
                    self?.onStateChange?()
                }
            } catch {
                await MainActor.run {
                    self?.state = .error
                    self?.lastError = "\(error)"
                    self?.statusLine = "error"
                    self?.onStateChange?()
                }
            }
        }
    }
    func disconnect() {
        statusLine = ""
        Task.detached { [weak self] in
            try? await XrayController.setSystemProxy(enable: false)
            await self?.xray.stop()
            await MainActor.run {
                guard let self else { return }
                self.state = .disconnected
                self.bytesUp = 0; self.bytesDown = 0
                self.statsTimer?.invalidate(); self.statsTimer = nil
                self.onStateChange?()
            }
        }
    }
    private func startStatsPolling() {
        statsTimer?.invalidate()
        statsTimer = Timer.scheduledTimer(withTimeInterval: 2, repeats: true) { _ in
            Task { @MainActor in self.pollStats() }
        }
    }
    private func pollStats() {
        Task.detached { [weak self] in
            if let (up, down) = await self?.xray.queryStats() {
                await MainActor.run { self?.bytesUp = up; self?.bytesDown = down }
            }
        }
    }

    // MARK: - Parsing

    static func decodeBase64Sub(_ s: String) -> String? {
        let clean = s.replacingOccurrences(of: "\n", with: "").replacingOccurrences(of: "\r", with: "")
        guard let data = Data(base64Encoded: clean) else { return nil }
        return String(data: data, encoding: .utf8)
    }
    static func parseVless(_ line: String) -> VPNServer? {
        guard line.hasPrefix("vless://"), let url = URLComponents(string: line) else { return nil }
        guard let user = url.user, let h = url.host, let p = url.port else { return nil }
        let q = Dictionary(uniqueKeysWithValues: (url.queryItems ?? []).map { ($0.name, $0.value ?? "") })
        let fragment = url.fragment?.removingPercentEncoding ?? h
        let parts = fragment.split(separator: " ", maxSplits: 2, omittingEmptySubsequences: true)
        var flag = ""; var rest = fragment
        if let first = parts.first,
           first.unicodeScalars.allSatisfy({ $0.properties.isEmojiPresentation || (0x1F1E6...0x1F1FF).contains($0.value) }) {
            flag = String(first); rest = parts.dropFirst().joined(separator: " ")
        }
        let cc = rest.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true).map(String.init)
        let city = cc.first ?? h
        let country = cc.count > 1 ? cc[1] : ""
        return VPNServer(id: h, city: city, country: country, flag: flag, host: h,
                         port: UInt16(p), uuid: user, pbk: q["pbk"] ?? "", sid: q["sid"] ?? "",
                         sni: q["sni"] ?? "", flow: q["flow"] ?? "xtls-rprx-vision",
                         fp: q["fp"] ?? "chrome", pingMs: nil, source: .subscription("default"))
    }
}
