import SwiftUI
import AppKit
import ServiceManagement

@main
struct MontanaApp: App {
    @StateObject private var meshModel   = MeshModel()
    @StateObject private var vpnModel    = VPNModel()
    @StateObject private var walletModel = WalletModel()

    init() {
        NSApplication.shared.setActivationPolicy(.regular)
    }

    var body: some Scene {
        WindowGroup("Montana Ядро 0.1") {
            RootView(meshModel: meshModel, vpnModel: vpnModel, walletModel: walletModel)
                .frame(minWidth: 720, minHeight: 520)
                .onAppear {
                    meshModel.startPolling()
                    vpnModel.bootstrap()
                }
        }
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(replacing: .help) {
                Button("Сайт Montana") {
                    if let u = URL(string: "https://montana.quest/") { NSWorkspace.shared.open(u) }
                }
                Button("Эксплорер") {
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
    @State private var tab: MainTab = .wallet

    var body: some View {
        NavigationSplitView {
            sidebar
        } detail: {
            detail
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .navigationSplitViewStyle(.balanced)
    }

    private var sidebar: some View {
        List(selection: $tab) {
            ForEach(MainTab.allCases) { t in
                Label(t.rawValue, systemImage: t.systemImage).tag(t)
            }
            Section {
                Text("Montana Ядро 0.1")
                    .font(.caption2).foregroundColor(.secondary)
                if let s = meshModel.doc?.network_summary {
                    HStack(spacing: 6) {
                        Circle().fill(.green).frame(width: 6, height: 6)
                        Text("сеть: W \(s.max_window)").font(.caption2.monospacedDigit())
                    }
                }
                if vpnModel.state == .connected, let s = vpnModel.currentServer {
                    HStack(spacing: 6) {
                        Circle().fill(.green).frame(width: 6, height: 6)
                        Text("ВПН: \(s.flag) \(s.city)").font(.caption2)
                    }
                }
            } header: { Text("статус").font(.caption2) }
        }
        .listStyle(.sidebar)
        .navigationSplitViewColumnWidth(min: 160, ideal: 180, max: 220)
    }

    @ViewBuilder private var detail: some View {
        switch tab {
        case .wallet:  WalletView(model: walletModel)
        case .vpn:     VPNView(model: vpnModel)
        case .network: NetworkView(model: meshModel)
        }
    }
}
