import SwiftUI

struct WalletView: View {
    @ObservedObject var service: NodeService
    @State private var showSeed = false
    @State private var seedHex: String? = nil
    @State private var seedRevealed = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                receiveCard
                identityCard
                backupCard
                sendCard
            }
            .padding(20)
        }
        .sheet(isPresented: $showSeed, onDismiss: { seedHex = nil; seedRevealed = false }) {
            seedSheet
        }
    }

    private var receiveCard: some View {
        Card(title: "Получение Ɉ") {
            HStack(alignment: .top, spacing: 18) {
                if let qr = QRGenerator.image(from: service.identity.accountId), !service.identity.accountId.isEmpty {
                    Image(nsImage: qr)
                        .interpolation(.none)
                        .resizable()
                        .scaledToFit()
                        .frame(width: 180, height: 180)
                        .padding(8)
                        .background(Color.white)
                        .cornerRadius(8)
                } else {
                    RoundedRectangle(cornerRadius: 8)
                        .fill(Color.secondary.opacity(0.15))
                        .frame(width: 180, height: 180)
                        .overlay(Text("QR недоступен").font(.system(size: 11)))
                }
                VStack(alignment: .leading, spacing: 8) {
                    Text("account_id").font(.system(size: 11)).foregroundColor(.secondary)
                    Text(service.identity.accountId.isEmpty ? "—" : service.identity.accountId)
                        .font(.system(size: 11, design: .monospaced))
                        .textSelection(.enabled)
                        .lineLimit(2)
                        .truncationMode(.middle)
                    Button("Скопировать account_id") {
                        service.copyToPasteboard(service.identity.accountId)
                    }
                    .disabled(service.identity.accountId.isEmpty)
                    Spacer()
                    Text("Передайте этот QR-код или 64-hex строку отправителю. account_id неизменен на всё время существования узла.")
                        .font(.system(size: 11))
                        .foregroundColor(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
        }
    }

    private var identityCard: some View {
        Card(title: "Идентификация") {
            VStack(alignment: .leading, spacing: 8) {
                idRow("node_id", service.identity.nodeId)
                idRow("master_seed_fp", service.identity.masterSeedFp)
                idRow("libp2p_peer_id", service.identity.libp2pPeerId)
                Divider().padding(.vertical, 4)
                row("suite", service.identity.suite)
                row("account_pk[..16]", service.identity.accountPkPrefix)
                row("node_pk[..16]", service.identity.nodePkPrefix)
                row("mlkem_pk[..16]", service.identity.mlkemPkPrefix)
            }
        }
    }

    private var backupCard: some View {
        Card(title: "Backup и восстановление") {
            VStack(alignment: .leading, spacing: 12) {
                Text("Master seed (32 байта = 64 hex) — единственный источник восстановления identity. Без него identity.bin потерян безвозвратно.")
                    .font(.system(size: 12))
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
                HStack {
                    Button("Показать master seed…") { showSeed = true }
                        .controlSize(.large)
                    Spacer()
                    Button("Открыть identity.bin в Finder") {
                        let url = URL(fileURLWithPath: service.identity.identityPath)
                        NSWorkspace.shared.activateFileViewerSelecting([url])
                    }
                    .disabled(service.identity.identityPath.isEmpty)
                }
                Text("Восстановление на другом Mac: запустите установщик с переданным seed:\n  `INSTALL_MNEMONIC_OR_SEED='<64-hex>' bash install-anywhere.sh`")
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }

    private var sendCard: some View {
        Card(title: "Отправка Ɉ") {
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 8) {
                    Image(systemName: "exclamationmark.triangle.fill").foregroundColor(.orange)
                    Text("Отправка временно недоступна").font(.system(size: 13, weight: .medium))
                }
                Text("Узел работает в singleton-режиме (M5 closed; M6+ networking — следующий milestone). CLI-команда `montana-node transfer` появится в будущих релизах ядра. Сейчас приём Ɉ через QR работает (адрес запоминается отправителями), отправлять можно будет после публичного 3-node genesis ceremony.")
                    .font(.system(size: 12))
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }

    private var seedSheet: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Image(systemName: "exclamationmark.shield.fill").foregroundColor(.red).font(.title)
                Text("Master seed — секрет").font(.title2).bold()
            }
            Text("Кто владеет seed — владеет identity.bin, а значит всеми Ɉ на этом аккаунте. Никогда не передавайте, не фотографируйте экраном смартфона, не загружайте в облако. Запишите на бумагу и храните физически.")
                .font(.system(size: 12))
                .fixedSize(horizontal: false, vertical: true)

            if !seedRevealed {
                Button("Я понимаю риск, показать") {
                    Task {
                        seedHex = await service.revealMasterSeed()
                        seedRevealed = true
                    }
                }
                .controlSize(.large)
            } else if let hex = seedHex {
                VStack(alignment: .leading, spacing: 6) {
                    Text("master_seed (hex):").font(.system(size: 11)).foregroundColor(.secondary)
                    Text(hex)
                        .font(.system(size: 13, design: .monospaced))
                        .textSelection(.enabled)
                        .padding(8)
                        .background(Color.secondary.opacity(0.1))
                        .cornerRadius(6)
                }
                HStack {
                    Button("Скопировать") { service.copyToPasteboard(hex) }
                    Spacer()
                }
            } else {
                Text("Не удалось прочитать seed (проверьте `montana-node inspect --reveal-master-seed`)")
                    .foregroundColor(.red)
                    .font(.system(size: 12))
            }

            Spacer()
            HStack {
                Spacer()
                Button("Закрыть") { showSeed = false }.keyboardShortcut(.cancelAction)
            }
        }
        .padding(24)
        .frame(width: 560, height: 420)
    }

    private func idRow(_ label: String, _ value: String) -> some View {
        HStack(alignment: .top) {
            Text(label).font(.system(size: 11)).foregroundColor(.secondary).frame(width: 130, alignment: .leading)
            Text(value.isEmpty ? "—" : value)
                .font(.system(size: 11, design: .monospaced))
                .textSelection(.enabled)
                .lineLimit(2)
                .truncationMode(.middle)
            Spacer()
            Button(action: { service.copyToPasteboard(value) }) {
                Image(systemName: "doc.on.doc").imageScale(.small)
            }
            .buttonStyle(.borderless)
            .disabled(value.isEmpty)
        }
    }

    private func row(_ label: String, _ value: String) -> some View {
        HStack {
            Text(label).font(.system(size: 11)).foregroundColor(.secondary)
            Spacer()
            Text(value.isEmpty ? "—" : value).font(.system(size: 11, design: .monospaced))
        }
    }
}

