import Foundation
import AppKit

struct NodeStatus {
    var currentWindow: Int = 0
    var phase: String = "Загрузка…"
    var d: UInt64 = 0
    var candidateDone: Int = 0
    var candidateTotal: Int = 0
    var accountId: String = ""
    var nodeId: String = ""
    var accountTable: Int = 0
    var nodeTable: Int = 0
    var candidatePool: Int = 0
    var balance: String = "—"
    var isNodeOperator: Bool = false
    var accountChainLength: Int = 0
    var supply: String = "—"
    var sumBalances: String = "—"
    var lastUpdated: Date = .init()
    var error: String? = nil

    var candidateProgress: Double {
        guard candidateTotal > 0 else { return 0 }
        return Double(candidateDone) / Double(candidateTotal)
    }
}

@MainActor
final class NodeService: ObservableObject {
    @Published var status = NodeStatus()
    @Published var identity = IdentityInfo()
    @Published var isRunning = false
    @Published var lastLogLines: [String] = []

    let home = FileManager.default.homeDirectoryForCurrentUser
    var binary: URL { home.appendingPathComponent("Applications/Montana/montana-node") }
    var dataDir: URL { home.appendingPathComponent("Applications/Montana/data") }
    var logFile: URL { dataDir.appendingPathComponent("logs/montana.log") }
    var plistPath: URL { home.appendingPathComponent("Library/LaunchAgents/org.montana.node.plist") }

    func refresh() async {
        async let s = runStatus()
        async let r = runIsLoaded()
        async let l = readLogTail(lines: 8)
        let (status, running, log) = await (s, r, l)
        if let status { self.status = status }
        self.isRunning = running
        self.lastLogLines = log
        if !identity.loaded { await loadIdentity() }
    }

    func loadIdentity() async {
        guard FileManager.default.fileExists(atPath: binary.path) else { return }
        let result = await runProcess(binary.path, ["inspect", "--data-dir", dataDir.path])
        guard result.exit == 0 else { return }
        var info = IdentityInfo()
        for raw in result.stdout.split(separator: "\n") {
            let line = String(raw)
            guard let colon = line.firstIndex(of: ":") else { continue }
            let key = line[..<colon].trimmingCharacters(in: .whitespaces)
            let val = line[line.index(after: colon)...].trimmingCharacters(in: .whitespaces)
            switch key {
            case "data-dir": info.dataDir = val
            case "identity": info.identityPath = val
            case "suite": info.suite = val
            case "account_id": info.accountId = val
            case "node_id": info.nodeId = val
            case "master_seed_fp": info.masterSeedFp = val
            case "libp2p_peer_id": info.libp2pPeerId = val
            case "account_pk[..16]": info.accountPkPrefix = val
            case "node_pk[..16]": info.nodePkPrefix = val
            case "mlkem_pk[..16]": info.mlkemPkPrefix = val
            default: break
            }
        }
        info.loaded = !info.accountId.isEmpty
        self.identity = info
    }

    func revealMasterSeed() async -> String? {
        guard FileManager.default.fileExists(atPath: binary.path) else { return nil }
        let r = await runProcess(binary.path, ["inspect", "--data-dir", dataDir.path, "--reveal-master-seed"])
        guard r.exit == 0 else { return nil }
        for line in r.stdout.split(separator: "\n") {
            let s = String(line)
            if s.contains("master_seed") && !s.contains("master_seed_fp") {
                if let colon = s.firstIndex(of: ":") {
                    let val = s[s.index(after: colon)...].trimmingCharacters(in: .whitespaces)
                    if val.count >= 64 { return val }
                }
            }
        }
        return nil
    }

    private func runStatus() async -> NodeStatus? {
        guard FileManager.default.fileExists(atPath: binary.path) else {
            var s = NodeStatus()
            s.error = "Бинарь не найден: \(binary.path)"
            return s
        }
        let r = await runProcess(binary.path, ["status", "--data-dir", dataDir.path])
        guard r.exit == 0 else {
            var s = NodeStatus()
            s.error = "exit \(r.exit): \(r.stderr.prefix(200))"
            return s
        }
        return parseStatus(r.stdout)
    }

