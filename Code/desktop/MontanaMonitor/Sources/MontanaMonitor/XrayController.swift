import Foundation

enum XrayError: Error, LocalizedError, Equatable {
    case binaryMissing(String)
    case launchFailed(String)
    case proxyFailed(String)
    case timeout
    case portBusy

    var errorDescription: String? {
        switch self {
        case .binaryMissing(let p): return "xray не найден: \(p)"
        case .launchFailed(let s):  return "не удалось запустить xray: \(s)"
        case .proxyFailed(let s):   return "не удалось настроить системный прокси: \(s)"
        case .timeout:              return "таймаут"
        case .portBusy:             return "PORT_BUSY"
        }
    }
}

actor XrayController {
    private var process: Process?
    private var stdoutPipe: Pipe?
    private var stderrPipe: Pipe?
    private let socksPort: UInt16 = 10808
    private let httpPort: UInt16 = 10809
    private let apiPort: UInt16 = 10810

    func start(server: VPNServer) async throws {
        try stopSync()
        // Если 10808 уже слушает чужой процесс (Happ и т.п.) — наш xray не сможет
        // забиндиться и Reality сломается. Сразу понятная ошибка.
        if Self.isPortListening(UInt16(Self.socksPort)) {
            throw XrayError.portBusy
        }
        let bin = try Self.xrayBinaryPath()
        let cfg = try writeConfig(server: server)
        let p = Process()
        p.executableURL = URL(fileURLWithPath: bin)
        p.arguments = ["-c", cfg.path]
        let out = Pipe(); let err = Pipe()
        p.standardOutput = out
        p.standardError = err
        try p.run()
        self.process = p
        self.stdoutPipe = out
        self.stderrPipe = err
        try await waitForListener(port: socksPort, timeoutMs: 4000)
    }

    func stop() async {
        try? stopSync()
    }

    private func stopSync() throws {
        if let p = process, p.isRunning {
            p.terminate()
            let deadline = Date().addingTimeInterval(2)
            while p.isRunning && Date() < deadline { Thread.sleep(forTimeInterval: 0.05) }
            if p.isRunning { kill(p.processIdentifier, SIGKILL) }
        }
        process = nil
        stdoutPipe = nil
        stderrPipe = nil
    }

    func queryStats() async -> (UInt64, UInt64)? {
        return nil
    }

    private func writeConfig(server s: VPNServer) throws -> URL {
        let dir = try Self.appSupportDir()
        let cfgURL = dir.appendingPathComponent("xray-config.json")
        let cfg: [String: Any] = [
            "log": ["loglevel": "warning"],
            "inbounds": [
                [
                    "tag": "socks-in",
                    "listen": "127.0.0.1",
                    "port": Int(socksPort),
                    "protocol": "socks",
                    "settings": ["auth": "noauth", "udp": true],
                    "sniffing": ["enabled": true, "destOverride": ["http", "tls"]]
                ],
                [
                    "tag": "http-in",
                    "listen": "127.0.0.1",
                    "port": Int(httpPort),
                    "protocol": "http",
                    "sniffing": ["enabled": true, "destOverride": ["http", "tls"]]
                ]
            ],
            "outbounds": [
                [
                    "tag": "proxy",
                    "protocol": "vless",
                    "settings": [
                        "vnext": [[
                            "address": s.host,
                            "port": Int(s.port),
                            "users": [[
                                "id": s.uuid,
                                "encryption": "none",
                                "flow": s.flow
                            ]]
                        ]]
                    ],
                    "streamSettings": [
                        "network": "tcp",
                        "security": "reality",
                        "realitySettings": [
                            "fingerprint": s.fp,
                            "serverName": s.sni,
                            "publicKey": s.pbk,
                            "shortId": s.sid,
                            "spiderX": ""
                        ]
                    ]
                ],
                ["tag": "direct", "protocol": "freedom"],
                ["tag": "block",  "protocol": "blackhole"]
            ],
        ]
        let data = try JSONSerialization.data(withJSONObject: cfg, options: [.prettyPrinted])
        try data.write(to: cfgURL)
        return cfgURL
    }

    private func waitForListener(port: UInt16, timeoutMs: Int) async throws {
        let deadline = Date().addingTimeInterval(TimeInterval(timeoutMs) / 1000.0)
        while Date() < deadline {
            if Self.isPortListening(port) { return }
            try await Task.sleep(nanoseconds: 100_000_000)
        }
        throw XrayError.timeout
    }

    private static func isPortListening(_ port: UInt16) -> Bool {
        let sock = socket(AF_INET, SOCK_STREAM, 0)
        if sock < 0 { return false }
        defer { close(sock) }
        var addr = sockaddr_in()
        addr.sin_family = sa_family_t(AF_INET)
        addr.sin_port = port.bigEndian
        addr.sin_addr.s_addr = inet_addr("127.0.0.1")
        let addrLen = socklen_t(MemoryLayout<sockaddr_in>.size)
        let rc = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { connect(sock, $0, addrLen) }
        }
        return rc == 0
    }

    static func xrayBinaryPath() throws -> String {
        let candidates = [
            Bundle.main.bundleURL.appendingPathComponent("Contents/Resources/xray").path,
            "/opt/homebrew/bin/xray",
            "/usr/local/bin/xray"
        ]
        for c in candidates where FileManager.default.isExecutableFile(atPath: c) {
            return c
        }
        throw XrayError.binaryMissing(candidates.joined(separator: " | "))
    }

    static func appSupportDir() throws -> URL {
        let base = try FileManager.default.url(
            for: .applicationSupportDirectory, in: .userDomainMask,
            appropriateFor: nil, create: true
        )
        let dir = base.appendingPathComponent("Montana", isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    // Стабильные пути для production main net.
    static let helperInstallPath = "/usr/local/bin/montana-vpn-proxy"
    static let sudoersPath = "/etc/sudoers.d/montana-vpn"
    static var helperInstalled: Bool {
        let fm = FileManager.default
        return fm.isExecutableFile(atPath: helperInstallPath) && fm.fileExists(atPath: sudoersPath)
    }
    static let socksPort = 10808
    static let httpPort = 10809

    /// Переключить системный прокси. Первый вызов устанавливает Montana-хелпер
    /// и NOPASSWD-правило (один запрос пароля). Далее — молча через `sudo -n`.
    static func setSystemProxy(enable: Bool) async throws {
        try ensureHelperInstalled()
        let action = enable ? "on" : "off"
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "/usr/bin/sudo")
        p.arguments = ["-n", helperInstallPath, action, "\(socksPort)", "\(httpPort)"]
        let err = Pipe()
        p.standardError = err
        try p.run()
        p.waitUntilExit()
        if p.terminationStatus != 0 {
            let msg = String(data: err.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
            throw XrayError.proxyFailed(msg.trimmingCharacters(in: .whitespacesAndNewlines))
        }
    }

    /// Гарантировать что Montana-хелпер скопирован в /usr/local/bin и NOPASSWD-правило
    /// установлено. Если уже на месте — ничего не делает (нет промта). Иначе — ОДИН
    /// privileged-вызов, который ставит и хелпер, и sudoers-правило сразу.
    static func ensureHelperInstalled() throws {
        let fm = FileManager.default
        let helperOK = fm.isExecutableFile(atPath: helperInstallPath)
        let sudoersOK = fm.fileExists(atPath: sudoersPath)
        if helperOK && sudoersOK { return }

        guard let bundled = Bundle.main.url(forResource: "montana-vpn-proxy", withExtension: nil)?.path else {
            throw XrayError.proxyFailed("montana-vpn-proxy не найден в бандле")
        }
        let user = NSUserName()
        // Единый privileged-скрипт: установить хелпер + sudoers-правило атомарно.
        let install = """
        /bin/mkdir -p /usr/local/bin && \
        /bin/cp '\(bundled)' '\(helperInstallPath)' && \
        /usr/sbin/chown root:wheel '\(helperInstallPath)' && \
        /bin/chmod 755 '\(helperInstallPath)' && \
        /bin/echo '\(user) ALL=(root) NOPASSWD: \(helperInstallPath)' > '\(sudoersPath)' && \
        /usr/sbin/chown root:wheel '\(sudoersPath)' && \
        /bin/chmod 440 '\(sudoersPath)'
        """
        try runAsAdmin(install)
    }

    private static func runAsAdmin(_ shell: String) throws {
        let escaped = shell
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        let osa = "do shell script \"\(escaped)\" with administrator privileges"
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
        p.arguments = ["-e", osa]
        let err = Pipe()
        p.standardError = err
        try p.run()
        p.waitUntilExit()
        if p.terminationStatus != 0 {
            let msg = String(data: err.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
            throw XrayError.proxyFailed(msg.trimmingCharacters(in: .whitespacesAndNewlines))
        }
    }
}
