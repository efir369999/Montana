import SwiftUI

/// Палитра и шрифты Монтаны — единые с Android `/vpn/app/` и iOS Montana app.
/// Источник CSS: montana.quest/vpn/app/index.html — :root { --gold #ca9335, --bg #0c0a08, … }
enum MontanaTheme {
    static let bg       = Color(red: 0x0c/255, green: 0x0a/255, blue: 0x08/255)
    static let bgSoft   = Color(red: 0x14/255, green: 0x11/255, blue: 0x0d/255)
    static let line     = Color(red: 0x2a/255, green: 0x25/255, blue: 0x20/255)
    static let ink      = Color(red: 0xe8/255, green: 0xe0/255, blue: 0xd0/255)
    static let inkMute  = Color(red: 0x8a/255, green: 0x81/255, blue: 0x75/255)
    static let inkFaint = Color(red: 0x4a/255, green: 0x44/255, blue: 0x3c/255)
    static let gold     = Color(red: 0xca/255, green: 0x93/255, blue: 0x35/255)
    static let goldSoft = Color(red: 0xd9/255, green: 0xa7/255, blue: 0x55/255)
    static let goldDeep = Color(red: 0x8e/255, green: 0x68/255, blue: 0x24/255)
    static let ok       = Color(red: 0x6e/255, green: 0xc1/255, blue: 0x8a/255)
    static let err      = Color(red: 0xd6/255, green: 0x52/255, blue: 0x52/255)

    static let titleFont  = Font.system(size: 24, weight: .medium, design: .default)
    static let bodyFont   = Font.system(size: 14, weight: .regular, design: .default)
    static let captionFont = Font.system(size: 11, weight: .regular, design: .default).monospaced()
    static let monoFont   = Font.system(size: 13, design: .monospaced)
}

struct MontanaButtonStyle: ButtonStyle {
    enum Variant { case solid, outline, ghost }
    var variant: Variant = .solid

    func makeBody(configuration: Configuration) -> some View {
        let baseFG: Color = {
            switch variant {
            case .solid:   return MontanaTheme.bg
            case .outline: return MontanaTheme.gold
            case .ghost:   return MontanaTheme.inkMute
            }
        }()
        let bg: Color = variant == .solid ? MontanaTheme.gold : .clear
        let border: Color = variant == .ghost ? MontanaTheme.line : MontanaTheme.goldDeep
        return configuration.label
            .font(.system(size: 13).weight(variant == .solid ? .semibold : .regular))
            .tracking(1.2)
            .textCase(.uppercase)
            .padding(.horizontal, 18).padding(.vertical, 12)
            .frame(maxWidth: .infinity)
            .background(bg)
            .foregroundColor(baseFG)
            .overlay(RoundedRectangle(cornerRadius: 0).strokeBorder(border, lineWidth: 1))
            .opacity(configuration.isPressed ? 0.7 : 1.0)
    }
}

extension View {
    func montanaCard() -> some View {
        self.padding(14)
            .background(MontanaTheme.bgSoft)
            .overlay(Rectangle().strokeBorder(MontanaTheme.line, lineWidth: 1))
    }
}
