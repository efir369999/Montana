import SwiftUI
import MontanaBindings

struct WalletView: View {
    @ObservedObject var model: WalletModel
    @ObservedObject var runner: NodeRunner
    @ObservedObject var loc: AppLocale
    @State private var restoreMnemonic: String = ""
    @State private var showingMnemonic: String? = nil

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if !runner.hasIdentity { onboardingView } else { walletView }
            }
            .padding(24)
        }
        .background(MontanaTheme.bg)
    }

    private var onboardingView: some View {
        VStack(alignment: .leading, spacing: 18) {
            Text(loc.t("onboard.title")).font(.title.bold()).foregroundColor(MontanaTheme.ink)
            Text(loc.t("onboard.subtitle"))
                .font(.title3).foregroundColor(MontanaTheme.ink)
                .fixedSize(horizontal: false, vertical: true)

            VStack(alignment: .leading, spacing: 10) {
                explainStep(num: 1, key: "onboard.step1")
                explainStep(num: 2, key: "onboard.step2")
                explainStep(num: 3, key: "onboard.step3")
                explainStep(num: 4, key: "onboard.step4")
                explainStep(num: 5, key: "onboard.step5")
            }
            .padding(14)
            .background(MontanaTheme.bgSoft)
            .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))

            VStack(alignment: .leading, spacing: 6) {
                Text(loc.t("onboard.lifespan.t")).font(.caption.bold()).foregroundColor(MontanaTheme.ink)
                Text(loc.t("onboard.lifespan.b")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Divider().background(MontanaTheme.line).padding(.vertical, 4)

            Button { createNew() } label: {
                Label(loc.t("onboard.btn.create"), systemImage: "plus.circle.fill")
                    .frame(maxWidth: .infinity).padding(8)
            }
            .buttonStyle(.borderedProminent).tint(MontanaTheme.gold)
            .controlSize(.large)

            DisclosureGroup(loc.t("onboard.recover")) {
                VStack(alignment: .leading, spacing: 8) {
                    TextEditor(text: $restoreMnemonic)
                        .font(.system(.body, design: .monospaced))
                        .frame(height: 80)
                        .padding(6)
                        .background(MontanaTheme.bgSoft)
                        .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
                    HStack {
                        Text(String(format: loc.t("onboard.words"), wordCount))
                            .font(.caption).foregroundColor(wordCount == 24 ? MontanaTheme.ok : MontanaTheme.inkMute)
                        Spacer()
                        Button(loc.t("onboard.btn.restore")) { restore() }
                            .buttonStyle(.bordered).disabled(wordCount != 24)
                    }
                }
                .padding(.top, 6)
            }
            .foregroundColor(MontanaTheme.inkMute)
            .padding(.top, 4)

            if let m = showingMnemonic {
                Divider().background(MontanaTheme.line).padding(.top, 8)
                mnemonicCaptureCard(m)
            }
            if let err = model.lastError {
                Text(err).foregroundColor(MontanaTheme.err).font(.callout)
            }
            if let err = runner.lastError {
                Text(err).foregroundColor(MontanaTheme.err).font(.callout)
            }
        }
    }

    private func explainStep(num: Int, key: String) -> some View {
        HStack(alignment: .top, spacing: 12) {
            ZStack {
                Circle().fill(MontanaTheme.gold).frame(width: 22, height: 22)
                Text("\(num)").font(.caption.bold()).foregroundColor(MontanaTheme.bg)
            }
            VStack(alignment: .leading, spacing: 2) {
                Text(loc.t(key + ".t")).font(.subheadline.bold()).foregroundColor(MontanaTheme.ink)
                Text(loc.t(key + ".b")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }

    private var wordCount: Int {
        restoreMnemonic.split(whereSeparator: { $0.isWhitespace }).count
    }

    private func mnemonicCaptureCard(_ m: String) -> some View {
        let words = m.split(whereSeparator: { $0.isWhitespace }).map(String.init)
        return VStack(alignment: .leading, spacing: 10) {
            HStack {
                Image(systemName: "exclamationmark.triangle.fill").foregroundColor(MontanaTheme.gold)
                Text(loc.t("mnemonic.warn")).font(.subheadline.bold()).foregroundColor(MontanaTheme.ink)
            }
            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 8), count: 4), spacing: 8) {
                ForEach(Array(words.enumerated()), id: \.offset) { i, w in
                    HStack(spacing: 4) {
                        Text("\(i+1).").font(.caption.monospacedDigit())
                            .foregroundColor(MontanaTheme.inkFaint).frame(width: 24, alignment: .trailing)
                        Text(w).font(.system(.body, design: .monospaced)).foregroundColor(MontanaTheme.ink)
                        Spacer()
                    }
                    .padding(.horizontal, 8).padding(.vertical, 4)
                    .background(MontanaTheme.bgSoft)
                    .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
                }
            }
            HStack {
                Button(loc.t("mnemonic.copy")) {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(m, forType: .string)
                }
                Spacer()
                Button(loc.t("mnemonic.saved")) {
                    showingMnemonic = nil
                    runner.start()
                }
                .buttonStyle(.borderedProminent).tint(MontanaTheme.gold)
            }
        }
        .padding(14)
        .background(MontanaTheme.gold.opacity(0.06))
        .overlay(Rectangle().strokeBorder(MontanaTheme.goldDeep, lineWidth: 1))
    }

    private func createNew() {
        let entropy = WalletModel.cryptoRandom(32)
        guard let mnemonic = entropyToMnemonic(entropy) else {
            model.errorString("derive failed")
            return
        }
        do {
            try runner.installIdentity(mnemonic: mnemonic)
            showingMnemonic = mnemonic
        } catch {
            model.errorString("init: \(error.localizedDescription)")
        }
    }

    private func restore() {
        let m = restoreMnemonic.trimmingCharacters(in: .whitespacesAndNewlines)
        do {
            try runner.installIdentity(mnemonic: m)
            restoreMnemonic = ""
            runner.start()
        } catch {
            model.errorString("restore: \(error.localizedDescription)")
        }
    }

    private func entropyToMnemonic(_ data: Data) -> String? {
        let cap = 512
        var buf = [UInt8](repeating: 0, count: cap)
        var written: size_t = 0
        let rc = data.withUnsafeBytes { ent in
            mt_entropy_to_mnemonic(ent.bindMemory(to: UInt8.self).baseAddress, &buf, cap, &written)
        }
        guard rc == MT_OK else { return nil }
        return String(bytes: buf.prefix(Int(written)), encoding: .utf8)
    }

    // ---------- Loaded wallet ----------

    private var walletView: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(loc.t("wallet.title")).font(.title.bold()).foregroundColor(MontanaTheme.ink)
                    Text(loc.t("wallet.subtitle")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                }
                Spacer()
                phaseBadge
            }
            balanceCard
            accountIdCard
            actionButtons
            if !runner.isRunning { startNodeBanner } else { vdfProgressCard }
        }
    }

    private var phaseBadge: some View {
        HStack(spacing: 6) {
            Circle().fill(runner.status.phase.color).frame(width: 10, height: 10)
            Text(loc.t(runner.status.phase.l10nKey)).font(.caption).foregroundColor(MontanaTheme.ink)
        }
        .padding(.horizontal, 10).padding(.vertical, 4)
        .background(runner.status.phase.color.opacity(0.15))
        .overlay(Rectangle().strokeBorder(runner.status.phase.color.opacity(0.5), lineWidth: 1))
    }

    private var balanceCard: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(loc.t("wallet.balance")).font(.caption).foregroundColor(MontanaTheme.inkMute).textCase(.uppercase)
            HStack(alignment: .firstTextBaseline, spacing: 6) {
                Text(format_balance(runner.status.balanceNJ))
                    .font(.system(.largeTitle, design: .rounded).monospacedDigit().bold())
                    .foregroundColor(MontanaTheme.gold)
                Text("Ɉ").font(.title2).foregroundColor(MontanaTheme.inkMute)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(20)
        .background(MontanaTheme.bgSoft)
        .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
    }

    private var accountIdCard: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(loc.t("wallet.account_id")).font(.caption).foregroundColor(MontanaTheme.inkMute).textCase(.uppercase)
            HStack {
                Text(runner.status.accountIdHex.isEmpty ? "—" : runner.status.accountIdHex)
                    .font(.system(.caption, design: .monospaced))
                    .textSelection(.enabled).lineLimit(2).foregroundColor(MontanaTheme.ink)
                Spacer()
                Button {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(runner.status.accountIdHex, forType: .string)
                } label: { Image(systemName: "doc.on.doc") }.buttonStyle(.plain).foregroundColor(MontanaTheme.inkMute)
            }
        }
    }

    private var actionButtons: some View {
        HStack(spacing: 10) {
            Button {} label: { Label(loc.t("wallet.send"), systemImage: "arrow.up.right") }.disabled(true)
            Button {} label: { Label(loc.t("wallet.receive"), systemImage: "qrcode") }.disabled(true)
            Spacer()
            Menu {
                Button(role: .destructive) { confirmWipe() } label: {
                    Label(loc.t("wallet.wipe"), systemImage: "trash")
                }
            } label: { Image(systemName: "ellipsis.circle") }
        }
    }

    private var startNodeBanner: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(loc.t("wallet.node_stopped")).font(.headline).foregroundColor(MontanaTheme.ink)
            Text(loc.t("wallet.node_off_note")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                .fixedSize(horizontal: false, vertical: true)
            Button { runner.start() } label: {
                Label(loc.t("wallet.start_node"), systemImage: "play.fill")
            }
            .buttonStyle(.borderedProminent).tint(MontanaTheme.gold)
        }
        .padding(14)
        .background(MontanaTheme.bgSoft)
        .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
    }

    private var vdfProgressCard: some View {
        let chain = runner.status.chainLength
        let tau2 = max(runner.status.tau2, 1)
        let progress = min(Double(chain) / Double(tau2), 1.0)
        return VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(loc.t("wallet.candidate")).font(.headline).foregroundColor(MontanaTheme.ink)
                Spacer()
                Text(String(format: loc.t("wallet.candidate_of"), "\(chain)", "\(tau2)"))
                    .font(.caption.monospacedDigit()).foregroundColor(MontanaTheme.inkMute)
            }
            ProgressView(value: progress).tint(MontanaTheme.gold)
            Text(loc.t("wallet.candidate_note")).font(.caption).foregroundColor(MontanaTheme.inkMute)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(14)
        .background(MontanaTheme.bgSoft)
        .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
    }

    private func confirmWipe() {
        let alert = NSAlert()
        alert.messageText = loc.t("wipe.title")
        alert.informativeText = loc.t("wipe.info")
        alert.alertStyle = .warning
        alert.addButton(withTitle: loc.t("wipe.ok"))
        alert.addButton(withTitle: loc.t("wipe.cancel"))
        if alert.runModal() == .alertFirstButtonReturn {
            runner.stop()
            try? FileManager.default.removeItem(at: runner.dataDir)
            try? FileManager.default.createDirectory(at: runner.dataDir, withIntermediateDirectories: true)
        }
    }

    private func format_balance(_ n: UInt64) -> String {
        let coin = Double(n) / 1_000_000_000_000.0
        if coin == 0 { return "0" }
        return String(format: "%.6f", coin)
    }
}
