import SwiftUI
import ServiceManagement

@MainActor
final class AppSettings: ObservableObject {
    @AppStorage("settings.autostart_app")  var autostartApp: Bool = false {
        didSet { applyLaunchAtLogin() }
    }
    @AppStorage("settings.autostart_node") var autostartNode: Bool = true
    @AppStorage("settings.autostart_vpn")  var autostartVPN: Bool = false
    @AppStorage("settings.refresh_min")    var subRefreshMin: Int = 30
    @AppStorage("settings.killswitch")     var killSwitch: Bool = false

    init() {
        autostartApp = (SMAppService.mainApp.status == .enabled)
    }

    private func applyLaunchAtLogin() {
        do {
            if autostartApp { try SMAppService.mainApp.register() }
            else            { try SMAppService.mainApp.unregister() }
        } catch {}
    }
}

struct SettingsSheet: View {
    @ObservedObject var loc: AppLocale
    @ObservedObject var settings: AppSettings
    @Environment(\.dismiss) var dismiss

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                HStack {
                    Text(loc.t("settings.title")).font(.title2.bold()).foregroundColor(MontanaTheme.ink)
                    Spacer()
                    Button(loc.t("settings.done")) { dismiss() }
                        .buttonStyle(.borderedProminent).tint(MontanaTheme.gold)
                }

                groupLabel(loc.t("settings.language"))
                HStack(spacing: 8) {
                    ForEach(AppLang.allCases) { l in
                        Button { loc.lang = l } label: {
                            Text(l.label)
                                .font(.system(size: 12).weight(.medium))
                                .padding(.horizontal, 14).padding(.vertical, 6)
                                .background(loc.lang == l ? MontanaTheme.gold.opacity(0.18) : Color.clear)
                                .foregroundColor(loc.lang == l ? MontanaTheme.gold : MontanaTheme.ink)
                                .overlay(Rectangle().strokeBorder(loc.lang == l ? MontanaTheme.goldDeep : MontanaTheme.line, lineWidth: 1))
                        }.buttonStyle(.plain)
                    }
                    Spacer()
                }

                Toggle(loc.t("settings.autostart_app"),  isOn: $settings.autostartApp)
                Toggle(loc.t("settings.autostart_node"), isOn: $settings.autostartNode)
                Toggle(loc.t("settings.autostart_vpn"),  isOn: $settings.autostartVPN)

                VStack(alignment: .leading, spacing: 6) {
                    Text(loc.t("settings.refresh_min")).foregroundColor(MontanaTheme.ink)
                    Stepper(value: $settings.subRefreshMin, in: 5...240, step: 5) {
                        Text("\(settings.subRefreshMin) мин").foregroundColor(MontanaTheme.inkMute)
                    }
                }

                VStack(alignment: .leading, spacing: 4) {
                    Toggle(loc.t("settings.killswitch"), isOn: $settings.killSwitch)
                    Text(loc.t("settings.killswitch_hint")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                        .fixedSize(horizontal: false, vertical: true)
                }

                Divider().background(MontanaTheme.line)
                VStack(alignment: .leading, spacing: 4) {
                    Text(loc.t("settings.about")).font(.headline).foregroundColor(MontanaTheme.ink)
                    Text(loc.t("settings.about.body")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            .padding(24)
        }
        .frame(minWidth: 460, idealWidth: 480, minHeight: 440, idealHeight: 500)
        .background(MontanaTheme.bg)
    }

    private func groupLabel(_ s: String) -> some View {
        Text(s).font(.caption.bold()).foregroundColor(MontanaTheme.inkMute).textCase(.uppercase)
    }
}
