import Foundation

enum XrayError: Error, LocalizedError {
    case binaryMissing(String)
    case launchFailed(String)
    case proxyFailed(String)
    case timeout

    var errorDescription: String? {
        switch self {
        case .binaryMissing(let p): return "xray не найден: \(p)"
        case .launchFailed(let s):  return "не удалось запустить xray: \(s)"
        case .proxyFailed(let s):   return "не удалось настроить системный прокси: \(s)"
        case .timeout:              return "таймаут"
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

    static func setSystemProxy(enable: Bool) async throws {
        let script: String
        if enable {
            script = """
            for svc in $(/usr/sbin/networksetup -listallnetworkservices | tail -n +2 | grep -v '^\\*'); do
              /usr/sbin/networksetup -setsocksfirewallproxy "$svc" 127.0.0.1 10808 || true
              /usr/sbin/networksetup -setsocksfirewallproxystate "$svc" on || true
              /usr/sbin/networksetup -setwebproxy "$svc" 127.0.0.1 10809 || true
              /usr/sbin/networksetup -setwebproxystate "$svc" on || true
              /usr/sbin/networksetup -setsecurewebproxy "$svc" 127.0.0.1 10809 || true
              /usr/sbin/networksetup -setsecurewebproxystate "$svc" on || true
            done
            """
        } else {
            script = """
            for svc in $(/usr/sbin/networksetup -listallnetworkservices | tail -n +2 | grep -v '^\\*'); do
              /usr/sbin/networksetup -setsocksfirewallproxystate "$svc" off || true
              /usr/sbin/networksetup -setwebproxystate "$svc" off || true
              /usr/sbin/networksetup -setsecurewebproxystate "$svc" off || true
            done
            """
        }
        try runAsAdmin(script)
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
