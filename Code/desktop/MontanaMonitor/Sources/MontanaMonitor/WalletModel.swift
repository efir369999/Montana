import Foundation
import SwiftUI
import Security

/// Минимальная модель для wallet-вкладки.
/// Identity и баланс живут в локальном узле — owns NodeRunner.
/// Здесь только last-error для UI и хелперы.
@MainActor
final class WalletModel: ObservableObject {
    @Published var lastError: String? = nil

    func errorString(_ msg: String) { self.lastError = msg }
    func clearError() { self.lastError = nil }

    static func cryptoRandom(_ n: Int) -> Data {
        var bytes = [UInt8](repeating: 0, count: n)
        let rc = SecRandomCopyBytes(kSecRandomDefault, n, &bytes)
        precondition(rc == errSecSuccess, "SecRandom failed")
        return Data(bytes)
    }
}