    private func runIsLoaded() async -> Bool {
        let r = await runProcess("/bin/launchctl", ["list"])
        return r.stdout.contains("org.montana.node")
    }

    private func readLogTail(lines: Int) async -> [String] {
        guard let data = try? Data(contentsOf: logFile),
              let text = String(data: data, encoding: .utf8) else { return [] }
        let all = text.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
        return Array(all.suffix(lines))
    }

    func restart() async {
        _ = await runProcess("/bin/launchctl", ["unload", plistPath.path])
        _ = await runProcess("/bin/launchctl", ["load", "-w", plistPath.path])
        try? await Task.sleep(nanoseconds: 500_000_000)
        await refresh()
    }

    func stop() async {
        _ = await runProcess("/bin/launchctl", ["unload", plistPath.path])
        await refresh()
    }

    func start() async {
        _ = await runProcess("/bin/launchctl", ["load", "-w", plistPath.path])
        await refresh()
    }

    func openLogs() {
        NSWorkspace.shared.open(logFile)
    }

    func revealDataFolder() {
        NSWorkspace.shared.activateFileViewerSelecting([dataDir])
    }

    func copyToPasteboard(_ text: String) {
        let pb = NSPasteboard.general
        pb.clearContents()
        pb.setString(text, forType: .string)
    }

    private func parseStatus(_ text: String) -> NodeStatus {
        var s = NodeStatus()
        s.lastUpdated = Date()
        for raw in text.split(separator: "\n") {
            let line = String(raw)
            guard let colon = line.firstIndex(of: ":") else { continue }
            let key = line[..<colon].trimmingCharacters(in: .whitespaces)
            let val = line[line.index(after: colon)...].trimmingCharacters(in: .whitespaces)
            switch key {
            case "current_window": s.currentWindow = Int(val) ?? 0
            case "phase": s.phase = val
            case "D (current)": s.d = UInt64(val) ?? 0
            case "candidate VDF":
                let parts = val.split(separator: "/")
                if parts.count >= 2 {
                    s.candidateDone = Int(parts[0].trimmingCharacters(in: .whitespaces)) ?? 0
                    let rest = parts[1].split(separator: " ").first.map(String.init) ?? ""
                    s.candidateTotal = Int(rest) ?? 0
                }
            case "account_id": s.accountId = val
            case "node_id": s.nodeId = val
            case "AccountTable":
                s.accountTable = Int(val.split(separator: " ").first ?? "0") ?? 0
            case "NodeTable":
                s.nodeTable = Int(val.split(separator: " ").first ?? "0") ?? 0
            case "CandidatePool":
                s.candidatePool = Int(val.split(separator: " ").first ?? "0") ?? 0
            case "balance": s.balance = val
            case "is_node_operator": s.isNodeOperator = (val == "true")
            case "account_chain_length": s.accountChainLength = Int(val) ?? 0
            case "supply (closed-form)": s.supply = val
            case "Σ balances": s.sumBalances = val
            default: break
            }
        }
        return s
    }

    private struct ProcessResult { let exit: Int32; let stdout: String; let stderr: String }

    private func runProcess(_ path: String, _ args: [String]) async -> ProcessResult {
        await withCheckedContinuation { (cont: CheckedContinuation<ProcessResult, Never>) in
            DispatchQueue.global(qos: .userInitiated).async {
                let task = Process()
                task.launchPath = path
                task.arguments = args
                let outPipe = Pipe()
                let errPipe = Pipe()
                task.standardOutput = outPipe
                task.standardError = errPipe
                do { try task.run() } catch {
                    cont.resume(returning: ProcessResult(exit: -1, stdout: "", stderr: "\(error)"))
                    return
                }
                let outData = outPipe.fileHandleForReading.readDataToEndOfFile()
                let errData = errPipe.fileHandleForReading.readDataToEndOfFile()
                task.waitUntilExit()
                cont.resume(returning: ProcessResult(
                    exit: task.terminationStatus,
                    stdout: String(data: outData, encoding: .utf8) ?? "",
                    stderr: String(data: errData, encoding: .utf8) ?? ""
                ))
            }
        }
    }
}
