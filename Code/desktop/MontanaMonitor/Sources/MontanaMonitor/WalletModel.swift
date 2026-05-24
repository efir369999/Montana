import Foundation
import SwiftUI
import MontanaBindings

enum WalletState: Equatable {
    case empty
    case loaded(MontanaWallet)
    case error(String)
}

struct MontanaWallet: Equatable {
    let accountIdHex: String        // 64 hex (32 bytes from derive_account_id)
    let pubkey: Data                // 1952 bytes ML-DSA-65
    let mnemonicShownOnce: String?  // только пока не сохранили — потом nil
}

private struct StoredWallet: Codable {
    let accountIdHex: String
    let pubkey: Data
    let seckey: Data
}

@MainActor
final class WalletModel: ObservableObject {
    @Published private(set) var state: WalletState = .empty
    @Published var balanceNJ: UInt64? = nil

    private let storageKey = "MontanaWallet_v1"
    private var balanceTimer: Timer?

    init() {
        loadFromKeychain()
    }

    func abiVersion() -> UInt32 { mt_abi_version() }

    /// Создать новый кошелёк из случайной мнемоники (24 слова Montana-spec).
    /// ВНИМАНИЕ: мнемоника возвращается ОДИН раз — далее живёт только в Keychain как seckey.
    func createNew(entropyOverride: Data? = nil) {
        let entropy = entropyOverride ?? Self.cryptoRandom(32)
        guard let mnemonic = mnemonicFromEntropy(entropy) else {
            state = .error("не удалось закодировать мнемонику")
            return
        }
        install(mnemonic: mnemonic, showOnce: true)
    }

    func restore(mnemonic: String) {
        install(mnemonic: mnemonic, showOnce: false)
    }

    func wipe() {
        KeychainStore.delete(key: storageKey)
        state = .empty
        balanceNJ = nil
        balanceTimer?.invalidate()
        balanceTimer = nil
    }

    private func install(mnemonic: String, showOnce: Bool) {
        var pubkey = [UInt8](repeating: 0, count: Int(MT_MLDSA_PUBKEY_SIZE))
        var seckey = [UInt8](repeating: 0, count: Int(MT_MLDSA_SECKEY_SIZE))
        var accountId = [UInt8](repeating: 0, count: Int(MT_ACCOUNT_ID_LEN))

        let rc = mnemonic.withCString { ptr in
            mt_account_from_mnemonic(ptr, &pubkey, &seckey, &accountId)
        }
        guard rc == MT_OK else {
            state = .error(errorString(rc))
            return
        }
        let accountIdHex = accountId.map { String(format: "%02x", $0) }.joined()
        let pkData = Data(pubkey)
        let skData = Data(seckey)

        do {
            let stored = StoredWallet(accountIdHex: accountIdHex, pubkey: pkData, seckey: skData)
            let data = try JSONEncoder().encode(stored)
            KeychainStore.write(key: storageKey, value: data)
        } catch {
            state = .error("keychain write: \(error)")
            return
        }
        state = .loaded(MontanaWallet(
            accountIdHex: accountIdHex,
            pubkey: pkData,
            mnemonicShownOnce: showOnce ? mnemonic : nil
        ))
        startBalancePolling(accountIdHex: accountIdHex)
    }

    private func loadFromKeychain() {
        guard let data = KeychainStore.read(key: storageKey),
              let stored = try? JSONDecoder().decode(StoredWallet.self, from: data) else {
            return
        }
        state = .loaded(MontanaWallet(
            accountIdHex: stored.accountIdHex,
            pubkey: stored.pubkey,
            mnemonicShownOnce: nil
        ))
        startBalancePolling(accountIdHex: stored.accountIdHex)
    }

    private func startBalancePolling(accountIdHex: String) {
        balanceTimer?.invalidate()
        fetchBalance(accountIdHex: accountIdHex)
        balanceTimer = Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { _ in
            Task { @MainActor in self.fetchBalance(accountIdHex: accountIdHex) }
        }
    }

    private func fetchBalance(accountIdHex: String) {
        guard let url = URL(string: "https://efir.org/explorer/account/\(accountIdHex).json") else { return }
        var req = URLRequest(url: url)
        req.timeoutInterval = 8
        URLSession.shared.dataTask(with: req) { [weak self] data, _, _ in
            guard let data,
                  let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let n = json["balance_n"] as? UInt64 else { return }
            DispatchQueue.main.async { self?.balanceNJ = n }
        }.resume()
    }

    static func cryptoRandom(_ n: Int) -> Data {
        var bytes = [UInt8](repeating: 0, count: n)
        let rc = SecRandomCopyBytes(kSecRandomDefault, n, &bytes)
        precondition(rc == errSecSuccess, "SecRandom failed")
        return Data(bytes)
    }

    /// Кодирование 32 байт энтропии в 24 слова через сам Rust (вызовом mt_mnemonic_to_master_seed
    /// в обратную сторону недоступно — потому здесь делаем простой вызов через node CLI).
    /// Для production: добавить mt_entropy_to_mnemonic в FFI.
    /// Временно — используем `montana-node init --entropy <hex32> --force` чтобы получить
    /// мнемонику, либо генерим её случайно на стороне Swift (placeholder).
    private func mnemonicFromEntropy(_ entropy: Data) -> String? {
        precondition(entropy.count == 32, "entropy must be 32 bytes")
        let capacity = 512
        var buf = [UInt8](repeating: 0, count: capacity)
        var written: size_t = 0
        let rc = entropy.withUnsafeBytes { ent in
            mt_entropy_to_mnemonic(ent.bindMemory(to: UInt8.self).baseAddress, &buf, capacity, &written)
        }
        guard rc == MT_OK else { return nil }
        return String(bytes: buf.prefix(Int(written)), encoding: .utf8)
    }

    private func errorString(_ rc: Int32) -> String {
        switch rc {
        case MT_ERR_NULL_PTR:              return "null pointer"
        case MT_ERR_INVALID_UTF8:          return "не UTF-8"
        case MT_ERR_MNEMONIC_WORD_COUNT:   return "не 24 слова"
        case MT_ERR_MNEMONIC_UNKNOWN_WORD: return "слово не из wordlist"
        case MT_ERR_MNEMONIC_CHECKSUM:     return "контрольная сумма не совпала"
        case MT_ERR_KEYGEN_FAILED:         return "keygen failed"
        case MT_ERR_SIGN_FAILED:           return "sign failed"
        case MT_ERR_VERIFY_FAILED:         return "verify failed"
        case MT_ERR_PANIC:                 return "rust panic"
        default:                            return "FFI rc=\(rc)"
        }
    }
}

enum KeychainStore {
    static func write(key: String, value: Data) {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key
        ]
        SecItemDelete(q as CFDictionary)
        var item = q
        item[kSecValueData as String] = value
        item[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        SecItemAdd(item as CFDictionary, nil)
    }

    static func read(key: String) -> Data? {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]
        var out: AnyObject?
        guard SecItemCopyMatching(q as CFDictionary, &out) == errSecSuccess else { return nil }
        return out as? Data
    }

    static func delete(key: String) {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key
        ]
        SecItemDelete(q as CFDictionary)
    }
}
