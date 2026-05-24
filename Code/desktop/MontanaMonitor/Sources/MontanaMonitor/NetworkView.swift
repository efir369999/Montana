import SwiftUI
import AppKit

/// Эксплорер сети Montana — сводка как в Bitcoin Core "Overview / Network",
/// без листинга каждого узла. Данные — из публичного API:
///   https://montana.quest/net/network.json  — сводный статус узлов
///   https://efir.org/explorer/data.json     — текущий supply, окно, peers
struct NetworkView: View {
    @ObservedObject var model: MeshModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                header
                Divider()
                syncCard
                statsGrid
                Divider()
                exploreLinks
                Spacer(minLength: 0)
            }
            .padding(24)
        }
    }

    private var header: some View {
        HStack(alignment: .firstTextBaseline) {
            VStack(alignment: .leading, spacing: 2) {
                Text("Сеть Montana").font(.title.bold())
                Text("Montana Ядро 0.1 — мейннет").font(.caption).foregroundColor(.secondary)
            }
            Spacer()
            if let at = model.lastFetchAt {
                Text("обновлено \(timeFmt.string(from: at))")
                    .font(.caption).foregroundColor(.secondary)
            }
        }
    }

    @ViewBuilder private var syncCard: some View {
        if let s = model.doc?.network_summary {
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 10) {
                    Circle().fill(.green).frame(width: 10, height: 10)
                    Text("синхронизирован с сетью").font(.headline)
                    Spacer()
                    Text("W \(s.max_window)").font(.system(.title2, design: .monospaced)).bold()
                }
                Text("Активных узлов \(s.active_nodes) из \(s.total_nodes); внешних операторов: \(s.discovered_peer_count).")
                    .font(.caption).foregroundColor(.secondary)
            }
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(RoundedRectangle(cornerRadius: 10).fill(Color.green.opacity(0.07)))
        } else if let err = model.lastError {
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Circle().fill(.orange).frame(width: 10, height: 10)
                    Text("нет связи с сетью").font(.headline)
                }
                Text(err).font(.caption).foregroundColor(.secondary).lineLimit(2)
            }
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(RoundedRectangle(cornerRadius: 10).fill(Color.orange.opacity(0.08)))
        } else {
            ProgressView("подключение к сети…")
                .padding(20)
                .frame(maxWidth: .infinity)
        }
    }

    @ViewBuilder private var statsGrid: some View {
        if let s = model.doc?.network_summary {
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                StatCard(label: "supply", value: format_supply(s.total_supply_nj), unit: "Ɉ")
                StatCard(label: "окно", value: "\(s.max_window)", unit: "W")
                StatCard(label: "узлов", value: "\(s.active_nodes)/\(s.total_nodes)", unit: "active/total")
            }
        }
    }

    private var exploreLinks: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Эксплорер").font(.headline)
            HStack(spacing: 8) {
                Link("Сеть онлайн", destination: URL(string: "https://montana.quest/net/")!)
                Text("·").foregroundColor(.secondary)
                Link("Подробный data.json", destination: URL(string: "https://efir.org/explorer/data.json")!)
                Text("·").foregroundColor(.secondary)
                Link("GitHub", destination: URL(string: "https://github.com/efir369999/Montana")!)
            }
            Text("Любой может развернуть узел Monтана — `curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-vps.sh | sudo bash` на чистом Linux VPS.")
                .font(.caption).foregroundColor(.secondary).padding(.top, 4)
        }
    }

    private let timeFmt: DateFormatter = {
        let f = DateFormatter(); f.dateFormat = "HH:mm:ss"; return f
    }()

    private func format_supply(_ nj: UInt64) -> String {
        let coin = Double(nj) / 1_000_000_000_000.0
        if coin >= 1_000_000 { return String(format: "%.2f М", coin / 1_000_000.0) }
        if coin >= 1_000     { return String(format: "%.2f К", coin / 1_000.0) }
        return String(format: "%.2f", coin)
    }
}

struct StatCard: View {
    let label: String
    let value: String
    let unit: String
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label).font(.caption).foregroundColor(.secondary).textCase(.uppercase)
            HStack(alignment: .firstTextBaseline, spacing: 4) {
                Text(value).font(.title2.bold().monospacedDigit())
                Text(unit).font(.caption).foregroundColor(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(RoundedRectangle(cornerRadius: 8).fill(Color.gray.opacity(0.08)))
    }
}
