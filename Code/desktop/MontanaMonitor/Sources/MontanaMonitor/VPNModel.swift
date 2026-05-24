import SwiftUI
import Foundation
import Network

enum VPNState: String { case disconnected, connecting, connected, error }

struct VPNServer: Identifiable, Hashable {
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
}

@MainActor
final class VPNModel: ObservableObject {
    @Published var servers: [VPNServer] = []
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

    var currentServer: VPNServer? {
        guard let id = selected else { return servers.first }
        return servers.first(where: { $0.id == id }) ?? servers.first
    }

    func bootstrap() {
        refreshSubscription()
        refreshTimer = Timer.scheduledTimer(withTimeInterval: 300, repeats: true) { _ in
            Task { @MainActor in self.refreshSubscription() }
        }
    }

    func refreshSubscription() {
        guard let url = URL(string: "https://montana.quest/vpn/sub") else { return }
        var req = URLRequest(url: url)
        req.cachePolicy = .reloadIgnoringLocalAndRemoteCacheData
        req.timeoutInterval = 8
        URLSession.shared.dataTask(with: req) { [weak self] data, _, _ in
            guard let data, let text = String(data: data, encoding: .utf8) else { return }
            let decoded = Self.decodeBase64Sub(text) ?? text
            let parsed = decoded.split(separator: "\n")
                .compactMap { Self.parseVless(String($0)) }
            Task { @MainActor in self?.applyServers(parsed) }
        }.resume()
    }

    private func applyServers(_ list: [VPNServer]) {
        servers = list
        if selected == nil { selected = list.first?.id }
        for s in list { pingServer(s) }
    }

    private func pingServer(_ s: VPNServer) {
        let host = NWEndpoint.Host(s.host)
        let port = NWEndpoint.Port(rawValue: s.port)!
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
        if let idx = servers.firstIndex(where: { $0.id == id }) {
            servers[idx].pingMs = ms
        }
    }

    func connect() {
        guard let s = currentServer else { return }
        state = .connecting
        statusLine = "запускаем xray → \(s.city)…"
        onStateChange?()
        Task.detached { [weak self] in
            do {
                try await self?.xray.start(server: s)
                try await XrayController.setSystemProxy(enable: true)
                await MainActor.run {
                    guard let self else { return }
                    self.state = .connected
                    self.statusLine = "подключено к \(s.flag) \(s.city)"
                    self.lastError = nil
                    self.startStatsPolling()
                    self.onStateChange?()
                }
            } catch {
                await MainActor.run {
                    self?.state = .error
                    self?.lastError = "\(error)"
                    self?.statusLine = "ошибка"
                    self?.onStateChange?()
                }
            }
        }
    }

    func disconnect() {
        statusLine = "отключаем…"
        Task.detached { [weak self] in
            try? await XrayController.setSystemProxy(enable: false)
            await self?.xray.stop()
            await MainActor.run {
                guard let self else { return }
                self.state = .disconnected
                self.statusLine = ""
                self.bytesUp = 0
                self.bytesDown = 0
                self.statsTimer?.invalidate()
                self.statsTimer = nil
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
                await MainActor.run {
                    self?.bytesUp = up
                    self?.bytesDown = down
                }
            }
        }
    }

    static func decodeBase64Sub(_ s: String) -> String? {
        let clean = s.replacingOccurrences(of: "\n", with: "")
                     .replacingOccurrences(of: "\r", with: "")
        guard let data = Data(base64Encoded: clean) else { return nil }
        return String(data: data, encoding: .utf8)
    }

    static func parseVless(_ line: String) -> VPNServer? {
        guard line.hasPrefix("vless://"),
              let url = URLComponents(string: line) else { return nil }
        guard let user = url.user, let h = url.host, let p = url.port else { return nil }
        let q = Dictionary(uniqueKeysWithValues: (url.queryItems ?? []).map { ($0.name, $0.value ?? "") })
        let fragment = url.fragment?.removingPercentEncoding ?? h
        let parts = fragment.split(separator: " ", maxSplits: 2, omittingEmptySubsequences: true)
        var flag = ""
        var rest = fragment
        if let first = parts.first, first.unicodeScalars.allSatisfy({ $0.properties.isEmojiPresentation || (0x1F1E6...0x1F1FF).contains($0.value) }) {
            flag = String(first)
            rest = parts.dropFirst().joined(separator: " ")
        }
        let cityCountry = rest.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true).map(String.init)
        let city = cityCountry.first ?? h
        let country = cityCountry.count > 1 ? cityCountry[1] : ""
        return VPNServer(
            id: h,
            city: city,
            country: country,
            flag: flag,
            host: h,
            port: UInt16(p),
            uuid: user,
            pbk: q["pbk"] ?? "",
            sid: q["sid"] ?? "",
            sni: q["sni"] ?? "",
            flow: q["flow"] ?? "xtls-rprx-vision",
            fp: q["fp"] ?? "chrome"
        )
    }
}
