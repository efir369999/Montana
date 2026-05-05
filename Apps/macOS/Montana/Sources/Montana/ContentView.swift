import SwiftUI

struct ContentView: View {
    @StateObject private var service = NodeService()
    @State private var refreshTimer: Timer?

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    statusCard
                    operatorCard
                    networkCard
                    logCard
                }
                .padding(20)
            }
            Divider()
            actionBar
        }
        .frame(minWidth: 720, minHeight: 620)
        .task {
            await service.refresh()
            startTimer()
        }
        .onDisappear { refreshTimer?.invalidate() }
    }

    private var header: some View {
        HStack(spacing: 14) {
            if let url = Bundle.module.url(forResource: "Montana_icon_1024", withExtension: "png"), let icon = NSImage(contentsOf: url) {
                Image(nsImage: icon)
                    .resizable()
                    .interpolation(.high)
                    .frame(width: 56, height: 56)
                    .cornerRadius(12)
            }
            VStack(alignment: .leading, spacing: 2) {
                Text("Montana").font(.system(size: 22, weight: .semibold))
                Text("ядро 0.1.0 · spec v35.25.0")
                    .font(.system(size: 12, design: .monospaced))
                    .foregroundColor(.secondary)
            }
            Spacer()
            statusPill
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 14)
    }

    private var statusPill: some View {
        HStack(spacing: 6) {
            Circle().fill(service.isRunning ? Color.green : Color.gray).frame(width: 10, height: 10)
            Text(service.isRunning ? "узел работает" : "узел остановлен")
                .font(.system(size: 12, weight: .medium))
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color.secondary.opacity(0.1))
        .clipShape(Capsule())
    }

    private var statusCard: some View {
        Card(title: "Состояние узла") {
            VStack(alignment: .leading, spacing: 10) {
                row("Фаза", service.status.phase, mono: true)
                row("Текущее окно", "\(service.status.currentWindow)", mono: true)
                row("D (итераций SHA-256/окно)", formatNumber(service.status.d), mono: true)
                if service.status.candidateTotal > 0 {
                    VStack(alignment: .leading, spacing: 6) {
                        HStack {
                            Text("Candidate VDF").font(.system(size: 12)).foregroundColor(.secondary)
                            Spacer()
                            Text("\(service.status.candidateDone) / \(service.status.candidateTotal)")
                                .font(.system(size: 12, design: .monospaced))
                        }
                        ProgressView(value: service.status.candidateProgress)
                            .progressViewStyle(.linear)
                    }
                }
                if let err = service.status.error {
                    Text(err).font(.system(size: 11, design: .monospaced)).foregroundColor(.red)
                }
            }
        }
    }

    private var operatorCard: some View {
        Card(title: "Оператор") {
            VStack(alignment: .leading, spacing: 10) {
                row("account_id", service.status.accountId.prefix16, mono: true)
                row("node_id", service.status.nodeId.prefix16, mono: true)
                row("balance", service.status.balance, mono: true)
                row("account_chain_length", "\(service.status.accountChainLength)", mono: true)
                row("is_node_operator", service.status.isNodeOperator ? "true" : "false", mono: true)
            }
        }
    }

    private var networkCard: some View {
        Card(title: "Сеть · state") {
            VStack(alignment: .leading, spacing: 10) {
                row("AccountTable", "\(service.status.accountTable) записей", mono: true)
                row("NodeTable", "\(service.status.nodeTable) активных узлов", mono: true)
                row("CandidatePool", "\(service.status.candidatePool) ожидают selection", mono: true)
                row("supply", service.status.supply, mono: true)
                row("Σ balances", service.status.sumBalances, mono: true)
            }
        }
    }

    private var logCard: some View {
        Card(title: "Лог (последние строки)") {
            VStack(alignment: .leading, spacing: 4) {
                if service.lastLogLines.isEmpty {
                    Text("лог пуст или недоступен").foregroundColor(.secondary).font(.system(size: 12))
                } else {
                    ForEach(Array(service.lastLogLines.enumerated()), id: \.offset) { _, line in
                        Text(line)
                            .font(.system(size: 11, design: .monospaced))
                            .lineLimit(1)
                            .truncationMode(.tail)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }
        }
    }

    private var actionBar: some View {
        HStack(spacing: 10) {
            Button("Перезапустить") { Task { await service.restart() } }
            Button("Остановить") { Task { await service.stop() } }
                .disabled(!service.isRunning)
            Button("Запустить") { Task { await service.start() } }
                .disabled(service.isRunning)
            Spacer()
            Button("Логи") { service.openLogs() }
            Button("Папка данных") { service.revealDataFolder() }
            Button(action: { Task { await service.refresh() } }) {
                Image(systemName: "arrow.clockwise")
            }
            .help("Обновить статус")
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
    }

    private func row(_ label: String, _ value: String, mono: Bool = false) -> some View {
        HStack {
            Text(label).font(.system(size: 12)).foregroundColor(.secondary)
            Spacer()
            Text(value)
                .font(.system(size: 12, design: mono ? .monospaced : .default))
                .textSelection(.enabled)
        }
    }

    private func formatNumber(_ n: UInt64) -> String {
        let f = NumberFormatter()
        f.numberStyle = .decimal
        f.groupingSeparator = " "
        return f.string(from: NSNumber(value: n)) ?? "\(n)"
    }

    private func startTimer() {
        refreshTimer?.invalidate()
        refreshTimer = Timer.scheduledTimer(withTimeInterval: 3.0, repeats: true) { _ in
            Task { @MainActor in await service.refresh() }
        }
    }
}

private struct Card<Content: View>: View {
    let title: String
    @ViewBuilder let content: () -> Content
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(title).font(.system(size: 11, weight: .semibold))
                .foregroundColor(.secondary).textCase(.uppercase).tracking(1)
            content()
        }
        .padding(14)
        .background(RoundedRectangle(cornerRadius: 10).fill(Color.secondary.opacity(0.06)))
    }
}

private extension String {
    var prefix16: String { count <= 16 ? self : String(prefix(16)) + "…" }
}
