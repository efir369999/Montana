import Foundation
import SwiftUI

enum NodePhase: String, Equatable, Codable {
    case unknown      = "Unknown"
    case bootstrap    = "Bootstrap"
    case candidateVdf = "CandidateVdf"
    case registered   = "Registered"
    case active       = "Active"

    var color: Color {
        switch self {
        case .unknown:      return .gray
        case .bootstrap:    return .orange
        case .candidateVdf: return .yellow
        case .registered:   return .blue
        case .active:       return .green
        }
    }
}

struct NodeStatus: Equatable {
    var phase: NodePhase = .unknown
    var accountIdHex: String = ""
    var nodeIdHex: String = ""
    var currentWindow: UInt64 = 0
    var chainLength: UInt64 = 0
    var tau2: UInt64 = 20160
    var supplyNJ: UInt64 = 0
    var balanceNJ: UInt64 = 0
    var isActive: Bool = false
    var lastUpdate: Date? = nil
}

@MainActor
final class NodeRunner: ObservableObject {
    @Published private(set) var status = NodeStatus()
    @Published private(set) var isRunning = false
    @Published private(set) var lastError: String? = nil
    @Published private(set) var lastLogLines: [String] = []

    private var process: Process?
    private var statusTimer: Timer?
    private var stdoutPipe: Pipe?
    private var stderrPipe: Pipe?

