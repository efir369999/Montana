import SwiftUI
import AppKit

struct NetworkView: View {
    @ObservedObject var model: MeshModel
    @ObservedObject var runner: NodeRunner
    @ObservedObject var loc: AppLocale

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                header
                Divider().background(MontanaTheme.line)
                syncCard
                if runner.hasIdentity { myNodeCard }
                statsGrid
                Divider().background(MontanaTheme.line)
                installNodeBlock
                Spacer(minLength: 0)
            }
            .padding(24)
        }
        .background(MontanaTheme.bg)
    }

    private var header: some View {
        HStack(alignment: .firstTextBaseline) {
            VStack(alignment: .leading, spacing: 2) {
                Text(loc.t("net.title")).font(.title.bold()).foregroundColor(MontanaTheme.ink)
                Text("\(loc.t("side.version")) — \(loc.t("net.subtitle"))")
                    .font(.caption).foregroundColor(MontanaTheme.inkMute)
            }
            Spacer()
            if let at = model.lastFetchAt {
                Text(String(format: loc.t("net.last_update"), timeFmt.string(from: at)))
                    .font(.caption).foregroundColor(MontanaTheme.inkMute)
            }
        }
    }

    @ViewBuilder private var syncCard: some View {
        if let s = model.doc?.network_summary {
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 10) {
                    Circle().fill(MontanaTheme.ok).frame(width: 10, height: 10)
                    Text(loc.t("net.synced")).font(.headline).foregroundColor(MontanaTheme.ink)
                    Spacer()
                    Text("\(s.max_window)").font(.system(.title2, design: .monospaced)).bold().foregroundColor(MontanaTheme.gold)
                }
                Text(String(format: loc.t("net.summary"), s.active_nodes, s.total_nodes, s.discovered_peer_count))
                    .font(.caption).foregroundColor(MontanaTheme.inkMute)
            }
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(MontanaTheme.bgSoft)
            .overlay(Rectangle().strokeBorder(MontanaTheme.ok.opacity(0.4), lineWidth: 1))
        } else if let err = model.lastError {
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Circle().fill(MontanaTheme.err).frame(width: 10, height: 10)
                    Text(loc.t("net.no_link")).font(.headline).foregroundColor(MontanaTheme.ink)
                }
                Text(err).font(.caption).foregroundColor(MontanaTheme.inkMute).lineLimit(2)
            }
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(MontanaTheme.bgSoft)
            .overlay(Rectangle().strokeBorder(MontanaTheme.err.opacity(0.4), lineWidth: 1))
        } else {
            ProgressView(loc.t("net.connecting")).tint(MontanaTheme.gold)
                .padding(20).frame(maxWidth: .infinity)
        }
    }

    @ViewBuilder private var statsGrid: some View {
        if let s = model.doc?.network_summary {
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                StatCard(label: loc.t("net.supply"), value: format_supply(s.total_supply_nj), unit: "Ɉ")
                StatCard(label: loc.t("net.window"), value: "\(s.max_window)", unit: "W")
                StatCard(label: loc.t("net.nodes"), value: "\(s.active_nodes)/\(s.total_nodes)", unit: loc.t("net.active_total"))
            }
        }
    }

    private var installNodeBlock: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(loc.t("net.explorer")).font(.headline).foregroundColor(MontanaTheme.ink)
            HStack(spacing: 8) {
                Link("montana.quest/net/", destination: URL(string: "https://montana.quest/net/")!)
                    .foregroundColor(MontanaTheme.gold)
                Text("·").foregroundColor(MontanaTheme.inkFaint)
                Link("data.json", destination: URL(string: "https://efir.org/explorer/data.json")!)
                    .foregroundColor(MontanaTheme.gold)
                Text("·").foregroundColor(MontanaTheme.inkFaint)
                Link("GitHub", destination: URL(string: "https://github.com/efir369999/Montana")!)
                    .foregroundColor(MontanaTheme.gold)
            }
            Text(loc.t("net.install_node")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                .padding(.top, 4).fixedSize(horizontal: false, vertical: true)
            Text("curl -sSL https://raw.githubusercontent.com/efir369999/Montana/main/Code/scripts/install-vps.sh | sudo bash")
                .font(.caption.monospaced()).foregroundColor(MontanaTheme.ink)
                .textSelection(.enabled)
                .padding(8)
                .background(MontanaTheme.bgSoft)
                .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
        }
    }

    @ViewBuilder private var myNodeCard: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "person.crop.circle.badge.checkmark").foregroundColor(MontanaTheme.gold)
                Text(loc.t("net.my_node")).font(.headline).foregroundColor(MontanaTheme.ink)
                Spacer()
                HStack(spacing: 6) {
                    Circle().fill(runner.status.phase.color).frame(width: 8, height: 8)
                    Text(loc.t(runner.status.phase.l10nKey)).font(.caption).foregroundColor(MontanaTheme.ink)
                }
            }
            if runner.status.currentWindow > 0,
               let mesh = model.doc?.network_summary, mesh.max_window > 0 {
                HStack {
                    Text(String(format: loc.t("net.synced_to"), "\(runner.status.currentWindow)", "\(mesh.max_window)"))
                        .font(.caption.monospacedDigit()).foregroundColor(MontanaTheme.inkMute)
                    Spacer()
                    let pct = Double(runner.status.currentWindow) / Double(max(mesh.max_window, 1)) * 100
                    Text(String(format: "%.1f%%", pct)).font(.caption.bold()).foregroundColor(MontanaTheme.gold)
                }
                ProgressView(value: min(Double(runner.status.currentWindow) / Double(mesh.max_window), 1.0))
                    .tint(MontanaTheme.gold)
            } else if !runner.isRunning {
                Text(loc.t("net.not_started")).font(.caption).foregroundColor(MontanaTheme.inkMute)
            } else {
                ProgressView(loc.t("net.connecting")).controlSize(.small).tint(MontanaTheme.gold)
            }
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(MontanaTheme.bgSoft)
        .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
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
            Text(label).font(.caption).foregroundColor(MontanaTheme.inkMute).textCase(.uppercase)
            HStack(alignment: .firstTextBaseline, spacing: 4) {
                Text(value).font(.title2.bold().monospacedDigit()).foregroundColor(MontanaTheme.ink)
                Text(unit).font(.caption).foregroundColor(MontanaTheme.inkMute)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(MontanaTheme.bgSoft)
        .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
    }
}
