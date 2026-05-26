import SwiftUI

struct VPNView: View {
    @ObservedObject var model: VPNModel
    @ObservedObject var loc: AppLocale

    @State private var showAddServer = false
    @State private var showAddSub = false
    @State private var newServerUrl = ""
    @State private var newServerName = ""
    @State private var newSubUrl = ""

    private let timeFmt: DateFormatter = {
        let f = DateFormatter(); f.dateFormat = "HH:mm"; return f
    }()

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header
            Divider().background(MontanaTheme.line)
            connectBlock
            Divider().background(MontanaTheme.line)
            subsBar
            Divider().background(MontanaTheme.line)
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    if model.servers.isEmpty && model.manualServers.isEmpty {
                        Text(loc.t("vpn.empty")).foregroundColor(MontanaTheme.inkMute).font(.callout)
                            .frame(maxWidth: .infinity, alignment: .center).padding(24)
                    }
                    if !model.servers.isEmpty {
                        sectionHeader(loc.t("vpn.section.auto"))
                        ForEach(model.subscriptions) { sub in
                            subscriptionGroup(sub)
                        }
                    }
                    if !model.manualServers.isEmpty {
                        sectionHeader(loc.t("vpn.section.manual"))
                        ForEach(model.manualServers) { s in
                            serverRow(s, deletable: true)
                        }
                    }
                }
            }
            if let err = model.lastError {
                let msg = err == "PORT_BUSY" ? loc.t("vpn.port_busy") : String(format: loc.t("vpn.error_label"), err)
                Text(msg).font(.caption).foregroundColor(MontanaTheme.err).lineLimit(3)
            }
        }
        .padding(24)
        .background(MontanaTheme.bg)
        .sheet(isPresented: $showAddServer) { addServerSheet }
        .sheet(isPresented: $showAddSub) { addSubSheet }
        .onAppear {
            model.confirmFirstSetup = {
                let a = NSAlert()
                a.messageText = loc.t("vpn.first_setup.title")
                a.informativeText = loc.t("vpn.first_setup.body")
                a.alertStyle = .informational
                a.addButton(withTitle: loc.t("vpn.first_setup.ok"))
                a.addButton(withTitle: loc.t("vpn.first_setup.cancel"))
                return a.runModal() == .alertFirstButtonReturn
            }
        }
    }

    // ---------- Header / state pill ----------

    private var header: some View {
        HStack {
            Text(loc.t("vpn.title")).font(.title.bold()).foregroundColor(MontanaTheme.ink)
            Spacer()
            statePill
        }
    }
    private var statePill: some View {
        let (key, color): (String, Color) = {
            switch model.state {
            case .disconnected: return ("vpn.state.off",  MontanaTheme.inkMute)
            case .connecting:   return ("vpn.state.conn", MontanaTheme.gold)
            case .connected:    return ("vpn.state.on",   MontanaTheme.ok)
            case .error:        return ("vpn.state.err",  MontanaTheme.err)
            }
        }()
        return HStack(spacing: 6) {
            Circle().fill(color).frame(width: 8, height: 8)
            Text(loc.t(key)).font(.caption).foregroundColor(MontanaTheme.ink)
        }
        .padding(.horizontal, 8).padding(.vertical, 3)
        .background(color.opacity(0.15))
        .overlay(Rectangle().strokeBorder(color.opacity(0.4), lineWidth: 1))
    }

    // ---------- Connect block ----------

    private var connectBlock: some View {
        HStack(spacing: 14) {
            if let s = model.currentServer {
                VStack(alignment: .leading, spacing: 2) {
                    Text("\(s.flag) \(s.city)").font(.headline).foregroundColor(MontanaTheme.ink)
                    Text(s.host).font(.caption).foregroundColor(MontanaTheme.inkMute)
                }
            } else {
                Text(loc.t("vpn.loading")).foregroundColor(MontanaTheme.inkMute)
            }
            Spacer()
            if model.state == .connected {
                VStack(alignment: .trailing, spacing: 2) {
                    Text("↑ \(format_bytes(model.bytesUp))").font(.caption.monospacedDigit()).foregroundColor(MontanaTheme.inkMute)
                    Text("↓ \(format_bytes(model.bytesDown))").font(.caption.monospacedDigit()).foregroundColor(MontanaTheme.inkMute)
                }
            }
            connectButton
        }
    }
    @ViewBuilder private var connectButton: some View {
        switch model.state {
        case .disconnected, .error:
            Button { model.connect() } label: { Text(loc.t("vpn.connect")).bold() }
                .buttonStyle(.borderedProminent).tint(MontanaTheme.gold)
                .disabled(model.currentServer == nil)
        case .connecting:
            ProgressView().scaleEffect(0.6).tint(MontanaTheme.gold)
        case .connected:
            Button { model.disconnect() } label: { Text(loc.t("vpn.disconnect")).bold() }
                .buttonStyle(.bordered).tint(MontanaTheme.err)
        }
    }

    // ---------- Subscriptions bar ----------

    private var subsBar: some View {
        HStack(spacing: 8) {
            Text(loc.t("vpn.subs")).font(.caption.bold()).foregroundColor(MontanaTheme.inkMute).textCase(.uppercase)
            Spacer()
            Button { model.refreshAll() } label: {
                Label(loc.t("vpn.sub.refresh_all"), systemImage: "arrow.clockwise")
            }.buttonStyle(.bordered).controlSize(.small)
            Button { model.pingAll() } label: {
                Label(loc.t("vpn.ping_all"), systemImage: "bolt.fill")
            }.buttonStyle(.bordered).controlSize(.small)
            Button { showAddSub = true } label: {
                Label(loc.t("vpn.sub.add"), systemImage: "plus")
            }.buttonStyle(.bordered).controlSize(.small)
            Button { showAddServer = true } label: {
                Label(loc.t("vpn.add_server"), systemImage: "plus.circle")
            }.buttonStyle(.bordered).controlSize(.small).tint(MontanaTheme.gold)
        }
    }

    private func sectionHeader(_ s: String) -> some View {
        Text(s).font(.caption.bold()).foregroundColor(MontanaTheme.inkFaint).textCase(.uppercase).padding(.top, 4)
    }

    private func subscriptionGroup(_ sub: Subscription) -> some View {
        let subServers = model.servers.filter {
            if case .subscription(let id) = $0.source { return id == sub.id }
            return false
        }
        return VStack(alignment: .leading, spacing: 6) {
            HStack {
                Image(systemName: "link").font(.caption).foregroundColor(MontanaTheme.inkMute)
                Text(sub.name.isEmpty ? loc.t("vpn.sub.default_name") : sub.name)
                    .font(.subheadline).foregroundColor(MontanaTheme.ink)
                Text("·").foregroundColor(MontanaTheme.inkFaint)
                Text(String(format: loc.t("vpn.sub.servers_count"), subServers.count))
                    .font(.caption).foregroundColor(MontanaTheme.inkMute)
                Spacer()
                let updated = sub.lastRefreshed.map { timeFmt.string(from: $0) } ?? loc.t("vpn.sub.never")
                Text(String(format: loc.t("vpn.sub.last_update"), updated))
                    .font(.caption.monospacedDigit()).foregroundColor(MontanaTheme.inkFaint)
                Menu {
                    Button(loc.t("vpn.sub.refresh")) { model.refresh(subId: sub.id) }
                    if sub.id != "default" {
                        Button(role: .destructive) { model.removeSubscription(sub) } label: {
                            Text(loc.t("vpn.sub.delete"))
                        }
                    }
                } label: {
                    Image(systemName: "ellipsis.circle").foregroundColor(MontanaTheme.inkMute)
                }.menuStyle(.borderlessButton).frame(width: 24)
            }
            ForEach(subServers) { s in serverRow(s, deletable: false) }
        }
        .padding(10)
        .background(MontanaTheme.bgSoft)
        .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
    }

    private func serverRow(_ s: VPNServer, deletable: Bool) -> some View {
        let isSel = model.selected == s.id || (model.selected == nil && model.allServers.first?.id == s.id)
        return HStack(spacing: 10) {
            Image(systemName: isSel ? "largecircle.fill.circle" : "circle")
                .foregroundColor(isSel ? MontanaTheme.gold : MontanaTheme.inkMute)
            Text(s.flag).font(.body)
            VStack(alignment: .leading, spacing: 0) {
                Text(s.city).font(.body).foregroundColor(MontanaTheme.ink)
                Text(s.host).font(.caption).foregroundColor(MontanaTheme.inkMute)
            }
            Spacer()
            if let ms = s.pingMs {
                Text("\(ms) ms").font(.caption.monospacedDigit()).foregroundColor(pingColor(ms))
            } else {
                Text("…").font(.caption).foregroundColor(MontanaTheme.inkMute)
            }
            if deletable {
                Button { model.removeManualServer(s) } label: { Image(systemName: "xmark.circle.fill") }
                    .buttonStyle(.plain).foregroundColor(MontanaTheme.inkFaint)
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 5)
        .background(isSel ? MontanaTheme.gold.opacity(0.08) : Color.clear)
        .contentShape(Rectangle())
        .onTapGesture { model.select(s.id) }
    }

    // ---------- Sheets ----------

    private var addServerSheet: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text(loc.t("vpn.add_server")).font(.title3.bold()).foregroundColor(MontanaTheme.ink)
            TextField(loc.t("vpn.server.name_ph"), text: $newServerName)
                .textFieldStyle(.roundedBorder)
            VStack(alignment: .leading, spacing: 4) {
                Text(loc.t("vpn.server.url_ph")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                TextEditor(text: $newServerUrl)
                    .font(.system(.caption, design: .monospaced))
                    .frame(minHeight: 100)
                    .padding(4)
                    .background(MontanaTheme.bgSoft)
                    .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
            }
            HStack {
                Spacer()
                Button(loc.t("vpn.server.cancel")) {
                    newServerUrl = ""; newServerName = ""; showAddServer = false
                }
                Button(loc.t("vpn.server.add")) {
                    if model.addManualServer(url: newServerUrl.trimmingCharacters(in: .whitespacesAndNewlines),
                                             customName: newServerName.trimmingCharacters(in: .whitespacesAndNewlines)) {
                        newServerUrl = ""; newServerName = ""; showAddServer = false
                    }
                }
                .buttonStyle(.borderedProminent).tint(MontanaTheme.gold)
                .disabled(!newServerUrl.hasPrefix("vless://"))
            }
        }
        .padding(20)
        .frame(width: 460)
        .background(MontanaTheme.bg)
    }
    private var addSubSheet: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text(loc.t("vpn.sub.add")).font(.title3.bold()).foregroundColor(MontanaTheme.ink)
            TextField(loc.t("vpn.sub.url_ph"), text: $newSubUrl)
                .textFieldStyle(.roundedBorder)
            HStack {
                Spacer()
                Button(loc.t("vpn.server.cancel")) { newSubUrl = ""; showAddSub = false }
                Button(loc.t("vpn.server.add")) {
                    let url = newSubUrl.trimmingCharacters(in: .whitespacesAndNewlines)
                    guard !url.isEmpty, URL(string: url) != nil else { return }
                    model.addSubscription(url: url, name: nil)
                    newSubUrl = ""; showAddSub = false
                }
                .buttonStyle(.borderedProminent).tint(MontanaTheme.gold)
                .disabled(URL(string: newSubUrl) == nil)
            }
        }
        .padding(20)
        .frame(width: 460)
        .background(MontanaTheme.bg)
    }

    private func format_bytes(_ b: UInt64) -> String {
        let units = ["B","KB","MB","GB","TB"]
        var v = Double(b); var i = 0
        while v >= 1024 && i < units.count - 1 { v /= 1024; i += 1 }
        return String(format: "%.1f %@", v, units[i])
    }
    private func pingColor(_ ms: Int) -> Color {
        switch ms {
        case ..<80: return MontanaTheme.ok
        case 80..<200: return MontanaTheme.gold
        default: return MontanaTheme.err
        }
    }
}