    var dataDir: URL {
        let base = (try? FileManager.default.url(
            for: .applicationSupportDirectory, in: .userDomainMask,
            appropriateFor: nil, create: true
        )) ?? URL(fileURLWithPath: NSHomeDirectory() + "/Library/Application Support")
        let dir = base.appendingPathComponent("Montana/node", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    var hasIdentity: Bool {
        FileManager.default.fileExists(atPath: dataDir.appendingPathComponent("identity.bin").path)
    }

    private static func binaryPath() -> String? {
        if let p = Bundle.main.url(forResource: "montana-node", withExtension: nil)?.path,
           FileManager.default.isExecutableFile(atPath: p) { return p }
        for c in ["/opt/homebrew/bin/montana-node", "/usr/local/bin/montana-node"] {
            if FileManager.default.isExecutableFile(atPath: c) { return c }
        }
        return nil
    }

    // MARK: - Identity install via init

    /// Создать identity (новую mnemonic либо восстановить переданную).
    /// Если node-data уже содержит identity.bin — используется `--force`.
    func installIdentity(mnemonic: String) throws {
        guard let bin = Self.binaryPath() else {
            throw NSError(domain: "NodeRunner", code: 1, userInfo: [NSLocalizedDescriptionKey: "montana-node binary не найден"])
        }
        let p = Process()
        p.executableURL = URL(fileURLWithPath: bin)
        p.arguments = ["init",
                       "--data-dir", dataDir.path,
                       "--mnemonic-stdin",
                       "--force"]
        let stdin = Pipe()
        p.standardInput = stdin
        let stderr = Pipe()
        p.standardError = stderr
        try p.run()
        let line = mnemonic.trimmingCharacters(in: .whitespacesAndNewlines) + "\n"
        try stdin.fileHandleForWriting.write(contentsOf: Data(line.utf8))
        try stdin.fileHandleForWriting.close()
        p.waitUntilExit()
        if p.terminationStatus != 0 {
            let err = String(data: stderr.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
            throw NSError(domain: "NodeRunner", code: Int(p.terminationStatus),
                          userInfo: [NSLocalizedDescriptionKey: "init failed: \(err.trimmingCharacters(in: .whitespacesAndNewlines))"])
        }
    }

    // MARK: - Start / stop node

    func start() {
        guard let bin = Self.binaryPath() else {
            lastError = "montana-node не найден в Resources"
            return
        }
        guard hasIdentity else {
            lastError = "сначала создайте кошелёк"
            return
        }
        stop()
        let p = Process()
        p.executableURL = URL(fileURLWithPath: bin)
        p.arguments = ["start", "--data-dir", dataDir.path]
        let out = Pipe(); let err = Pipe()
        p.standardOutput = out
        p.standardError = err
        p.terminationHandler = { [weak self] _ in
            Task { @MainActor in
                self?.isRunning = false
            }
        }
        do {
            try p.run()
            self.process = p
            self.stdoutPipe = out
            self.stderrPipe = err
            self.isRunning = true
            self.lastError = nil
            startLogPump(pipe: out)
            startLogPump(pipe: err)
            startStatusPolling()
        } catch {
            self.lastError = "не удалось запустить узел: \(error)"
        }
    }

    func stop() {
        statusTimer?.invalidate()
        statusTimer = nil
        if let p = process, p.isRunning {
            p.terminate()
            let deadline = Date().addingTimeInterval(2)
            while p.isRunning && Date() < deadline { Thread.sleep(forTimeInterval: 0.05) }
        }
        process = nil
        stdoutPipe = nil
        stderrPipe = nil
        isRunning = false
    }

    private func startLogPump(pipe: Pipe) {
        let handle = pipe.fileHandleForReading
        handle.readabilityHandler = { [weak self] h in
            let data = h.availableData
            guard !data.isEmpty, let text = String(data: data, encoding: .utf8) else { return }
            DispatchQueue.main.async {
                guard let self else { return }
                for line in text.split(whereSeparator: { $0 == "\n" || $0 == "\r" }) {
                    self.lastLogLines.append(String(line))
                    if self.lastLogLines.count > 200 {
                        self.lastLogLines.removeFirst(self.lastLogLines.count - 200)
                    }
                }
            }
        }
    }

    // MARK: - Status polling via `montana-node status`

    private func startStatusPolling() {
        statusTimer?.invalidate()
        pollStatus()
        statusTimer = Timer.scheduledTimer(withTimeInterval: 5, repeats: true) { _ in
            Task { @MainActor in self.pollStatus() }
        }
    }

    private func pollStatus() {
        guard let bin = Self.binaryPath() else { return }
        Task.detached { [weak self] in
            guard let self else { return }
            let out = Pipe()
            let p = Process()
            p.executableURL = URL(fileURLWithPath: bin)
            p.arguments = ["status", "--data-dir", await self.dataDir.path]
            p.standardOutput = out
            p.standardError = Pipe()
            do { try p.run() } catch { return }
            p.waitUntilExit()
            let data = out.fileHandleForReading.readDataToEndOfFile()
            guard let text = String(data: data, encoding: .utf8) else { return }
            let parsed = await Self.parseStatusOutput(text)
            await MainActor.run { self.status = parsed }
        }
    }

    static func parseStatusOutput(_ text: String) -> NodeStatus {
        var s = NodeStatus()
        s.lastUpdate = Date()
        for line in text.split(separator: "\n") {
            let kv = line.split(separator: ":", maxSplits: 1, omittingEmptySubsequences: true).map { $0.trimmingCharacters(in: .whitespaces) }
            guard kv.count == 2 else { continue }
            let key = kv[0]
            let val = kv[1]
            switch key {
            case "phase":
                let raw = val.trimmingCharacters(in: CharacterSet(charactersIn: "\""))
                s.phase = NodePhase(rawValue: raw) ?? .unknown
            case "account_id":
                s.accountIdHex = val
            case "node_id":
                s.nodeIdHex = val
            case "current_window":
                s.currentWindow = UInt64(val) ?? 0
            case "chain_length":
                s.chainLength = UInt64(val) ?? 0
            case "tau2":
                s.tau2 = UInt64(val) ?? 20160
            case "supply_n":
                s.supplyNJ = UInt64(val) ?? 0
            case "balance_n":
                s.balanceNJ = UInt64(val) ?? 0
            case "is_node_operator":
                s.isActive = (val == "true")
            default: break
            }
        }
        return s
    }
}
