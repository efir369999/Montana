import SwiftUI
import AppKit
import ServiceManagement

@main
struct MontanaMonitorApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    var body: some Scene { Settings { EmptyView() } }
}

enum MainTab: String, CaseIterable, Identifiable {
    case wallet = "Кошелёк"
    case mesh   = "Сеть"
    case vpn    = "ВПН"
    var id: String { rawValue }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private var popover: NSPopover!
    let meshModel   = MeshModel()
    let vpnModel    = VPNModel()
    let walletModel = WalletModel()

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        refreshStatusButton(nodes: 0, total: 0, vpnOn: false)
        statusItem.button?.action = #selector(togglePopover(_:))
        statusItem.button?.target = self
        popover = NSPopover()
        popover.contentSize = NSSize(width: 560, height: 620)
        popover.behavior = .transient
        popover.contentViewController = NSHostingController(
            rootView: RootView(meshModel: meshModel, vpnModel: vpnModel, walletModel: walletModel)
        )
        meshModel.onUpdate = { [weak self] active, total in
            guard let self else { return }
            self.refreshStatusButton(nodes: active, total: total, vpnOn: self.vpnModel.state == .connected)
        }
        vpnModel.onStateChange = { [weak self] in
            guard let self else { return }
            let s = self.meshModel.doc?.network_summary
            self.refreshStatusButton(
                nodes: s?.active_nodes ?? 0,
                total: s?.total_nodes ?? 0,
                vpnOn: self.vpnModel.state == .connected
            )
        }
        meshModel.startPolling()
        vpnModel.bootstrap()
    }

    private func refreshStatusButton(nodes: Int, total: Int, vpnOn: Bool) {
        guard let btn = statusItem.button else { return }
        let title = total == 0 ? "Mt …" : "Mt \(nodes)/\(total)"
        let symbol = vpnOn ? "lock.shield.fill" : "circle.grid.3x3.fill"
        let icon = NSImage(systemSymbolName: symbol, accessibilityDescription: "Montana")
        icon?.isTemplate = !vpnOn
        btn.image = icon
        btn.title = vpnOn ? "Mt 🔒 \(nodes)/\(total)" : title
        btn.imagePosition = .imageLeading
    }

    @objc func togglePopover(_ sender: AnyObject?) {
        guard let btn = statusItem.button else { return }
        if popover.isShown { popover.performClose(sender) }
        else { popover.show(relativeTo: btn.bounds, of: btn, preferredEdge: .minY) }
    }
}

struct RootView: View {
    @ObservedObject var meshModel: MeshModel
    @ObservedObject var vpnModel: VPNModel
    @ObservedObject var walletModel: WalletModel
    @State private var tab: MainTab = .wallet
    @State private var launchAtLogin = false

    var body: some View {
        VStack(spacing: 0) {
            Picker("", selection: $tab) {
                ForEach(MainTab.allCases) { Text($0.rawValue).tag($0) }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal, 14)
            .padding(.top, 12)
            .padding(.bottom, 8)

            Group {
                switch tab {
                case .wallet: WalletView(model: walletModel)
                case .mesh:   MeshView(model: meshModel)
                case .vpn:    VPNView(model: vpnModel)
                }
            }
            Divider()
            footer
        }
        .frame(width: 560)
        .onAppear { launchAtLogin = SMAppService.mainApp.status == .enabled }
    }

    private var footer: some View {
        HStack {
            Toggle("При входе в систему", isOn: $launchAtLogin).font(.caption)
                .onChange(of: launchAtLogin) { _, new in setLaunchAtLogin(new) }
            Spacer()
            Button("Сеть") { open("https://montana.quest/net/") }
            Button("Репо") { open("https://github.com/efir369999/Montana") }
            Button("Закрыть") { NSApp.terminate(nil) }
        }
        .padding(12)
    }

    private func setLaunchAtLogin(_ enabled: Bool) {
        do {
            if enabled { try SMAppService.mainApp.register() }
            else { try SMAppService.mainApp.unregister() }
        } catch {
            launchAtLogin = SMAppService.mainApp.status == .enabled
        }
    }

    private func open(_ s: String) { if let u = URL(string: s) { NSWorkspace.shared.open(u) } }
}
