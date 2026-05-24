import SwiftUI

struct VPNView: View {
    @ObservedObject var model: VPNModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header
            Divider()
            connectBlock
            Divider()
            Text("Серверы").font(.subheadline.bold())
            ScrollView {
                VStack(spacing: 4) {
                    ForEach(model.servers) { s in
                        ServerRow(server: s, isSelected: s.id == effectiveSelectedId)
                            .onTapGesture { model.selected = s.id }
                            .contentShape(Rectangle())
                    }
                }
            }
            .frame(maxHeight: 220)
            if let err = model.lastError {
                Text("ошибка: \(err)").font(.caption).foregroundColor(.red).lineLimit(2)
            }
            Spacer(minLength: 0)
        }
        .padding(14)
    }

    private var effectiveSelectedId: String? {
        model.selected ?? model.servers.first?.id
    }

    private var header: some View {
        HStack {
            Text("ВПН Монтана").font(.title3.bold())
            Spacer()
            statePill
        }
    }

    private var statePill: some View {
        let (label, color): (String, Color) = {
            switch model.state {
            case .disconnected: return ("отключено", .gray)
            case .connecting:   return ("подключение…", .orange)
            case .connected:    return ("подключено", .green)
            case .error:        return ("ошибка", .red)
            }
        }()
        return HStack(spacing: 6) {
            Circle().fill(color).frame(width: 8, height: 8)
            Text(label).font(.caption)
        }
        .padding(.horizontal, 8).padding(.vertical, 3)
        .background(RoundedRectangle(cornerRadius: 6).fill(color.opacity(0.15)))
    }

    private var connectBlock: some View {
        HStack(spacing: 14) {
            if let s = model.currentServer {
                VStack(alignment: .leading, spacing: 2) {
                    Text("\(s.flag) \(s.city)").font(.headline)
                    Text(s.host).font(.caption).foregroundColor(.secondary)
                }
            } else {
                Text("загрузка серверов…").foregroundColor(.secondary)
            }
            Spacer()
            if model.state == .connected {
                VStack(alignment: .trailing, spacing: 2) {
                    Text("↑ \(format_bytes(model.bytesUp))").font(.caption.monospacedDigit())
                    Text("↓ \(format_bytes(model.bytesDown))").font(.caption.monospacedDigit())
                }
            }
            connectButton
        }
    }

    @ViewBuilder private var connectButton: some View {
        switch model.state {
        case .disconnected, .error:
            Button(action: { model.connect() }) {
                Text("Подключить").bold()
            }
            .buttonStyle(.borderedProminent)
            .disabled(model.currentServer == nil)
        case .connecting:
            ProgressView().scaleEffect(0.6)
        case .connected:
            Button(action: { model.disconnect() }) {
                Text("Отключить").bold()
            }
            .buttonStyle(.bordered)
            .tint(.red)
        }
    }

    private func format_bytes(_ b: UInt64) -> String {
        let units = ["B","KB","MB","GB","TB"]
        var v = Double(b); var i = 0
        while v >= 1024 && i < units.count - 1 { v /= 1024; i += 1 }
        return String(format: "%.1f %@", v, units[i])
    }
}

struct ServerRow: View {
    let server: VPNServer
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: isSelected ? "largecircle.fill.circle" : "circle")
                .foregroundColor(isSelected ? .accentColor : .secondary)
            Text(server.flag).font(.body)
            VStack(alignment: .leading, spacing: 0) {
                Text(server.city).font(.body)
                Text(server.host).font(.caption).foregroundColor(.secondary)
            }
            Spacer()
            if let ms = server.pingMs {
                Text("\(ms) мс")
                    .font(.caption.monospacedDigit())
                    .foregroundColor(pingColor(ms))
            } else {
                Text("…").font(.caption).foregroundColor(.secondary)
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 4)
        .background(isSelected
            ? RoundedRectangle(cornerRadius: 6).fill(Color.accentColor.opacity(0.10))
            : nil)
    }

    private func pingColor(_ ms: Int) -> Color {
        switch ms {
        case ..<80:   return .green
        case 80..<200: return .yellow
        default:      return .orange
        }
    }
}
