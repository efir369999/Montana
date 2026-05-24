import SwiftUI

struct WalletView: View {
    @ObservedObject var model: WalletModel
    @State private var restoreMnemonic: String = ""
    @State private var showingNewMnemonic: String?
    @State private var copiedFlash: Bool = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header
            Divider()
            content
            Spacer(minLength: 0)
        }
        .padding(14)
    }

    private var header: some View {
        HStack {
            Text("Кошелёк").font(.title3.bold())
            Spacer()
            Text("ABI v\(model.abiVersion())").font(.caption).foregroundColor(.secondary)
        }
    }

    @ViewBuilder private var content: some View {
        switch model.state {
        case .empty:
            emptyView
        case .loaded(let w):
            loadedView(w)
        case .error(let msg):
            errorView(msg)
        }
    }

    private var emptyView: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Создайте новый кошелёк или восстановите по 24 словам").font(.subheadline)
            HStack {
                Button("Создать новый") { model.createNew() }
                    .buttonStyle(.borderedProminent)
                Button("Помощь") { open("https://montana.quest/docs/wallet") }
            }
            Divider().padding(.vertical, 4)
            Text("Восстановить из 24 слов:").font(.subheadline)
            TextEditor(text: $restoreMnemonic)
                .font(.system(.body, design: .monospaced))
                .frame(height: 80)
                .border(Color.gray.opacity(0.3))
            HStack {
                Spacer()
                Button("Восстановить") {
                    model.restore(mnemonic: restoreMnemonic.trimmingCharacters(in: .whitespacesAndNewlines))
                    restoreMnemonic = ""
                }
                .disabled(restoreMnemonic.split(whereSeparator: { $0.isWhitespace }).count != 24)
            }
            mnemonicShownView
        }
    }

    @ViewBuilder private var mnemonicShownView: some View {
        if case .loaded(let w) = model.state, let m = w.mnemonicShownOnce ?? showingNewMnemonic {
            Divider()
            Text("🗝 Запишите 24 слова. После закрытия их больше не показать.")
                .font(.subheadline.bold()).foregroundColor(.orange)
            mnemonicGrid(m)
            HStack {
                Button("Скопировать") {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(m, forType: .string)
                    copiedFlash = true
                    DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { copiedFlash = false }
                }
                if copiedFlash { Text("скопировано").font(.caption).foregroundColor(.green) }
                Spacer()
                Button("Я записал, скрыть") { showingNewMnemonic = nil }
                    .buttonStyle(.borderedProminent)
            }
        }
    }

    private func mnemonicGrid(_ m: String) -> some View {
        let words = m.split(whereSeparator: { $0.isWhitespace }).map(String.init)
        return LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 6), count: 4), spacing: 6) {
            ForEach(Array(words.enumerated()), id: \.offset) { i, w in
                HStack(spacing: 4) {
                    Text("\(i+1).").font(.caption.monospacedDigit()).foregroundColor(.secondary).frame(width: 22, alignment: .trailing)
                    Text(w).font(.system(.body, design: .monospaced))
                    Spacer()
                }
                .padding(.horizontal, 6).padding(.vertical, 3)
                .background(RoundedRectangle(cornerRadius: 4).fill(Color.gray.opacity(0.08)))
            }
        }
    }

    private func loadedView(_ w: MontanaWallet) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            VStack(alignment: .leading, spacing: 4) {
                Text("account_id").font(.caption).foregroundColor(.secondary)
                HStack {
                    Text(w.accountIdHex)
                        .font(.system(.caption, design: .monospaced))
                        .textSelection(.enabled)
                    Button(action: {
                        NSPasteboard.general.clearContents()
                        NSPasteboard.general.setString(w.accountIdHex, forType: .string)
                    }) { Image(systemName: "doc.on.doc") }
                    .buttonStyle(.plain)
                }
            }
            HStack(spacing: 24) {
                VStack(alignment: .leading, spacing: 2) {
                    Text("баланс").font(.caption).foregroundColor(.secondary)
                    if let n = model.balanceNJ {
                        Text(format_balance(n))
                            .font(.title2.bold().monospacedDigit())
                    } else {
                        Text("—").font(.title2.bold().monospacedDigit()).foregroundColor(.secondary)
                    }
                }
                VStack(alignment: .leading, spacing: 2) {
                    Text("криптосистема").font(.caption).foregroundColor(.secondary)
                    Text("ML-DSA-65").font(.body.monospacedDigit())
                }
            }
            Divider()
            HStack {
                Button("Получить (QR)") { /* TODO */ }
                    .disabled(true)
                Button("Отправить") { /* TODO */ }
                    .disabled(true)
                Spacer()
                Button("Стереть") { model.wipe() }
                    .foregroundColor(.red)
            }
            mnemonicShownView
            Text("Подписи через mt-bindings (Rust SSOT). Перевод средств — будет в следующей итерации (требует RPC к узлу).")
                .font(.caption).foregroundColor(.secondary)
        }
    }

    private func errorView(_ msg: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Ошибка").foregroundColor(.red).bold()
            Text(msg).font(.body)
            Button("Сбросить") { model.wipe() }
        }
    }

    private func format_balance(_ n: UInt64) -> String {
        let coin = Double(n) / 1_000_000_000_000.0
        return String(format: "%.6f Ɉ", coin)
    }

    private func open(_ s: String) { if let u = URL(string: s) { NSWorkspace.shared.open(u) } }
}
