import SwiftUI
import AppKit
import ServiceManagement

@main
struct MontanaApp: App {
    @StateObject private var meshModel   = MeshModel()
    @StateObject private var vpnModel    = VPNModel()
    @StateObject private var walletModel = WalletModel()
    @StateObject private var nodeRunner  = NodeRunner()
    @StateObject private var loc         = AppLocale()
    @StateObject private var settings    = AppSettings()

    init() {
        NSApplication.shared.setActivationPolicy(.regular)
    }

    var body: some Scene {
        WindowGroup("Montana") {
            RootView(meshModel: meshModel, vpnModel: vpnModel, walletModel: walletModel, nodeRunner: nodeRunner, loc: loc, settings: settings)
                .frame(minWidth: 760, minHeight: 560)
                .onAppear {
                    meshModel.startPolling()
                    vpnModel.bootstrap()
                    if nodeRunner.hasIdentity { nodeRunner.start() }
                }
        }
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(replacing: .help) {
                Button(loc.t("menu.site")) {
                    if let u = URL(string: "https://montana.quest/") { NSWorkspace.shared.open(u) }
                }
                Button(loc.t("menu.explorer")) {
                    if let u = URL(string: "https://montana.quest/net/") { NSWorkspace.shared.open(u) }
                }
            }
        }
    }
}

enum MainTab: String, CaseIterable, Identifiable {
    case wallet   = "Кошелёк"
    case vpn      = "ВПН"
    case network  = "Сеть"
    var id: String { rawValue }

    var systemImage: String {
        switch self {
        case .wallet:  return "key.fill"
        case .vpn:     return "lock.shield.fill"
        case .network: return "globe"
        }
    }
}

struct RootView: View {
    @ObservedObject var meshModel: MeshModel
    @ObservedObject var vpnModel: VPNModel
    @ObservedObject var walletModel: WalletModel
    @ObservedObject var nodeRunner: NodeRunner
    @ObservedObject var loc: AppLocale
    @ObservedObject var settings: AppSettings
    @State private var tab: MainTab = .wallet
    @State private var showSettings = false

    var body: some View {
        NavigationSplitView {
            sidebarWithLangPicker
        } detail: {
            detail
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .background(MontanaTheme.bg)
        }
        .navigationSplitViewStyle(.balanced)
        .navigationTitle(loc.t("side.version"))
    }

    private var sidebarWithLangPicker: some View {
        VStack(spacing: 0) {
            sidebar
            Divider().background(MontanaTheme.line)
            profileChip
                .padding(.horizontal, 12).padding(.vertical, 10)
        }
        .background(MontanaTheme.bgSoft)
        .sheet(isPresented: $showSettings) {
            SettingsSheet(loc: loc, settings: settings)
        }
    }

    private var profileChip: some View {
        Button { showSettings = true } label: {
            HStack(spacing: 10) {
                ZStack {
                    Circle().fill(MontanaTheme.gold.opacity(0.18)).frame(width: 32, height: 32)
                    Image(systemName: "person.fill").foregroundColor(MontanaTheme.gold)
                }
                VStack(alignment: .leading, spacing: 0) {
                    Text(loc.t("profile.title")).font(.caption.bold()).foregroundColor(MontanaTheme.ink)
                    Text("\(loc.lang.label) · \(loc.t("profile.settings"))")
                        .font(.caption2).foregroundColor(MontanaTheme.inkMute)
                }
                Spacer()
                Image(systemName: "gearshape").foregroundColor(MontanaTheme.inkMute)
            }
            .padding(.horizontal, 8).padding(.vertical, 6)
            .background(MontanaTheme.bg)
            .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
        }
        .buttonStyle(.plain)
    }

    private var sidebar: some View {
        List(selection: $tab) {
            ForEach(MainTab.allCases) { t in
                Label(loc.t(t.l10nKey), systemImage: t.systemImage)
                    .foregroundColor(tab == t ? MontanaTheme.gold : MontanaTheme.ink)
                    .tag(t)
            }
            Section {
                Text(loc.t("side.version"))
                    .font(.caption2).foregroundColor(MontanaTheme.inkFaint)
                HStack(spacing: 6) {
                    Circle().fill(nodeRunner.status.phase.color).frame(width: 6, height: 6)
                    Text(nodeRunner.isRunning
                        ? String(format: loc.t("side.node.running"), loc.t(nodeRunner.status.phase.l10nKey))
                        : loc.t("side.node.stopped"))
                        .font(.caption2).foregroundColor(MontanaTheme.inkMute)
                }
                if let sum = meshModel.doc?.network_summary {
                    HStack(spacing: 6) {
                        Circle().fill(MontanaTheme.ok).frame(width: 6, height: 6)
                        Text(String(format: loc.t("side.net.window"), "\(sum.max_window)"))
                            .font(.caption2.monospacedDigit()).foregroundColor(MontanaTheme.inkMute)
                    }
                }
                if vpnModel.state == .connected, let vs = vpnModel.currentServer {
                    HStack(spacing: 6) {
                        Circle().fill(MontanaTheme.ok).frame(width: 6, height: 6)
                        Text(String(format: loc.t("side.vpn.on"), "\(vs.flag) \(vs.city)"))
                            .font(.caption2).foregroundColor(MontanaTheme.inkMute)
                    }
                }
            } header: { Text(loc.t("side.status")).font(.caption2).foregroundColor(MontanaTheme.inkFaint) }
        }
        .listStyle(.sidebar)
        .scrollContentBackground(.hidden)
        .background(MontanaTheme.bgSoft)
        .navigationSplitViewColumnWidth(min: 180, ideal: 200, max: 240)
    }

    @ViewBuilder private var detail: some View {
        switch tab {
        case .wallet:  WalletView(model: walletModel, runner: nodeRunner, loc: loc)
        case .vpn:     VPNView(model: vpnModel, loc: loc)
        case .network: NetworkView(model: meshModel, runner: nodeRunner, loc: loc)
        }
    }
}

extension MainTab {
    var l10nKey: String {
        switch self {
        case .wallet:  return "tab.wallet"
        case .vpn:     return "tab.vpn"
        case .network: return "tab.network"
        }
    }
}

extension NodePhase {
    var l10nKey: String {
        switch self {
        case .unknown:      return "phase.unknown"
        case .bootstrap:    return "phase.bootstrap"
        case .candidateVdf: return "phase.candidateVdf"
        case .registered:   return "phase.registered"
        case .active:       return "phase.active"
        }
    }
}
